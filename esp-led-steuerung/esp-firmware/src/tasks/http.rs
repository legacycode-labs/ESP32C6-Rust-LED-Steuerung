// HTTP Server Task - Serviert HTML und WebSocket
use core::future::pending;
use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_net::Stack;
use embassy_time::{Duration, Instant};
use picoserve::{io::embedded_io_async, response::IntoResponse, response::ws, routing::get};

use crate::config::*;
use crate::web::{
    INDEX_HTML,
    protocol::{OperationMode, RgbColor, WsClientMessage, WsServerMessage},
};
use crate::{LedColorChannel, LedColorMessage, LedColorSubscriber, LedCommand, LedCommandSender};
use serde_json_core;

/// Response-Enum für WebSocket-Endpoint
/// Ermöglicht Rückgabe von entweder WebSocket-Upgrade oder HTTP-Fehler
enum WebSocketResponse {
    Upgrade(
        ws::UpgradedWebSocket<ws::UnspecifiedProtocol, ws::CallbackNotUsingState<WebSocketHandler>>,
    ),
    ServiceUnavailable,
}

impl IntoResponse for WebSocketResponse {
    async fn write_to<
        R: embedded_io_async::Read,
        W: picoserve::response::ResponseWriter<Error = R::Error>,
    >(
        self,
        connection: picoserve::response::Connection<'_, R>,
        response_writer: W,
    ) -> Result<picoserve::ResponseSent, W::Error> {
        match self {
            WebSocketResponse::Upgrade(ws) => ws.write_to(connection, response_writer).await,
            WebSocketResponse::ServiceUnavailable => {
                picoserve::response::Response::new(
                    picoserve::response::StatusCode::new(503),
                    "Service Unavailable: Too many WebSocket connections (max 10)",
                )
                .with_header("Retry-After", "5")
                .write_to(connection, response_writer)
                .await
            }
        }
    }
}

/// HTTP Server Task - läuft parallel zu anderen Tasks
///
/// Dieser Task stellt den HTTP-Server bereit:
/// - Serviert index.html auf GET /
/// - WebSocket-Endpoint auf /ws für bidirektionale Kommunikation
/// - Empfängt LED-Farb-Updates via Channel
/// - Sendet Kommandos an LED Task via Channel
///
/// **Task Pool:** Diese Task wird 4x gespawnt für concurrent connections:
/// - Ermöglicht gleichzeitiges Laden von HTML + WebSocket-Verbindungen
/// - Verhindert Blockierung wenn eine Connection aktiv ist
///
/// # Parameter
/// - `task_id`: Eindeutige ID für diese Server-Instanz (0..3)
/// - `stack`: embassy-net Stack für Netzwerk-Zugriff
/// - `color_channel`: PubSub Channel für LED-Farb-Broadcasts (WebSocketHandler erstellt Subscriber)
/// - `command_sender`: Channel Sender für LED-Kommandos
#[embassy_executor::task(pool_size = 4)]
pub async fn http_server_task(
    task_id: usize,
    stack: &'static Stack<'static>,
    _color_channel: &'static LedColorChannel,
    command_sender: LedCommandSender,
) {
    info!("HTTP: Server task {} starting on port 80...", task_id);

    // Router-Konfiguration
    // WebSocket-Route mit async block
    let app = picoserve::Router::new().route("/", get(serve_html)).route(
        "/ws",
        get(
            |upgrade: picoserve::response::WebSocketUpgrade| async move {
                info!("HTTP: WebSocket upgrade requested");

                // Erstelle Subscriber für diese WebSocket-Connection
                // Mit 10 max. Subscribers (PubSubChannel<..., 2, 10, 1>) und 4 HTTP-Tasks
                // kann bei > 10 gleichzeitigen WebSocket-Clients die Subscriber-Allokation fehlschlagen.
                // Statt Panic senden wir HTTP 503 an den Client.
                match _color_channel.subscriber() {
                    Ok(color_subscriber) => {
                        info!("HTTP: Subscriber created, upgrading to WebSocket");
                        let handler = WebSocketHandler {
                            command_sender,
                            color_subscriber,
                        };
                        WebSocketResponse::Upgrade(upgrade.on_upgrade(handler))
                    }
                    Err(_) => {
                        info!(
                            "HTTP: No subscriber slots available (10/10 in use), sending HTTP 503"
                        );
                        WebSocketResponse::ServiceUnavailable
                    }
                }
            },
        ),
    );

    // Server-Konfiguration
    let config = picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
        persistent_start_read_request: Some(Duration::from_secs(5)),
    })
    .keep_connection_alive();

    // HTTP-Buffer für Requests/Responses
    let mut http_buffer = [0u8; HTTP_BUFFER_SIZE];

    // TCP-Buffers für Socket
    let mut rx_buffer = [0u8; TCP_RX_BUFFER_SIZE];
    let mut tx_buffer = [0u8; TCP_TX_BUFFER_SIZE];

    // Server erstellen
    let server = picoserve::Server::new(&app, &config, &mut http_buffer);

    // Server starten (lauscht auf Port 80)
    // task_id ermöglicht mehrere concurrent Server-Instanzen
    let _ = server
        .listen_and_serve(task_id, *stack, 80, &mut rx_buffer, &mut tx_buffer)
        .await;

    info!("HTTP: Server task {} ended", task_id);
}

/// Serviert die HTML-Hauptseite
async fn serve_html() -> impl IntoResponse {
    picoserve::response::Response::new(picoserve::response::StatusCode::OK, INDEX_HTML)
        .with_header("Content-Type", "text/html; charset=utf-8")
}

/// WebSocket-Handler State
/// Speichert Command Sender und Color Subscriber für bidirektionale Kommunikation
struct WebSocketHandler {
    command_sender: LedCommandSender,
    color_subscriber: LedColorSubscriber,
}

impl ws::WebSocketCallback for WebSocketHandler {
    async fn run<R: embedded_io_async::Read, W: embedded_io_async::Write<Error = R::Error>>(
        mut self,
        mut rx: ws::SocketRx<R>,
        mut tx: ws::SocketTx<W>,
    ) -> Result<(), W::Error> {
        info!("HTTP: WebSocket connection established");

        // Buffer für eingehende WebSocket-Nachrichten
        let mut buffer = [0u8; WEBSOCKET_BUFFER_SIZE];

        // Sende initiales Status-Update wenn Subscriber Messages hat
        if let Some(msg) = self.color_subscriber.try_next_message_pure() {
            let mode = if msg.is_auto_mode {
                OperationMode::Auto
            } else {
                OperationMode::Manual
            };
            Self::send_status_update(&mut tx, &msg, mode).await.ok();
        }

        let close_reason = loop {
            // Gleichzeitig auf zwei Events lauschen mit embassy_futures::select:
            // 1. WebSocket-Messages vom Browser
            // 2. LED-Color-Broadcasts vom PubSubChannel
            //
            // Dies ist effizienter als Polling mit Timer, da beide Futures
            // gleichzeitig awaited werden und nur bei tatsächlichen Events aufwachen.
            match select(
                rx.next_message(&mut buffer, pending()),
                self.color_subscriber.next_message_pure(),
            )
            .await
            {
                // WebSocket-Nachricht vom Browser empfangen
                Either::First(ws_result) => {
                    let ws_result = ws_result?.ignore_never_b();

                    match ws_result {
                        Ok(ws::Message::Text(data)) => {
                            info!("HTTP: Received text message: {} bytes", data.len());

                            // Parse JSON-Nachricht (konvertiere &str zu &[u8])
                            match serde_json_core::from_slice::<WsClientMessage>(data.as_bytes()) {
                                Ok((msg, _)) => {
                                    use crate::web::protocol::MessageType;

                                    match msg.msg_type {
                                        MessageType::SetColor => {
                                            info!("HTTP: Received set_color command");

                                            if let Some(color) = msg.color {
                                                let color_str = color.as_str();
                                                if let Ok(command) = LedCommand::try_from(color_str)
                                                {
                                                    info!(
                                                        "HTTP: Sending command to LED: {}",
                                                        color_str
                                                    );

                                                    // Sende Command an LED Task (infallible)
                                                    // Der Browser erhält Status-Update automatisch via PubSubChannel,
                                                    // wenn der LED-Task die Farbe geändert hat (Single Source of Truth)
                                                    self.command_sender.send(command).await;
                                                } else {
                                                    info!(
                                                        "HTTP: Unknown color name: {}",
                                                        color_str
                                                    );
                                                }
                                            }
                                        }
                                        MessageType::SetMode => {
                                            info!("HTTP: Received set_mode command");

                                            if let Some(mode) = msg.mode {
                                                if mode == OperationMode::Auto {
                                                    info!("HTTP: Enabling auto mode");
                                                    self.command_sender
                                                        .send(LedCommand::EnableAuto)
                                                        .await;
                                                    info!("HTTP: Auto mode enabled");
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    info!("HTTP: JSON parse error");
                                    // Sende Error-Response
                                    let error = WsServerMessage::Error {
                                        message: "JSON parse error",
                                    };
                                    let mut json_buffer = [0u8; JSON_ERROR_BUFFER_SIZE];
                                    if let Ok(n) =
                                        serde_json_core::to_slice(&error, &mut json_buffer)
                                    {
                                        let json_str =
                                            core::str::from_utf8(&json_buffer[..n]).unwrap();
                                        let _ = tx.send_text(json_str).await;
                                    }
                                }
                            }
                        }
                        Ok(ws::Message::Binary(data)) => {
                            info!(
                                "HTTP: Received binary message: {} bytes (ignored)",
                                data.len()
                            );
                        }
                        Ok(ws::Message::Ping(data)) => {
                            info!("HTTP: Received ping");
                            tx.send_pong(data).await?;
                        }
                        Ok(ws::Message::Pong(_)) => {
                            info!("HTTP: Received pong");
                        }
                        Ok(ws::Message::Close(_reason)) => {
                            info!("HTTP: WebSocket close received");
                            break None;
                        }
                        Err(error) => {
                            info!("HTTP: WebSocket error");
                            break Some((error.code(), "WebSocket Error"));
                        }
                    }
                }
                // LED-Color-Update vom PubSubChannel empfangen
                Either::Second(led_msg) => {
                    let mode = if led_msg.is_auto_mode {
                        OperationMode::Auto
                    } else {
                        OperationMode::Manual
                    };
                    info!(
                        "HTTP: LED color changed to '{}' ({}), notifying client",
                        led_msg.name,
                        if led_msg.is_auto_mode {
                            "Auto"
                        } else {
                            "Manuell"
                        }
                    );
                    Self::send_status_update(&mut tx, &led_msg, mode).await.ok();
                }
            }
        };

        info!("HTTP: WebSocket connection closed");
        tx.close(close_reason).await
    }
}

impl WebSocketHandler {
    /// Sendet Status-Update an WebSocket-Client
    async fn send_status_update<W: embedded_io_async::Write>(
        tx: &mut ws::SocketTx<W>,
        led_msg: &LedColorMessage,
        mode: OperationMode,
    ) -> Result<(), W::Error> {
        let rgb = RgbColor {
            r: led_msg.color.r,
            g: led_msg.color.g,
            b: led_msg.color.b,
        };

        // ColorName aus dem String-Namen erstellen
        let color = match led_msg.name {
            "Rot" => crate::web::protocol::ColorName::Red,
            "Grün" => crate::web::protocol::ColorName::Green,
            "Blau" => crate::web::protocol::ColorName::Blue,
            _ => return Ok(()), // Unbekannte Farbe ignorieren
        };

        let status = WsServerMessage::Status {
            color,
            rgb,
            timestamp_ms: Instant::now().as_millis(),
            mode,
        };

        // Serialisiere und sende
        let mut json_buffer = [0u8; JSON_STATUS_BUFFER_SIZE];
        if let Ok(n) = serde_json_core::to_slice(&status, &mut json_buffer) {
            let json_str = core::str::from_utf8(&json_buffer[..n]).unwrap();
            tx.send_text(json_str).await?;
        }

        Ok(())
    }
}

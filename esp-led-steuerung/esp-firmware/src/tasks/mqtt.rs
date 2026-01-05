// MQTT Task - Published LED-Farben an MQTT Broker
use defmt::{Debug2Format, error, info, warn};
use embassy_net::{IpAddress, Stack, dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::{Duration, Timer, with_timeout};

use rust_mqtt::client::client::MqttClient;
use rust_mqtt::client::client_config::{ClientConfig, MqttVersion};
use rust_mqtt::packet::v5::publish_packet::QualityOfService;
use rust_mqtt::utils::rng_generator::CountingRng;
use rust_mqtt::utils::types::EncodedString;

use crate::LedColorSubscriber;
use crate::config::*;

/// MQTT Task - läuft parallel zu anderen Tasks
///
/// Dieser Task übernimmt das MQTT-Publishing:
/// - Wartet auf Netzwerk-Verbindung
/// - Verbindet sich mit MQTT Broker
/// - Empfängt LED-Farb-Updates via Channel
/// - Published Farbnamen **sofort bei Änderung** (event-basiert)
/// - Automatisches Reconnect bei Fehlern
///
/// # Parameter
/// - `stack`: embassy-net Stack für Netzwerk-Zugriff
/// - `color_subscriber`: PubSub Subscriber für LED-Farb-Broadcasts
#[embassy_executor::task]
pub async fn mqtt_task(stack: &'static Stack<'static>, mut color_subscriber: LedColorSubscriber) {
    info!("MQTT: Task started, waiting for network...");
    wait_for_network(stack).await;
    info!("MQTT: Network ready");

    loop {
        match mqtt_connect_and_publish(stack, &mut color_subscriber).await {
            Ok(_) => warn!("MQTT: Connection closed normally"),
            Err(e) => error!("MQTT: Error: {}", Debug2Format(&e)),
        }
        info!("MQTT: Reconnecting in {}s...", MQTT_RECONNECT_DELAY_SECS);
        Timer::after(Duration::from_secs(MQTT_RECONNECT_DELAY_SECS)).await;
    }
}

/// Wartet bis Netzwerk-Verbindung verfügbar ist
///
/// Prüft kontinuierlich Link-Status und DHCP-Konfiguration.
async fn wait_for_network(stack: &'static Stack<'static>) {
    loop {
        if stack.is_link_up() {
            if let Some(_) = stack.config_v4() {
                break;
            }
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Verbindet mit MQTT Broker und published Farb-Updates
///
/// Diese Funktion übernimmt den kompletten MQTT-Lifecycle:
/// 1. DNS-Auflösung des Broker-Hostnames
/// 2. TCP-Verbindung aufbauen
/// 3. MQTT CONNECT senden
/// 4. Farb-Updates empfangen und periodisch publishen
///
/// Bei jedem Fehler wird die Funktion beendet und der Haupt-Loop
/// startet automatisch einen Reconnect-Versuch.
async fn mqtt_connect_and_publish(
    stack: &'static Stack<'static>,
    color_subscriber: &mut LedColorSubscriber,
) -> Result<(), MqttError> {
    // DNS Lookup
    info!("MQTT: Resolving '{}'...", MQTT_BROKER);
    let broker_ip = resolve_hostname(stack, MQTT_BROKER).await?;
    info!("MQTT: Resolved to {}", Debug2Format(&broker_ip));

    // TCP Connect
    let mut rx_buffer = [0u8; 4096];
    let mut tx_buffer = [0u8; 4096];
    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    socket
        .connect((broker_ip, MQTT_PORT))
        .await
        .map_err(|_| MqttError::ConnectionFailed)?;
    info!("MQTT: TCP connected");

    // MQTT Client Configuration
    let rng = CountingRng(20000);
    let mut config = ClientConfig::<5, _>::new(MqttVersion::MQTTv5, rng);
    config.client_id = EncodedString {
        string: MQTT_CLIENT_ID,
        len: MQTT_CLIENT_ID.len() as u16,
    };
    config.keep_alive = 30;
    config.max_packet_size = MQTT_BUFFER_SIZE as u32;

    // MQTT Buffer
    let mut send_buffer = [0u8; MQTT_BUFFER_SIZE];
    let mut recv_buffer = [0u8; MQTT_BUFFER_SIZE];

    // MQTT Client erstellen
    let mut client = MqttClient::<_, 5, _>::new(
        socket,
        &mut send_buffer,
        MQTT_BUFFER_SIZE,
        &mut recv_buffer,
        MQTT_BUFFER_SIZE,
        config,
    );

    // MQTT CONNECT
    client
        .connect_to_broker()
        .await
        .map_err(|_| MqttError::ProtocolError)?;
    info!("MQTT: Connected to broker");

    // Publish Loop - Event-basiert
    // Wartet blockierend auf neue Farb-Updates und published diese sofort
    loop {
        // Warte auf neue Farbe (blockiert bis Broadcast kommt)
        let msg = color_subscriber.next_message_pure().await;

        let mode_str = if msg.is_auto_mode { "Auto" } else { "Manuell" };
        info!(
            "MQTT: Color changed to '{}' ({}), publishing...",
            msg.name, mode_str
        );

        // Publishe Farbe auf erstes Topic
        client
            .send_message(
                MQTT_TOPIC_COLOR,
                msg.name.as_bytes(),
                QualityOfService::QoS0,
                false,
            )
            .await
            .map_err(|_| MqttError::PublishFailed)?;

        // Publishe Modus auf zweites Topic
        client
            .send_message(
                MQTT_TOPIC_MODE,
                mode_str.as_bytes(),
                QualityOfService::QoS0,
                false,
            )
            .await
            .map_err(|_| MqttError::PublishFailed)?;

        info!("MQTT: Published color='{}' mode='{}'", msg.name, mode_str);
    }
}

/// Löst Hostname zu IPv4-Adresse auf
///
/// Nutzt embassy-net DNS-Stack mit konfigurierbarem Timeout.
async fn resolve_hostname(
    stack: &'static Stack<'static>,
    hostname: &str,
) -> Result<embassy_net::Ipv4Address, MqttError> {
    let result = with_timeout(
        Duration::from_secs(DNS_TIMEOUT_SECS),
        stack.dns_query(hostname, DnsQueryType::A),
    )
    .await;

    match result {
        Ok(Ok(addrs)) => {
            for addr in addrs {
                if let IpAddress::Ipv4(ipv4) = addr {
                    return Ok(ipv4);
                }
            }
            Err(MqttError::DnsResolutionFailed)
        }
        Ok(Err(_)) => Err(MqttError::DnsResolutionFailed),
        Err(_) => Err(MqttError::DnsTimeout),
    }
}

/// MQTT Fehler-Typen
///
/// Alle möglichen Fehler die während MQTT-Operationen auftreten können.
#[derive(Debug)]
enum MqttError {
    DnsResolutionFailed,
    DnsTimeout,
    ConnectionFailed,
    ProtocolError,
    PublishFailed,
}

impl defmt::Format for MqttError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            MqttError::DnsResolutionFailed => defmt::write!(fmt, "DNS failed"),
            MqttError::DnsTimeout => defmt::write!(fmt, "DNS timeout"),
            MqttError::ConnectionFailed => defmt::write!(fmt, "Connection failed"),
            MqttError::ProtocolError => defmt::write!(fmt, "Protocol error"),
            MqttError::PublishFailed => defmt::write!(fmt, "Publish failed"),
        }
    }
}

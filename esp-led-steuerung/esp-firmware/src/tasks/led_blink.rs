// LED Blink Task - Steuert RGB LED über RMT Peripheral
use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal_smartled::smart_led_buffer;
use rgb::RGB8;

use crate::config::{BLINK_INTERVAL_SECS, LED_BRIGHTNESS, RMT_CLOCK_MHZ};
use crate::hal::{RmtLedWriter, SmartLedWriter};
use crate::{LedColorMessage, LedColorPublisher, LedCommand, LedCommandReceiver, rotate_color};

/// LED Blink Logic - Testbare Business Logic ohne Hardware-Abhängigkeit
///
/// Diese Funktion enthält die komplette LED-Steuerungs-Logik:
/// - Rotiert Farben automatisch (Rot → Blau → Grün) oder
/// - Empfängt manuelle Farb-Kommandos vom WebSocket
/// - Blinkt mit konfigurierbarem Intervall
/// - Sendet Farb-Updates an MQTT und HTTP Tasks via Channel
///
/// # Trait-basierte Abstraktion
/// Der generische Parameter `L: SmartLedWriter` ermöglicht:
/// - Real Hardware (RmtLedWriter) im Production-Code
/// - Mock Implementation (MockLedWriter) in Unit Tests
///
/// # Parameter
/// - `led`: LED Writer (Hardware oder Mock)
/// - `color_publisher`: PubSub Publisher für LED-Farb-Broadcasts
/// - `command_receiver`: Channel Receiver für WebSocket-Kommandos
pub async fn led_blink_logic<L: SmartLedWriter>(
    mut led: L,
    color_publisher: LedColorPublisher,
    command_receiver: LedCommandReceiver,
) {
    // Farbe initialisieren: starte mit Rot
    let mut color: RGB8 = RGB8::default();
    color.r = LED_BRIGHTNESS;

    // Modus-Flag: automatische Rotation vs. manuelle Steuerung
    let mut auto_rotate = true;

    // Hauptschleife: blinkt LED endlos
    loop {
        let mut color_changed = false;

        // Prüfe auf eingehende Kommandos vom WebSocket (non-blocking)
        if let Ok(cmd) = command_receiver.try_receive() {
            match cmd {
                LedCommand::SetColor { target_color, name } => {
                    info!("Command received: SetColor {}", name);
                    color = target_color;
                    auto_rotate = false; // Wechsel zu manueller Steuerung
                    color_changed = true; // Farbe hat sich geändert
                }
                LedCommand::EnableAuto => {
                    info!("Command received: EnableAuto");
                    auto_rotate = true; // Wechsel zu Auto-Rotation
                    // Keine Farb-Änderung, nur Modus-Wechsel
                }
            }
        }

        // Farb-Rotation nur im Auto-Modus
        if auto_rotate {
            color = rotate_color(color);
            color_changed = true; // Farbe hat sich geändert
        }

        info!("Blink!");

        // Farbe an LED senden (via Trait - Hardware oder Mock)
        if let Err(_e) = led.write(color) {
            error!("Failed to write to LED");
        }

        // Nur publishen wenn sich Farbe geändert hat
        if color_changed {
            let msg = LedColorMessage::from_color(color, auto_rotate);
            color_publisher.publish_immediate(msg); // Broadcast an alle Subscribers
            info!(
                "Published color update: {} ({})",
                msg.name,
                if auto_rotate { "Auto" } else { "Manuell" }
            );
        }

        // Async Delay: gibt CPU an andere Tasks zurück
        Timer::after(Duration::from_secs(BLINK_INTERVAL_SECS)).await;
    }
}

/// LED Blink Task - Embassy Task für parallele Ausführung
///
/// Dieser Task übernimmt die Hardware-Initialisierung und ruft dann
/// die testbare `led_blink_logic()` Funktion auf.
///
/// # Parameter
/// - `gpio8`: GPIO8 Peripheral für LED-Datenleitung
/// - `rmt_peripheral`: RMT Peripheral für präzises Timing
/// - `color_publisher`: PubSub Publisher für LED-Farb-Broadcasts
/// - `command_receiver`: Channel Receiver für WebSocket-Kommandos
#[embassy_executor::task]
pub async fn led_blink_task(
    gpio8: esp_hal::peripherals::GPIO8<'static>,
    rmt_peripheral: esp_hal::peripherals::RMT<'static>,
    color_publisher: LedColorPublisher,
    command_receiver: LedCommandReceiver,
) {
    // Buffer für SmartLED Daten erstellen (1 LED)
    // Macro allokiert Speicher im richtigen Format für RMT
    let mut rmt_buffer = smart_led_buffer!(1);

    // Hardware initialisieren: RmtLedWriter kapselt RMT + SmartLED
    let led = RmtLedWriter::new(gpio8, rmt_peripheral, RMT_CLOCK_MHZ, &mut rmt_buffer);

    // Business Logic aufrufen (jetzt testbar!)
    led_blink_logic(led, color_publisher, command_receiver).await;
}

// Library-Root: Wiederverwendbare Logik und Module
// Keine Standard-Bibliothek (Embedded System)
#![no_std]

// Module
pub mod config;
pub mod hal;
pub mod tasks;
pub mod web;

// Re-exports von esp-core
pub use esp_core::{LedColorMessage, LedCommand, LedError, SmartLedWriter, rotate_color};

// RGB Farb-Typ (direkt von rgb crate)
use rgb::RGB8;

// Embassy Channel-Typen
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};

// Konfigurationswerte
use crate::config::LED_BRIGHTNESS;

// ============================================================================
// Firmware-spezifische Implementierungen
// ============================================================================
//
// defmt::Format Implementations wurden nach esp-core verschoben (optional feature)

/// Helper-Funktion: Erstellt LedCommand mit Firmware-spezifischem LED_BRIGHTNESS
///
/// Dies überschreibt die Default-Implementierung aus esp-core die DEFAULT_BRIGHTNESS = 10 nutzt.
pub fn led_command_from_name(name: &str) -> Result<LedCommand, ()> {
    match name {
        "Rot" => Ok(LedCommand::SetColor {
            target_color: RGB8 {
                r: LED_BRIGHTNESS,
                g: 0,
                b: 0,
            },
            name: "Rot",
        }),
        "Grün" => Ok(LedCommand::SetColor {
            target_color: RGB8 {
                r: 0,
                g: LED_BRIGHTNESS,
                b: 0,
            },
            name: "Grün",
        }),
        "Blau" => Ok(LedCommand::SetColor {
            target_color: RGB8 {
                r: 0,
                g: 0,
                b: LED_BRIGHTNESS,
            },
            name: "Blau",
        }),
        _ => Err(()),
    }
}

// ============================================================================
// Type-Aliase für Channel-Typen
// ============================================================================
//
// Diese Type-Aliase vereinfachen die Lesbarkeit der Funktionssignaturen.
// Statt:  Publisher<'static, NoopRawMutex, LedColorMessage, 2, 10, 1>
// Nutze:  LedColorPublisher

/// PubSubChannel für LED-Farb-Broadcasts
/// - 2: Nachrichten-Kapazität im Queue
/// - 10: Maximale Anzahl Subscribers (1 MQTT + bis zu 9 WebSockets)
/// - 1: Publish WaitResult Slots
pub type LedColorChannel = PubSubChannel<NoopRawMutex, LedColorMessage, 2, 10, 1>;

/// Publisher für LED-Farb-Broadcasts
/// Erzeugt aus LedColorChannel
pub type LedColorPublisher = Publisher<'static, NoopRawMutex, LedColorMessage, 2, 10, 1>;

/// Subscriber für LED-Farb-Broadcasts
/// Empfängt Broadcasts von LedColorPublisher
pub type LedColorSubscriber = Subscriber<'static, NoopRawMutex, LedColorMessage, 2, 10, 1>;

/// Channel für LED-Kommandos (WebSocket → LED Task)
/// - 1: Nachrichten-Kapazität (nur ein Command zur Zeit)
pub type LedCommandChannel = embassy_sync::channel::Channel<NoopRawMutex, LedCommand, 1>;

/// Sender für LED-Kommandos (WebSocket → LED Task)
/// Erzeugt aus LedCommandChannel
pub type LedCommandSender = Sender<'static, NoopRawMutex, LedCommand, 1>;

/// Receiver für LED-Kommandos (LED Task empfängt)
/// Empfängt Commands von LedCommandSender
pub type LedCommandReceiver = Receiver<'static, NoopRawMutex, LedCommand, 1>;

// ============================================================================
// Testing-Strategie für Embedded no_std Crates
// ============================================================================
//
// Problem: Standard Rust Unit Tests (#[test]) funktionieren nicht, weil:
//
// 1. **no_std Environment:**
//    - Dieses Crate nutzt #![no_std] für Embedded-Systeme
//    - Das Rust test Framework benötigt std (nicht verfügbar)
//
// 2. **Platform-spezifische Dependencies:**
//    - esp-hal, embassy-sync, etc. kompilieren nur für riscv32imac-unknown-none-elf
//    - Cargo test versucht für Host-Target (x86_64) zu kompilieren → schlägt fehl
//
// 3. **Keine echten "Pure Functions":**
//    - Selbst rotate_color() nutzt RGB8 (von rgb crate)
//    - LedCommand nutzt LED_BRIGHTNESS aus config.rs
//    - Type-Aliase nutzen embassy-sync Types
//
// Lösungsansätze:
//
// **Option A: defmt-test (Target Tests auf ESP32 Hardware)**
//    - Tests laufen auf dem ESP32 selbst
//    - Komplex, braucht Hardware-Setup
//    - User hat "keine Target-Tests" gewünscht ❌
//
// **Option B: Refactoring für Testbarkeit (Option 2 aus Plan)**
//    - Trait-Abstraktionen für Hardware (SmartLedWriter)
//    - Logic-Funktionen ohne Hardware-Dependencies
//    - Mock-Implementierungen für Tests
//    - Benötigt Refactoring der Tasks ✅
//
// **Option C: Manuelle Verifikation**
//    - Code-Review der Logik
//    - Funktionstest auf Hardware
//    - Keine automatisierten Tests ⚠️
//
// **Empfehlung:**
// Für automatisierte Tests ist Option B (Refactoring) notwendig.
// Die aktuelle Code-Struktur ist zu eng mit Hardware gekoppelt für pure Unit Tests.
//
// **Nächste Schritte:**
// Um testbar zu machen, müssten wir:
// 1. `rotate_color()` als standalone pure function (✅ schon der Fall)
// 2. LED-Logic von Hardware trennen (braucht Trait-Abstraktion)
// 3. Separate Testable-Logic-Funktionen erstellen
//
// Siehe Plan-File: /home/chefkoch/.claude/plans/giggly-marinating-coral.md

//! Hardware Abstraction Traits
//!
//! Diese Traits definieren Schnittstellen für Hardware-Zugriff
//! ohne konkrete Implementierung.

use rgb::RGB8;

/// Fehler-Typ für LED-Operationen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedError {
    WriteFailed,
}

/// Trait für SmartLED Hardware-Zugriff
///
/// Abstrahiert den Zugriff auf RGB LEDs (WS2812/Neopixel).
///
/// # Implementierungen
/// - **Production:** RmtLedWriter (ESP32 RMT Peripheral)
/// - **Testing:** MockLedWriter (in-memory Mock)
pub trait SmartLedWriter: Send {
    /// Schreibt eine RGB-Farbe auf die LED
    ///
    /// # Fehlerbehandlung
    /// Gibt `LedError::WriteFailed` zurück wenn Hardware-Zugriff fehlschlägt
    fn write(&mut self, color: RGB8) -> Result<(), LedError>;
}

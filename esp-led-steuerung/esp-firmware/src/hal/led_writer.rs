// SmartLED Writer Trait und Implementierungen
//
// Abstrahiert den Zugriff auf RGB LEDs (WS2812/Neopixel)
// um Tests mit Mock-Implementierungen zu ermöglichen.

use rgb::RGB8;

/// Fehler-Typ für LED-Schreiboperationen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedError {
    WriteFailed,
}

/// Trait für LED-Writer
///
/// Abstrahiert den Zugriff auf SmartLEDs (WS2812/Neopixel).
/// Ermöglicht Mock-Implementierungen für Tests.
pub trait SmartLedWriter: Send {
    /// Schreibt eine RGB-Farbe auf die LED
    ///
    /// # Fehlerbehandlung
    /// Gibt LedError::WriteFailed zurück wenn Hardware-Zugriff fehlschlägt
    fn write(&mut self, color: RGB8) -> Result<(), LedError>;
}

// ============================================================================
// Real Hardware Implementation (nur für ESP32-Target)
// ============================================================================

#[cfg(not(test))]
mod real_impl {
    use super::*;
    use esp_hal::Blocking;
    use esp_hal::rmt::Rmt;
    use esp_hal::time::Rate;
    use esp_hal_smartled::SmartLedsAdapter;
    use smart_leds_trait::SmartLedsWrite;

    // Buffer-Größe für 1 LED (3 Farben * 8 Bits + 1 Reset)
    const LED_BUFFER_SIZE: usize = 25;

    /// Real Hardware LED Writer
    ///
    /// Nutzt ESP32 RMT Peripheral um WS2812 LEDs anzusteuern.
    ///
    /// Hinweis: Der Buffer muss 'static sein, daher wird er im Task erstellt
    /// und als Parameter übergeben statt im Constructor allokiert.
    pub struct RmtLedWriter<'a> {
        led: SmartLedsAdapter<'a, LED_BUFFER_SIZE>,
    }

    impl<'a> RmtLedWriter<'a> {
        /// Erstellt einen neuen RmtLedWriter
        ///
        /// # Parameter
        /// - `gpio8`: GPIO8 Peripheral für LED-Datenleitung
        /// - `rmt_peripheral`: RMT Peripheral
        /// - `rmt_clock_mhz`: RMT Clock Frequenz in MHz (z.B. 80)
        /// - `buffer`: Buffer für LED-Daten (erstellt mit smart_led_buffer!(1) Macro)
        pub fn new(
            gpio8: esp_hal::peripherals::GPIO8<'a>,
            rmt_peripheral: esp_hal::peripherals::RMT<'a>,
            rmt_clock_mhz: u32,
            buffer: &'a mut [esp_hal::rmt::PulseCode; LED_BUFFER_SIZE],
        ) -> Self {
            // RMT initialisieren
            let rmt: Rmt<'a, Blocking> =
                Rmt::new(rmt_peripheral, Rate::from_mhz(rmt_clock_mhz)).unwrap();

            // SmartLED Adapter erstellen
            let led = SmartLedsAdapter::new(rmt.channel0, gpio8, buffer);

            Self { led }
        }
    }

    impl<'a> SmartLedWriter for RmtLedWriter<'a> {
        fn write(&mut self, color: RGB8) -> Result<(), LedError> {
            self.led
                .write([color].into_iter())
                .map_err(|_| LedError::WriteFailed)
        }
    }
}

#[cfg(not(test))]
pub use real_impl::RmtLedWriter;

// ============================================================================
// Mock Implementation (nur für Tests)
// ============================================================================

#[cfg(test)]
pub struct MockLedWriter {
    /// Zuletzt geschriebene Farbe (für Assertions in Tests)
    pub last_color: Option<RGB8>,
    /// Anzahl der write() Aufrufe
    pub write_count: usize,
    /// Simuliere Fehler beim nächsten write()
    pub fail_next_write: bool,
}

#[cfg(test)]
impl MockLedWriter {
    pub fn new() -> Self {
        Self {
            last_color: None,
            write_count: 0,
            fail_next_write: false,
        }
    }
}

#[cfg(test)]
impl SmartLedWriter for MockLedWriter {
    fn write(&mut self, color: RGB8) -> Result<(), LedError> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(LedError::WriteFailed);
        }

        self.last_color = Some(color);
        self.write_count += 1;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_led_writer_write() {
        let mut mock = MockLedWriter::new();
        let color = RGB8 { r: 10, g: 0, b: 0 };

        assert_eq!(mock.write_count, 0);
        assert_eq!(mock.last_color, None);

        mock.write(color).unwrap();

        assert_eq!(mock.write_count, 1);
        assert_eq!(mock.last_color, Some(color));
    }

    #[test]
    fn test_mock_led_writer_multiple_writes() {
        let mut mock = MockLedWriter::new();

        mock.write(RGB8 { r: 10, g: 0, b: 0 }).unwrap();
        mock.write(RGB8 { r: 0, g: 10, b: 0 }).unwrap();
        mock.write(RGB8 { r: 0, g: 0, b: 10 }).unwrap();

        assert_eq!(mock.write_count, 3);
        assert_eq!(mock.last_color, Some(RGB8 { r: 0, g: 0, b: 10 }));
    }

    #[test]
    fn test_mock_led_writer_fail() {
        let mut mock = MockLedWriter::new();
        mock.fail_next_write = true;

        let result = mock.write(RGB8 { r: 10, g: 0, b: 0 });
        assert_eq!(result, Err(LedError::WriteFailed));
        assert_eq!(mock.write_count, 0);
        assert_eq!(mock.last_color, None);
    }
}

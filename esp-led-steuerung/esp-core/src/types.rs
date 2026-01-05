//! Core Types für LED-Steuerung
//!
//! Datenstrukturen ohne Hardware-Dependencies

use rgb::RGB8;

/// LED Color Message für Channel-Kommunikation
///
/// Wird zwischen LED-Task und anderen Tasks ausgetauscht.
#[derive(Clone, Copy)]
pub struct LedColorMessage {
    pub color: RGB8,
    pub name: &'static str,
    pub is_auto_mode: bool,
}

impl LedColorMessage {
    /// Erstellt eine LedColorMessage aus einer RGB8-Farbe und Modus
    ///
    /// Die Funktion erkennt automatisch die Farbe basierend auf RGB-Werten.
    pub fn from_color(color: RGB8, is_auto_mode: bool) -> Self {
        let name = match (color.r, color.g, color.b) {
            (r, 0, 0) if r > 0 => "Rot",
            (0, g, 0) if g > 0 => "Grün",
            (0, 0, b) if b > 0 => "Blau",
            _ => "Unbekannt",
        };
        Self {
            color,
            name,
            is_auto_mode,
        }
    }
}

/// LED Command für manuelle Steuerung
///
/// Wird vom WebSocket an den LED-Task gesendet.
#[derive(Clone, Copy)]
pub enum LedCommand {
    /// Setze LED auf eine spezifische Farbe (manueller Modus)
    SetColor {
        target_color: RGB8,
        name: &'static str,
    },
    /// Aktiviere Auto-Rotation
    EnableAuto,
}

impl core::convert::TryFrom<&str> for LedCommand {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        // Brightness constant - in esp-core haben wir keinen Zugriff auf config.rs
        // Daher nutzen wir einen Default-Wert
        const DEFAULT_BRIGHTNESS: u8 = 10;

        match name {
            "Rot" => Ok(Self::SetColor {
                target_color: RGB8 {
                    r: DEFAULT_BRIGHTNESS,
                    g: 0,
                    b: 0,
                },
                name: "Rot",
            }),
            "Grün" => Ok(Self::SetColor {
                target_color: RGB8 {
                    r: 0,
                    g: DEFAULT_BRIGHTNESS,
                    b: 0,
                },
                name: "Grün",
            }),
            "Blau" => Ok(Self::SetColor {
                target_color: RGB8 {
                    r: 0,
                    g: 0,
                    b: DEFAULT_BRIGHTNESS,
                },
                name: "Blau",
            }),
            _ => Err(()),
        }
    }
}

// ============================================================================
// defmt::Format Implementations (optional feature)
// ============================================================================

#[cfg(feature = "defmt")]
impl defmt::Format for LedColorMessage {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "LedColorMessage {{ name: {}, rgb: ({}, {}, {}), auto: {} }}",
            self.name,
            self.color.r,
            self.color.g,
            self.color.b,
            self.is_auto_mode
        )
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for LedCommand {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            LedCommand::SetColor { target_color, name } => {
                defmt::write!(
                    fmt,
                    "SetColor {{ name: {}, rgb: ({}, {}, {}) }}",
                    name,
                    target_color.r,
                    target_color.g,
                    target_color.b
                )
            }
            LedCommand::EnableAuto => {
                defmt::write!(fmt, "EnableAuto")
            }
        }
    }
}

// WebSocket-Protokoll-Definitionen
// Definiert die JSON-Nachrichten für Client ↔ Server Kommunikation

use serde::{Deserialize, Serialize};

/// Farbname-Enum für die drei unterstützten LED-Farben
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, defmt::Format)]
pub enum ColorName {
    #[serde(rename = "Rot")]
    Red,
    #[serde(rename = "Grün")]
    Green,
    #[serde(rename = "Blau")]
    Blue,
}

/// RGB-Struct für JSON-Serialisierung
/// Repräsentiert eine Farbe mit r, g, b Werten (0-255)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Client → Server Nachrichten
/// Kommandos vom Browser an den ESP32
///
/// Hinweis: Verwendet einfache untagged enum Struktur für serde-json-core Kompatibilität
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct WsClientMessage {
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<OperationMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    SetColor,
    SetMode,
}

/// Server → Client Nachrichten
/// Status-Updates und Fehler vom ESP32 an den Browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum WsServerMessage {
    #[serde(rename = "status")]
    Status {
        color: ColorName,
        rgb: RgbColor,
        timestamp_ms: u64,
        mode: OperationMode,
    },
    #[serde(rename = "error")]
    Error { message: &'static str },
}

/// Betriebs-Modus der LED
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationMode {
    Auto,   // Automatische Farb-Rotation
    Manual, // Manuelle Steuerung vom Browser
}

impl ColorName {
    /// Konvertiert ColorName zu einem deutschen String
    pub fn as_str(self) -> &'static str {
        match self {
            ColorName::Red => "Rot",
            ColorName::Green => "Grün",
            ColorName::Blue => "Blau",
        }
    }
}

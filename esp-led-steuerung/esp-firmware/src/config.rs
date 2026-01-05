// Projekt-Konfiguration: Konstanten und Hardware-Zuordnungen
#![allow(dead_code)]

// ============================================================================
// LED Konfiguration
// ============================================================================

/// GPIO-Pin für die RGB LED (WS2812/Neopixel)
pub const LED_GPIO_PIN: u8 = 8;

/// Helligkeits-Level für die LED (0-255)
/// Wert ist gedimmt für Augenschonung
pub const LED_BRIGHTNESS: u8 = 10;

/// RMT Taktfrequenz in MHz
/// 80 MHz ist optimal für WS2812 LED-Timing
pub const RMT_CLOCK_MHZ: u32 = 80;

/// Anzahl der LEDs im Strip
pub const LED_COUNT: usize = 1;

/// Blink-Intervall in Sekunden
pub const BLINK_INTERVAL_SECS: u64 = 1;

// ============================================================================
// WiFi Konfiguration
// ============================================================================

/// WiFi SSID (Netzwerk-Name)
/// Wird zur Build-Zeit aus der Environment Variable WIFI_SSID geladen
/// Setze diese in .env file (siehe .env.example)
pub const WIFI_SSID: &str = env!(
    "WIFI_SSID",
    "WiFi SSID nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// WiFi Passwort
/// Wird zur Build-Zeit aus der Environment Variable WIFI_PASSWORD geladen
/// Setze diese in .env file (siehe .env.example)
pub const WIFI_PASSWORD: &str = env!(
    "WIFI_PASSWORD",
    "WiFi Password nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// Heap-Größe für WiFi (Bytes)
/// WiFi benötigt dynamischen Speicher für Pakete
pub const WIFI_HEAP_SIZE: usize = 65536; // 64 KB

/// Zusätzliche Heap-Größe (Bytes)
pub const EXTRA_HEAP_SIZE: usize = 36864; // 36 KB

// Gesamt-Heap: ~100 KB für WiFi-Stack

// ============================================================================
// MQTT Konfiguration
// ============================================================================

/// MQTT Broker Hostname oder IP-Adresse
/// Wird zur Build-Zeit aus der Environment Variable MQTT_BROKER geladen
/// Setze diese in .env file (siehe .env.example)
pub const MQTT_BROKER: &str = env!(
    "MQTT_BROKER",
    "MQTT Broker nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// MQTT Broker Port
/// Standard: 1883 (unverschlüsselt), 8883 (TLS)
/// Kann in .env überschrieben werden, falls nötig
pub const MQTT_PORT: u16 = 1883;

/// MQTT Client ID
/// Eindeutige Kennung für diesen ESP32-C6
/// Wird zur Build-Zeit aus der Environment Variable MQTT_CLIENT_ID geladen
/// Setze diese in .env file (siehe .env.example)
pub const MQTT_CLIENT_ID: &str = env!(
    "MQTT_CLIENT_ID",
    "MQTT Client ID nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// MQTT Publish Topic für LED-Farbe
/// Topic für LED-Farb-Updates (z.B. "Rot", "Grün", "Blau")
/// Wird zur Build-Zeit aus der Environment Variable MQTT_TOPIC_COLOR geladen
/// Setze diese in .env file (siehe .env.example)
pub const MQTT_TOPIC_COLOR: &str = env!(
    "MQTT_TOPIC_COLOR",
    "MQTT Topic Color nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// MQTT Publish Topic für LED-Modus
/// Topic für LED-Modus-Updates (z.B. "Auto", "Manuell")
/// Wird zur Build-Zeit aus der Environment Variable MQTT_TOPIC_MODE geladen
/// Setze diese in .env file (siehe .env.example)
pub const MQTT_TOPIC_MODE: &str = env!(
    "MQTT_TOPIC_MODE",
    "MQTT Topic Mode nicht gesetzt! Erstelle .env file (siehe .env.example)"
);

/// MQTT Reconnect Delay in Sekunden
/// Wartezeit nach Verbindungsfehler vor erneutem Versuch
pub const MQTT_RECONNECT_DELAY_SECS: u64 = 5;

/// MQTT Buffer-Größe in Bytes
/// Muss groß genug für MQTT-Pakete sein
pub const MQTT_BUFFER_SIZE: usize = 1024;

/// DNS Query Timeout in Sekunden
pub const DNS_TIMEOUT_SECS: u64 = 10;

// ============================================================================
// mDNS-Konfiguration
// ============================================================================

/// mDNS Hostname (ohne .local suffix)
/// Der ESP32 wird erreichbar sein unter: <MDNS_HOSTNAME>.local
pub const MDNS_HOSTNAME: &str = "led";

/// mDNS TTL (Time To Live) in Sekunden
/// Gibt an, wie lange andere Geräte die mDNS-Antwort cachen dürfen
pub const MDNS_TTL_SECS: u32 = 120;

/// mDNS Reconnect Delay in Sekunden
/// Wartezeit nach Fehler vor erneutem Versuch
pub const MDNS_RECONNECT_DELAY_SECS: u64 = 5;

/// mDNS Port (Standard: 5353)
/// Multicast DNS nutzt Port 5353 laut RFC 6762
pub const MDNS_PORT: u16 = 5353;

/// mDNS IPv4 Multicast-Adresse (224.0.0.251)
/// Standard mDNS Multicast-Gruppe laut RFC 6762
pub const MDNS_MULTICAST_ADDR: [u8; 4] = [224, 0, 0, 251];

/// UDP Buffer-Größen für mDNS (TX, RX in Bytes)
/// edge-nal-embassy benötigt Buffer für UDP-Pakete
pub const MDNS_UDP_BUFFER_SIZE: usize = 512;

/// mDNS Receive/Send Buffer-Größen in Bytes
/// 1500 Bytes = Standard MTU für Ethernet/WiFi
pub const MDNS_PACKET_BUFFER_SIZE: usize = 1500;

// ============================================================================
// HTTP Server Konfiguration
// ============================================================================

/// HTTP Buffer-Größe in Bytes
/// Für HTTP Request/Response Headers und Body
/// 1024 Bytes reicht dank Chunked Transfer Encoding (HTML ist 8 KB, wird in Chunks gesendet)
pub const HTTP_BUFFER_SIZE: usize = 1024;

/// TCP RX Buffer-Größe in Bytes
/// Für eingehende TCP-Daten vom Client
pub const TCP_RX_BUFFER_SIZE: usize = 1024;

/// TCP TX Buffer-Größe in Bytes
/// Für ausgehende TCP-Daten zum Client
pub const TCP_TX_BUFFER_SIZE: usize = 1024;

/// WebSocket Message Buffer-Größe in Bytes
/// Für eingehende WebSocket-Nachrichten vom Browser
/// 512 Bytes reicht für JSON-Messages (< 256 Bytes)
pub const WEBSOCKET_BUFFER_SIZE: usize = 512;

/// JSON Serialisierungs-Buffer für WebSocket Status-Updates
/// Für {"type":"status","color":"Rot","rgb":{...},"timestamp_ms":...,"mode":"auto"}
pub const JSON_STATUS_BUFFER_SIZE: usize = 256;

/// JSON Serialisierungs-Buffer für WebSocket Error-Messages
/// Für {"type":"error","message":"..."}
pub const JSON_ERROR_BUFFER_SIZE: usize = 128;

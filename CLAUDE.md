# ESP32-C6 LED Controller - Entwicklungsnotizen

Embedded Rust Projekt für ESP32-C6 mit WiFi, MQTT, HTTP/WebSocket und umfangreicher Test-Infrastruktur.

## Projekt-Übersicht

**Features:**
- RGB LED Steuerung (WS2812) mit Auto-Rotation und manueller Steuerung
- WiFi 6 Konnektivität mit DHCP und DNS
- MQTT v5 Publishing (dual topics: led-color + led-mode)
- HTTP Server mit WebSocket für Browser-Steuerung
- mDNS Responder für einfache Geräteerkennung
- Embassy Async Runtime (7 parallele Tasks)
- Moderne Web UI mit Pico.css + Alpine.js
- Umfassende Tests (97% Coverage, 19 Tests)

**Technologie-Stack:**
- **Hardware:** ESP32-C6 RISC-V @ 160MHz, 8MB Flash, 512KB RAM
- **Sprache:** Rust (no_std, embedded, Edition 2024)
- **Runtime:** Embassy Async
- **Networking:** WiFi 6, embassy-net, smoltcp
- **Protokolle:** MQTT v5 (rust-mqtt 0.3.0), HTTP/WebSocket (picoserve 0.17.1)
- **LED:** WS2812 via RMT Peripheral
- **Logging:** defmt (binary logging)
- **Testing:** cargo-llvm-cov

**Version:** 1.0.0

## Quick Start

```bash
# 1. .env File erstellen
cd esp-led-steuerung/esp-firmware
cp .env.example .env
# .env editieren: WIFI_SSID, WIFI_PASSWORD, MQTT_BROKER eintragen

# 2. Bauen und Flashen
cargo run --release

# 3. Nutzung
# → LED rotiert automatisch: Rot → Blau → Grün
# → WiFi verbindet sich
# → MQTT published auf: devices/esp32c6/led-color, devices/esp32c6/led-mode
# → HTTP Server auf Port 80
# → mDNS: esp32c6.local
# → Browser: http://esp32c6.local/ oder http://<ESP32-IP>/
```

**Wichtig:** Bei .env Änderungen: `cargo clean && cargo run --release`

## Hardware

**Board:** nanoESP32-C6 V1.0 (ESP32-C6-DevKitM-1 kompatibel)

**Chip:** ESP32-C6 RISC-V 32-bit @ 160MHz
- Flash: 8 MB
- RAM: 512 KB SRAM
- Features: WiFi 6, Bluetooth 5.3 LE, IEEE 802.15.4

**USB-Ports:**
1. **CH343 Port** - Empfohlen für Flashen und Serial Monitor (`/dev/ttyACM0`)
2. **ESP32C6 Port** - Für JTAG-Debugging

## Architektur

### Workspace-Struktur (3 Crates)

```
esp-led-steuerung/
├── esp-core/           # Platform-agnostic (keine ESP-Deps)
│   ├── traits.rs       # SmartLedWriter Trait
│   ├── types.rs        # LedColorMessage, LedCommand
│   └── logic.rs        # rotate_color() + Tests
├── esp-firmware/       # ESP32 Hardware Implementation
│   ├── hal/            # RmtLedWriter
│   ├── tasks/          # WiFi, MQTT, HTTP, LED
│   ├── web/            # HTML + WebSocket Protocol
│   └── config.rs       # WiFi, MQTT, Buffer-Größen
└── esp-tests/          # Integration Tests (x86_64)
    └── tests/          # MockLedWriter + 15 Tests
```

### Task-Struktur (7 Tasks)

1. `led_blink_task` - LED steuern (Auto/Manuell)
2. `connection_task` - WiFi Connect/Reconnect
3. `net_task` - embassy-net Stack Runner
4. `dhcp_task` - DHCP Client
5. `mqtt_task` - MQTT Publishing
6. `mdns_responder_task` - mDNS Responder
7. `http_server_task` ×4 - HTTP/WebSocket Pool

### Kommunikation

**PubSubChannel** (1→N Broadcast):
- LED Task → MQTT Task + HTTP Tasks
- Alle Subscriber erhalten LED-Updates

**Command Channel** (N→1):
- WebSocket → LED Task
- Single Source of Truth

### Trait-basierte Abstraktion

```rust
// Trait Definition (esp-core)
pub trait SmartLedWriter: Send {
    fn write(&mut self, color: RGB8) -> Result<(), LedError>;
}

// Hardware Implementation (esp-firmware)
pub struct RmtLedWriter<'a> { ... }

// Mock Implementation (esp-tests)
pub struct MockLedWriter { ... }
```

## Testing

**Status:** 97% Coverage, 19 Tests

```bash
# Tests ausführen
cargo test

# Coverage-Report
cargo llvm-cov --package esp-core --package esp-tests --html
```

**Details:** Siehe [TESTING.md](esp-led-steuerung/TESTING.md)

## CI/CD

**Status:** [![CI](https://github.com/legacycode-labs/ESP32C6-Rust-LED-Steuerung/actions/workflows/ci.yml/badge.svg)](https://github.com/legacycode-labs/ESP32C6-Rust-LED-Steuerung/actions)

**Static Analysis Tools:**
- **CodeQL:** Security-Analyse (GitHub default setup, läuft automatisch)
- **clippy:** Code-Qualität + Embedded-Lints (stack-size-threshold: 1024)
- **cargo-audit:** Dependency-Schwachstellen via RustSec Database
- **cargo test:** 97% Coverage, 19 Tests

**Workflow:**
- Läuft bei Push auf main und Pull Requests
- 3 parallele Jobs: test, clippy, audit
- Analysiert nur esp-core + esp-tests (esp-firmware excluded)
- Caching für schnellere Builds (~1-2 Minuten)

**Dependabot:**
- Wöchentliche Dependency-Updates für esp-core und esp-tests
- Automatische PRs bei verfügbaren Updates

## Build & Deploy

### Entwicklung

```bash
cd esp-led-steuerung/esp-firmware
cargo check          # Code prüfen
cargo clippy         # Linter
cargo fmt            # Formatierung
cargo build --release # Build
cargo run --release  # Build + Flash + Monitor
```

### Konfiguration

**Build-Profile:**
- opt-level = "s" (Größe)
- LTO = 'fat'
- build-std = ["core"]

**Binary-Größe:** ~29 KB (0.35% von 8 MB Flash)

### WiFi & MQTT Config

Credentials werden zur Build-Zeit via `.env` eingebettet:

```bash
WIFI_SSID=dein-wifi
WIFI_PASSWORD=dein-passwort
MQTT_BROKER=mqtt.home
MQTT_PORT=1883
MQTT_CLIENT_ID=esp32c6-led-publisher
MQTT_TOPIC_COLOR=devices/esp32c6/led-color
MQTT_TOPIC_MODE=devices/esp32c6/led-mode
```

## Troubleshooting

**Serial Monitor zeigt Binär-Daten:**
- defmt sendet binäre Logs
- Lösung: `cargo run --release` (macht automatische Dekodierung)

**USB-Port nicht gefunden (WSL2):**
```bash
# Windows PowerShell (Administrator):
usbipd list
usbipd bind --busid <BUSID>
usbipd attach --wsl --busid <BUSID>
```

**.env Änderungen werden ignoriert:**
- `cargo clean && cargo run --release`

**MQTT-Verbindung fehlschlägt:**
- DNS-Auflösung prüfen
- Broker erreichbar?
- ESP32 hat IP via DHCP?

## Key Learnings

**Build & Tooling:**
- espflash reicht für Flashen (kein probe-rs nötig)
- defmt benötigt `--elf` oder `cargo run` für Dekodierung
- .env Änderungen triggern kein Auto-Rebuild

**Embedded Rust:**
- Workspace-Struktur für testbaren no_std Code
- Trait-Abstraktion statt Repository Pattern
- embassy-futures::select für Event-basierte Tasks
- PubSubChannel für Broadcasts, Channel für Commands

**ESP32-spezifisch:**
- rust-mqtt 0.3.0 kompatibel mit embassy-net 0.7.1
- NoopRawMutex für Single-Core ESP32-C6
- Task Pool (`pool_size = 4`) für concurrent Connections
- defmt::Format als optional feature in shared crates

## Externe Links

- [ESP32-C6 Docs](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitm-1/)
- [esp-hal Examples](https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples)
- [Embassy Book](https://embassy.dev/book/)
- [GitHub Repository](https://github.com/legacycode-labs/ESP32C6-Rust-LED-Steuerung)

# esp-led-steuerung - ESP32-C6 LED Controller

ESP32-C6 Embedded Rust Projekt mit WiFi, MQTT und WebSocket-Steuerung einer RGB LED.

## 🎯 Was macht das Projekt?

Ein ESP32-C6 Mikrocontroller steuert eine RGB LED bidirektional:
- **Auto-Modus:** LED rotiert automatisch durch Farben (Rot → Blau → Grün)
- **Manuell-Modus:** LED steuerbar über Browser-WebSocket-Interface
- **MQTT Publishing:** Status wird an MQTT-Broker gesendet (Farbe + Modus)
- **HTTP Server:** WebUI für Browser-Steuerung

## ⚡ Quick Start

```bash
# 1. .env File erstellen
cd esp-led-steuerung
cp .env.example .env
# .env editieren: WIFI_SSID, WIFI_PASSWORD eintragen

# 2. Flashen und Starten
cargo run --release

# 3. Browser öffnen
# → ESP32-IP-Adresse aus Serial Monitor ablesen
# → http://<ESP32-IP>/ im Browser öffnen
# → LED über WebSocket steuern
```

**Wichtig:** Bei .env Änderungen: `cargo clean && cargo run --release`

## 🎨 Features

✅ **RGB LED Control**
- WS2812 SmartLED auf GPIO8
- Auto-Rotation oder manuelle Steuerung
- Helligkeit: 10/255 (gedimmt)

✅ **WiFi & Networking**
- WiFi 6 (802.11ax)
- DHCP für automatische IP-Konfiguration
- DNS-Auflösung

✅ **MQTT v5 Publishing**
- Event-basiert (nur bei Änderung)
- Dual Topics: `led-color` + `led-mode`
- Automatisches Reconnect

✅ **HTTP/WebSocket Server**
- HTTP Server auf Port 80
- WebSocket bidirektional
- 4 parallele Connections (Task Pool)
- Graceful degradation bei > 10 Clients

✅ **Embassy Async Runtime**
- 7 parallel laufende Tasks
- PubSubChannel (1→N Broadcast)
- Command Channel (N→1)
- defmt Binary Logging

## 🔧 Hardware

**Board:** nanoESP32-C6 V1.0 (ESP32-C6-DevKitM-1 kompatibel)
- **Chip:** ESP32-C6 RISC-V @ 160MHz
- **Flash:** 8 MB
- **RAM:** 512 KB
- **Features:** WiFi 6, Bluetooth 5.3 LE, IEEE 802.15.4

**USB-Anschlüsse:**
1. **CH343 Port** ← **Empfohlen** (`/dev/ttyACM0`)
2. **ESP32C6 Port** - Für JTAG-Debugging

## 📦 Tech Stack

- **Sprache:** Rust (no_std, embedded)
- **Runtime:** Embassy Async
- **Networking:** embassy-net + smoltcp
- **HTTP/WebSocket:** picoserve 0.17.1
- **MQTT:** rust-mqtt 0.3.0
- **LED:** esp-hal-smartled (RMT Peripheral)
- **Logging:** defmt

## 🚀 Development

### Nur bauen
```bash
cargo build --release
```

### Board-Info
```bash
espflash board-info
```

### Code-Qualität
```bash
cargo clippy
cargo fmt
```

## 🐛 Troubleshooting

**USB-Port nicht gefunden (WSL2):**
```powershell
# Windows PowerShell (Administrator):
usbipd list
usbipd bind --busid 1-4
usbipd attach --wsl --busid 1-4
```

**Sonderzeichen im Serial Monitor:**
- defmt sendet Binärdaten!
- Lösung: `cargo run --release` nutzen

**.env Änderungen werden ignoriert:**
- `cargo clean && cargo run --release`

**VS Code: "can't find crate for test":**
- `.vscode/settings.json` muss im Workspace-Root liegen
- Nach Verschieben: "Developer: Reload Window"

## 📚 Dokumentation

**Ausführliche Dokumentation siehe:**
- **[CLAUDE.md](../CLAUDE.md)** - Vollständige Entwickler-Dokumentation
  - Projekt-Übersicht & Tech-Stack
  - WiFi, MQTT, HTTP/WebSocket Implementation
  - Task-Struktur & Architektur-Diagramme
  - Build-Konfiguration
  - Lessons Learned
  - Troubleshooting-Details

**Inline-Code-Kommentare:**
- Alle Projekt-Dateien sind vollständig auf Deutsch kommentiert
- `src/bin/main.rs` - Kompletter Code-Walkthrough
- `src/lib.rs` - Core Types mit Dokumentation
- `src/tasks/*.rs` - Task-spezifische Implementierungen

**Externe Ressourcen:**
- [ESP32-C6 Docs](https://docs.espressif.com/projects/rust/esp-hal/1.0.0/esp32c6/esp_hal/)
- [Embassy Book](https://embassy.dev/book/)
- [esp-hal Examples](https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples)

## 📊 Projekt-Status

**Aktuell:** ✅ Produktiv einsatzbereit
- 7 Tasks laufen stabil
- RAM-Optimiert (12 KB gespart)
- Fehlerbehandlung implementiert
- Type-Aliase für Lesbarkeit
- defmt::Format für Debugging

**Binary-Größe:** ~29 KB (0.35% von 8 MB Flash)

**Task-Struktur:**
1. LED Control (Auto/Manuell)
2-4. WiFi (Connection, Net Stack, DHCP)
5. MQTT Publishing
6. HTTP/WebSocket Server (×4 Pool)

## 📄 Lizenz

Template/Beispiel für ESP32-C6 Entwicklung mit Rust.

---

**Entwickelt mit:** Rust + Embassy + esp-hal 1.0 + picoserve + rust-mqtt

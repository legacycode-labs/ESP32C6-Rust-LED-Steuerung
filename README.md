# Rust Embedded Development für ESP32-C6

Dieses Repository enthält eine vollständige Entwicklungsumgebung für Embedded Rust auf dem ESP32-C6 (RISC-V).

## Hardware: nanoESP32-C6 V1.0

**Platinen-Aufdruck:** V1711 nanoESP32-C6 V1.0
**Board-Typ:** ESP32-C6-DevKitM-1 kompatibel
**Dokumentation:** https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitm-1/index.html

**Chip:** ESP32-C6 (RISC-V 32-bit @ 160MHz)
- Flash: 8 MB
- RAM: 512 KB SRAM
- WiFi 6, Bluetooth 5.3 LE, Zigbee/Thread

## Dev Container

Eine fertig konfigurierte Entwicklungsumgebung mit:
- Ubuntu 24.04 LTS Base Image
- Rust Toolchain mit riscv32imac-unknown-none-elf Target
- espflash für USB-Flashen (kein Debug-Probe nötig!)
- cargo-espflash, cargo-binutils
- VS Code Extensions (rust-analyzer, even-better-toml, errorlens, crates)

**Start:** Öffne das Projekt in VS Code und wähle "Reopen in Container"

## Projekt: esp-led-steuerung

ESP32-C6 Projekt erstellt mit `esp-generate` - dem offiziellen ESP-RS Template Generator.
- Moderne esp-hal Version (1.0+)
- defmt Logging-Framework für effiziente Binärdaten-Übertragung
- ESP-IDF Bootloader Integration
- Komplett konfiguriertes Setup mit build.rs und rust-toolchain.toml

Siehe [esp-led-steuerung/README.md](esp-led-steuerung/README.md) für Details.

## Schnellstart

1. Repository klonen
2. In VS Code öffnen
3. "Reopen in Container" wählen
4. Im Container:
   ```bash
   cd esp-led-steuerung
   cargo check    # Code prüfen
   cargo build --release    # Projekt bauen
   cargo run --release      # Flashen und Monitor starten
   ```

## Hardware-Zugriff (Windows/WSL2)

Für direktes Flashen vom Dev Container muss das USB-Board von Windows an WSL2 durchgereicht werden:

### 1. usbipd installieren (Windows Host)

**Auf Windows PowerShell (als Admin):**
```powershell
winget install usbipd
```

### 2. USB-Board durchreichen (Windows Host)

**Auf Windows PowerShell:**
```powershell
# Board finden (zeigt alle USB-Geräte)
usbipd list

# Beispiel-Ausgabe:
# BUSID  VID:PID    DEVICE                    STATE
# 3-6    1a86:55d4  USB Single Serial         Not attached

# Board für WSL2 freigeben (einmalig als Admin)
usbipd bind --busid 3-6

# Board an WSL2 anhängen (jede Session neu)
usbipd attach --wsl --busid 3-6
```

**Wichtig:** `usbipd attach` muss nach jedem Windows-Neustart oder USB-Reconnect neu ausgeführt werden!

### 3. USB-Gerät überprüfen (WSL2 / Dev Container)

**In WSL2 oder im Dev Container:**
```bash
# Alle USB-Geräte anzeigen
lsusb

# Serial-Devices anzeigen
ls /dev/ttyACM* /dev/ttyUSB*
# Sollte zeigen: /dev/ttyACM0
```

## Flashen

```bash
cd esp-led-steuerung
cargo run --release
```

Das Board wird automatisch erkannt und geflasht - kein Debug-Probe nötig!
Der Monitor startet automatisch und zeigt die defmt-Ausgaben.

## Weitere Informationen

Siehe `CLAUDE.md` für detaillierte Entwicklungsnotizen.

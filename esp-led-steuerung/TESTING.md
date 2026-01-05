# Testing Strategy - ESP32-C6 LED Controller

## ✅ Status: Tests funktionieren!

**Testabdeckung:** 97.01% (Regions) | 100% (Functions) | 79.71% (Lines)

Das Projekt nutzt eine **Workspace-Struktur** mit Trait-basierter Abstraktion für testbaren Code.

## 📁 Workspace-Struktur

```
esp-led-steuerung/
├── Cargo.toml              # Workspace Root
├── esp-core/               # ✅ Platform-agnostic (KEINE ESP-Deps!)
│   ├── src/
│   │   ├── traits.rs       # SmartLedWriter Trait
│   │   ├── types.rs        # LedColorMessage, LedCommand
│   │   └── logic.rs        # rotate_color() + Tests
│   └── Cargo.toml
├── esp-firmware/           # ESP32 Hardware Implementation
│   ├── src/
│   │   ├── hal/led_writer.rs   # RmtLedWriter (echte Hardware)
│   │   ├── tasks/              # WiFi, MQTT, HTTP, LED
│   │   └── bin/main.rs
│   ├── .cargo/config.toml      # ESP32 Target Config
│   └── Cargo.toml
└── esp-tests/              # ✅ Integration Tests (x86_64)
    ├── tests/
    │   └── led_tests.rs    # MockLedWriter + 15 Tests
    └── Cargo.toml
```

## 🎯 Keine Code-Duplikation

**Ein Trait, mehrere Implementierungen:**

```rust
// esp-core/src/traits.rs - Trait Definition (1x)
pub trait SmartLedWriter: Send {
    fn write(&mut self, color: RGB8) -> Result<(), LedError>;
}

// esp-firmware/src/hal/led_writer.rs - Hardware (1x)
pub struct RmtLedWriter<'a> { ... }
impl SmartLedWriter for RmtLedWriter { ... }

// esp-tests/tests/led_tests.rs - Mock (1x)
pub struct MockLedWriter { ... }
impl SmartLedWriter for MockLedWriter { ... }
```

✅ **Kein duplizierter Code** - nur verschiedene Implementierungen desselben Traits!

## 🚀 Tests ausführen

### Alle Tests

```bash
cargo test
```

**Output:**
```
running 4 tests   (esp-core/src/logic.rs)
running 15 tests  (esp-tests/tests/led_tests.rs)

test result: ok. 19 passed; 0 failed
```

### Einzelne Packages

```bash
# Nur esp-core Tests
cargo test -p esp-core

# Nur Integration Tests
cargo test -p esp-tests
```

### ESP32 Firmware bauen

```bash
cd esp-firmware
cargo build --release
cargo run --release  # Build + Flash + Monitor
```

**Wichtig:** esp-firmware kann NICHT für x86_64 gebaut werden - nur für ESP32 RISC-V!

## 📊 Test Coverage

### Coverage mit HTML Report

```bash
cargo llvm-cov --package esp-core --package esp-tests --html
```

**Öffnet automatisch:** `target/llvm-cov/html/index.html`

### Coverage Summary

```bash
cargo llvm-cov --package esp-core --package esp-tests --summary-only
```

**Aktuelle Coverage:**
```
Filename      Regions    Cover    Functions  Cover    Lines   Cover
───────────────────────────────────────────────────────────────────
logic.rs         36     100.00%      5      100.00%    29    100.00%
types.rs         31      93.55%      2      100.00%    40     65.00%
───────────────────────────────────────────────────────────────────
TOTAL            67      97.01%      7      100.00%    69     79.71%
```

### Coverage für CI/CD

```bash
# lcov Format (für Tools wie Codecov, Coveralls)
cargo llvm-cov --package esp-core --package esp-tests --lcov --output-path lcov.info
```

## 🧪 Was wird getestet?

### esp-core/src/logic.rs (100% Coverage)

**Pure Functions:**
- `rotate_color()` - RGB Farb-Rotation (Rot → Grün → Blau → Rot)

**Tests:**
- ✅ `test_rotate_color_red_to_green()`
- ✅ `test_rotate_color_green_to_blue()`
- ✅ `test_rotate_color_blue_to_red()`
- ✅ `test_rotate_color_full_cycle()`

### esp-core/src/types.rs (93.55% Coverage)

**Types:**
- `LedColorMessage` - LED Status mit Farbe + Modus
- `LedCommand` - Kommandos für LED-Steuerung

**Tests in esp-tests/tests/led_tests.rs:**
- ✅ `test_led_color_message_red_auto()`
- ✅ `test_led_color_message_green_manual()`
- ✅ `test_led_color_message_blue()`
- ✅ `test_led_color_message_unknown()`
- ✅ `test_led_command_try_from_rot()`
- ✅ `test_led_command_try_from_invalid()`
- ✅ `test_led_command_enable_auto()`

### MockLedWriter Tests

**Mock Implementation Tests:**
- ✅ `test_mock_led_writer_write()`
- ✅ `test_mock_led_writer_multiple_writes()`
- ✅ `test_mock_led_writer_fail()`
- ✅ `test_mock_led_writer_recovers_after_fail()`

## 🏗️ Architektur-Entscheidungen

### Warum Workspace statt Monolith?

**Problem:** ESP32-Dependencies (esp-hal, esp-rom-sys) kompilieren nur für RISC-V
- `cargo test` würde versuchen für x86_64 zu bauen → **FAIL**

**Lösung:** Workspace mit 3 Crates
- **esp-core:** Keine Hardware-Deps, kompiliert für x86_64 ✅
- **esp-firmware:** ESP32-only, ausgeklammert via `default-members`
- **esp-tests:** Integration Tests, kompiliert für x86_64 ✅

### Warum Trait statt Repository Pattern?

**Rust Best Practice:** Trait-basierte Abstraktion
- Kein Runtime-Overhead
- Compile-time Garantien
- Idiomatisches Rust (kein Java-Stil)

### Warum MockLedWriter in Tests statt Core?

**Separation of Concerns:**
- esp-core: Nur Trait-Definition (production code)
- esp-firmware: Hardware-Implementierung (production code)
- esp-tests: Mock-Implementierung (test code)

## 📝 Lessons Learned

### ❌ Was NICHT funktioniert

1. **tests/ im Workspace Root**
   - Würde gegen Root-Package gebaut
   - Root hat alle Dependencies → esp-hal → 💥
   - **Lösung:** Gelöscht, Tests in esp-tests/

2. **cargo test ohne -p flag im Workspace mit ESP-Deps**
   - Würde esp-firmware für x86_64 bauen → 💥
   - **Lösung:** `default-members` in Cargo.toml

3. **TryFrom in esp-core mit LED_BRIGHTNESS**
   - esp-core hat keinen Zugriff auf config.rs (firmware-only)
   - **Lösung:** DEFAULT_BRIGHTNESS in esp-core, Override in firmware

### ✅ Was funktioniert

1. **Trait-basierte Abstraktion**
   - Ein Trait, mehrere Implementierungen
   - Keine Code-Duplikation

2. **default-members in Workspace**
   - Schließt esp-firmware aus `cargo test` aus
   - Tests laufen auf x86_64 Host

3. **cargo-llvm-cov für Coverage**
   - Moderne Coverage-Analyse
   - Bessere Workspace-Unterstützung als tarpaulin
   - HTML Reports + lcov Format

## 🔧 Tools Installation

```bash
# Coverage Tool installieren
cargo install cargo-llvm-cov

# Verwendung
cargo llvm-cov --package esp-core --package esp-tests --html
```

## 🎯 Nächste Schritte (Optional)

**Weitere 3% Coverage erreichen:**
- Edge-Cases in `LedCommand::try_from()` testen
- Error-Paths in `LedColorMessage::from_color()` testen

**CI/CD Integration:**
```yaml
# .github/workflows/test.yml
- name: Run tests
  run: cargo test

- name: Coverage
  run: |
    cargo install cargo-llvm-cov
    cargo llvm-cov --package esp-core --package esp-tests --lcov --output-path lcov.info

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: lcov.info
```

**Mehr Tests:**
- WebSocket Handler Logic extrahieren und testen
- MQTT Message Formatting testen
- Config Validation testen

## 📚 Referenzen

- **Cargo Workspaces:** https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
- **cargo-llvm-cov:** https://github.com/taiki-e/cargo-llvm-cov
- **Rust Testing:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **Trait Objects:** https://doc.rust-lang.org/book/ch17-02-trait-objects.html

// Build-Script: Wird vor dem Kompilieren ausgeführt
// Konfiguriert den Linker für ESP32-C6 Embedded Rust

fn main() {
    // Lade .env file für WiFi-Credentials
    // Fehler ignorieren wenn .env nicht existiert (dann müssen ENV vars gesetzt sein)
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("⚠️  .env file nicht gefunden: {}", e);
        eprintln!("   Setze WIFI_SSID und WIFI_PASSWORD als Environment-Variablen");
    }

    // Gebe WiFi-Credentials an Rust-Compiler weiter
    // Die Werte werden zur Compile-Zeit in den Code eingebacken
    if let Ok(ssid) = std::env::var("WIFI_SSID") {
        println!("cargo:rustc-env=WIFI_SSID={}", ssid);
    }
    if let Ok(password) = std::env::var("WIFI_PASSWORD") {
        println!("cargo:rustc-env=WIFI_PASSWORD={}", password);
    }

    // Gebe MQTT-Konfiguration an Rust-Compiler weiter
    // Die Werte werden zur Compile-Zeit in den Code eingebacken
    if let Ok(broker) = std::env::var("MQTT_BROKER") {
        println!("cargo:rustc-env=MQTT_BROKER={}", broker);
    }
    if let Ok(port) = std::env::var("MQTT_PORT") {
        println!("cargo:rustc-env=MQTT_PORT={}", port);
    }
    if let Ok(client_id) = std::env::var("MQTT_CLIENT_ID") {
        println!("cargo:rustc-env=MQTT_CLIENT_ID={}", client_id);
    }
    if let Ok(topic_color) = std::env::var("MQTT_TOPIC_COLOR") {
        println!("cargo:rustc-env=MQTT_TOPIC_COLOR={}", topic_color);
    }
    if let Ok(topic_mode) = std::env::var("MQTT_TOPIC_MODE") {
        println!("cargo:rustc-env=MQTT_TOPIC_MODE={}", topic_mode);
    }

    // Registriere hilfsbereiten Error-Handler für Linker-Fehler
    linker_be_nice();

    // Füge Linker-Skripte hinzu:

    // 1. defmt.x - defmt Logging-Support
    //    Definiert Symbole für defmt's binäres Log-Format
    println!("cargo:rustc-link-arg=-Tdefmt.x");

    // 2. linkall.x - ESP32 Memory-Layout
    //    WICHTIG: Muss als LETZTES kommen (sonst Probleme mit flip-link)
    //    Definiert Flash/RAM-Layout und Startup-Code
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

// Error-Handler: Zeigt hilfreiche Tipps bei Linker-Fehlern
// Wird vom Linker als "--error-handling-script" aufgerufen
fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();

    // Wenn vom Linker aufgerufen (mit Error-Typ und Symbol-Name)
    if args.len() > 1 {
        let kind = &args[1]; // Fehler-Typ (z.B. "undefined-symbol")
        let what = &args[2]; // Symbol-Name (z.B. "_defmt_...")

        match kind.as_str() {
            // Undefiniertes Symbol gefunden
            "undefined-symbol" => match what.as_str() {
                what if what.starts_with("_defmt_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`"
                    );
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                what if what.starts_with("esp_rtos_") => {
                    eprintln!();
                    eprintln!(
                        "💡 `esp-radio` has no scheduler enabled. Make sure you have initialized `esp-rtos` or provided an external scheduler."
                    );
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!(
                        "💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests"
                    );
                    eprintln!();
                }
                "free"
                | "malloc"
                | "calloc"
                | "get_free_internal_heap_size"
                | "malloc_internal"
                | "realloc_internal"
                | "calloc_internal"
                | "free_internal" => {
                    eprintln!();
                    eprintln!(
                        "💡 Did you forget the `esp-alloc` dependency or didn't enable the `compat` feature on it?"
                    );
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}

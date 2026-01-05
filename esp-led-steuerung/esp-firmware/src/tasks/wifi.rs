// WiFi Task - Verbindet mit WLAN und managed Connection
use defmt::{Debug2Format, error, info, warn};
use embassy_net::{Runner, Stack};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice};

use crate::config::{WIFI_PASSWORD, WIFI_SSID};

/// WiFi Connection Task
///
/// Managed die WiFi-Verbindung:
/// - Verbindet mit Access Point
/// - Holt IP-Adresse via DHCP
/// - Überwacht Verbindung und reconnected bei Bedarf
#[embassy_executor::task]
pub async fn connection_task(mut controller: WifiController<'static>) {
    info!("WiFi: Starting connection task");

    loop {
        if matches!(controller.is_started(), Ok(false)) {
            info!("WiFi: Configuring and starting...");

            // Configure WiFi station mode
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(WIFI_SSID.into())
                    .with_password(WIFI_PASSWORD.into()),
            );

            if let Err(e) = controller.set_config(&client_config) {
                error!("WiFi: Failed to set configuration: {}", Debug2Format(&e));
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }

            if let Err(e) = controller.start_async().await {
                error!("WiFi: Failed to start: {}", Debug2Format(&e));
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }

            info!("WiFi: Started successfully");
        }

        // Scan for networks (optional, für Debugging)
        match controller
            .scan_with_config_async(ScanConfig::default())
            .await
        {
            Ok(ap_infos) => {
                info!("WiFi: Found {} access points", ap_infos.len());
                for ap_info in &ap_infos {
                    if ap_info.ssid.as_str() == WIFI_SSID {
                        info!(
                            "WiFi: Target AP found - SSID: {}, Signal: {} dBm",
                            WIFI_SSID, ap_info.signal_strength
                        );
                    }
                }
            }
            Err(e) => {
                warn!("WiFi: Scan failed: {}", Debug2Format(&e));
            }
        }

        // Connect to AP
        info!("WiFi: Connecting to '{}'...", WIFI_SSID);
        match controller.connect_async().await {
            Ok(_) => {
                info!("WiFi: Connected successfully!");
            }
            Err(e) => {
                error!("WiFi: Connection failed: {}", Debug2Format(&e));
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }

        // Wait for disconnect
        info!("WiFi: Waiting for disconnect event...");
        controller
            .wait_for_event(esp_radio::wifi::WifiEvent::StaDisconnected)
            .await;
        warn!("WiFi: Disconnected from AP, will retry...");

        Timer::after(Duration::from_secs(2)).await;
    }
}

/// Network Task
///
/// Überwacht den Netzwerk-Stack:
/// - Prozessiert Netzwerk-Pakete
/// - Managed TCP/IP Stack
#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}

/// DHCP Monitor Task
///
/// Wartet bis eine IP-Adresse vom DHCP-Server erhalten wurde
/// und loggt dann die Netzwerk-Konfiguration
#[embassy_executor::task]
pub async fn dhcp_task(stack: &'static Stack<'static>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("WiFi: Link is up, waiting for IP address...");

    loop {
        if let Some(config) = stack.config_v4() {
            info!("WiFi: Got IP address!");
            info!("  IP:      {}", Debug2Format(&config.address.address()));
            info!("  Gateway: {}", Debug2Format(&config.gateway));
            info!("  DNS:     {}", Debug2Format(&config.dns_servers));
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

// mDNS Responder Task - Advertised Hostname via Multicast DNS
//
// Dieser Task implementiert einen mDNS (Multicast DNS) Responder nach RFC 6762.
// Der ESP32-C6 wird damit unter einem lesbaren Hostnamen (z.B. "led.local")
// im lokalen Netzwerk erreichbar, ohne dass ein DNS-Server benötigt wird.
//
// Technische Details:
// - Protokoll: mDNS (RFC 6762)
// - Transport: UDP Multicast auf 224.0.0.251:5353
// - Unterstützt: A-Records (IPv4 Hostname-Auflösung)
// - Library: edge-mdns 0.6.1 (no_std)
// - Adapter: edge-nal-embassy 0.7.0 (embassy-net Integration)

use defmt::{Debug2Format, error, info, warn};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};

use core::net::{Ipv4Addr, SocketAddr};
use core::sync::atomic::{AtomicU32, Ordering};

use edge_mdns::{HostAnswersMdnsHandler, buf::VecBufAccess, domain::base::Ttl, host::Host, io};
use edge_nal::{MulticastV4, UdpBind, UdpSplit};
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;

use crate::config::{
    MDNS_HOSTNAME, MDNS_MULTICAST_ADDR, MDNS_PACKET_BUFFER_SIZE, MDNS_PORT,
    MDNS_RECONNECT_DELAY_SECS, MDNS_TTL_SECS, MDNS_UDP_BUFFER_SIZE,
};

/// Atomischer Counter für Random Number Generator
///
/// Wird für mDNS Transaction IDs verwendet. Ein einfacher Counter
/// ist für mDNS ausreichend, da keine kryptographische Sicherheit
/// benötigt wird.
static RNG_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Random Number Generator für mDNS
///
/// Generiert Pseudo-Zufallszahlen basierend auf einem atomischen Counter.
/// Wird von edge-mdns für Transaction IDs und Query IDs verwendet.
///
/// # Parameter
/// - `buf`: Buffer der mit Zufallsbytes gefüllt werden soll
///
/// # Implementierung
/// Nutzt einen wrapping counter statt echter Zufallszahlen.
/// Für mDNS-Protokoll ausreichend, da nur Eindeutigkeit benötigt wird.
fn mdns_rng(buf: &mut [u8]) {
    let mut counter = RNG_COUNTER.fetch_add(1, Ordering::Relaxed);
    for chunk in buf.chunks_mut(4) {
        let bytes = counter.to_le_bytes();
        let len = chunk.len().min(4);
        chunk[..len].copy_from_slice(&bytes[..len]);
        counter = counter.wrapping_add(1);
    }
}

/// mDNS Responder Task
///
/// Dieser Task advertised den ESP32-C6 via mDNS unter dem Hostnamen
/// definiert in `MDNS_HOSTNAME` (konfigurierbar in `src/config.rs`).
///
/// # Funktionsweise
///
/// Der Task läuft kontinuierlich und übernimmt folgende Aufgaben:
///
/// 1. **Netzwerk-Initialisierung**
///    - Wartet auf WiFi-Link (siehe `connection_task`)
///    - Wartet auf DHCP IP-Adresse (siehe `dhcp_task`)
///
/// 2. **UDP-Socket Setup**
///    - Bindet auf `0.0.0.0:5353` (MDNS_PORT)
///    - Joined IPv4 Multicast-Gruppe `224.0.0.251` (MDNS_MULTICAST_ADDR)
///
/// 3. **mDNS Responder Loop**
///    - Empfängt mDNS-Queries von anderen Geräten
///    - Antwortet mit A-Records (Hostname → IP-Adresse)
///    - TTL für Antworten: MDNS_TTL_SECS (Standard: 120 Sekunden)
///
/// 4. **Fehlerbehandlung & Reconnect**
///    - Bei jedem Fehler: Automatisches Reconnect
///    - Wartezeit vor Retry: MDNS_RECONNECT_DELAY_SECS (Standard: 5 Sekunden)
///
/// # Netzwerk-Erreichbarkeit
///
/// Nach erfolgreicher Initialisierung ist der ESP32 erreichbar unter:
/// - **Hostname:** `<MDNS_HOSTNAME>.local` (z.B. "led.local")
/// - **IP-Adresse:** Vom DHCP zugewiesene IPv4-Adresse
///
/// # Beispiel-Nutzung
///
/// ```bash
/// # Hostname auflösen (Linux/macOS)
/// avahi-resolve -n led.local
/// ping led.local
///
/// # HTTP-Zugriff via Hostname
/// curl http://led.local/
///
/// # Im Browser
/// http://led.local/
/// ```
///
/// # Konfiguration
///
/// Alle mDNS-Parameter sind in `src/config.rs` konfigurierbar:
/// - `MDNS_HOSTNAME` - Hostname ohne .local Suffix
/// - `MDNS_TTL_SECS` - Cache-Dauer für Antworten
/// - `MDNS_PORT` - UDP-Port (Standard: 5353)
/// - `MDNS_MULTICAST_ADDR` - Multicast-Gruppe (Standard: 224.0.0.251)
/// - `MDNS_RECONNECT_DELAY_SECS` - Wartezeit nach Fehler
/// - `MDNS_UDP_BUFFER_SIZE` - UDP TX/RX Buffer-Größe
/// - `MDNS_PACKET_BUFFER_SIZE` - mDNS Packet Buffer-Größe
///
/// # Parameter
/// - `stack`: embassy-net Stack für Netzwerk-Operationen (shared mit allen Tasks)
///
/// # Resourcen-Nutzung
/// - **RAM:** ~4.2 KB (UDP Buffers + mDNS State)
/// - **Flash:** ~19 KB (edge-mdns Library)
/// - **Sockets:** 1 UDP Socket (von 13 verfügbaren)
#[embassy_executor::task]
pub async fn mdns_responder_task(stack: &'static Stack<'static>) {
    info!("mDNS: Task started, waiting for network...");
    wait_for_network(stack).await;
    info!("mDNS: Network ready");

    loop {
        match run_mdns_responder(stack).await {
            Ok(_) => warn!("mDNS: Responder stopped normally"),
            Err(e) => error!("mDNS: Error: {}", Debug2Format(&e)),
        }
        info!("mDNS: Reconnecting in {}s...", MDNS_RECONNECT_DELAY_SECS);
        Timer::after(Duration::from_secs(MDNS_RECONNECT_DELAY_SECS)).await;
    }
}

/// Wartet bis Netzwerk-Verbindung verfügbar ist
///
/// Prüft kontinuierlich Link-Status und DHCP-Konfiguration.
/// Identisches Pattern wie in `mqtt.rs` und `wifi.rs` verwendet.
///
/// # Wartet auf
/// 1. WiFi Link ist up (`stack.is_link_up()`)
/// 2. IPv4-Konfiguration vom DHCP verfügbar (`stack.config_v4()`)
///
/// # Parameter
/// - `stack`: embassy-net Stack
///
/// # Polling-Intervall
/// Prüft alle 500ms - Balance zwischen Reaktivität und CPU-Last
async fn wait_for_network(stack: &'static Stack<'static>) {
    loop {
        if stack.is_link_up() {
            if let Some(_) = stack.config_v4() {
                break;
            }
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Führt mDNS Responder Loop aus
///
/// Diese Funktion implementiert den Haupt-Lifecycle des mDNS Responders:
///
/// 1. **IP-Adresse holen** - Liest IPv4-Adresse vom DHCP
/// 2. **UDP-Stack Setup** - Erstellt edge-nal-embassy UDP Adapter
/// 3. **Socket Binding** - Bindet auf `0.0.0.0:MDNS_PORT`
/// 4. **Multicast Join** - Joined Gruppe `MDNS_MULTICAST_ADDR`
/// 5. **Host Setup** - Konfiguriert Hostname, IP, TTL
/// 6. **Responder Start** - Startet blocking mDNS Loop
///
/// # UDP-Stack Details
///
/// Der UDP-Stack wird via `edge-nal-embassy` erstellt, welcher als Adapter
/// zwischen `edge-mdns` (benötigt edge-nal traits) und `embassy-net`
/// (ESP32 Netzwerk-Stack) dient.
///
/// **StaticCell-Pattern:**
/// - UDP_BUFFERS werden nur **einmal** initialisiert
/// - Weitere Calls zu `init_with()` nach Reconnects geben vorhandene Referenz zurück
/// - Verhindert Panic bei wiederholten Aufrufen
///
/// # mDNS-Buffers
///
/// - **RX/TX Buffers:** MDNS_PACKET_BUFFER_SIZE (1500 Bytes = Standard MTU)
/// - **Format:** VecBufAccess (stack-allocated, kein Heap)
/// - **Mutex:** NoopRawMutex (single-core safe, effizienter als CriticalSection)
///
/// # Fehlerbehandlung
///
/// Bei jedem Fehler wird die Funktion beendet und der Haupt-Loop
/// startet automatisch einen Reconnect-Versuch nach MDNS_RECONNECT_DELAY_SECS.
///
/// # Parameter
/// - `stack`: embassy-net Stack für Netzwerk-Operationen
///
/// # Returns
/// - `Ok(())` - Responder gestoppt (unwahrscheinlich, normalerweise blocking)
/// - `Err(MdnsError)` - Socket-Fehler, Multicast-Fehler oder Responder-Fehler
async fn run_mdns_responder(stack: &'static Stack<'static>) -> Result<(), MdnsError> {
    // IP-Adresse vom DHCP holen
    let our_ip = stack.config_v4().unwrap().address.address();
    info!("mDNS: Using IP {}", Debug2Format(&our_ip));

    // UDP Adapter erstellen (edge-nal-embassy → embassy-net)
    // StaticCell wird nur einmal initialisiert, weitere Calls returnen existierende Referenz
    // Wichtig für Reconnect-Loop: Verhindert Panic bei wiederholter Initialisierung
    static UDP_BUFFERS: static_cell::StaticCell<
        UdpBuffers<1, MDNS_UDP_BUFFER_SIZE, MDNS_UDP_BUFFER_SIZE>,
    > = static_cell::StaticCell::new();
    let udp_buffers = UDP_BUFFERS.init_with(|| UdpBuffers::new());
    let udp_stack = Udp::new(*stack, udp_buffers);

    // Multicast Socket auf 0.0.0.0:MDNS_PORT binden
    // UNSPECIFIED = alle Interfaces (WiFi in unserem Fall)
    let mut socket = udp_stack
        .bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), MDNS_PORT))
        .await
        .map_err(|_| MdnsError::SocketBindFailed)?;

    // Join Multicast-Gruppe (mDNS IPv4)
    // Ermöglicht Empfang von mDNS-Queries auf 224.0.0.251
    socket
        .join_v4(Ipv4Addr::from(MDNS_MULTICAST_ADDR), Ipv4Addr::UNSPECIFIED)
        .await
        .map_err(|_| MdnsError::MulticastJoinFailed)?;

    // Socket in RX/TX splitten für edge-mdns API
    let (recv, send) = socket.split();

    // Host-Konfiguration für mDNS Responses
    let host = Host {
        hostname: MDNS_HOSTNAME,            // Hostname ohne .local Suffix
        ipv4: our_ip.into(),                // Unsere IPv4-Adresse vom DHCP
        ipv6: [0u8; 16].into(),             // IPv6 nicht unterstützt (kein proto-ipv6 in smoltcp)
        ttl: Ttl::from_secs(MDNS_TTL_SECS), // Cache-Dauer für Clients
    };

    // mDNS Packet Buffers (stack-allocated)
    // Größe: MDNS_PACKET_BUFFER_SIZE (1500 Bytes = Standard MTU)
    let recv_buf = VecBufAccess::<NoopRawMutex, MDNS_PACKET_BUFFER_SIZE>::new();
    let send_buf = VecBufAccess::<NoopRawMutex, MDNS_PACKET_BUFFER_SIZE>::new();

    // Signal für Broadcast-Notifications (nicht verwendet, aber von API benötigt)
    let signal = Signal::<NoopRawMutex, ()>::new();

    // mDNS Responder erstellen
    let mdns = io::Mdns::new(
        Some(our_ip), // IPv4 Interface
        None,         // Kein IPv6
        recv,         // UDP RX
        send,         // UDP TX
        recv_buf,     // RX Buffer
        send_buf,     // TX Buffer
        mdns_rng,     // RNG für Transaction IDs
        &signal,      // Broadcast Signal
    );

    info!(
        "mDNS: Responder running, advertising '{}.local'",
        MDNS_HOSTNAME
    );

    // Blocking: Läuft bis Fehler auftritt
    // HostAnswersMdnsHandler implementiert einfache A-Record Responses
    // (nur Hostname → IP, kein Service Discovery)
    mdns.run(HostAnswersMdnsHandler::new(&host))
        .await
        .map_err(|_| MdnsError::ResponderFailed)?;

    Ok(())
}

/// mDNS Fehler-Typen
///
/// Alle möglichen Fehler die während mDNS-Operationen auftreten können.
/// Jeder Fehler führt zu einem Reconnect-Versuch im Haupt-Loop.
#[derive(Debug)]
enum MdnsError {
    /// UDP Socket konnte nicht auf Port MDNS_PORT gebunden werden
    ///
    /// Mögliche Ursachen:
    /// - Port bereits belegt (unwahrscheinlich)
    /// - Keine Socket-Ressourcen verfügbar
    SocketBindFailed,

    /// Multicast-Gruppe konnte nicht gejoint werden
    ///
    /// Mögliche Ursachen:
    /// - Netzwerk-Interface nicht bereit
    /// - Multicast nicht unterstützt (sehr unwahrscheinlich bei WiFi)
    MulticastJoinFailed,

    /// mDNS Responder Loop ist fehlgeschlagen
    ///
    /// Mögliche Ursachen:
    /// - Netzwerk-Verbindung verloren
    /// - UDP Socket-Fehler
    /// - Buffer-Overflow (sehr unwahrscheinlich mit 1500 Byte Buffers)
    ResponderFailed,
}

impl defmt::Format for MdnsError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            MdnsError::SocketBindFailed => defmt::write!(fmt, "Socket bind failed"),
            MdnsError::MulticastJoinFailed => defmt::write!(fmt, "Multicast join failed"),
            MdnsError::ResponderFailed => defmt::write!(fmt, "Responder failed"),
        }
    }
}

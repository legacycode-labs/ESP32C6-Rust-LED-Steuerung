// Keine Standard-Bibliothek verwenden (Embedded System)
#![no_std]
// Kein normaler main() Einstiegspunkt (wird von esp_rtos bereitgestellt)
#![no_main]
// Verbiete mem::forget - gefährlich bei ESP HAL Types mit DMA-Buffern
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
// Verbiete große Stack-Frames (Stack ist auf Embedded Systemen begrenzt)
#![deny(clippy::large_stack_frames)]

// Heap Allocator (WiFi benötigt dynamischen Speicher)
extern crate alloc;

// Embassy Async Runtime
use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, Stack, StackResources};
use embassy_time::{Duration, Timer};

// ESP32-C6 HAL
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;

// Backtrace bei Panic und println!() Support
use {esp_backtrace as _, esp_println as _};

// Projekt-Module und Konfiguration
use esp_led_steuerung::config::{EXTRA_HEAP_SIZE, WIFI_HEAP_SIZE};
use esp_led_steuerung::tasks::{
    connection_task, dhcp_task, http_server_task, led_blink_task, mdns_responder_task, mqtt_task,
    net_task,
};
use esp_led_steuerung::{LedColorChannel, LedCommandChannel};

// ESP-IDF App Descriptor - erforderlich für den Bootloader!
// Ohne diesen schlägt das Flashen mit "ESP-IDF App Descriptor missing" fehl
esp_bootloader_esp_idf::esp_app_desc!();

/// Main Entry Point
///
/// Initialisiert Hardware, WiFi, startet Embassy Runtime und spawnt Tasks.
/// Danach schläft main() - alle Arbeit läuft in Tasks.
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // ESP32-C6 Konfiguration: CPU auf maximale Taktfrequenz (160 MHz)
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Heap Allocator initialisieren (WiFi braucht dynamischen Speicher!)
    // Zwei Bereiche: reclaimed RAM (64 KB) + extra (36 KB) = 100 KB total
    esp_alloc::heap_allocator!(
        #[esp_hal::ram(reclaimed)]
        size: WIFI_HEAP_SIZE
    );
    esp_alloc::heap_allocator!(size: EXTRA_HEAP_SIZE);

    // Embassy Runtime initialisieren (Timer + Software Interrupt)
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // WiFi Hardware initialisieren
    static RADIO_INIT: static_cell::StaticCell<esp_radio::Controller> =
        static_cell::StaticCell::new();
    let radio_init =
        RADIO_INIT.init(esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller"));

    let (wifi_controller, wifi_interface) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi");

    // Netzwerk-Stack erstellen
    // Random seed für TCP/IP Stack (von Hardware RNG)
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Static resources für embassy-net
    // Erhöht auf 12 Sockets: MQTT (1) + HTTP-Listener (1) + ~10 WebSocket-Clients
    static RESOURCES: static_cell::StaticCell<StackResources<12>> = static_cell::StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());

    // embassy-net erstellt Stack + Runner (nutzt STA interface für Client-Modus)
    let (stack, runner) = embassy_net::new(
        wifi_interface.sta,
        NetConfig::dhcpv4(Default::default()),
        resources,
        seed,
    );

    // Stack muss 'static sein für Tasks
    static STACK: static_cell::StaticCell<Stack<'static>> = static_cell::StaticCell::new();
    let stack = &*STACK.init(stack);

    // LED Farb-Channel erstellen (für LED → MQTT + HTTP Kommunikation)
    // PubSubChannel für Broadcast: alle Subscribers bekommen jede Nachricht
    // Params: <Mutex, Message, Capacity, MaxSubscribers, MaxPublishers>
    // 10 Subscribers: 1 MQTT + bis zu 9 WebSocket-Connections (mehr als genug)
    static COLOR_CHANNEL: static_cell::StaticCell<LedColorChannel> = static_cell::StaticCell::new();
    let color_channel = &*COLOR_CHANNEL.init(LedColorChannel::new());
    let color_publisher = color_channel.publisher().unwrap();

    // LED Command-Channel erstellen (für HTTP → LED Kommunikation)
    static COMMAND_CHANNEL: static_cell::StaticCell<LedCommandChannel> =
        static_cell::StaticCell::new();
    let command_channel = COMMAND_CHANNEL.init(LedCommandChannel::new());
    let command_sender = command_channel.sender();
    let command_receiver = command_channel.receiver();

    // Spawn LED Task (mit Publisher für Farb-Broadcasts und Receiver für Kommandos)
    spawner
        .spawn(led_blink_task(
            peripherals.GPIO8,
            peripherals.RMT,
            color_publisher,
            command_receiver,
        ))
        .unwrap();

    // Spawn WiFi Tasks
    spawner.spawn(connection_task(wifi_controller)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();
    spawner.spawn(dhcp_task(stack)).unwrap();

    // Spawn MQTT Task (mit Subscriber für LED-Farb-Updates)
    let mqtt_subscriber = color_channel.subscriber().unwrap();
    spawner.spawn(mqtt_task(stack, mqtt_subscriber)).unwrap();

    // Spawn HTTP Server Tasks (4x für concurrent connections)
    // Jede Task-Instanz kann eine Connection gleichzeitig handeln
    // Jede bekommt Referenz zum Color-Channel um Subscribers zu erstellen
    for task_id in 0..4 {
        spawner
            .spawn(http_server_task(
                task_id,
                stack,
                color_channel,
                command_sender,
            ))
            .unwrap();
    }

    // Spawn mDNS Responder Task (für led.local Hostname)
    spawner.spawn(mdns_responder_task(stack)).unwrap();

    // Main-Loop: schläft (alle Arbeit läuft in Tasks)
    loop {
        Timer::after(Duration::from_secs(3600)).await;
    }
}

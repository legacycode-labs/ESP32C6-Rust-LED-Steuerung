// Task-Modul: Enthält alle Embassy Tasks
//
// Jeder Task läuft asynchron und unabhängig.
// Tasks kommunizieren über Embassy Channels (LED → MQTT, HTTP ↔ LED).

pub mod http;
pub mod led_blink;
pub mod mdns;
pub mod mqtt;
pub mod wifi;

// Re-export Tasks für einfachen Import
pub use http::http_server_task;
pub use led_blink::led_blink_task;
pub use mdns::mdns_responder_task;
pub use mqtt::mqtt_task;
pub use wifi::{connection_task, dhcp_task, net_task};

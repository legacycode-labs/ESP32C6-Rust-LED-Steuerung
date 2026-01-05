// Web-Modul für HTTP Server und WebSocket
// Organisiert alle Web-bezogenen Komponenten

pub mod protocol;

// HTML-Datei zur Compile-Zeit einbinden
// Die Datei wird direkt ins Binary eingebettet
pub const INDEX_HTML: &str = include_str!("index.html");

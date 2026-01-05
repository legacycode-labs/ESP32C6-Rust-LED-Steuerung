// Hardware Abstraction Layer (HAL) Module
//
// Dieses Modul kapselt Hardware-Zugriffe hinter Traits,
// um Testbarkeit und Wartbarkeit zu verbessern.

pub mod led_writer;

pub use led_writer::{LedError, RmtLedWriter, SmartLedWriter};

#[cfg(test)]
pub use led_writer::MockLedWriter;

//! ESP Core - Platform-agnostic Logic and Traits
//!
//! Diese Crate enthält KEINE Hardware-Dependencies.
//! Sie definiert nur Traits und Pure Functions.

#![no_std]

pub mod logic;
pub mod traits;
pub mod types;

// Re-exports für einfachen Zugriff
pub use logic::rotate_color;
pub use traits::{LedError, SmartLedWriter};
pub use types::{LedColorMessage, LedCommand};

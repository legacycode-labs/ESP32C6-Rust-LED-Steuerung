//! Integration Tests für LED Logic
//!
//! Diese Tests laufen auf dem Host (x86_64) und nutzen MockLedWriter

use esp_core::{LedColorMessage, LedCommand, LedError, SmartLedWriter, rotate_color};
use rgb::RGB8;

// ============================================================================
// Mock LED Writer
// ============================================================================

#[derive(Default)]
pub struct MockLedWriter {
    pub last_color: Option<RGB8>,
    pub write_count: usize,
    pub fail_next_write: bool,
}

impl MockLedWriter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SmartLedWriter for MockLedWriter {
    fn write(&mut self, color: RGB8) -> Result<(), LedError> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(LedError::WriteFailed);
        }

        self.last_color = Some(color);
        self.write_count += 1;
        Ok(())
    }
}

// ============================================================================
// Tests: MockLedWriter
// ============================================================================

#[test]
fn test_mock_led_writer_write() {
    let mut mock = MockLedWriter::new();
    let color = RGB8 { r: 10, g: 0, b: 0 };

    assert_eq!(mock.write_count, 0);
    assert_eq!(mock.last_color, None);

    mock.write(color).unwrap();

    assert_eq!(mock.write_count, 1);
    assert_eq!(mock.last_color, Some(color));
}

#[test]
fn test_mock_led_writer_multiple_writes() {
    let mut mock = MockLedWriter::new();

    mock.write(RGB8 { r: 10, g: 0, b: 0 }).unwrap();
    mock.write(RGB8 { r: 0, g: 10, b: 0 }).unwrap();
    mock.write(RGB8 { r: 0, g: 0, b: 10 }).unwrap();

    assert_eq!(mock.write_count, 3);
    assert_eq!(mock.last_color, Some(RGB8 { r: 0, g: 0, b: 10 }));
}

#[test]
fn test_mock_led_writer_fail() {
    let mut mock = MockLedWriter::new();
    mock.fail_next_write = true;

    let result = mock.write(RGB8 { r: 10, g: 0, b: 0 });
    assert_eq!(result, Err(LedError::WriteFailed));
    assert_eq!(mock.write_count, 0);
    assert_eq!(mock.last_color, None);
}

#[test]
fn test_mock_led_writer_recovers_after_fail() {
    let mut mock = MockLedWriter::new();
    mock.fail_next_write = true;

    // First write fails
    let result1 = mock.write(RGB8 { r: 10, g: 0, b: 0 });
    assert!(result1.is_err());

    // Second write succeeds
    let result2 = mock.write(RGB8 { r: 0, g: 10, b: 0 });
    assert!(result2.is_ok());
    assert_eq!(mock.write_count, 1);
    assert_eq!(mock.last_color, Some(RGB8 { r: 0, g: 10, b: 0 }));
}

// ============================================================================
// Tests: rotate_color()
// ============================================================================

#[test]
fn test_rotate_color_red_to_green() {
    let red = RGB8 { r: 10, g: 0, b: 0 };
    let green = rotate_color(red);
    assert_eq!(green, RGB8 { r: 0, g: 10, b: 0 });
}

#[test]
fn test_rotate_color_green_to_blue() {
    let green = RGB8 { r: 0, g: 10, b: 0 };
    let blue = rotate_color(green);
    assert_eq!(blue, RGB8 { r: 0, g: 0, b: 10 });
}

#[test]
fn test_rotate_color_blue_to_red() {
    let blue = RGB8 { r: 0, g: 0, b: 10 };
    let red = rotate_color(blue);
    assert_eq!(red, RGB8 { r: 10, g: 0, b: 0 });
}

#[test]
fn test_rotate_color_full_cycle() {
    let mut color = RGB8 { r: 10, g: 0, b: 0 };
    color = rotate_color(color); // Rot → Grün
    color = rotate_color(color); // Grün → Blau
    color = rotate_color(color); // Blau → Rot
    assert_eq!(color, RGB8 { r: 10, g: 0, b: 0 });
}

// ============================================================================
// Tests: LedColorMessage
// ============================================================================

#[test]
fn test_led_color_message_red_auto() {
    let color = RGB8 { r: 10, g: 0, b: 0 };
    let msg = LedColorMessage::from_color(color, true);
    assert_eq!(msg.name, "Rot");
    assert_eq!(msg.color, color);
    assert!(msg.is_auto_mode);
}

#[test]
fn test_led_color_message_green_manual() {
    let color = RGB8 { r: 0, g: 10, b: 0 };
    let msg = LedColorMessage::from_color(color, false);
    assert_eq!(msg.name, "Grün");
    assert_eq!(msg.color, color);
    assert!(!msg.is_auto_mode);
}

#[test]
fn test_led_color_message_blue() {
    let color = RGB8 { r: 0, g: 0, b: 10 };
    let msg = LedColorMessage::from_color(color, true);
    assert_eq!(msg.name, "Blau");
    assert_eq!(msg.color, color);
}

#[test]
fn test_led_color_message_unknown() {
    let color = RGB8 {
        r: 10,
        g: 10,
        b: 10,
    };
    let msg = LedColorMessage::from_color(color, false);
    assert_eq!(msg.name, "Unbekannt");
}

// ============================================================================
// Tests: LedCommand
// ============================================================================

#[test]
fn test_led_command_try_from_rot() {
    use core::convert::TryFrom;
    let cmd = LedCommand::try_from("Rot");
    assert!(cmd.is_ok());
    match cmd.unwrap() {
        LedCommand::SetColor { target_color, name } => {
            assert_eq!(name, "Rot");
            assert_eq!(target_color.r, 10); // DEFAULT_BRIGHTNESS
            assert_eq!(target_color.g, 0);
            assert_eq!(target_color.b, 0);
        }
        _ => panic!("Expected SetColor variant"),
    }
}

#[test]
fn test_led_command_try_from_invalid() {
    use core::convert::TryFrom;
    let cmd = LedCommand::try_from("Gelb");
    assert!(cmd.is_err());
}

#[test]
fn test_led_command_enable_auto() {
    let cmd = LedCommand::EnableAuto;
    match cmd {
        LedCommand::EnableAuto => {}
        _ => panic!("Expected EnableAuto variant"),
    }
}

//! Pure Business Logic Functions
//!
//! Funktionen ohne Hardware-Dependencies (testbar!)

use rgb::RGB8;

/// Rotiert RGB-Farbwerte zyklisch: Rot → Grün → Blau → Rot
///
/// # Beispiele
///
/// ```
/// # use rgb::RGB8;
/// # use esp_core::rotate_color;
/// let mut color = RGB8 { r: 10, g: 0, b: 0 };  // Rot
/// color = rotate_color(color);                  // → Grün
/// assert_eq!(color, RGB8 { r: 0, g: 10, b: 0 });
/// ```
pub fn rotate_color(color: RGB8) -> RGB8 {
    RGB8 {
        r: color.b, // Rot wird zu Blau
        g: color.r, // Grün wird zu altem Rot
        b: color.g, // Blau wird zu Grün
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

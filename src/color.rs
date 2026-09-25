//! Colors.

/// An RGB color, one byte (0-255) per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const GRAY: Color = Color::rgb(128, 128, 128);
    pub const RED: Color = Color::rgb(230, 60, 60);
    pub const ORANGE: Color = Color::rgb(245, 150, 50);
    pub const YELLOW: Color = Color::rgb(245, 220, 70);
    pub const GREEN: Color = Color::rgb(80, 200, 100);
    pub const BLUE: Color = Color::rgb(70, 130, 230);
    pub const PURPLE: Color = Color::rgb(160, 90, 220);

    /// `const fn` means this can run at compile time, which is what lets
    /// us use it to define the constants above.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Builds a color from a hex literal like `0xFF8800`.
    pub const fn from_hex(hex: u32) -> Self {
        Self::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
    }

    /// Mixes two colors. `t = 0.0` gives `self`, `t = 1.0` gives `other`.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Color::rgb(
            mix(self.r, other.r),
            mix(self.g, other.g),
            mix(self.b, other.b),
        )
    }

    /// Packs the color into the `0x00RRGGBB` format used by the pixel buffer.
    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let c = Color::from_hex(0x12_34_56);
        assert_eq!(c, Color::rgb(0x12, 0x34, 0x56));
        assert_eq!(c.to_u32(), 0x12_34_56);
    }

    #[test]
    fn lerp_endpoints_and_middle() {
        assert_eq!(Color::BLACK.lerp(Color::WHITE, 0.0), Color::BLACK);
        assert_eq!(Color::BLACK.lerp(Color::WHITE, 1.0), Color::WHITE);
        assert_eq!(
            Color::BLACK.lerp(Color::WHITE, 0.5),
            Color::rgb(128, 128, 128)
        );
    }
}

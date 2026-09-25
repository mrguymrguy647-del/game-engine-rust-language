//! A tiny built-in bitmap font, so games can draw text without loading files.
//!
//! Every character is 3 pixels wide and 5 pixels tall. A glyph is stored as
//! five rows; in each row the three lowest bits say which pixels are lit,
//! left to right. For example the letter `A`:
//!
//! ```text
//! 0b010   . # .
//! 0b101   # . #
//! 0b111   # # #
//! 0b101   # . #
//! 0b101   # . #
//! ```

pub const GLYPH_WIDTH: usize = 3;
pub const GLYPH_HEIGHT: usize = 5;

/// Pixels from the start of one character to the start of the next (glyph + 1 px gap).
pub const ADVANCE: usize = GLYPH_WIDTH + 1;
/// Pixels from the top of one line of text to the top of the next.
pub const LINE_HEIGHT: usize = GLYPH_HEIGHT + 2;

type Glyph = [u8; GLYPH_HEIGHT];

const UNKNOWN: Glyph = [0b111, 0b001, 0b010, 0b000, 0b010]; // drawn as '?'

/// Looks up the bitmap for a character. Lowercase letters are drawn as uppercase.
pub fn glyph(c: char) -> Glyph {
    match c.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b001, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        ' ' => [0b000, 0b000, 0b000, 0b000, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => UNKNOWN,
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '(' => [0b001, 0b010, 0b010, 0b010, 0b001],
        ')' => [0b100, 0b010, 0b010, 0b010, 0b100],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
        _ => UNKNOWN,
    }
}

/// Is the pixel at (`col`, `row`) of this glyph lit? `col` 0 is the left column.
pub fn is_lit(glyph: &Glyph, col: usize, row: usize) -> bool {
    let bit = GLYPH_WIDTH - 1 - col;
    (glyph[row] >> bit) & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_digits_have_real_glyphs() {
        for c in ('A'..='Z').chain('0'..='9') {
            assert_ne!(glyph(c), UNKNOWN, "missing glyph for {c:?}");
        }
    }

    #[test]
    fn lowercase_maps_to_uppercase() {
        assert_eq!(glyph('a'), glyph('A'));
    }

    #[test]
    fn reads_bits_left_to_right() {
        let a = glyph('A');
        assert!(!is_lit(&a, 0, 0));
        assert!(is_lit(&a, 1, 0));
        assert!(is_lit(&a, 0, 1));
        assert!(!is_lit(&a, 1, 1));
    }
}

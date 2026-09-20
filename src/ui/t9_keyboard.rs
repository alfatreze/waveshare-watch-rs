// T9 Keyboard - French multi-tap input
// Ported from C++ T9Keyboard.cpp
// 12 buttons in 4x3 grid, tap to cycle characters, 1.5s auto-commit

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::{Alignment, Text};

const KEYS_COLS: usize = 3;
const KEYS_ROWS: usize = 4;
const KEY_W: i32 = 120;
const KEY_H: i32 = 72;
const KEY_GAP: i32 = 8;
const KB_X: i32 = (410 - KEYS_COLS as i32 * KEY_W - (KEYS_COLS as i32 - 1) * KEY_GAP) / 2;
const KB_Y: i32 = 175;
// A T9 sequence is measured from its first tap. 800ms was short enough for a
// normal three-letter cycle to commit before the third tap landed.
const COMMIT_MS: u32 = 1_500;

struct KeyDef {
    chars_lower: &'static [&'static str],
    chars_upper: &'static [&'static str],
}

static KEYS: [KeyDef; 12] = [
    KeyDef { chars_lower: &[".", ",", "?", "!", "1"], chars_upper: &[".", ",", "?", "!", "1"] },
    KeyDef { chars_lower: &["a","b","c","a","a","c","2"], chars_upper: &["A","B","C","2"] },
    KeyDef { chars_lower: &["d","e","f","e","e","e","3"], chars_upper: &["D","E","F","3"] },
    KeyDef { chars_lower: &["g","h","i","i","i","4"], chars_upper: &["G","H","I","4"] },
    KeyDef { chars_lower: &["j","k","l","5"], chars_upper: &["J","K","L","5"] },
    KeyDef { chars_lower: &["m","n","o","o","6"], chars_upper: &["M","N","O","6"] },
    KeyDef { chars_lower: &["p","q","r","s","7"], chars_upper: &["P","Q","R","S","7"] },
    KeyDef { chars_lower: &["t","u","v","u","u","8"], chars_upper: &["T","U","V","8"] },
    KeyDef { chars_lower: &["w","x","y","z","9"], chars_upper: &["W","X","Y","Z","9"] },
    KeyDef { chars_lower: &[], chars_upper: &[] },
    KeyDef { chars_lower: &[" ", "0"], chars_upper: &[" ", "0"] },
    KeyDef { chars_lower: &[], chars_upper: &[] },
];

const LOWER_LABELS: [&str; 12] = [
    "1 . , ?", "2 abc", "3 def", "4 ghi", "5 jkl", "6 mno", "7 pqrs", "8 tuv", "9 wxyz", "abc", "0 space", "delete",
];
const UPPER_LABELS: [&str; 12] = [
    "1 . , ?", "2 ABC", "3 DEF", "4 GHI", "5 JKL", "6 MNO", "7 PQRS", "8 TUV", "9 WXYZ", "ABC", "0 SPACE", "DELETE",
];
const NUMERIC_LABELS: [&str; 12] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "123", "0", "DELETE",
];

#[derive(Clone, Copy, PartialEq)]
enum Mode { Lower, Upper, Numeric }

pub struct T9Keyboard {
    text: [u8; 128],
    text_len: usize,
    mode: Mode,
    last_key: i8,
    char_index: usize,
    commit_timer: u32,
    active: bool,
    pending_char: bool, // char not yet committed
}

impl T9Keyboard {
    pub fn new() -> Self {
        Self {
            text: [0; 128], text_len: 0, mode: Mode::Lower,
            last_key: -1, char_index: 0, commit_timer: 0,
            active: false, pending_char: false,
        }
    }

    pub fn show(&mut self) { self.active = true; }
    pub fn hide(&mut self) { self.active = false; self.commit(); }
    pub fn is_active(&self) -> bool { self.active }

    pub fn get_text(&self) -> &str {
        core::str::from_utf8(&self.text[..self.text_len]).unwrap_or("")
    }

    pub fn clear_text(&mut self) { self.text_len = 0; }

    pub fn set_text(&mut self, value: &str) {
        self.clear_text();
        self.add_char(value);
        self.commit();
    }

    fn commit(&mut self) {
        self.last_key = -1;
        self.char_index = 0;
        self.commit_timer = 0;
        self.pending_char = false;
    }

    fn add_char(&mut self, ch: &str) {
        for &b in ch.as_bytes() {
            if self.text_len < self.text.len() - 1 {
                self.text[self.text_len] = b;
                self.text_len += 1;
            }
        }
    }

    fn delete_last(&mut self) {
        if self.text_len > 0 { self.text_len -= 1; }
    }

    fn key_label(&self, index: usize) -> &'static str {
        match self.mode {
            Mode::Lower => LOWER_LABELS[index],
            Mode::Upper => UPPER_LABELS[index],
            Mode::Numeric => NUMERIC_LABELS[index],
        }
    }

    /// Call every frame with dt_ms. Returns true if display needs update.
    pub fn update(&mut self, dt_ms: u32) -> bool {
        if !self.active { return false; }
        if self.pending_char {
            self.commit_timer += dt_ms;
            if self.commit_timer >= COMMIT_MS {
                self.commit();
                return true;
            }
        }
        false
    }

    /// Handle tap at screen coordinate. Returns true if consumed.
    pub fn handle_tap(&mut self, x: u16, y: u16) -> bool {
        if !self.active { return false; }

        // Find which key was tapped
        let local_x = x as i32 - KB_X;
        let local_y = y as i32 - KB_Y;
        if local_x < 0 || local_y < 0 { return false; }
        let kx = local_x / (KEY_W + KEY_GAP);
        let ky = local_y / (KEY_H + KEY_GAP);
        if kx < 0 || kx >= KEYS_COLS as i32 || ky < 0 || ky >= KEYS_ROWS as i32 { return false; }
        if local_x % (KEY_W + KEY_GAP) >= KEY_W || local_y % (KEY_H + KEY_GAP) >= KEY_H { return false; }
        let idx = (ky * KEYS_COLS as i32 + kx) as usize;
        if idx >= 12 { return false; }

        // Shift key
        if idx == 9 {
            self.commit();
            self.mode = match self.mode {
                Mode::Lower => Mode::Upper,
                Mode::Upper => Mode::Numeric,
                Mode::Numeric => Mode::Lower,
            };
            return true;
        }

        // Delete key
        if idx == 11 {
            self.commit();
            self.delete_last();
            return true;
        }

        // Character key
        let key = &KEYS[idx];
        let chars = match self.mode {
            Mode::Lower => key.chars_lower,
            Mode::Upper | Mode::Numeric => key.chars_upper,
        };
        if chars.is_empty() { return true; }

        if self.mode == Mode::Numeric {
            self.commit();
            // Last char is the digit
            self.add_char(chars[chars.len() - 1]);
        } else if idx as i8 == self.last_key && self.pending_char {
            // Same key: cycle to next character
            self.char_index = (self.char_index + 1) % chars.len();
            self.delete_last(); // remove previous cycling char
            self.add_char(chars[self.char_index]);
            self.commit_timer = 0;
        } else {
            // New key
            self.commit();
            self.last_key = idx as i8;
            self.char_index = 0;
            self.add_char(chars[0]);
            self.pending_char = true;
            self.commit_timer = 0;
        }
        true
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        if !self.active { return; }

        // Text area background
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(16, 96), Size::new(378, 68)),
            Size::new(10, 10),
        )
            .into_styled(PrimitiveStyle::with_fill(Rgb565::new(2, 5, 4)))
            .draw(d);
        // Text
        let txt = self.get_text();
        // The bundled font set stops at 10x20. Draw each glyph twice with a
        // 2px horizontal offset to create a clear 12px-wide input treatment.
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        for (index, ch) in txt.chars().enumerate() {
            let mut glyph = [0u8; 4];
            let ch_text = ch.encode_utf8(&mut glyph);
            let x = 26 + index as i32 * 12;
            let _ = Text::new(ch_text, Point::new(x, 142), style).draw(d);
            let _ = Text::new(ch_text, Point::new(x + 2, 142), style).draw(d);
        }
        // Cursor blink
        let cursor_x = 26 + txt.chars().count() as i32 * 12;
        let _ = Rectangle::new(Point::new(cursor_x, 112), Size::new(3, 32))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(d);

        // Mode indicator
        let mode_str = match self.mode {
            Mode::Lower => "LOWERCASE",
            Mode::Upper => "CAPITALS",
            Mode::Numeric => "NUMBERS",
        };
        let dim = MonoTextStyle::new(&FONT_10X20, Rgb565::CYAN);
        let _ = Text::with_alignment(mode_str, Point::new(205, 88), dim, Alignment::Center).draw(d);

        // Keyboard buttons
        let normal = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        for row in 0..KEYS_ROWS {
            for col in 0..KEYS_COLS {
                let idx = row * KEYS_COLS + col;
                let x = KB_X + col as i32 * (KEY_W + KEY_GAP);
                let y = KB_Y + row as i32 * (KEY_H + KEY_GAP);

                let bg = if idx == 9 {
                    Rgb565::new(0, 18, 22)
                } else if self.pending_char && idx as i8 == self.last_key {
                    Rgb565::new(10, 18, 16) // highlight active key
                } else {
                    Rgb565::new(6, 12, 10)
                };

                let _ = RoundedRectangle::with_equal_corners(
                    Rectangle::new(Point::new(x, y), Size::new(KEY_W as u32, KEY_H as u32)),
                    Size::new(12, 12),
                ).into_styled(PrimitiveStyle::with_fill(bg)).draw(d);

                let _ = Text::with_alignment(
                    self.key_label(idx),
                    Point::new(x + KEY_W / 2, y + KEY_H / 2 + 5),
                    normal,
                    Alignment::Center,
                ).draw(d);
            }
        }
    }
}

//! Project bitmap fonts.
//!
//! Pixel Operator Mono is by Jayvee Enaguas (HarvettFox96) and is released
//! under CC0. Its original license is retained at assets/fonts/.

use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::{mapping::ASCII, DecorationDimensions, MonoFont};
use embedded_graphics::prelude::Size;

/// Pixel Operator Mono rasterized at a compact 12 by 24 pixel cell.
pub const PIXEL_OPERATOR_MONO_12X24: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("assets/pixel_operator_mono_12x24.raw"), 1_152),
    glyph_mapping: &ASCII,
    character_size: Size::new(12, 24),
    character_spacing: 0,
    baseline: 20,
    underline: DecorationDimensions::new(22, 1),
    strikethrough: DecorationDimensions::new(12, 1),
};

/// Larger Pixel Operator Mono input face with a 16 by 24 pixel cell.
pub const PIXEL_OPERATOR_MONO_16X24: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("assets/pixel_operator_mono_16x24.raw"), 1_536),
    glyph_mapping: &ASCII,
    character_size: Size::new(16, 24),
    character_spacing: 0,
    baseline: 20,
    underline: DecorationDimensions::new(22, 1),
    strikethrough: DecorationDimensions::new(12, 1),
};

/// Standard UI face: wider and clearer than the previous built-in 10x20
/// bitmap while remaining narrow enough for watch controls and lists.
pub const PIXEL_OPERATOR_MONO_14X24: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("assets/pixel_operator_mono_14x24.raw"), 1_344),
    glyph_mapping: &ASCII,
    character_size: Size::new(14, 24),
    character_spacing: 0,
    baseline: 20,
    underline: DecorationDimensions::new(14, 1),
    strikethrough: DecorationDimensions::new(14, 1),
};

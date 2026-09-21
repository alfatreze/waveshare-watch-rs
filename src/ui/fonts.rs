//! Project bitmap fonts.
//!
//! Pixel Operator Mono is by Jayvee Enaguas (HarvettFox96) and is released
//! under CC0. Its original license is retained at assets/fonts/.

use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::{mapping::ASCII, DecorationDimensions, MonoFont};
use embedded_graphics::pixelcolor::BinaryColor;
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

// Keeps the pixel type explicit beside the font declaration and makes the
// intended one-bit source format clear to future font conversions.
const _: core::marker::PhantomData<BinaryColor> = core::marker::PhantomData;

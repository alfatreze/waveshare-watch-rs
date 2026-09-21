use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyle, Rectangle, RoundedRectangle}, text::{Alignment, Text}};
use crate::{apps::{App, AppInput, AppResult}, ui::fonts::PIXEL_OPERATOR_MONO_14X24};

pub struct Stopwatch { elapsed_ms: u32, running: bool }
impl Stopwatch { pub fn new() -> Self { Self { elapsed_ms: 0, running: false } } }
impl App for Stopwatch {
    fn name(&self) -> &str { "Stopwatch" }
    fn setup(&mut self) {}
    fn update(&mut self, input: &AppInput) -> AppResult {
        if let Some(point) = input.touch {
            if point.y > 330 { self.running = !self.running; }
            else if point.y > 230 && !self.running { self.elapsed_ms = 0; }
        }
        if self.running { self.elapsed_ms = self.elapsed_ms.saturating_add(input.dt_ms); }
        AppResult::Continue
    }
    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502)).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(d);
        let title = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CYAN);
        let value = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE);
        let _ = Text::with_alignment("STOPWATCH", Point::new(205, 42), title, Alignment::Center).draw(d);
        let total = self.elapsed_ms / 1000; let mins = total / 60; let secs = total % 60;
        let mut time = [b'0'; 5]; time[0] = b'0' + (mins / 10 % 10) as u8; time[1] = b'0' + (mins % 10) as u8; time[2] = b':'; time[3] = b'0' + (secs / 10) as u8; time[4] = b'0' + (secs % 10) as u8;
        let _ = Text::with_alignment(core::str::from_utf8(&time).unwrap_or("00:00"), Point::new(205, 132), value, Alignment::Center).draw(d);
        for (y, label) in [(230, "RESET"), (330, if self.running { "PAUSE" } else { "START" })] {
            let _ = RoundedRectangle::with_equal_corners(Rectangle::new(Point::new(16,y), Size::new(378,65)), Size::new(12,12)).into_styled(PrimitiveStyle::with_fill(Rgb565::new(4,9,8))).draw(d);
            let _ = Text::with_alignment(label, Point::new(205,y+40), value, Alignment::Center).draw(d);
        }
    }
}

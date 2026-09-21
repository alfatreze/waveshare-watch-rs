use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyle, Rectangle, RoundedRectangle}, text::{Alignment, Text}};
use crate::{apps::{App, AppInput, AppResult}, ui::fonts::PIXEL_OPERATOR_MONO_14X24};

pub struct CountdownTimer { remaining_ms: u32, running: bool, alert_pending: bool }
impl CountdownTimer {
    pub fn new() -> Self { Self { remaining_ms: 60_000, running: false, alert_pending: false } }
    pub fn take_alert(&mut self) -> bool { let value = self.alert_pending; self.alert_pending = false; value }
}
impl App for CountdownTimer {
    fn name(&self) -> &str { "Timer" }
    fn setup(&mut self) {}
    fn update(&mut self, input: &AppInput) -> AppResult {
        if let Some(point) = input.touch {
            if point.y > 345 { self.running = !self.running; }
            else if !self.running && point.y > 245 { self.remaining_ms = self.remaining_ms.saturating_add(60_000).min(99 * 60_000); }
            else if !self.running && point.y > 145 { self.remaining_ms = self.remaining_ms.saturating_sub(60_000); }
        }
        if self.running {
            self.remaining_ms = self.remaining_ms.saturating_sub(input.dt_ms);
            if self.remaining_ms == 0 { self.running = false; self.alert_pending = true; }
        }
        AppResult::Continue
    }
    fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(Point::zero(), Size::new(410, 502)).into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK)).draw(d);
        let style = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CYAN);
        let value = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE);
        let _ = Text::with_alignment("TIMER", Point::new(205, 42), style, Alignment::Center).draw(d);
        let secs = self.remaining_ms / 1000;
        let mut time = [b'0'; 5]; time[1] = b'0' + (secs / 60 % 10) as u8; time[2] = b':'; time[3] = b'0' + (secs % 60 / 10) as u8; time[4] = b'0' + (secs % 10) as u8;
        let text = core::str::from_utf8(&time).unwrap_or("00:00");
        let _ = Text::with_alignment(text, Point::new(205, 120), value, Alignment::Center).draw(d);
        for (y, label) in [(145, "-1 MIN"), (245, "+1 MIN"), (345, if self.running { "PAUSE" } else { "START" })] {
            let _ = RoundedRectangle::with_equal_corners(Rectangle::new(Point::new(16,y), Size::new(378,65)), Size::new(12,12)).into_styled(PrimitiveStyle::with_fill(Rgb565::new(4,9,8))).draw(d);
            let _ = Text::with_alignment(label, Point::new(205,y+40), value, Alignment::Center).draw(d);
        }
    }
}

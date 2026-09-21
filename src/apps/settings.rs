// On-watch WiFi setup: one field per screen with a large T9 keyboard.

use embedded_graphics::geometry::Point as EgPoint;
use crate::ui::fonts::PIXEL_OPERATOR_MONO_14X24;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};

use crate::peripherals::wifi::{WifiConfig, WifiState};
use crate::product::settings::WatchSettings;
use crate::ui::t9_keyboard::T9Keyboard;

const ACTION_X: i32 = 278;
const ACTION_Y: i32 = 46;
const ACTION_W: u32 = 116;
const ACTION_H: u32 = 42;

#[derive(Clone, Copy, PartialEq)]
enum SetupStep {
    Ssid,
    Password,
    Review,
}

#[derive(Clone, Copy, PartialEq)]
enum SettingsView {
    Home,
    Wifi,
    TimeZone,
}

pub struct SettingsApp {
    pub wifi_config: WifiConfig,
    pub wifi_state: WifiState,
    pub keyboard: T9Keyboard,
    step: SetupStep,
    view: SettingsView,
    watch_settings: WatchSettings,
    settings_changed: bool,
    connection_request_pending: bool,
}

impl SettingsApp {
    pub fn new() -> Self {
        let mut keyboard = T9Keyboard::new();
        keyboard.show();
        Self {
            wifi_config: WifiConfig::new(),
            wifi_state: WifiState::Disconnected,
            keyboard,
            step: SetupStep::Ssid,
            view: SettingsView::Home,
            watch_settings: WatchSettings::default(),
            settings_changed: false,
            connection_request_pending: false,
        }
    }

    pub fn set_watch_settings(&mut self, settings: WatchSettings) {
        self.watch_settings = settings;
        self.settings_changed = false;
    }

    pub fn take_watch_settings_change(&mut self) -> Option<WatchSettings> {
        if self.settings_changed {
            self.settings_changed = false;
            Some(self.watch_settings)
        } else {
            None
        }
    }

    fn action_hit(x: u16, y: u16) -> bool {
        (x as i32) >= ACTION_X
            && (x as i32) < ACTION_X + ACTION_W as i32
            && (y as i32) >= ACTION_Y
            && (y as i32) < ACTION_Y + ACTION_H as i32
    }

    fn save_active_field(&mut self) {
        match self.step {
            SetupStep::Ssid => self.wifi_config.set_ssid(self.keyboard.get_text()),
            SetupStep::Password => self.wifi_config.set_password(self.keyboard.get_text()),
            SetupStep::Review => {}
        }
    }

    fn edit_ssid(&mut self) {
        self.step = SetupStep::Ssid;
        self.keyboard.set_text(self.wifi_config.ssid_str());
        self.keyboard.show();
        self.wifi_state = WifiState::Disconnected;
    }

    fn edit_password(&mut self) {
        self.step = SetupStep::Password;
        self.keyboard.set_text(self.wifi_config.password_str());
        self.keyboard.show();
        self.wifi_state = WifiState::Disconnected;
    }

    /// Handle tap at screen position. Returns true if consumed.
    pub fn handle_tap(&mut self, x: u16, y: u16) -> bool {
        match self.view {
            SettingsView::Home => return self.handle_home_tap(x, y),
            SettingsView::TimeZone => return self.handle_timezone_tap(x, y),
            SettingsView::Wifi => {}
        }
        if self.wifi_state == WifiState::Connecting || self.wifi_state == WifiState::Connected {
            return true;
        }

        match self.step {
            SetupStep::Ssid => {
                if Self::action_hit(x, y) {
                    self.save_active_field();
                    if self.wifi_config.is_ready() {
                        self.edit_password();
                    } else {
                        self.wifi_state = WifiState::Error;
                    }
                    return true;
                }
                if self.keyboard.handle_tap(x, y) {
                    self.save_active_field();
                    self.wifi_state = WifiState::Disconnected;
                    return true;
                }
            }
            SetupStep::Password => {
                if Self::action_hit(x, y) {
                    self.save_active_field();
                    self.step = SetupStep::Review;
                    self.keyboard.hide();
                    self.wifi_state = WifiState::Disconnected;
                    return true;
                }
                if self.keyboard.handle_tap(x, y) {
                    self.save_active_field();
                    self.wifi_state = WifiState::Disconnected;
                    return true;
                }
            }
            SetupStep::Review => {
                if Self::action_hit(x, y) {
                    if self.wifi_config.is_ready() {
                        self.wifi_state = WifiState::Connecting;
                        self.connection_request_pending = true;
                    } else {
                        self.edit_ssid();
                        self.wifi_state = WifiState::Error;
                    }
                    return true;
                }
                // A large dedicated edit target avoids returning to a dense form.
                if (16..=394).contains(&(x as i32)) && (280..=355).contains(&(y as i32)) {
                    self.edit_password();
                    return true;
                }
                if (16..=394).contains(&(x as i32)) && (190..=265).contains(&(y as i32)) {
                    self.edit_ssid();
                    return true;
                }
            }
        }
        false
    }

    fn handle_home_tap(&mut self, x: u16, y: u16) -> bool {
        if !(16..=394).contains(&(x as i32)) {
            return false;
        }
        match y as i32 {
            92..=157 => {
                self.view = SettingsView::Wifi;
                self.edit_ssid();
                true
            }
            170..=235 => {
                self.watch_settings
                    .set_use_24_hour_clock(!self.watch_settings.use_24_hour_clock());
                self.settings_changed = true;
                true
            }
            248..=313 => {
                self.view = SettingsView::TimeZone;
                true
            }
            _ => false,
        }
    }

    fn handle_timezone_tap(&mut self, x: u16, y: u16) -> bool {
        if (16..=394).contains(&(x as i32)) && (150..=230).contains(&(y as i32)) {
            self.watch_settings.set_utc_offset_seconds(
                self.watch_settings.utc_offset_seconds() - 60 * 60,
            );
            self.settings_changed = true;
            return true;
        }
        if (16..=394).contains(&(x as i32)) && (245..=325).contains(&(y as i32)) {
            self.watch_settings.set_utc_offset_seconds(
                self.watch_settings.utc_offset_seconds() + 60 * 60,
            );
            self.settings_changed = true;
            return true;
        }
        if (16..=394).contains(&(x as i32)) && (360..=435).contains(&(y as i32)) {
            self.view = SettingsView::Home;
            return true;
        }
        false
    }

    pub fn update(&mut self, dt_ms: u32) {
        if self.keyboard.update(dt_ms) {
            self.save_active_field();
        }
    }

    /// Consume the single request generated by the review screen's CONNECT action.
    pub fn take_connection_request(&mut self) -> bool {
        let requested = self.connection_request_pending;
        self.connection_request_pending = false;
        requested
    }

    fn action_label(&self) -> (&'static str, Rgb565) {
        match self.step {
            SetupStep::Ssid => ("NEXT", Rgb565::new(0, 20, 20)),
            SetupStep::Password => ("REVIEW", Rgb565::new(0, 20, 20)),
            SetupStep::Review => match self.wifi_state {
                WifiState::Error => ("RETRY", Rgb565::RED),
                _ => ("CONNECT", Rgb565::new(0, 20, 20)),
            },
        }
    }

    fn draw_action<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let (label, colour) = self.action_label();
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(ACTION_X, ACTION_Y), Size::new(ACTION_W, ACTION_H)),
            Size::new(12, 12),
        )
        .into_styled(PrimitiveStyle::with_fill(colour))
        .draw(d);
        let _ = Text::with_alignment(
            label,
            EgPoint::new(ACTION_X + ACTION_W as i32 / 2, ACTION_Y + 28),
            MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE),
            Alignment::Center,
        )
        .draw(d);
    }

    fn draw_review<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let label = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CSS_GRAY);
        let value = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE);
        let card = Rgb565::new(4, 9, 8);

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(16, 112), Size::new(378, 66)),
            Size::new(12, 12),
        )
        .into_styled(PrimitiveStyle::with_fill(card))
        .draw(d);
        let _ = Text::new("NETWORK", EgPoint::new(28, 136), label).draw(d);
        let ssid = self.wifi_config.ssid_str();
        let _ = Text::new(if ssid.is_empty() { "NOT SET" } else { ssid }, EgPoint::new(28, 160), value).draw(d);

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(16, 190), Size::new(378, 66)),
            Size::new(12, 12),
        )
        .into_styled(PrimitiveStyle::with_fill(card))
        .draw(d);
        let _ = Text::new("PASSWORD", EgPoint::new(28, 214), label).draw(d);
        let pass = if self.wifi_config.pass_len == 0 { "OPEN NETWORK" } else { "********" };
        let _ = Text::new(pass, EgPoint::new(28, 238), value).draw(d);

        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(16, 280), Size::new(378, 62)),
            Size::new(12, 12),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::new(6, 12, 10)))
        .draw(d);
        let _ = Text::with_alignment("EDIT PASSWORD", EgPoint::new(205, 318), value, Alignment::Center).draw(d);
        let _ = Text::with_alignment("SESSION ONLY - NOT SAVED", EgPoint::new(205, 395), label, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Tap a card to edit", EgPoint::new(205, 462), label, Alignment::Center).draw(d);
    }

    fn draw_card<D: DrawTarget<Color = Rgb565>>(
        d: &mut D, y: i32, title: &str, value: &str, accent: Rgb565,
    ) {
        let _ = RoundedRectangle::with_equal_corners(
            Rectangle::new(EgPoint::new(16, y), Size::new(378, 65)), Size::new(12, 12),
        ).into_styled(PrimitiveStyle::with_fill(Rgb565::new(4, 9, 8))).draw(d);
        let _ = Rectangle::new(EgPoint::new(16, y + 8), Size::new(4, 49))
            .into_styled(PrimitiveStyle::with_fill(accent)).draw(d);
        let _ = Text::new(title, EgPoint::new(32, y + 27), MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE)).draw(d);
        let _ = Text::with_alignment(value, EgPoint::new(375, y + 47), MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, accent), Alignment::Right).draw(d);
    }

    fn timezone_label<'a>(&self, buf: &'a mut [u8; 8]) -> &'a str {
        let offset = self.watch_settings.utc_offset_seconds() / 3600;
        let sign = if offset < 0 { b'-' } else { b'+' };
        let value = offset.unsigned_abs() as u8;
        buf[..6].copy_from_slice(b"UTC+00");
        buf[3] = sign;
        buf[4] = b'0' + value / 10;
        buf[5] = b'0' + value % 10;
        core::str::from_utf8(&buf[..6]).unwrap_or("UTC+00")
    }

    fn draw_home<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let title = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CYAN);
        let hint = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CSS_GRAY);
        let _ = Text::new("SETTINGS", EgPoint::new(16, 42), title).draw(d);
        Self::draw_card(d, 92, "WI-FI", "SETUP", Rgb565::CYAN);
        Self::draw_card(d, 170, "CLOCK FORMAT", if self.watch_settings.use_24_hour_clock() { "24 H" } else { "12 H" }, Rgb565::GREEN);
        let mut timezone = [0; 8];
        Self::draw_card(d, 248, "TIME ZONE", self.timezone_label(&mut timezone), Rgb565::YELLOW);
        Self::draw_card(d, 326, "WATCH FACE", "DIGITAL", Rgb565::new(20, 40, 20));
        let _ = Text::with_alignment("Tap a setting to change it", EgPoint::new(205, 452), hint, Alignment::Center).draw(d);
    }

    fn draw_timezone<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let title = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CYAN);
        let value = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE);
        let hint = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CSS_GRAY);
        let mut timezone = [0; 8];
        let _ = Text::new("TIME ZONE", EgPoint::new(16, 42), title).draw(d);
        let _ = Text::with_alignment(self.timezone_label(&mut timezone), EgPoint::new(205, 110), value, Alignment::Center).draw(d);
        Self::draw_card(d, 150, "ONE HOUR", "-", Rgb565::YELLOW);
        Self::draw_card(d, 245, "ONE HOUR", "+", Rgb565::YELLOW);
        let _ = RoundedRectangle::with_equal_corners(Rectangle::new(EgPoint::new(16, 360), Size::new(378, 65)), Size::new(12, 12))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::new(4, 9, 8))).draw(d);
        let _ = Text::with_alignment("DONE", EgPoint::new(205, 400), value, Alignment::Center).draw(d);
        let _ = Text::with_alignment("Applied on the next time sync", EgPoint::new(205, 466), hint, Alignment::Center).draw(d);
    }

    pub fn render<D: DrawTarget<Color = Rgb565>>(&self, d: &mut D) {
        let _ = Rectangle::new(EgPoint::zero(), Size::new(410, 502))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::new(1, 2, 2)))
            .draw(d);

        if self.view == SettingsView::Home {
            self.draw_home(d);
            return;
        }
        if self.view == SettingsView::TimeZone {
            self.draw_timezone(d);
            return;
        }

        let title = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::CYAN);
        let prompt = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::WHITE);
        let _ = Text::new("WI-FI SETUP", EgPoint::new(16, 34), title).draw(d);
        let heading = match self.step {
            SetupStep::Ssid => "1/3  ENTER NETWORK NAME",
            SetupStep::Password => "2/3  ENTER PASSWORD",
            SetupStep::Review => "3/3  REVIEW AND CONNECT",
        };
        let _ = Text::new(heading, EgPoint::new(16, 72), prompt).draw(d);
        self.draw_action(d);

        match self.step {
            SetupStep::Ssid | SetupStep::Password => self.keyboard.render(d),
            SetupStep::Review => self.draw_review(d),
        }

        if self.wifi_state == WifiState::Error {
            let error = MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, Rgb565::RED);
            let message = if self.step == SetupStep::Ssid { "ENTER A NETWORK NAME" } else { "CONNECTION FAILED - EDIT OR RETRY" };
            let _ = Text::with_alignment(message, EgPoint::new(205, 96), error, Alignment::Center).draw(d);
        }
        if self.wifi_state == WifiState::Connecting || self.wifi_state == WifiState::Connected {
            let status = if self.wifi_state == WifiState::Connecting { "CONNECTING..." } else { "CONNECTED" };
            let colour = if self.wifi_state == WifiState::Connecting { Rgb565::YELLOW } else { Rgb565::GREEN };
            let _ = Text::with_alignment(status, EgPoint::new(205, 482), MonoTextStyle::new(&PIXEL_OPERATOR_MONO_14X24, colour), Alignment::Center).draw(d);
        }
    }
}

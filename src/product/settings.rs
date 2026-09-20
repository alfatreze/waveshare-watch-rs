//! Durable user settings and the storage boundary for them.

/// The selected watchface. New designs become new variants; the numeric value
/// is stable so it can be stored without coupling persistence to Rust names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WatchfaceId {
    Digital = 0,
}

/// User-owned choices that should survive restart and firmware updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WatchSettings {
    brightness: u8,
    active_watchface: WatchfaceId,
    use_24_hour_clock: bool,
    utc_offset_seconds: i32,
}

impl Default for WatchSettings {
    fn default() -> Self {
        Self {
            brightness: 0xA0,
            active_watchface: WatchfaceId::Digital,
            use_24_hour_clock: true,
            // UTC is predictable and safe until a user-selectable time zone
            // is implemented. A build-time offset remains available for the
            // current firmware-validation path.
            utc_offset_seconds: 0,
        }
    }
}

impl WatchSettings {
    pub const MIN_BRIGHTNESS: u8 = 0x10;

    pub fn brightness(self) -> u8 {
        self.brightness
    }

    pub fn set_brightness(&mut self, value: u8) {
        self.brightness = value.max(Self::MIN_BRIGHTNESS);
    }

    pub fn active_watchface(self) -> WatchfaceId {
        self.active_watchface
    }

    pub fn use_24_hour_clock(self) -> bool {
        self.use_24_hour_clock
    }

    pub fn utc_offset_seconds(self) -> i32 {
        self.utc_offset_seconds
    }
}

/// Hardware-agnostic persistence contract. The eventual NVS adapter belongs
/// below this model; watchfaces and UI should never write flash directly.
pub trait SettingsStore {
    type Error;

    fn load(&mut self) -> Result<WatchSettings, Self::Error>;
    fn save(&mut self, settings: WatchSettings) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::WatchSettings;

    #[test]
    fn brightness_never_turns_the_panel_off() {
        let mut settings = WatchSettings::default();
        settings.set_brightness(0);
        assert_eq!(settings.brightness(), WatchSettings::MIN_BRIGHTNESS);
    }
}

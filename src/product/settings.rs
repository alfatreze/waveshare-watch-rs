//! Durable user settings and the storage boundary for them.

use embedded_storage::nor_flash::NorFlash;

/// Two dedicated 4 KiB sectors inside the ESP-IDF compatible NVS data
/// partition. The bootloader's default table reserves 0x9000..0xf000 for
/// application data; this firmware owns only the first two sectors.
pub const SETTINGS_SLOT_A: u32 = 0x9000;
pub const SETTINGS_SLOT_B: u32 = 0xa000;
pub const SETTINGS_SLOT_SIZE: u32 = 4096;

const RECORD_MAGIC: [u8; 4] = *b"WWRS";
const RECORD_VERSION: u8 = 2;
const RECORD_LEN: usize = 24;
const V1_RECORD_LEN: usize = 20;

/// The selected watchface. New designs become new variants; the numeric value
/// is stable so it can be stored without coupling persistence to Rust names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WatchfaceId {
    Digital = 0,
}

impl WatchfaceId {
    fn from_storage(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Digital),
            _ => None,
        }
    }
}

/// User-owned choices that should survive restart and firmware updates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WatchSettings {
    brightness: u8,
    active_watchface: WatchfaceId,
    use_24_hour_clock: bool,
    utc_offset_seconds: i32,
    alarm_enabled: bool,
    alarm_hour: u8,
    alarm_minute: u8,
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
            alarm_enabled: false,
            alarm_hour: 7,
            alarm_minute: 0,
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

    pub fn set_active_watchface(&mut self, value: WatchfaceId) {
        self.active_watchface = value;
    }

    pub fn use_24_hour_clock(self) -> bool {
        self.use_24_hour_clock
    }

    pub fn set_use_24_hour_clock(&mut self, value: bool) {
        self.use_24_hour_clock = value;
    }

    pub fn utc_offset_seconds(self) -> i32 {
        self.utc_offset_seconds
    }

    /// Accept offsets within the real-world UTC range (including the uncommon
    /// 30- and 45-minute zones), keeping corrupt settings from producing
    /// surprising clock output.
    pub fn set_utc_offset_seconds(&mut self, value: i32) {
        self.utc_offset_seconds = value.clamp(-12 * 60 * 60, 14 * 60 * 60);
    }

    pub fn alarm_enabled(self) -> bool { self.alarm_enabled }
    pub fn alarm_hour(self) -> u8 { self.alarm_hour }
    pub fn alarm_minute(self) -> u8 { self.alarm_minute }
    pub fn set_alarm(&mut self, enabled: bool, hour: u8, minute: u8) {
        self.alarm_enabled = enabled;
        self.alarm_hour = hour.min(23);
        self.alarm_minute = minute.min(59);
    }
}

/// Hardware-agnostic persistence contract. The eventual NVS adapter belongs
/// below this model; watchfaces and UI should never write flash directly.
pub trait SettingsStore {
    type Error;

    fn load(&mut self) -> Result<WatchSettings, Self::Error>;
    fn save(&mut self, settings: WatchSettings) -> Result<(), Self::Error>;
}

/// A power-loss-tolerant settings store over a reserved flash region.
///
/// Each save erases and writes the *other* slot, leaving the most recent
/// validated record untouched until its replacement is complete. On boot, a
/// damaged or incomplete record is ignored and the newest valid slot wins.
pub struct FlashSettingsStore<F> {
    flash: F,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashSettingsError<E> {
    Flash(E),
}

#[repr(align(4))]
struct AlignedRecord([u8; RECORD_LEN]);

impl<F> FlashSettingsStore<F>
where
    F: NorFlash,
{
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    fn read_slot(&mut self, offset: u32) -> Result<Option<(u32, WatchSettings)>, F::Error> {
        let mut record = AlignedRecord([0; RECORD_LEN]);
        self.flash.read(offset, &mut record.0)?;
        Ok(decode_record(&record.0))
    }

    fn newest(&mut self) -> Result<Option<(u32, u32, WatchSettings)>, F::Error> {
        let a = self.read_slot(SETTINGS_SLOT_A)?;
        let b = self.read_slot(SETTINGS_SLOT_B)?;
        Ok(match (a, b) {
            (None, None) => None,
            (Some((generation, settings)), None) => Some((SETTINGS_SLOT_A, generation, settings)),
            (None, Some((generation, settings))) => Some((SETTINGS_SLOT_B, generation, settings)),
            (Some((a_generation, a_settings)), Some((b_generation, b_settings))) => {
                if generation_is_newer(a_generation, b_generation) {
                    Some((SETTINGS_SLOT_A, a_generation, a_settings))
                } else {
                    Some((SETTINGS_SLOT_B, b_generation, b_settings))
                }
            }
        })
    }
}

impl<F> SettingsStore for FlashSettingsStore<F>
where
    F: NorFlash,
{
    type Error = FlashSettingsError<F::Error>;

    fn load(&mut self) -> Result<WatchSettings, Self::Error> {
        Ok(self
            .newest()
            .map_err(FlashSettingsError::Flash)?
            .map(|(_, _, settings)| settings)
            .unwrap_or_default())
    }

    fn save(&mut self, settings: WatchSettings) -> Result<(), Self::Error> {
        let (target, generation) = match self.newest().map_err(FlashSettingsError::Flash)? {
            Some((SETTINGS_SLOT_A, generation, _)) => (SETTINGS_SLOT_B, generation.wrapping_add(1)),
            Some((SETTINGS_SLOT_B, generation, _)) => (SETTINGS_SLOT_A, generation.wrapping_add(1)),
            Some(_) => unreachable!(),
            None => (SETTINGS_SLOT_A, 1),
        };
        let record = AlignedRecord(encode_record(generation, settings));
        self.flash
            .erase(target, target + SETTINGS_SLOT_SIZE)
            .map_err(FlashSettingsError::Flash)?;
        self.flash
            .write(target, &record.0)
            .map_err(FlashSettingsError::Flash)
    }
}

fn encode_record(generation: u32, settings: WatchSettings) -> [u8; RECORD_LEN] {
    let mut record = [0xff; RECORD_LEN];
    record[..4].copy_from_slice(&RECORD_MAGIC);
    record[4] = RECORD_VERSION;
    record[5] = settings.brightness();
    record[6] = settings.active_watchface() as u8;
    record[7] = u8::from(settings.use_24_hour_clock());
    record[8..12].copy_from_slice(&settings.utc_offset_seconds().to_le_bytes());
    record[12..16].copy_from_slice(&generation.to_le_bytes());
    record[16] = u8::from(settings.alarm_enabled());
    record[17] = settings.alarm_hour();
    record[18] = settings.alarm_minute();
    let record_checksum = checksum(&record[..20]).to_le_bytes();
    record[20..24].copy_from_slice(&record_checksum);
    record
}

fn decode_record(record: &[u8; RECORD_LEN]) -> Option<(u32, WatchSettings)> {
    if record[..4] != RECORD_MAGIC || record[7] > 1 {
        return None;
    }
    let version = record[4];
    let (checksum_end, checksum_start) = match version {
        1 => (16, V1_RECORD_LEN - 4),
        RECORD_VERSION if record[16] <= 1 && record[17] <= 23 && record[18] <= 59 => (20, RECORD_LEN - 4),
        _ => return None,
    };
    let expected_checksum = u32::from_le_bytes(record[checksum_end..checksum_end + 4].try_into().ok()?);
    if checksum(&record[..checksum_start]) != expected_checksum {
        return None;
    }
    let mut settings = WatchSettings::default();
    settings.set_brightness(record[5]);
    settings.set_active_watchface(WatchfaceId::from_storage(record[6])?);
    settings.set_use_24_hour_clock(record[7] != 0);
    settings.set_utc_offset_seconds(i32::from_le_bytes(record[8..12].try_into().ok()?));
    if version == RECORD_VERSION {
        settings.set_alarm(record[16] != 0, record[17], record[18]);
    }
    let generation = u32::from_le_bytes(record[12..16].try_into().ok()?);
    Some((generation, settings))
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c_9dc5, |hash, byte| {
        hash.wrapping_mul(0x0100_0193) ^ u32::from(*byte)
    })
}

fn generation_is_newer(candidate: u32, baseline: u32) -> bool {
    candidate != baseline && candidate.wrapping_sub(baseline) < (1 << 31)
}

#[cfg(test)]
mod tests {
    use super::{decode_record, encode_record, WatchSettings, WatchfaceId};

    #[test]
    fn brightness_never_turns_the_panel_off() {
        let mut settings = WatchSettings::default();
        settings.set_brightness(0);
        assert_eq!(settings.brightness(), WatchSettings::MIN_BRIGHTNESS);
    }

    #[test]
    fn stored_record_round_trips_every_user_choice() {
        let mut settings = WatchSettings::default();
        settings.set_brightness(0x6a);
        settings.set_active_watchface(WatchfaceId::Digital);
        settings.set_use_24_hour_clock(false);
        settings.set_utc_offset_seconds(5 * 60 * 60 + 45 * 60);
        assert_eq!(decode_record(&encode_record(42, settings)), Some((42, settings)));
    }

    #[test]
    fn damaged_record_is_rejected() {
        let mut record = encode_record(7, WatchSettings::default());
        record[5] ^= 1;
        assert_eq!(decode_record(&record), None);
    }
}

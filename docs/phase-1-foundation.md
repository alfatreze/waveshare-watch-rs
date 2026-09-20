# Phase 1 — Foundation and Hardware Validation

## Objective

Establish a reproducible, recoverable baseline on the Waveshare ESP32-S3 Touch AMOLED 2.06 before adding new product features or watchfaces.

## Current baseline

- Source: `waveshare-watch-rs` upstream commit `5eb445b`.
- Branch: `phase-1-foundation`.
- Target: `xtensa-esp32s3-none-elf`.
- Toolchain: Espressif Rust (`esp`), installed with `espup`.
- Release build: succeeds with `cargo build --release`.
- The existing build reports warnings; none currently prevent the target binary from linking. Treat warnings as debt to reduce while touching their owning modules, rather than as an unrelated cleanup exercise.

## Verified device baseline (2026-09-20)

The release binary was flashed to the attached watch over `/dev/cu.usbmodem1101` and restarted through the serial monitor.

| Check | Result |
| --- | --- |
| Bootloader and application boot | Pass |
| Display and TE/VSync initialization | Pass |
| PSRAM framebuffer initialization | Pass |
| FT3168 touch controller initialization | Pass |
| PCF85063A RTC initialization | Pass |
| QMI8658 IMU initialization | Pass |
| AXP2101 power-management initialization | Pass |
| SD card detection | Pass — 15,256 MB detected |
| ES8311 codec and I2S initialization | Pass |
| Wi-Fi/radio initialization | Pass — intentionally disabled until credentials are configured |
| BLE connector initialization | Pass — advertising intentionally disabled |

The serial boot log ends with `All systems GO!`. Screen appearance and touch/tap interaction were confirmed on the physical device. Sleep/wake behavior and NTP remain to be checked.

## Device validation checklist

Run these checks on a physical watch and record pass/fail plus serial output in the issue or pull request that adds the check.

1. Flash and recovery
   - Flash the release binary over USB.
   - Confirm serial monitor access, a clean boot, and a repeatable re-flash path.
2. Display and touch
   - Confirm correct orientation, full-screen rendering, touch coordinates, taps, and swipes.
   - Check wake from a fully-off display using both touch and the physical button.
3. Time and storage
   - Confirm RTC reads/writes survive a reboot.
   - Confirm NTP sync works after Wi-Fi is configured; verify the offline RTC path remains usable.
4. Power and sleep
   - Compare normal, dim, always-on, and display-off states.
   - Measure idle current and wake reliability; do not enable a new animation until its active and idle cost is known.
5. Peripherals
   - Validate PMIC battery/charge readings, IMU, SD detection, and the audio beep independently.
   - Record unavailable or unstable hardware as a known limitation rather than hiding it behind a UI state.

## Architecture guardrails

- Keep hardware drivers and board pin definitions below the UI and application layers.
- Keep rendering in the display/UI path; peripheral tasks publish state rather than drawing directly.
- Watchfaces consume a read-only state snapshot and provide separate active and always-on rendering paths.
- Animation is disabled in always-on and display-off states. Active faces use a bounded frame rate and redraw only changed regions.
- Keep credentials and device-specific configuration out of version control.

## Exit criteria

Phase 1 is complete when the release build is repeatable, the flash/recovery procedure is documented from an actual device run, and the display, touch, RTC, power states, and one Wi-Fi/NTP path have passed their hardware checks.

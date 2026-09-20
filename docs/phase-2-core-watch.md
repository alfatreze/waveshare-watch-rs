# Phase 2 — Core Watch Experience

## Outcome

Turn the verified firmware foundation into a dependable daily watch while keeping the rendering and hardware layers extensible for new watchface designs.

## Delivery order

1. **Durable settings**
   - Store display brightness, 12/24-hour preference, selected watchface, and time-zone choice in NVS.
   - Keep the UI dependent on `product::settings`, never on NVS directly.
2. **Watch utilities**
   - Add persistent alarms, a countdown timer, and a stopwatch.
   - Make alerts wake the display safely and use the existing audio path.
3. **First-run setup**
   - On-device Wi-Fi setup now uses a paged, session-only flow (network name, password, review); persist it only after the flash-storage safety spike.
   - Add time-zone selection and status/error feedback.
4. **Watchface platform**
   - A stable registry and selection screen.
   - Each face supplies active and always-on render paths, a frame budget, and an optional animation policy.
   - The existing digital face remains the baseline until supplied designs define the next faces.

## Initial boundary

`product::settings` is the sole owner of user choices. Its `SettingsStore` trait makes the eventual NVS implementation replaceable and prevents UI or watchfaces from directly managing flash storage.

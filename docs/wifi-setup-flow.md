# On-watch Wi-Fi setup flow

## Functional model

The wearer enters one network name and, if needed, one password. The watch applies that configuration to the radio for the current session, reports connection progress or failure, and can retry. Credentials are not written to flash yet.

## Information architecture and flow

1. **Network name** — a single editable value and a full-width keyboard; `NEXT` validates that a name exists.
2. **Password** — a single editable value and the same keyboard; `REVIEW` advances without requiring a password so open networks remain possible.
3. **Review** — shows network name and a masked password, with large cards to edit either value and `CONNECT` / `RETRY` to submit.

Auto-discovery is intentionally deferred. It will become the first screen in this flow once scanning is reliable and its radio/power cost is understood.

## Interaction and accessibility decisions

- One editable field per screen preserves vertical room for the keyboard and avoids a bottom-edge target.
- Keys are 120 by 72 pixels, separated by 8-pixel gaps, with 12-pixel rounded corners. Hit testing rejects the gaps rather than assigning them to a nearby key.
- The near-black backdrop and lighter key surfaces make individual targets visible without using colour as the only cue.
- Labels use the firmware's largest current mono font. Short labels, large buttons, and fewer simultaneous controls improve legibility.
- Connection state is stated in words (`CONNECTING`, `CONNECTED`, or an error instruction) as well as colour.

## State inventory

| State | Display | Recovery |
| --- | --- | --- |
| Empty network name | Page 1 with an explicit prompt after `NEXT` | Enter a name and tap `NEXT` |
| Editing network name | Page 1 keyboard | `NEXT` |
| Editing password | Page 2 keyboard | `REVIEW` |
| Ready to connect | Page 3 review | Edit a card or tap `CONNECT` |
| Connecting | Page 3 with textual progress | Wait for bounded network timeout |
| Connection failed | Page 3 with textual failure and `RETRY` | Edit credentials or retry |
| Connected | Page 3 with textual confirmation | Exit setup normally |

This is an accessibility-informed touch layout; it still needs physical-device validation with the final panel calibration.

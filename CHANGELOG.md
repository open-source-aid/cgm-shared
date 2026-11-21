# Changelog

## 0.1.0 (unreleased)

### API

- `CGM_API_VERSION = 2`. `CgmCommand::Pair` gains `pairing_secret: Option<Secret>`: the
  short secret a person reads off the device or its packaging when the protocol needs one
  to establish a credential — a pairing code on an applicator, a transmitter id, a sensor
  code read by NFC. It is not the credential and does not replace it: a credential is
  negotiated or derived and kept for the life of the sensor, this is typed once and then
  spent. Carrying both in `credential` would make a driver guess which it was handed.
  Drivers that need no such secret ignore it. Incompatible: hosts constructing `Pair` must
  name the new field.
- First version: `CGM_API_VERSION = 1`. Glucose, sensor-state, command and engine types
  implementing the guideline 06 CGM Proposal; glucose generators behind `sim`.

### Device-visible

- None (this crate talks to no device).

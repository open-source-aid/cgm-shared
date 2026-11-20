# Design notes

- **Why a separate `CgmEngine` trait.** `pump_shared::Engine: TherapyMachine` and speaks
  `pump_shared::Command`/`Event`. A CGM has no therapy state, and implementing a dummy one
  would let a CGM engine be handed to pump code. Open: Q5.
- **Why `Quality` flags instead of dropping readings.** Every app wants the same reading
  stream; what it forwards (warm-up, implausible, device-flagged) is app policy, but the
  reason must be decided once, in the driver.
- **Why `BackfillAvailable` instead of automatic backfill.** E3 says `submit` queues and the
  engine performs no work that was not submitted. The engine detects the gap; the adapter
  submits `ReadReadings { after }`.
- **Rate unit.** Tenths of mg/dL per minute in an `i16` (±3276.7 mg/dL/min), integers only.

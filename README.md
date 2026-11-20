# cgm-shared

Shared contract for sans-IO CGM drivers under the OS-AID driver guideline.

> **Experimental. Implements guideline Proposals.** The CGM data model (guideline
> 06, "CGM data model") and the `cgm-shared` crate (guideline 03, "Shared crates")
> are Proposals the group has not agreed on. This crate exists so a first CGM
> driver (`cgm-sibionics`) can be written against them and the Proposal can be
> judged from working code.

## Contents

| Item | Guideline |
|---|---|
| `MgDl(u16)`, `MgDl::from_mmol_tenths` (18.0182, rounded once) | 06, Glucose |
| `GlucoseReading`, `Trend`, `Rate` (tenths of mg/dL per minute), `Quality` flags | 06, CGM data model |
| `SensorState`, `Calibration` | 06, CGM data model |
| `CgmCommand`, `CgmOutcome`, `CgmReport`, `CgmFailure` | 06, Commands |
| `CgmEngine`, `CgmEvent`, `CgmDriver`, `CgmDriverInfo`, `CGM_API_VERSION` | 04 |
| `sim::GlucoseGenerator` with `Constant`, `Ramp`, `Replay`, `WithGaps` (feature `sim`) | 11 |

## Decisions taken provisionally

| Open question | What this crate does |
|---|---|
| Q5 / Q11 (splitting `pump-shared`) | Depends on `pump-shared` and re-exports its generic host contract (`Action`, `Completion`, `BleOp`, `Slot`, `Secret`, ...). `CgmEngine` mirrors `pump_shared::Engine` without `TherapyMachine`. `CgmDriverInfo` wraps `DriverInfo` with an empty pump `commands` list |
| Q1 (auxiliary inputs) | No `InputRequired`; pairing inputs known up front are `CgmCommand::Pair` parameters |

Readings are always reported, including unusable ones; `Quality` says why a
reading should not be used, and the adapter decides what to forward. Gaps are
announced with `CgmEvent::BackfillAvailable` and fetched with
`CgmCommand::ReadReadings`, so the engine never starts device work on its own
(E3).

## Checks

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo build --no-default-features --target thumbv7em-none-eabihf
cargo build --no-default-features --features sim --target thumbv7em-none-eabihf
cargo llvm-cov --all-features --fail-under-lines 100 --fail-under-regions 100 --fail-under-functions 100
```

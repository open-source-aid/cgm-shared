//! Shared contract for sans-IO CGM drivers.
//!
//! **Proposal.** This crate implements the CGM data model that guideline 06
//! lists as a Proposal, and the `cgm-shared` crate that guideline 03 proposes.
//! The group has not agreed on either yet.
//!
//! Open: Q5/Q11. The generic host contract is not split out of `pump-shared`
//! yet, so this crate depends on it and re-exports the generic part.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod command;
mod driver;
mod engine;
mod glucose;
mod sensor;
#[cfg(feature = "sim")]
pub mod sim;

pub use command::{
    CgmCommand, CgmCommandKind, CgmFailure, CgmOutcome, CgmReport, CgmStatus, PairReport,
};
pub use driver::{CgmDriver, CgmDriverInfo, CGM_API_VERSION};
pub use engine::{CgmDeviceInfo, CgmEngine, CgmEvent};
pub use glucose::{GlucoseReading, MgDl, Quality, Rate, Trend};
pub use sensor::{Calibration, SensorState};

// The generic host contract, re-exported from `pump-shared` (Open: Q5).
#[cfg(feature = "sim")]
pub use pump_shared::SimDevice;
pub use pump_shared::{
    Action, Address, Advertisement, BleMatch, BleOp, BleResult, Class, CommandId, Completion,
    CompletionError, Config, DriverInfo, EngineState, Feature, GattChar, Kind, Notification,
    NotifyKind, OpId, Refusal, Secret, Sequence, Slot, SubmitError, Timestamp, UtcTime, Uuid,
    Variant,
};

/// Re-exported so a driver depends on one shared crate only.
pub use pump_shared;

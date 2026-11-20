//! The CGM engine contract (guideline 04), without the pump-only
//! `TherapyMachine`.
//!
//! Open: Q5. `pump_shared::Engine` requires `TherapyMachine` and speaks the
//! pump `Command`/`Event`. Until the generic contract is split into its own
//! crate, CGM engines implement this trait instead. Its methods, and the rules
//! E1–E10 that govern them, are the same as `pump_shared::Engine`'s.

use alloc::boxed::Box;
use alloc::string::String;

use pump_shared::{
    Action, CommandId, Completion, CompletionError, EngineState, Notification, OpId, Sequence,
    SubmitError, Timestamp, UtcTime,
};

use crate::{CgmCommand, CgmOutcome, GlucoseReading, SensorState};

/// What the device reported about itself after a connect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CgmDeviceInfo {
    /// The serial used for deduplication (`<serial>-<sequence>`).
    pub serial: String,
    pub firmware: String,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub variant: &'static str,
    pub activation: Option<UtcTime>,
    /// Warm-up and wear durations in seconds, as the driver knows them for
    /// this device. Hard-coded values are documented by the driver.
    pub warmup_secs: u32,
    pub wear_secs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgmEvent {
    Started {
        id: CommandId,
    },
    Finished {
        id: CommandId,
        outcome: CgmOutcome,
    },
    /// A reading pushed by the device while connected.
    Reading(GlucoseReading),
    SensorStateChanged(SensorState),
    /// The device has readings after `after` that this engine has not
    /// reported (a gap). The adapter submits `ReadReadings { after }` to
    /// fetch them; the engine never starts device work on its own (E3).
    BackfillAvailable {
        after: Sequence,
    },
    DeviceInfo(Box<CgmDeviceInfo>),
    StateChanged(EngineState),
    Log {
        level: u8,
        line: String,
    },
}

/// Sans-IO, synchronous, poll-based (R5, R6, E2). See guideline 04 for the
/// host loop and rules E1–E10.
pub trait CgmEngine {
    fn submit(&mut self, command: CgmCommand) -> Result<CommandId, SubmitError>;

    fn stop(&mut self) -> CommandId;

    fn handle_completion(
        &mut self,
        id: OpId,
        completion: Completion,
    ) -> Result<(), CompletionError>;

    fn handle_timeout(&mut self, now: Timestamp);

    fn handle_wall_clock(&mut self, now: UtcTime);

    fn handle_link_lost(&mut self);

    fn handle_notification(&mut self, notification: Notification);

    fn poll_action(&mut self) -> Option<Action>;

    fn poll_event(&mut self) -> Option<CgmEvent>;

    fn poll_timeout(&self) -> Option<Timestamp>;

    fn state(&self) -> EngineState;
}

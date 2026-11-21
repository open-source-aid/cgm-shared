//! CGM commands and their outcomes (guideline 06, "CGM data model", Commands).

use alloc::string::String;
use alloc::vec::Vec;

use pump_shared::{Address, Refusal, Secret, Sequence, UtcTime};

use crate::{GlucoseReading, MgDl, SensorState};

/// A device operation. Like pump commands, none of these know anything about
/// dosing (R1).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgmCommand {
    /// Connect if needed and report the sensor's status. A connected engine
    /// keeps receiving live readings until the link drops.
    Status,
    /// Bind to a sensor and, if it is factory-new, start it.
    ///
    /// Inputs the user knows up front are parameters (07, auxiliary inputs);
    /// anything derived from the link stays inside the engine. Open: Q1
    /// (no mid-flow `InputRequired`).
    Pair {
        address: Address,
        /// The device-specific credential binding this app to the sensor, if
        /// one already exists. `None` lets the driver create one where the
        /// protocol allows it; the result reports it so the app can keep it.
        credential: Option<Secret>,
        /// The secret the user reads off the device or its packaging, when
        /// the protocol needs one to establish a credential: a pairing code
        /// printed on an applicator, a transmitter id, a sensor code read by
        /// NFC. Drivers that need none ignore it.
        ///
        /// This is not the credential and does not replace it. A credential
        /// is negotiated or derived, is usually long, and is kept for the life
        /// of the sensor; this is short, is typed or scanned by a person, is
        /// used once to get the credential, and is then of no further use.
        /// Carrying both in one field would make a driver guess which it was
        /// handed, and a wrong guess would either leak a reusable key into a
        /// one-shot exchange or treat a four-character code as a session key.
        ///
        /// It is a parameter rather than something the engine asks for
        /// mid-flow, because the user knows it before the exchange starts
        /// (07, auxiliary inputs). Open: Q1.
        pairing_secret: Option<Secret>,
        /// A serial read from the sensor's packaging or applicator (NFC), to
        /// check the sensor in range is the one the user means.
        expected_serial: Option<String>,
        /// When the app already knows the sensor's start time.
        known_activation: Option<UtcTime>,
    },
    Unpair,
    /// Readings strictly after `after`, from device history (backfill).
    ReadReadings {
        after: Sequence,
    },
    StartSensor,
    StopSensor,
    Calibrate {
        value: MgDl,
        time: UtcTime,
    },
}

impl CgmCommand {
    pub fn kind(&self) -> CgmCommandKind {
        match self {
            Self::Status => CgmCommandKind::Status,
            Self::Pair { .. } => CgmCommandKind::Pair,
            Self::Unpair => CgmCommandKind::Unpair,
            Self::ReadReadings { .. } => CgmCommandKind::ReadReadings,
            Self::StartSensor => CgmCommandKind::StartSensor,
            Self::StopSensor => CgmCommandKind::StopSensor,
            Self::Calibrate { .. } => CgmCommandKind::Calibrate,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CgmCommandKind {
    Status,
    Pair,
    Unpair,
    ReadReadings,
    StartSensor,
    StopSensor,
    Calibrate,
    Stop,
}

/// Snapshot returned by `Status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CgmStatus {
    pub state: SensorState,
    pub activation: Option<UtcTime>,
    /// The newest measurement index the device reports.
    pub latest_sequence: Option<Sequence>,
    /// Battery level in percent, when the device reports one.
    pub battery_percent: Option<u8>,
}

/// Result of a successful `Pair`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairReport {
    /// The credential now bound to the sensor (the one passed in, or the one
    /// the driver created). The app stores it in its keychain/keystore.
    pub credential: Secret,
    pub activation: Option<UtcTime>,
    /// True when this `Pair` started a factory-new sensor.
    pub started_sensor: bool,
    /// The product serial the sensor reported, if it was asked.
    pub product_serial: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgmReport {
    Done,
    Status(CgmStatus),
    Paired(PairReport),
    Readings(Vec<GlucoseReading>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CgmFailure {
    /// The sensor refused our credential.
    KeyRejected,
    /// The device answered with an error status.
    Status(u8),
    /// A host action (storage, randomness) failed.
    Host,
    Disconnected,
    /// The device sent something the protocol does not allow.
    Protocol,
    /// The device in range is not the one asked for (serial mismatch).
    WrongDevice,
    /// The device speaks a protocol generation this driver does not.
    UnsupportedDevice,
    /// The device did not answer in time.
    Timeout,
}

/// Exactly one per `submit` (E5).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CgmOutcome {
    Completed(CgmReport),
    Refused(Refusal),
    /// The command may or may not have taken effect on the device (E8). For
    /// a CGM this matters for `Pair` (sensor start), which is single-use.
    Unknown,
    Failed(CgmFailure),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_command_has_a_kind() {
        let t = UtcTime::from_unix_millis(0);
        let cases = [
            (CgmCommand::Status, CgmCommandKind::Status),
            (
                CgmCommand::Pair {
                    address: Address::new([0; 6]),
                    credential: None,
                    pairing_secret: None,
                    expected_serial: None,
                    known_activation: None,
                },
                CgmCommandKind::Pair,
            ),
            (CgmCommand::Unpair, CgmCommandKind::Unpair),
            (
                CgmCommand::ReadReadings {
                    after: Sequence::new(1),
                },
                CgmCommandKind::ReadReadings,
            ),
            (CgmCommand::StartSensor, CgmCommandKind::StartSensor),
            (CgmCommand::StopSensor, CgmCommandKind::StopSensor),
            (
                CgmCommand::Calibrate {
                    value: MgDl(100),
                    time: t,
                },
                CgmCommandKind::Calibrate,
            ),
        ];
        for (command, kind) in cases {
            assert_eq!(command.kind(), kind);
        }
    }
}

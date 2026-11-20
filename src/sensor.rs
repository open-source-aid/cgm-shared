//! Sensor lifecycle and calibration (guideline 06, "CGM data model").

use pump_shared::UtcTime;

use crate::MgDl;

/// Where the sensor is in its life. Driver-computed from what the device
/// reports, so every app shows the same state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorState {
    /// Readings are not yet reliable.
    WarmingUp { until: UtcTime },
    /// Producing readings.
    Active { expires: UtcTime },
    /// Past its wear period.
    Expired,
    /// The device or the driver declared the sensor unusable. The code is
    /// driver-defined and documented by the driver.
    Failed(u16),
    /// The sensor is not producing readings even though one is expected.
    SignalLost { since: UtcTime },
    /// Not enough is known (no activation time, no wall clock).
    Unknown,
}

/// A calibration entered by the user and sent to the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Calibration {
    pub value: MgDl,
    pub time: UtcTime,
    /// Whether the device accepted it; `None` until it has answered.
    pub accepted: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_compare_by_value() {
        let t = UtcTime::from_unix_millis(1);
        assert_eq!(
            SensorState::WarmingUp { until: t },
            SensorState::WarmingUp { until: t }
        );
        assert_ne!(SensorState::Active { expires: t }, SensorState::Expired);
        assert_ne!(SensorState::Failed(1), SensorState::Unknown);
        assert_ne!(SensorState::SignalLost { since: t }, SensorState::Unknown);
        let c = Calibration {
            value: MgDl(100),
            time: t,
            accepted: None,
        };
        assert_eq!(c.accepted, None);
    }
}

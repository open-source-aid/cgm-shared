//! Glucose values and readings (guideline 06, "Glucose" and "CGM data model").

use alloc::vec::Vec;

use pump_shared::{Sequence, UtcTime};

/// Integer mg/dL. The only glucose unit in the contract (06): apps convert
/// mmol/L at the user-interface edge, drivers at the protocol boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MgDl(pub u16);

impl MgDl {
    /// Numerator and denominator of 18.0182 mg/dL per mmol/L, the factor the
    /// guideline mandates (06, "Conversion factor").
    const FACTOR_NUM: u64 = 180_182;
    const FACTOR_DEN: u64 = 10_000;

    pub const fn get(self) -> u16 {
        self.0
    }

    /// Converts a device value in tenths of mmol/L to mg/dL, rounding to the
    /// nearest integer once (06). Exact integer arithmetic, no floating point.
    ///
    /// The largest input, `u16::MAX` tenths (6553.5 mmol/L), is about
    /// 118 000 mg/dL, which does not fit; such a value is not physiological and
    /// gives `None` rather than a clamped number (06: unknown is `None`).
    pub fn from_mmol_tenths(tenths: u16) -> Option<Self> {
        // tenths/10 * 18.0182 = tenths * 180182 / 100000
        let scaled = u64::from(tenths) * Self::FACTOR_NUM;
        let denominator = Self::FACTOR_DEN * 10;
        let rounded = (scaled + denominator / 2) / denominator;
        u16::try_from(rounded).ok().map(Self)
    }
}

/// Rate of change in tenths of mg/dL per minute (integers only, 06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rate(pub i16);

impl Rate {
    pub const fn deci_mgdl_per_min(self) -> i16 {
        self.0
    }
}

/// Five-step trend arrow, the set Loop, Trio and AAPS can all show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trend {
    FallingQuickly,
    Falling,
    Stable,
    Rising,
    RisingQuickly,
}

/// Why a reading should not be used for therapy, as flags. An empty set means
/// the reading is usable. Readings are always reported (06: nothing is
/// silently dropped); the adapter decides what to forward.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Quality(u8);

impl Quality {
    /// The sensor is still warming up.
    pub const WARMUP: Self = Self(1 << 0);
    /// The sensor is past its wear period.
    pub const EXPIRED: Self = Self(1 << 1);
    /// The driver's plausibility check considers the signal implausible.
    pub const IMPLAUSIBLE: Self = Self(1 << 2);
    /// The device itself flagged the record.
    pub const DEVICE_FLAGGED: Self = Self(1 << 3);
    /// The sensor has failed.
    pub const SENSOR_FAILED: Self = Self(1 << 4);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn is_usable(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// One CGM measurement (06, `GlucoseReading`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlucoseReading {
    /// The device's own index for this measurement; the deduplication key
    /// together with the sensor serial.
    pub sequence: Sequence,
    /// When the measurement was taken, in UTC.
    pub time: UtcTime,
    /// `None` when the device produced no value for this slot (06: no
    /// sentinel numbers).
    pub value: Option<MgDl>,
    pub trend: Option<Trend>,
    pub rate: Option<Rate>,
    pub quality: Quality,
    /// Read from device history (backfill) rather than pushed live.
    pub historical: bool,
    /// The record as the device sent it, for audit and later re-decoding.
    pub raw: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn converts_with_the_guideline_factor() {
        assert_eq!(MgDl::from_mmol_tenths(0), Some(MgDl(0)));
        // 5.5 mmol/L * 18.0182 = 99.1 -> 99
        assert_eq!(MgDl::from_mmol_tenths(55), Some(MgDl(99)));
        // 5.6 * 18.0182 = 100.9 -> 101
        assert_eq!(MgDl::from_mmol_tenths(56), Some(MgDl(101)));
        // 2.2 * 18.0182 = 39.64 -> 40
        assert_eq!(MgDl::from_mmol_tenths(22), Some(MgDl(40)));
        // 22.2 * 18.0182 = 400.0 -> 400
        assert_eq!(MgDl::from_mmol_tenths(222), Some(MgDl(400)));
        assert_eq!(MgDl(7).get(), 7);
    }

    #[test]
    fn refuses_values_that_do_not_fit() {
        // 36371 tenths -> 65534.0 mg/dL, the last that fits; 36372 -> 65535.8,
        // which rounds to 65536.
        assert_eq!(MgDl::from_mmol_tenths(36_371), Some(MgDl(65_534)));
        assert_eq!(MgDl::from_mmol_tenths(36_372), None);
        assert_eq!(MgDl::from_mmol_tenths(u16::MAX), None);
    }

    #[test]
    fn quality_flags_combine() {
        let q = Quality::empty();
        assert!(q.is_usable());
        let q = q.with(Quality::WARMUP).with(Quality::IMPLAUSIBLE);
        assert!(!q.is_usable());
        assert!(q.contains(Quality::WARMUP));
        assert!(q.contains(Quality::IMPLAUSIBLE));
        assert!(!q.contains(Quality::EXPIRED));
        assert_eq!(Quality::from_bits(q.bits()), q);
        assert_eq!(Quality::default(), Quality::empty());
        assert!(Quality::SENSOR_FAILED.contains(Quality::SENSOR_FAILED));
        assert!(Quality::DEVICE_FLAGGED.contains(Quality::DEVICE_FLAGGED));
    }

    #[test]
    fn rate_exposes_its_unit() {
        assert_eq!(Rate(-25).deci_mgdl_per_min(), -25);
    }

    proptest! {
        #[test]
        fn conversion_is_monotonic_and_within_half_a_unit(t in 0u16..36_371) {
            let a = MgDl::from_mmol_tenths(t).unwrap().get();
            let b = MgDl::from_mmol_tenths(t + 1).unwrap().get();
            prop_assert!(b >= a);
            // |a - t * 1.80182| <= 0.5, checked in integers (x 100000).
            let exact = u64::from(t) * 180_182;
            let ours = u64::from(a) * 100_000;
            prop_assert!(ours.abs_diff(exact) <= 50_000);
        }
    }
}

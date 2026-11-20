//! Glucose generators for CGM simulators and tests (guideline 11, Proposal).
//!
//! Deterministic: the value depends only on `t`. `Sine` and `Noisy` from the
//! guideline's table are not implemented yet; they need either floating-point
//! maths or a seeded generator and no CGM simulator needs them so far.

use alloc::boxed::Box;
use alloc::vec::Vec;

use pump_shared::Timestamp;

use crate::MgDl;

pub trait GlucoseGenerator {
    /// Glucose at `t`, or `None` for a gap (sensor error, signal loss).
    fn at(&mut self, t: Timestamp) -> Option<MgDl>;
}

/// The same value forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constant(pub MgDl);

impl GlucoseGenerator for Constant {
    fn at(&mut self, _t: Timestamp) -> Option<MgDl> {
        Some(self.0)
    }
}

/// A straight line from `from` at t = 0 to `to` at `over_ms`, then `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ramp {
    pub from: MgDl,
    pub to: MgDl,
    pub over_ms: u64,
}

impl GlucoseGenerator for Ramp {
    fn at(&mut self, t: Timestamp) -> Option<MgDl> {
        let t = t.as_millis().min(self.over_ms);
        if self.over_ms == 0 {
            return Some(self.to);
        }
        let from = i64::from(self.from.get());
        let span = i64::from(self.to.get()) - from;
        // |span| <= u16::MAX and t <= over_ms, so this cannot overflow i128.
        let offset = i128::from(span) * i128::from(t) / i128::from(self.over_ms);
        let value = i128::from(from) + offset;
        u16::try_from(value).ok().map(MgDl)
    }
}

/// A recorded trace of `(time, value)` points, held until the next point.
/// Before the first point there is no value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replay(pub Vec<(Timestamp, Option<MgDl>)>);

impl GlucoseGenerator for Replay {
    fn at(&mut self, t: Timestamp) -> Option<MgDl> {
        self.0
            .iter()
            .take_while(|(at, _)| *at <= t)
            .last()
            .and_then(|(_, value)| *value)
    }
}

/// Another generator with gaps `[start, end)` cut out of it.
pub struct WithGaps {
    pub inner: Box<dyn GlucoseGenerator>,
    pub gaps: Vec<(Timestamp, Timestamp)>,
}

impl GlucoseGenerator for WithGaps {
    fn at(&mut self, t: Timestamp) -> Option<MgDl> {
        if self.gaps.iter().any(|(start, end)| *start <= t && t < *end) {
            return None;
        }
        self.inner.at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn at(ms: u64) -> Timestamp {
        Timestamp::from_millis(ms)
    }

    #[test]
    fn constant_is_constant() {
        let mut g = Constant(MgDl(120));
        assert_eq!(g.at(at(0)), Some(MgDl(120)));
        assert_eq!(g.at(at(u64::MAX)), Some(MgDl(120)));
    }

    #[test]
    fn ramp_moves_linearly_then_holds() {
        let mut up = Ramp {
            from: MgDl(100),
            to: MgDl(200),
            over_ms: 1000,
        };
        assert_eq!(up.at(at(0)), Some(MgDl(100)));
        assert_eq!(up.at(at(500)), Some(MgDl(150)));
        assert_eq!(up.at(at(5000)), Some(MgDl(200)));
        let mut down = Ramp {
            from: MgDl(200),
            to: MgDl(100),
            over_ms: 1000,
        };
        assert_eq!(down.at(at(250)), Some(MgDl(175)));
        let mut instant = Ramp {
            from: MgDl(1),
            to: MgDl(2),
            over_ms: 0,
        };
        assert_eq!(instant.at(at(0)), Some(MgDl(2)));
    }

    #[test]
    fn replay_holds_each_point() {
        let mut g = Replay(vec![(at(10), Some(MgDl(90))), (at(20), None)]);
        assert_eq!(g.at(at(0)), None);
        assert_eq!(g.at(at(10)), Some(MgDl(90)));
        assert_eq!(g.at(at(19)), Some(MgDl(90)));
        assert_eq!(g.at(at(25)), None);
    }

    #[test]
    fn gaps_cut_holes() {
        let mut g = WithGaps {
            inner: Box::new(Constant(MgDl(100))),
            gaps: vec![(at(10), at(20))],
        };
        assert_eq!(g.at(at(9)), Some(MgDl(100)));
        assert_eq!(g.at(at(10)), None);
        assert_eq!(g.at(at(20)), Some(MgDl(100)));
    }
}

//! Static description of a CGM driver.

use alloc::boxed::Box;

use pump_shared::{Advertisement, Config, DriverInfo, Kind, Variant};

use crate::{CgmCommandKind, CgmEngine};

/// Version of the CGM contract in this crate. Bumped on any incompatible
/// change, independently of `pump_shared::API_VERSION`.
pub const CGM_API_VERSION: u16 = 1;

/// `DriverInfo` plus the CGM command set.
///
/// Open: Q5. `DriverInfo.commands` lists pump commands; a CGM driver leaves it
/// empty and lists its commands in `commands` here. `device.kind` is
/// `Kind::Cgm`. Everything else in `device` (GATT table, slots, features,
/// variants) means exactly what it does for a pump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CgmDriverInfo {
    pub device: DriverInfo,
    pub cgm_api_version: u16,
    pub commands: &'static [CgmCommandKind],
}

impl CgmDriverInfo {
    pub fn accepts(&self, kind: CgmCommandKind) -> bool {
        self.commands.contains(&kind)
    }

    /// Both contracts this driver was built against match this build.
    pub fn is_compatible(&self) -> bool {
        self.device.kind == Kind::Cgm
            && self.device.is_compatible()
            && self.device.commands.is_empty()
            && self.cgm_api_version == CGM_API_VERSION
    }
}

pub trait CgmDriver: Sync {
    fn info(&self) -> &'static CgmDriverInfo;

    fn matches(&self, adv: &Advertisement<'_>) -> Option<&'static Variant>;

    fn new_engine(&self, variant: &'static Variant, config: Config) -> Box<dyn CgmEngine>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pump_shared::{BleMatch, CommandKind};

    const fn info(kind: Kind, pump: &'static [CommandKind], cgm: u16) -> CgmDriverInfo {
        CgmDriverInfo {
            device: DriverInfo {
                id: "t",
                name: "T",
                kind,
                version: "0",
                api_version: pump_shared::API_VERSION,
                protocol: "t",
                ble: BleMatch {
                    name_prefixes: &[],
                    service_uuids: &[],
                    manufacturer_id: None,
                },
                gatt: &[],
                variants: &[],
                commands: pump,
                features: &[],
                slots: &[],
            },
            cgm_api_version: cgm,
            commands: &[CgmCommandKind::Status],
        }
    }

    #[test]
    fn checks_both_contracts() {
        let good = info(Kind::Cgm, &[], CGM_API_VERSION);
        assert!(good.is_compatible());
        assert!(good.accepts(CgmCommandKind::Status));
        assert!(!good.accepts(CgmCommandKind::Calibrate));
        assert!(!info(Kind::Pump, &[], CGM_API_VERSION).is_compatible());
        assert!(!info(Kind::Cgm, &[CommandKind::Status], CGM_API_VERSION).is_compatible());
        assert!(!info(Kind::Cgm, &[], CGM_API_VERSION + 1).is_compatible());
    }
}

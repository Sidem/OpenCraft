//! What only one kind of processor keeps (`Processor::extra`): a pumpjack's well, a research center's unit, a recycler's
//! debt, a launch site's count, a depot's wants, a drone port's fleet. Chosen from the spec when the processor is made
//! (`Extra::new`), so a processor never carries another kind's state.
//!
//! Invariant: the variant follows the spec alone (pick, or the launch site's block), so a kind's file may match on it in
//! place of checking the pick. The kinds' save bytes come after the shared ones, at most one kind's per processor.
//!
//! To give a kind state of its own: a variant here, its arm in `new`, and (if saved) in `write_state` and `read_state`,
//! read behind a version check.

use crate::block::LAUNCH_SITE;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::ItemId;
use crate::pods::CHANNELS;

use super::center::Study;
use super::depot::Want;
use super::hangar::Hangar;
use super::pods::CHANNEL_SAVED;
use super::pump::Pump;
use super::{Pick, ProcessSpec};

pub enum Extra {
    None,
    /// A pumpjack's well and the oil kept towards a canister (pump.rs; saved).
    Pump(Pump),
    /// A research center's or AI lab's unit in progress (center.rs; saved).
    Study(Study),
    /// Millicoins a recycler has earned and not yet paid out (recycler.rs; saved).
    Recycler {
        owed: u32,
    },
    /// Rockets a launch site has sent up (saved), and the satellite it carried up this tick until `Factory::update`
    /// puts it in orbit (launch.rs; derived).
    Launch {
        launches: u32,
        lifted: Option<ItemId>,
    },
    /// What a town depot takes in and up to how many it holds of each (depot.rs; derived, set by `Sim::sync_depots`).
    Depot {
        wants: Vec<Want>,
    },
    /// A drone port's fleet out and what helps it this tick (hangar.rs; derived).
    Hangar(Hangar),
    /// The channel a pod launcher or landing pad is tuned to (pods/channels.rs; saved from `pods::CHANNEL_SAVED`).
    Channel(u8),
    /// What a belt ejector threw out this tick, until `report_made` (ejector.rs; derived).
    Ejector(Vec<Stack>),
}

impl Extra {
    pub(super) fn new(spec: &ProcessSpec) -> Extra {
        match spec.pick {
            Pick::Pump => Extra::Pump(Pump::default()),
            Pick::Research => Extra::Study(Study::default()),
            Pick::Recycle => Extra::Recycler { owed: 0 },
            Pick::Depot => Extra::Depot { wants: Vec::new() },
            Pick::Hangar => Extra::Hangar(Hangar::default()),
            Pick::Pods | Pick::Land => Extra::Channel(0),
            Pick::Eject => Extra::Ejector(Vec::new()),
            _ if spec.block == LAUNCH_SITE => Extra::Launch { launches: 0, lifted: None },
            _ => Extra::None,
        }
    }

    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        match self {
            Extra::Pump(pump) => pump.write_state(w),
            Extra::Study(study) => study.write_state(w),
            Extra::Recycler { owed } => w.u32(*owed),
            Extra::Launch { launches, .. } => w.u32(*launches),
            Extra::Channel(channel) => w.u8(*channel),
            Extra::None | Extra::Depot { .. } | Extra::Hangar(_) | Extra::Ejector(_) => {}
        }
    }

    pub(super) fn read_state(&mut self, r: &mut ByteReader) -> Option<()> {
        match self {
            Extra::Pump(pump) => *pump = Pump::read_state(r)?,
            Extra::Study(study) => study.read_state(r)?,
            Extra::Recycler { owed } => *owed = r.u32()?,
            Extra::Launch { launches, .. } => *launches = r.u32()?,
            // Older saves have none: `Processor::load` set it from the facing.
            Extra::Channel(channel) if r.version >= CHANNEL_SAVED => *channel = r.u8()? % CHANNELS,
            Extra::Channel(_) | Extra::None | Extra::Depot { .. } | Extra::Hangar(_) | Extra::Ejector(_) => {}
        }
        Some(())
    }
}

/// Test-only views of a kind's state (panic for another kind).
#[cfg(test)]
impl super::Processor {
    pub fn pump_state(&self) -> &Pump {
        let Extra::Pump(pump) = &self.extra else { panic!("not a pumpjack") };
        pump
    }

    pub fn study_state(&self) -> &Study {
        let Extra::Study(study) = &self.extra else { panic!("not a research center") };
        study
    }

    /// Rockets sent up, and the satellite carried up this tick.
    pub fn launch_state(&self) -> (u32, Option<ItemId>) {
        let Extra::Launch { launches, lifted } = self.extra else { panic!("not a launch site") };
        (launches, lifted)
    }

    pub fn owed_mut(&mut self) -> &mut u32 {
        let Extra::Recycler { owed } = &mut self.extra else { panic!("not a recycler") };
        owed
    }

    pub fn owed(&self) -> u32 {
        let Extra::Recycler { owed } = self.extra else { panic!("not a recycler") };
        owed
    }
}

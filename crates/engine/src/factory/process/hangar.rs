//! The drone port (Milestone 7): a 3×3×1 landing pad, a processor that keeps drones instead of making
//! anything. Drones come in as items (`Action::Insert`, or belts into its inlets) up to its tier's fleet, and
//! what they do lives in the core's `drones/` module, which reads and edits the port through this file's
//! numbers. The port draws its tier's kW only while drones are out (`Hangar::busy`, set by `drones/`).
//!
//! - Stored drones are the `input` buffer's (so saves, breaking and belts need nothing new); `Hangar::away`
//!   counts the fleet that is flying, derived each tick by `drones/` so a port never holds more than its fleet.
//! - Reach (blocks, from the pad's centre) and fleet size come from [`TIERS`], by tier: Mk1 to Mk4.
//!
//! To change a tier: its row in [`TIERS`] (and the power in the spec's tier row).

use crate::block::tex;
use crate::item::DRONE;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor};

/// What a port tier gives: how far its drones work from the pad's centre, and how many it keeps.
pub struct HangarTier {
    pub reach: u32,
    pub fleet: u32,
}

pub const TIERS: [HangarTier; 4] = [
    HangarTier { reach: 32, fleet: 4 },
    HangarTier { reach: 48, fleet: 8 },
    HangarTier { reach: 64, fleet: 12 },
    HangarTier { reach: 96, fleet: 16 },
];

/// Derived each tick by `drones/` (never saved).
#[derive(Default)]
pub struct Hangar {
    /// Drones of this port that are out.
    pub away: u32,
    /// Whether any are: it then draws power.
    pub busy: bool,
}

const fn inlet(side: Side) -> Port {
    Port { side, role: Role::In, cell: Which::All }
}

pub const HANGAR_SPEC: ProcessSpec = ProcessSpec {
    block: crate::block::DRONE_PORT,
    categories: &[],
    pick: Pick::Hangar,
    buffers: [1, 0, 0],
    side: 0,
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 40 },
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 60 },
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 },
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 140 },
    ],
    footprint: Footprint { size: [3, 3, 1], ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right)] },
    verb: "Launching",
    products: "drones",
    waiting: "Idle",
    map_colour: 0xff8c42,
    parts: &PORT_PARTS,
};

const PAD: [u16; 3] = [tex::DRONE_PORT_TOP, tex::DRONE_PORT_SIDE, tex::FRAME];

/// A banded plinth, the landing pad, four corner masts with lamps and a status lamp.
const PORT_PARTS: [Part; 10] = [
    part([0.0, -0.42, 0.0], [2.96, 0.16, 2.96], Look::Band(tex::FRAME)),
    part([0.0, -0.3, 0.0], [2.8, 0.12, 2.8], Look::Tex(PAD)),
    part([-1.3, -0.08, -1.3], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([1.3, -0.08, -1.3], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([-1.3, -0.08, 1.3], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([1.3, -0.08, 1.3], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([-1.3, 0.14, -1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([1.3, 0.14, -1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([-1.3, 0.14, 1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([1.3, 0.14, 1.3], [0.14, 0.1, 0.14], Look::Lamp),
];

impl Processor {
    /// The tier's reach and fleet (`None`: not a port).
    pub fn hangar_tier(&self) -> Option<&'static HangarTier> {
        (self.spec.pick == Pick::Hangar).then(|| &TIERS[(self.tier as usize).min(TIERS.len() - 1)])
    }

    /// Drones at home on the pad.
    pub fn drones_home(&self) -> u32 {
        self.input.count(DRONE)
    }

    /// How many more drones it takes: its fleet less those home and those out.
    pub(super) fn hangar_room(&self, item: crate::item::ItemId) -> u32 {
        let Some(tier) = self.hangar_tier().filter(|_| item == DRONE) else { return 0 };
        tier.fleet.saturating_sub(self.drones_home() + self.hangar.away)
    }

    /// The status line of a port (`None`: another processor).
    pub(super) fn hangar_text(&self) -> Option<String> {
        let tier = self.hangar_tier()?;
        let home = self.drones_home();
        let wiring = if self.speed == 0 { " · No power: wire it to a pole" } else { "" };
        Some(format!(
            "{home} drones home, {} out · fleet {} · reach {} blocks{wiring}",
            self.hangar.away, tier.fleet, tier.reach
        ))
    }
}

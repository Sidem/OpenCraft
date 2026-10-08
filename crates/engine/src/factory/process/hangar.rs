//! The drone port (Milestone 7): a 3×3×1 landing pad, a processor that keeps drones instead of making
//! anything. Drones come in as items (`Action::Insert`, or belts into its inlets) up to its tier's fleet, and
//! what they do lives in the core's `drones/` module, which reads and edits the port through this file's
//! numbers. The port draws its tier's kW only while drones are out (`Hangar::busy`, set by `drones/`).
//!
//! - Stored drones are the `input` buffer's (so saves, breaking and belts need nothing new); `Hangar::away`
//!   counts the fleet that is flying, derived each tick by `drones/` so a port never holds more than its fleet.
//!   The buffer is one slot, so a port keeps construction drones or cargo drones (`drones/cargo.rs`), not both.
//! - Reach (blocks, from the pad's centre), cargo range and fleet size come from [`TIERS`], by tier: Mk1 to Mk5.
//!
//! To change a tier: its row in [`TIERS`] (and the power in the spec's tier row).

use crate::block::tex;
use crate::item::{ItemId, CARGO_DRONE, DRONE};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor};

/// What a port tier gives: how far its construction drones work from the pad's centre, how far its cargo drones fly
/// to another port, and how many drones it keeps.
pub struct HangarTier {
    pub reach: u32,
    pub fleet: u32,
    pub cargo_range: u32,
}

pub const TIERS: [HangarTier; 5] = [
    HangarTier { reach: 32, fleet: 4, cargo_range: 200 },
    HangarTier { reach: 48, fleet: 8, cargo_range: 400 },
    HangarTier { reach: 64, fleet: 12, cargo_range: 800 },
    HangarTier { reach: 96, fleet: 16, cargo_range: 1600 },
    HangarTier { reach: 128, fleet: 24, cargo_range: 3200 },
];

/// Derived each tick by `drones/` (never saved).
#[derive(Default)]
pub struct Hangar {
    /// Drones of this port that are out.
    pub away: u32,
    /// Whether any are: it then draws power.
    pub busy: bool,
    /// Which kind is out (`DRONE` or `CARGO_DRONE`; `NONE` when none): the port's one slot takes only that kind.
    pub kind: ItemId,
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
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 200 },
    ],
    footprint: Footprint { size: [3, 3, 1], ports: &[inlet(Side::Back), inlet(Side::Left), inlet(Side::Right)] },
    verb: "Launching",
    products: "drones",
    waiting: "Idle",
    map_colour: 0xff8c42,
    compute: 0,
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

    /// Construction drones at home on the pad.
    pub fn drones_home(&self) -> u32 {
        self.input.count(DRONE)
    }

    /// Cargo drones at home on the pad.
    pub fn couriers_home(&self) -> u32 {
        self.input.count(CARGO_DRONE)
    }

    /// How many more of `item` it takes: its fleet less those home and those out. A port keeps one kind of drone at
    /// a time (its slot, and the kind that is out), construction drones or cargo drones.
    pub(super) fn hangar_room(&self, item: ItemId) -> u32 {
        let Some(tier) = self.hangar_tier().filter(|_| item == DRONE || item == CARGO_DRONE) else { return 0 };
        let other = |kind: ItemId| kind != ItemId::NONE && kind != item;
        let home = self.input.slots.first().map_or(ItemId::NONE, |s| if s.is_empty() { ItemId::NONE } else { s.item });
        if other(home) || (self.hangar.away > 0 && other(self.hangar.kind)) {
            return 0;
        }
        tier.fleet.saturating_sub(self.input.total() + self.hangar.away)
    }

    /// The status line of a port (`None`: another processor).
    pub(super) fn hangar_text(&self) -> Option<String> {
        let tier = self.hangar_tier()?;
        let (home, cargo) = (self.drones_home() + self.couriers_home(), self.couriers_home() > 0);
        let wiring = if self.speed == 0 { " · No power: wire it to a pole" } else { "" };
        let (noun, range) = if cargo || self.hangar.kind == CARGO_DRONE {
            ("cargo drones", format!("range {} blocks", tier.cargo_range))
        } else {
            ("drones", format!("reach {} blocks", tier.reach))
        };
        Some(format!("{home} {noun} home, {} out · fleet {} · {range}{wiring}", self.hangar.away, tier.fleet))
    }
}

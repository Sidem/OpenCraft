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
use super::super::Factory;
use super::model::{part, Look, Part};
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor};

#[cfg(test)]
mod tests;

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
    /// The share of its fleet, in thousandths, the best swarm hub in range adds this tick (`hub.rs`; 0 for none).
    pub bonus: u32,
    /// Whether a comms satellite covers the pad this tick: reach and cargo range are multiplied (`LINK_REACH`, `LINK_RANGE`).
    pub linked: bool,
    /// Whether a supplied river town is near this tick (`settlements/perks/relay.rs`): both are multiplied by `RELAY`.
    pub relayed: bool,
}

/// What a comms satellite over the pad multiplies a port's construction reach and cargo range by.
pub const LINK_REACH: u32 = 4;
pub const LINK_RANGE: u32 = 2;
/// What a river town's drone relay multiplies them by.
pub const RELAY: u32 = 2;

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
    part([-1.29, -0.08, -1.29], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([1.29, -0.08, -1.29], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([-1.29, -0.08, 1.29], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([1.29, -0.08, 1.29], [0.2, 0.36, 0.2], Look::Tex([tex::DRONE_PORT_SIDE; 3])),
    part([-1.3, 0.14, -1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([1.3, 0.14, -1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([-1.3, 0.14, 1.3], [0.14, 0.1, 0.14], Look::Lamp),
    part([1.3, 0.14, 1.3], [0.14, 0.1, 0.14], Look::Lamp),
];

impl Processor {
    /// The tier's reach and fleet (`None`: not a port).
    pub fn hangar_tier(&self) -> Option<&'static HangarTier> {
        self.port().map(|(tier, _)| tier)
    }

    /// Its tier and this tick's fleet out and helpers (`None`: not a port).
    pub(super) fn port(&self) -> Option<(&'static HangarTier, &Hangar)> {
        let Extra::Hangar(port) = &self.extra else { return None };
        Some((&TIERS[(self.tier as usize).min(TIERS.len() - 1)], port))
    }

    /// The fleet out and helpers, for `drones/` and the factory tick to set (`None`: not a port).
    pub(in crate::factory) fn port_mut(&mut self) -> Option<&mut Hangar> {
        match &mut self.extra {
            Extra::Hangar(port) => Some(port),
            _ => None,
        }
    }

    /// How far its construction drones work, in blocks, and how far its cargo drones fly: the tier's, times what a comms
    /// satellite over the pad and a river town near add.
    pub fn reach_and_range(&self) -> Option<(u32, u32)> {
        let (t, port) = self.port()?;
        let (reach, range) = if port.linked { (LINK_REACH, LINK_RANGE) } else { (1, 1) };
        let relay = if port.relayed { RELAY } else { 1 };
        Some((t.reach * reach * relay, t.cargo_range * range * relay))
    }

    /// How many drones it keeps: its tier's fleet and what a swarm hub adds.
    pub fn fleet(&self) -> u32 {
        self.port().map_or(0, |(t, port)| t.fleet + t.fleet * port.bonus / 1000)
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
        let Some((_, port)) = self.port().filter(|_| item == DRONE || item == CARGO_DRONE) else { return 0 };
        let other = |kind: ItemId| kind != ItemId::NONE && kind != item;
        let home = self.input.slots.first().map_or(ItemId::NONE, |s| if s.is_empty() { ItemId::NONE } else { s.item });
        if other(home) || (port.away > 0 && other(port.kind)) {
            return 0;
        }
        self.fleet().saturating_sub(self.input.total() + port.away)
    }

    /// The status line of a port (`None`: another processor).
    pub(super) fn hangar_text(&self) -> Option<String> {
        let (reach, cargo_range) = self.reach_and_range()?;
        let (_, port) = self.port()?;
        let (home, cargo) = (self.drones_home() + self.couriers_home(), self.couriers_home() > 0);
        let wiring = if self.speed == 0 { " · No power: wire it to a pole" } else { "" };
        let (noun, range) = if cargo || port.kind == CARGO_DRONE {
            ("cargo drones", format!("range {cargo_range} blocks"))
        } else {
            ("drones", format!("reach {reach} blocks"))
        };
        let swarm = if port.bonus > 0 { " (swarm hub)" } else { "" };
        let comms = match (port.linked, port.relayed) {
            (true, true) => " (comms satellite, river town)",
            (true, false) => " (comms satellite)",
            (false, true) => " (river town)",
            (false, false) => "",
        };
        Some(format!("{home} {noun} home, {} out · fleet {}{swarm} · {range}{comms}{wiring}", port.away, self.fleet()))
    }
}

impl Factory {
    /// Marks each drone port relayed this tick or not, by its anchor (a river town's perk, `settlements/perks/`).
    pub fn relay_ports(&mut self, near: impl Fn(crate::math::IVec3) -> bool) {
        for p in self.processors.iter_mut() {
            let pos = p.pos;
            if let Some(port) = p.port_mut() {
                port.relayed = near(pos);
            }
        }
    }
}

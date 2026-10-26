//! The belt ejector (Underpasses tech): a 1×1 processor (`Pick::Eject`) at the end of a belt that throws what the belt
//! brings onto the ground in front of it (the way its placer faced). No power, no recipe, no panel.
//!
//! - It takes any item from any side, but only [`ROOM`] at a time, and drops one every [`EJECT_SECONDS`], so the rate
//!   is fixed by data (the core cannot see how many items already lie about) and the belt behind it backs up. A tool
//!   goes in one piece: its count is its uses left.
//! - A drop is a `SimEvent::Dropped` from the factory's tick; the host spawns the loose item (a client draws the
//!   host's) and `entities/merge.rs` joins them into stacks, also in chunks nobody has loaded. The items are not
//!   saved here: only what waits in its input buffer is, as for every processor.
//! - Items leave from above the block, a little forward, flying forward ([`FORWARD`], [`LIFT`]), so a wall or a
//!   machine in front cannot swallow them. They count as consumed in the analytics: the factory lost them.
//!
//! To change its pace or its throw: the constants.

use crate::block::{tex, BELT_EJECTOR};
use crate::inventory::Stack;
use crate::math::Vec3;
use crate::sim::SimEvent;
use crate::tools;

use super::super::footprint::SINGLE;
use super::super::power::FULL_SPEED;
use super::super::ticks;
use super::super::DIRS;
use super::model::{part, Look, Part};
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

/// Seconds between two drops (ten a second: faster than a Mk3 belt brings items).
const EJECT_SECONDS: f64 = 0.1;
/// Items it holds at once, over all its slots.
const ROOM: u32 = 4;
/// The centre of the muzzle: forward of the block's centre and above it. A drop starts here, so the model and the
/// throw cannot drift apart (`tests::an_item_leaves_the_muzzle`).
const MOUTH: [f64; 2] = [0.38, 0.31];
const FORWARD: f64 = MOUTH[0];
const START_HEIGHT: f64 = 0.5 + MOUTH[1];
/// How a drop is thrown (blocks per second): forward and up, about as steeply as the tube's last stretch rises.
const THROW: f64 = 2.0;
const LIFT: f64 = 1.5;

pub const EJECTOR_SPEC: ProcessSpec = ProcessSpec {
    block: BELT_EJECTOR,
    categories: &[],
    pick: Pick::Eject,
    buffers: [2, 0, 0],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Recipe, speed: 1000, fuel: 0, power: 0 }],
    footprint: SINGLE,
    verb: "Dropping",
    products: "items",
    waiting: "Waiting for a belt to bring items to drop: end a belt here",
    map_colour: 0x8a7a5a,
    compute: 0,
    parts: &PARTS,
};

const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// Front towards +z, so the muzzle is at the back (-z), where the items go: a plinth and a housing carrying a funnel
/// (a cup over a neck) and a steel tube that climbs in four steps towards the front and ends in a flared mouth with a
/// dark bore, then a status lamp on the side. The mouth is at [`MOUTH`].
const PARTS: [Part; 12] = {
    const TUBE: Look = Look::Tex([tex::MAT_BRASS; 3]);
    const DARK: Look = Look::Tex([tex::DRILL; 3]);
    let mouth = [0.0, MOUTH[1] as f32, -(MOUTH[0] as f32)];
    [
        part([0.0, -0.4, 0.0], [0.96, 0.2, 0.96], Look::Band(tex::FRAME)),
        part([0.0, -0.175, 0.1], [0.76, 0.25, 0.7], FRAME),
        part([0.0, 0.03, 0.3], [0.28, 0.16, 0.28], TUBE),
        part([0.0, 0.14, 0.3], [0.4, 0.06, 0.4], FRAME),
        part([0.0, 0.175, 0.3], [0.24, 0.01, 0.24], DARK),
        part([0.39, -0.17, 0.15], [0.08, 0.1, 0.12], Look::Lamp),
        part([0.0, -0.02, 0.12], [0.2, 0.2, 0.2], TUBE),
        part([0.0, 0.08, -0.02], [0.19, 0.19, 0.19], TUBE),
        part([0.0, 0.19, -0.16], [0.18, 0.18, 0.18], TUBE),
        part([0.0, 0.3, -0.3], [0.17, 0.17, 0.17], TUBE),
        part(mouth, [0.3, 0.26, 0.1], FRAME),
        part([0.0, mouth[1], mouth[2] - 0.055], [0.18, 0.18, 0.02], DARK),
    ]
};

/// The boxes of the item's icon and loose model (`item_models`): the same parts turned a quarter, so the mouth points
/// to the viewer's right (local -z becomes +x) and not away from them.
pub(crate) const ICON: [crate::item_models::Part; 12] = icon(&PARTS);

const fn icon(parts: &[Part; 12]) -> [crate::item_models::Part; 12] {
    let mut out = [crate::item_models::Part { center: [0.0; 3], size: [0.0; 3], yaw: 0.0, tex: [0; 3] }; 12];
    let mut i = 0;
    while i < 12 {
        let p = &parts[i];
        let tex = match p.look {
            Look::Tex(t) => t,
            Look::Band(top) => [top, tex::stripe(0), tex::FRAME],
            Look::Lamp => [tex::LAMP_GREEN; 3],
            _ => panic!("an ejector part has a look the icon does not know"),
        };
        out[i] = crate::item_models::Part {
            center: [-p.at[2], p.at[1], p.at[0]],
            size: [p.size[2], p.size[1], p.size[0]],
            yaw: 0.0,
            tex,
        };
        i += 1;
    }
    out
}

impl Processor {
    /// How many of an item it takes now: up to [`ROOM`] items in all.
    pub(super) fn eject_room(&self, item: crate::item::ItemId) -> u32 {
        self.input.space_for(item).min(ROOM.saturating_sub(self.input.total()))
    }

    /// One tick: count towards the next drop and, when it is due, queue the item in `Extra::Ejector`.
    pub(super) fn eject(&mut self) {
        let Some(slot) = self.input.slots.iter().position(|s| !s.is_empty()) else {
            (self.progress, self.status) = (0, Status::NoInput);
            return;
        };
        self.status = Status::Working;
        self.progress += self.stats().speed;
        if self.progress < ticks(EJECT_SECONDS) * FULL_SPEED {
            return;
        }
        self.progress = 0;
        let Stack { item, count } = self.input.slots[slot];
        let n = tools::lot(item, count);
        self.input.take(slot, n);
        self.used.push(Stack { item, count: 1 });
        if let Extra::Ejector(dropping) = &mut self.extra {
            dropping.push(Stack { item, count: n });
        }
    }

    /// Turns what `eject` queued this tick into `Dropped` events (from `report_made`).
    pub(super) fn report_drops(&mut self, events: &mut Vec<SimEvent>) {
        let Extra::Ejector(dropping) = &mut self.extra else { return };
        let d = DIRS[self.dir as usize];
        let (dx, dz) = (f64::from(d.x), f64::from(d.z));
        let pos = Vec3::new(
            f64::from(self.pos.x) + 0.5 + dx * FORWARD,
            f64::from(self.pos.y) + START_HEIGHT,
            f64::from(self.pos.z) + 0.5 + dz * FORWARD,
        );
        let vel = Vec3::new(dx * THROW, LIFT, dz * THROW);
        events.extend(dropping.drain(..).map(|s| SimEvent::Dropped { pos, vel, item: s.item, count: s.count }));
    }

    /// The status line of an ejector (`None`: another processor).
    pub(super) fn eject_text(&self) -> Option<String> {
        if self.spec.pick != Pick::Eject {
            return None;
        }
        Some(match self.status {
            Status::Working => "Dropping items on the ground in front of it".to_string(),
            _ => self.spec.waiting.to_string(),
        })
    }
}

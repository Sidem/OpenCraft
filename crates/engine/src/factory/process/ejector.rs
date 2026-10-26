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
/// Where a drop starts, forward of the block's centre and above its floor, and how it is thrown (blocks per second).
const FORWARD: f64 = 0.35;
const START_HEIGHT: f64 = 1.2;
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

/// Front towards +z, so the chute is at the back (-z), where the items go: a plinth, a housing, a funnel on top with a
/// conveyor tray running out to the back, and a status lamp.
const PARTS: [Part; 5] = [
    part([0.0, -0.4, 0.0], [0.96, 0.2, 0.96], Look::Band(tex::FRAME)),
    part([0.0, -0.05, 0.05], [0.8, 0.5, 0.7], FRAME),
    part([0.0, 0.3, 0.0], [0.7, 0.2, 0.7], Look::Tex([tex::BELT_TOP, tex::FRAME, tex::FRAME])),
    part([0.0, 0.42, -0.3], [0.4, 0.06, 0.5], Look::Tex([tex::BELT_TOP, tex::FRAME, tex::FRAME])),
    part([0.3, 0.1, 0.42], [0.12, 0.1, 0.12], Look::Lamp),
];

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

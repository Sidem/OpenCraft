//! The recycler (the Recycling tech): a 2×2×2 processor (`Pick::Recycle`) that destroys whatever its six inlet hatches
//! (back, left and right, two each) take and pays recycling coins out of its two front hatches. What an item pays is
//! worked out from the recipe tables (`recipes/recycling.rs`); the coin is `item::COIN`, 1024 to a stack.
//!
//! - It takes any item worth coins (everything but the coin) into its six input slots. At full power it destroys one
//!   a [`ITEM_SECONDS`] (a tool, whose stack count is its uses, all at once); the items' worth is added to
//!   `Processor::owed` in millicoins, and whole coins move into the two output slots as they have room (a belt takes
//!   one a tick), so a dear item never needs a whole stack of room at once and a plank's half coin is not lost.
//! - It stops taking work once [`OWED_MAX`] is owed (the status says the output is full) and draws power only while
//!   it has an item to destroy and room to be paid.
//! - Saved with what it owes; breaking it returns the whole coins with what its buffers hold.
//!
//! To change its pace or price: the constants, its spec's power, or `recipes/recycling.rs` for what items pay.

use crate::block::{tex, RECYCLER};
use crate::inventory::Stack;
use crate::item::{stack_size, COIN, COIN_STACK, MAX_STACK};
use crate::recipes::recycling::{millicoins, MILLI};

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::ticks;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

/// Seconds of work (at full power) to destroy one item.
const ITEM_SECONDS: f64 = 0.25;
/// Millicoins it may owe before it stops destroying items: its two output slots' worth of coins.
pub const OWED_MAX: u32 = 2 * COIN_STACK * MILLI;

pub const RECYCLER_SPEC: ProcessSpec = ProcessSpec {
    block: RECYCLER,
    categories: &[],
    pick: Pick::Recycle,
    buffers: [6, 0, 2],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 }],
    footprint: Footprint {
        size: [2, 2, 2],
        ports: &[
            Port { side: Side::Back, role: Role::In, cell: Which::All },
            Port { side: Side::Left, role: Role::In, cell: Which::All },
            Port { side: Side::Right, role: Role::In, cell: Which::All },
            Port { side: Side::Front, role: Role::Out, cell: Which::All },
        ],
    },
    verb: "Recycling",
    products: "coins",
    waiting: "Waiting for items to destroy: belts bring anything in",
    map_colour: 0x4a9a6a,
    parts: &PARTS,
};

const BODY: [u16; 3] = [tex::RECYCLER_TOP, tex::RECYCLER_SIDE, tex::FRAME];
const FRAME: Look = Look::Tex([tex::FRAME; 3]);

/// Across and deep 2, 2 high (±1): a banded plinth, a green housing, a wide hopper on top with a dark grate, a feed
/// chute on the back, a coin tray at the front (gold) and a status lamp.
const PARTS: [Part; 7] = [
    part([0.0, -0.9, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.2, 0.0], [1.8, 1.2, 1.8], Look::Tex(BODY)),
    part([0.0, 0.55, 0.0], [1.5, 0.3, 1.5], FRAME),
    part([0.0, 0.72, 0.0], [1.2, 0.08, 1.2], Look::Tex([tex::RECYCLER_TOP; 3])),
    part([0.0, 0.0, -0.95], [0.9, 0.7, 0.3], FRAME),
    part([0.0, -0.55, 0.92], [1.2, 0.3, 0.3], Look::Tex([tex::COIN; 3])),
    part([0.78, 0.2, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

impl Processor {
    /// One tick of destroying items at `power` (thousandths of full): pay out what is owed, then work on the next item.
    pub(super) fn recycle(&mut self, power: u32) {
        let pay = (self.owed / MILLI).min(self.out.space_for(COIN));
        if pay > 0 {
            self.out.add(COIN, pay);
            self.owed -= pay * MILLI;
            self.made.push(Stack { item: COIN, count: pay });
        }
        let Some(slot) = self.input.slots.iter().position(|s| !s.is_empty()) else {
            (self.progress, self.status) = (0, Status::NoInput);
            return;
        };
        if self.owed >= OWED_MAX {
            self.status = Status::OutputFull;
        } else if power == 0 {
            self.status = Status::NoPower;
        } else {
            self.status = Status::Working;
            self.progress += self.stats().speed * power / FULL_SPEED;
            if self.progress >= ticks(ITEM_SECONDS) * FULL_SPEED {
                self.progress = 0;
                let Stack { item, count } = self.input.slots[slot];
                // A tool's count is its uses left, not a pile: it goes in one piece.
                let n = if stack_size(item) > MAX_STACK { count } else { 1 };
                self.owed += millicoins(item) * n;
                self.input.take(slot, n);
            }
        }
    }

    /// Whether it would destroy an item this tick if powered.
    pub(super) fn recycle_wants_power(&self) -> bool {
        self.owed < OWED_MAX && self.input.total() > 0
    }

    /// The whole coins it still owes, to give back when it is broken (stacks no bigger than a coin stack holds).
    pub(super) fn owed_stacks(&self) -> Vec<Stack> {
        let mut left = self.owed / MILLI;
        let mut stacks = Vec::new();
        while left > 0 {
            let count = left.min(COIN_STACK);
            stacks.push(Stack { item: COIN, count });
            left -= count;
        }
        stacks
    }

    /// The status line of a recycler (`None`: another processor).
    pub(super) fn recycle_text(&self) -> Option<String> {
        (self.spec.pick == Pick::Recycle).then(|| match self.status {
            Status::NoPower => NOT_WIRED.to_string(),
            Status::OutputFull => {
                "Output full: put a belt leading away from a front hatch, or take the coins".to_string()
            }
            Status::Working => format!("Recycling · {} coins owed", self.owed / MILLI),
            _ => self.spec.waiting.to_string(),
        })
    }
}

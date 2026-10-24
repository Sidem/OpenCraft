//! The tag a belt item carries: where it was put on a belt and how long it has ridden belts since.
//! Core state (saved, in the state hash), so it is deterministic and the same on every co-op machine.
//!
//! - `from` is the cell of the machine that put the item on its first belt (a miner, quarry, box or processor).
//!   A splitter or filter only passes the tag on; a belt that hands over to a machine drops it, and the machine's own
//!   output starts a new one. [`Tag::UNKNOWN`] is for items from saves older than version 56.
//! - `ticks` counts the ticks spent on belts (`belt_step` adds one a tick; waiting counts, a router's one tick
//!   does not, but waiting in one does), saturating at `u32::MAX` (over two years).
//!
//! A tag is 16 bytes an item. Reading it for the crosshair is `belt_items.rs`; nothing here runs per frame.

use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::TICK_RATE;

/// The `y` of an origin nobody knows.
const NOWHERE: i32 = i32::MIN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag {
    pub ticks: u32,
    pub from: IVec3,
}

impl Tag {
    pub const UNKNOWN: Tag = Tag { ticks: 0, from: IVec3::new(0, NOWHERE, 0) };

    /// A fresh tag for an item the machine at `from` puts on a belt.
    pub fn made_at(from: IVec3) -> Tag {
        Tag { ticks: 0, from }
    }

    /// The machine cell it came from, unless that is unknown.
    pub fn origin(&self) -> Option<IVec3> {
        (self.from.y != NOWHERE).then_some(self.from)
    }

    /// Whole seconds on belts.
    pub fn seconds(&self) -> u32 {
        self.ticks / TICK_RATE
    }

    /// One more tick on a belt.
    pub fn age(&mut self) {
        self.ticks = self.ticks.saturating_add(1);
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.u32(self.ticks);
        w.ivec3(self.from);
    }

    /// Saves before version 56 have no tags.
    pub fn read_state(r: &mut ByteReader) -> Option<Tag> {
        if r.version < 56 {
            return Some(Tag::UNKNOWN);
        }
        Some(Tag { ticks: r.u32()?, from: r.ivec3()? })
    }
}

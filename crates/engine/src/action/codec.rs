//! Actions as bytes, for the wire (co-op, `net/`): a tag byte per variant, then its fields in
//! declaration order, through `ByteWriter` / `ByteReader`.
//!
//! Invariants: reads validate like save reads: an unknown tag, a bad bool, an unknown item or a short
//! buffer gives `None`, never a panic. `apply` still checks every value against the state, so a
//! well-formed but odd action (a slot past the end, a missing machine) does nothing. Tags are the
//! wire format: append, never renumber.
//! To add an action: the next tag in `write` (the compiler asks for the arm) and in `read`; the
//! round-trip test in `action/tests.rs` asks for a sample.

use super::Action;
use crate::bytes::{ByteReader, ByteWriter};

/// Number of tags in use; `read` refuses the rest.
#[cfg(test)]
pub const TAG_COUNT: u8 = 19;

impl Action {
    pub fn write(&self, w: &mut ByteWriter) {
        match *self {
            Action::BreakBlock { pos } => {
                w.u8(0);
                w.ivec3(pos);
            }
            Action::PlaceBlock { pos, slot, facing, against } => {
                w.u8(1);
                w.ivec3(pos);
                w.u8(slot);
                w.u8(facing);
                w.ivec3(against);
            }
            Action::TakeContents { pos } => {
                w.u8(2);
                w.ivec3(pos);
            }
            Action::SetRecipe { pos, recipe } => {
                w.u8(3);
                w.ivec3(pos);
                w.u16(recipe);
            }
            Action::SetFilter { pos, item } => {
                w.u8(4);
                w.ivec3(pos);
                w.item(item);
            }
            Action::SetResearch { tech } => {
                w.u8(5);
                w.u8(tech);
            }
            Action::Insert { pos, item } => {
                w.u8(6);
                w.ivec3(pos);
                w.item(item);
            }
            Action::Craft { recipe, times } => {
                w.u8(7);
                w.u16(recipe);
                w.u32(times);
            }
            Action::ClickSlot { slot, shift } => {
                w.u8(8);
                w.u8(slot);
                w.bool(shift);
            }
            Action::ClickBox { pos, slot, shift } => {
                w.u8(9);
                w.ivec3(pos);
                w.u8(slot);
                w.bool(shift);
            }
            Action::StoreSlot { pos, slot } => {
                w.u8(10);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::CloseInventory => w.u8(11),
            Action::SelectSlot { slot } => {
                w.u8(12);
                w.u8(slot);
            }
            Action::ScrollSlot { delta } => {
                w.u8(13);
                w.u8(delta as u8);
            }
            Action::DropSelected { count } => {
                w.u8(14);
                w.u32(count);
            }
            Action::PickUp { item, count } => {
                w.u8(15);
                w.item(item);
                w.u32(count);
            }
            Action::Give { item, count } => {
                w.u8(16);
                w.item(item);
                w.u32(count);
            }
            Action::Join { key } => {
                w.u8(17);
                w.u64(key);
            }
            Action::Leave { pos } => {
                w.u8(18);
                w.vec3(pos);
            }
        }
    }

    pub fn read(r: &mut ByteReader) -> Option<Action> {
        Some(match r.u8()? {
            0 => Action::BreakBlock { pos: r.ivec3()? },
            1 => Action::PlaceBlock { pos: r.ivec3()?, slot: r.u8()?, facing: r.u8()?, against: r.ivec3()? },
            2 => Action::TakeContents { pos: r.ivec3()? },
            3 => Action::SetRecipe { pos: r.ivec3()?, recipe: r.u16()? },
            4 => Action::SetFilter { pos: r.ivec3()?, item: r.item()? },
            5 => Action::SetResearch { tech: r.u8()? },
            6 => Action::Insert { pos: r.ivec3()?, item: r.item()? },
            7 => Action::Craft { recipe: r.u16()?, times: r.u32()? },
            8 => Action::ClickSlot { slot: r.u8()?, shift: r.bool()? },
            9 => Action::ClickBox { pos: r.ivec3()?, slot: r.u8()?, shift: r.bool()? },
            10 => Action::StoreSlot { pos: r.ivec3()?, slot: r.u8()? },
            11 => Action::CloseInventory,
            12 => Action::SelectSlot { slot: r.u8()? },
            13 => Action::ScrollSlot { delta: r.u8()? as i8 },
            14 => Action::DropSelected { count: r.u32()? },
            15 => Action::PickUp { item: r.item()?, count: r.u32()? },
            16 => Action::Give { item: r.item()?, count: r.u32()? },
            17 => Action::Join { key: r.u64()? },
            18 => Action::Leave { pos: r.vec3()? },
            _ => return None,
        })
    }

    /// Whether a peer may send this for its own player. Joining, leaving and pickups are the
    /// authority's (the host's) to issue.
    pub fn is_peer_input(&self) -> bool {
        !matches!(self, Action::Join { .. } | Action::Leave { .. } | Action::PickUp { .. })
    }
}

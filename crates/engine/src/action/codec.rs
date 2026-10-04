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
use crate::factory::Job;

/// Number of tags in use; `read` refuses the rest.
#[cfg(test)]
pub const TAG_COUNT: u8 = 39;

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
            Action::CancelCraft { order } => {
                w.u8(26);
                w.u16(order);
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
            Action::Rotate { pos } => {
                w.u8(19);
                w.ivec3(pos);
            }
            Action::SetQuarry { pos, width, depth, paused } => {
                w.u8(20);
                w.ivec3(pos);
                w.u8(width);
                w.u8(depth);
                w.bool(paused);
            }
            Action::MarkSite { a, b, level, job } => {
                w.u8(21);
                for v in [a.0, a.1, b.0, b.1, level] {
                    w.i32(v);
                }
                w.u8(job as u8);
            }
            Action::RemoveSite { id } => {
                w.u8(22);
                w.u32(id);
            }
            Action::Upgrade { pos } => {
                w.u8(23);
                w.ivec3(pos);
            }
            Action::Connect { pole, to } => {
                w.u8(27);
                w.ivec3(pole);
                w.ivec3(to);
            }
            Action::Disconnect { pole, to } => {
                w.u8(28);
                w.ivec3(pole);
                w.ivec3(to);
            }
            Action::QuickMoveAll { slot } => {
                w.u8(29);
                w.u8(slot);
            }
            Action::StoreAll { pos, slot } => {
                w.u8(30);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::TakeAll { pos, slot } => {
                w.u8(31);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::SetSensor { pos, rule } => {
                w.u8(32);
                w.ivec3(pos);
                w.u8(rule);
            }
            Action::PlaceGhost { pos, slot, facing } => {
                w.u8(33);
                w.ivec3(pos);
                w.u8(slot);
                w.u8(facing);
            }
            Action::RemoveGhost { pos } => {
                w.u8(34);
                w.ivec3(pos);
            }
            Action::PlantGhost { pos, block, facing, tier } => {
                w.u8(35);
                w.ivec3(pos);
                w.u8(block);
                w.u8(facing);
                w.u8(tier);
            }
            Action::MarkRemoval { pos } => {
                w.u8(36);
                w.ivec3(pos);
            }
            Action::Jetpack { on } => {
                w.u8(37);
                w.bool(on);
            }
            Action::Fetch { item, at } => {
                w.u8(38);
                w.item(item);
                w.ivec3(at);
            }
            Action::SortInventory => w.u8(24),
            Action::SortBox { pos } => {
                w.u8(25);
                w.ivec3(pos);
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
            19 => Action::Rotate { pos: r.ivec3()? },
            20 => Action::SetQuarry { pos: r.ivec3()?, width: r.u8()?, depth: r.u8()?, paused: r.bool()? },
            21 => Action::MarkSite {
                a: (r.i32()?, r.i32()?),
                b: (r.i32()?, r.i32()?),
                level: r.i32()?,
                job: Job::from_byte(r.u8()?)?,
            },
            22 => Action::RemoveSite { id: r.u32()? },
            23 => Action::Upgrade { pos: r.ivec3()? },
            24 => Action::SortInventory,
            25 => Action::SortBox { pos: r.ivec3()? },
            26 => Action::CancelCraft { order: r.u16()? },
            27 => Action::Connect { pole: r.ivec3()?, to: r.ivec3()? },
            28 => Action::Disconnect { pole: r.ivec3()?, to: r.ivec3()? },
            29 => Action::QuickMoveAll { slot: r.u8()? },
            30 => Action::StoreAll { pos: r.ivec3()?, slot: r.u8()? },
            31 => Action::TakeAll { pos: r.ivec3()?, slot: r.u8()? },
            32 => Action::SetSensor { pos: r.ivec3()?, rule: r.u8()? },
            33 => Action::PlaceGhost { pos: r.ivec3()?, slot: r.u8()?, facing: r.u8()? },
            34 => Action::RemoveGhost { pos: r.ivec3()? },
            36 => Action::MarkRemoval { pos: r.ivec3()? },
            37 => Action::Jetpack { on: r.bool()? },
            38 => Action::Fetch { item: r.item()?, at: r.ivec3()? },
            35 => Action::PlantGhost { pos: r.ivec3()?, block: r.block()?, facing: r.u8()?, tier: r.u8()? },
            _ => return None,
        })
    }

    /// Whether a peer may send this for its own player. Joining, leaving and pickups are the
    /// authority's (the host's) to issue.
    pub fn is_peer_input(&self) -> bool {
        !matches!(self, Action::Join { .. } | Action::Leave { .. } | Action::PickUp { .. })
    }
}

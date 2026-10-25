//! Actions as bytes, for the wire (co-op, `net/`): a tag byte per variant, then its fields in
//! declaration order, through `ByteWriter` / `ByteReader`.
//!
//! Invariants: reads validate like save reads: an unknown tag, a bad bool, an unknown item or a short
//! buffer gives `None`, never a panic. `apply` still checks every value against the state, so a
//! well-formed but odd action (a slot past the end, a missing machine) does nothing. Tags are the
//! wire format: append, never renumber.
//! To add an action: the next tag in `write` (the compiler asks for the arm) and in `read` (`codec/read.rs`); the
//! round-trip test in `action/tests.rs` asks for a sample.

use super::Action;
use crate::bytes::ByteWriter;

mod read;

/// Number of tags in use; `read` refuses the rest.
#[cfg(test)]
pub const TAG_COUNT: u8 = 65;

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
            Action::MarkTunnel { from, to, size } => {
                w.u8(40);
                w.ivec3(from);
                w.ivec3(to);
                w.u8(size);
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
            Action::ClickGear { slot, shift } => {
                w.u8(39);
                w.u8(slot);
                w.bool(shift);
            }
            Action::PlaceTrain { pos, slot } => {
                w.u8(41);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::TakeTrain { pos } => {
                w.u8(42);
                w.ivec3(pos);
            }
            Action::TrainStop { node, dock, clear } => {
                w.u8(43);
                w.ivec3(node);
                w.ivec3(dock);
                w.bool(clear);
            }
            Action::ToggleSignal { pos, slot } => {
                w.u8(44);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::Hover { on } => {
                w.u8(45);
                w.bool(on);
            }
            Action::Charge { pole, on } => {
                w.u8(46);
                w.ivec3(pole);
                w.bool(on);
            }
            Action::SetRoute { from, to, clear } => {
                w.u8(47);
                w.ivec3(from);
                w.ivec3(to);
                w.bool(clear);
            }
            Action::RightClickSlot { slot, shift } => {
                w.u8(48);
                w.u8(slot);
                w.bool(shift);
            }
            Action::RightClickBox { pos, slot, shift } => {
                w.u8(49);
                w.ivec3(pos);
                w.u8(slot);
                w.bool(shift);
            }
            Action::ThrowCursor => w.u8(50),
            Action::TakeOneSlot { slot } => {
                w.u8(54);
                w.u8(slot);
            }
            Action::TakeOneBox { pos, slot } => {
                w.u8(55);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::TakeMachineSlot { pos, slot } => {
                w.u8(56);
                w.ivec3(pos);
                w.u8(slot);
            }
            Action::InsertCursor { pos } => {
                w.u8(57);
                w.ivec3(pos);
            }
            Action::SetHarbourProduct { pos, item } => {
                w.u8(58);
                w.ivec3(pos);
                w.item(item);
            }
            Action::OpenVault { pos, dials } => {
                w.u8(59);
                w.ivec3(pos);
                w.u32(dials);
            }
            Action::SetChannel { pos, channel } => {
                w.u8(60);
                w.ivec3(pos);
                w.u8(channel);
            }
            Action::ClearCover { pos } => {
                w.u8(61);
                w.ivec3(pos);
            }
            Action::TakeBeltItem { pos, item, at } => {
                w.u8(62);
                w.ivec3(pos);
                w.item(item);
                w.u8(at);
            }
            Action::LessonTaught { bit } => {
                w.u8(63);
                w.u8(bit);
            }
            Action::UpgradePack => w.u8(53),
            Action::QueueResearch { tech } => {
                w.u8(51);
                w.u8(tech);
            }
            Action::UnqueueResearch { tech } => {
                w.u8(52);
                w.u8(tech);
            }
            Action::SortInventory => w.u8(24),
            Action::SortBox { pos } => {
                w.u8(25);
                w.ivec3(pos);
            }
            Action::TakeApart { recipe, times } => {
                w.u8(64);
                w.u16(recipe);
                w.u16(times);
            }
        }
    }

    /// Whether a peer may send this for its own player. Joining, leaving and pickups are the
    /// authority's (the host's) to issue.
    pub fn is_peer_input(&self) -> bool {
        !matches!(self, Action::Join { .. } | Action::Leave { .. } | Action::PickUp { .. })
    }
}

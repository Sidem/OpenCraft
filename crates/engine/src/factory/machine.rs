//! What every machine kind provides (`Machine`), and the two list helpers every kind shares: `add_to` appends a machine and
//! indexes its cells, `swap_out` removes one and re-indexes the machine that moved into its place.

use rustc_hash::FxHashMap;

use super::links::Slot;
use super::Factory;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};
/// What every machine kind provides. Static dispatch only: callers `match` on `Slot` or loop over
/// one kind's `Vec`.
pub(super) trait Machine: Sized {
    fn pos(&self) -> IVec3;
    /// Every cell it occupies, `pos` first: one, except for multi-block processors (`footprint/`).
    fn cells(&self) -> Vec<IVec3> {
        vec![self.pos()]
    }
    /// Its core state (derived data such as links is left out).
    fn write_state(&self, w: &mut ByteWriter);
    fn read_state(r: &mut ByteReader) -> Option<Self>;
    /// Everything it holds or carries, dropped when it is removed.
    fn contents(&self) -> Vec<Stack>;
    /// Readout lines for the HUD ("" for nothing to say).
    fn describe(&self, f: &Factory) -> String;
    /// Box instances for its model; `rel` is its cell centre relative to the camera.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64);
}

/// Appends `m` to its kind's list and indexes its cells.
pub(super) fn add_to<T: Machine>(list: &mut Vec<T>, m: T, at: &mut FxHashMap<IVec3, Slot>, slot: fn(u32) -> Slot) {
    for c in m.cells() {
        at.insert(c, slot(list.len() as u32));
    }
    list.push(m);
}

/// Removes entry `i` (its cells leave `at`), re-indexes the entry moved into its place, and returns
/// the removed machine's contents.
pub(super) fn swap_out<T: Machine>(
    list: &mut Vec<T>,
    i: u32,
    at: &mut FxHashMap<IVec3, Slot>,
    slot: fn(u32) -> Slot,
) -> Vec<Stack> {
    let m = list.swap_remove(i as usize);
    for c in m.cells() {
        at.remove(&c);
    }
    if let Some(moved) = list.get(i as usize) {
        for c in moved.cells() {
            at.insert(c, slot(i));
        }
    }
    m.contents()
}

//! LED lamps (`block/led.rs`): the lamp's brightness is which of three block ids stands in the cell (light is derived from
//! blocks only), its colour is a record kept here. A player sets both from the lamp's panel (`Action::SetLed`, the host's
//! `web/src/ui/led-panel.ts`); the halo and the tint box read the colour (`halos.rs`, `view.rs`). Core state, saved and
//! hashed. Unpowered, like the lamp.
//!
//! Invariants: a record exists only for a cell that holds an LED and whose colour is not [`DEFAULT_COLOUR`]; the list is
//! sorted by cell, so the bytes never depend on the order edits came in; colours are 0xRRGGBB. When the cell stops
//! being an LED (broken, dug, built over), `Sim::block_changed` calls `cell_changed`, which forgets the record.
//!
//! To change what a lamp holds: its fields, `write_state` and `read_state` (with a `SAVE_VERSION` bump).

use crate::block::{is_led, BlockId, LED_STEPS};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::sim::Sim;

pub mod view;

/// A lamp with no record shines this warm white, the colour of the plain lamp's halo.
pub const DEFAULT_COLOUR: u32 = 0xffe2b0;
/// Every colour's bits.
const RGB: u32 = 0xff_ffff;

/// The colours players picked, by cell (only those not [`DEFAULT_COLOUR`]).
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Leds {
    list: Vec<(IVec3, u32)>,
}

fn key(p: IVec3) -> (i32, i32, i32) {
    (p.x, p.y, p.z)
}

impl Leds {
    /// The colour of the lamp at `pos` (its record, else the default).
    pub fn colour(&self, pos: IVec3) -> u32 {
        self.picked(pos).unwrap_or(DEFAULT_COLOUR)
    }

    /// The colour a player picked for the lamp at `pos`, if any.
    pub fn picked(&self, pos: IVec3) -> Option<u32> {
        self.list.binary_search_by_key(&key(pos), |&(p, _)| key(p)).ok().map(|i| self.list[i].1)
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    fn set(&mut self, pos: IVec3, rgb: u32) {
        let at = self.list.binary_search_by_key(&key(pos), |&(p, _)| key(p));
        match (at, rgb & RGB == DEFAULT_COLOUR) {
            (Ok(i), true) => {
                self.list.remove(i);
            }
            (Ok(i), false) => self.list[i].1 = rgb & RGB,
            (Err(i), false) => self.list.insert(i, (pos, rgb & RGB)),
            (Err(_), true) => {}
        }
    }

    fn forget(&mut self, pos: IVec3) {
        if let Ok(i) = self.list.binary_search_by_key(&key(pos), |&(p, _)| key(p)) {
            self.list.remove(i);
        }
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for &(p, rgb) in &self.list {
            w.ivec3(p);
            w.u32(rgb);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Leds> {
        let mut list: Vec<(IVec3, u32)> = Vec::new();
        for _ in 0..r.count()? {
            let (p, rgb) = (r.ivec3()?, r.u32()?);
            let sorted = list.last().is_none_or(|&(q, _)| key(q) < key(p));
            if rgb > RGB || rgb == DEFAULT_COLOUR || !sorted {
                return None;
            }
            list.push((p, rgb));
        }
        Some(Leds { list })
    }
}

/// Which brightness step (an index of `LED_STEPS`) the block `b` is; `None` for any other block.
pub fn step_of(b: BlockId) -> Option<usize> {
    LED_STEPS.iter().position(|&s| s == b)
}

impl Sim {
    /// Sets the LED lamp at `pos` to brightness `step` (an index of `LED_STEPS`) and colour `rgb`; nothing when no
    /// lamp stands there or the step is not one.
    pub(crate) fn set_led(&mut self, pos: IVec3, step: u8, rgb: u32) {
        let old = self.block(pos);
        let Some(&new) = LED_STEPS.get(step as usize).filter(|_| is_led(old)) else { return };
        if new != old && self.world.set_block_anywhere(pos, new) {
            self.block_changed(pos, old);
        }
        self.leds.set(pos, rgb);
    }

    /// After the block at `pos` changed: a lamp that stopped being one forgets its colour.
    pub(crate) fn led_cell_changed(&mut self, pos: IVec3) {
        if !self.leds.is_empty() && !is_led(self.block(pos)) {
            self.leds.forget(pos);
        }
    }
}

#[cfg(test)]
mod tests;

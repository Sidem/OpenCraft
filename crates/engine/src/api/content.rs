//! Content tables and balance numbers the UI shows: block names and sound materials, item names and
//! looks, hand yield, miner recovery. Exposed as getters so TypeScript never mirrors engine constants.

use wasm_bindgen::prelude::*;

use crate::block::{self, BLOCK_COUNT};
use crate::deposits::HAND_YIELD;
use crate::factory::MINER_RECOVERY;
use crate::item::{self, ItemId};
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn block_count(&self) -> u32 {
        BLOCK_COUNT as u32
    }

    pub fn block_name(&self, id: u8) -> String {
        block::def(id).name.to_string()
    }

    /// Sound material a block uses (`block::sound`), i.e. which bank its dig/step/place sounds come from.
    pub fn block_sound(&self, id: u8) -> u8 {
        block::def(id).sound
    }

    /// An item's name ("" for unknown ids).
    pub fn item_name(&self, id: u16) -> String {
        item::name(ItemId(id)).to_string()
    }

    /// How to draw an item's icon: texture layers top, side, bottom, then its box proportions x, y, z
    /// (1 = a full cube). Empty for unknown ids.
    pub fn item_icon(&self, id: u16) -> Vec<f32> {
        item::def(ItemId(id)).map_or_else(Vec::new, |d| {
            vec![d.tex[0] as f32, d.tex[1] as f32, d.tex[2] as f32, d.size[0], d.size[1], d.size[2]]
        })
    }

    /// Ore kept per block mined by hand.
    pub fn hand_yield(&self) -> u32 {
        HAND_YIELD
    }

    /// Fraction of drilled ore a miner delivers.
    pub fn miner_recovery(&self) -> f64 {
        MINER_RECOVERY
    }
}

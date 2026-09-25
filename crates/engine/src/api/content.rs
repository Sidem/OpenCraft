//! Content tables and balance numbers the UI shows: block names and sound materials, hand yield,
//! miner recovery. Exposed as getters so TypeScript never mirrors engine constants.

use wasm_bindgen::prelude::*;

use crate::block::{self, BLOCK_COUNT};
use crate::deposits::HAND_YIELD;
use crate::factory::MINER_RECOVERY;
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

    /// Ore kept per block mined by hand.
    pub fn hand_yield(&self) -> u32 {
        HAND_YIELD
    }

    /// Fraction of drilled ore a miner delivers.
    pub fn miner_recovery(&self) -> f64 {
        MINER_RECOVERY
    }
}

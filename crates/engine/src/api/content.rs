//! Content tables and balance numbers the UI shows: block names and sound materials, item names and
//! looks, tool uses, hand yield, miner recovery. Exposed as getters so TypeScript never mirrors engine constants.

use wasm_bindgen::prelude::*;

use crate::block::{self, BLOCK_COUNT};
use crate::deposits::HAND_YIELD;
use crate::factory::MINER_RECOVERY;
use crate::item::{self, ItemId};
use crate::item_models;
use crate::tools;
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

    /// Box parts for a manufactured item's icon: centre xyz, size xyz, then top/side/bottom layers.
    /// Empty means the ordinary `item_icon` box is the complete model.
    pub fn item_model(&self, id: u16) -> Vec<f32> {
        let mut out = Vec::new();
        for p in item_models::parts(ItemId(id)) {
            out.extend_from_slice(&[
                p.center[0],
                p.center[1],
                p.center[2],
                p.size[0],
                p.size[1],
                p.size[2],
                p.tex[0] as f32,
                p.tex[1] as f32,
                p.tex[2] as f32,
            ]);
        }
        out
    }

    /// A tool's uses when new, which is also its stack size (its count is the uses left); 0 for
    /// anything that isn't a tool or doesn't wear (the prospecting devices).
    pub fn tool_uses(&self, id: u16) -> u32 {
        tools::tool(ItemId(id)).filter(|t| tools::device(t.item).is_none()).map_or(0, |t| t.tier.uses)
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

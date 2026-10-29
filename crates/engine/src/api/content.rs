//! Content tables and balance numbers the UI shows: block names and sound materials, item names and
//! looks, tool uses, hand yield, miner recovery. Exposed as getters so TypeScript never mirrors engine constants.

use wasm_bindgen::prelude::*;

use crate::block::{self, BLOCK_COUNT};
use crate::deposits::HAND_YIELD;
use crate::factory::tiers;
use crate::factory::upgrades::TIER_COLOURS;
use crate::factory::{Axis, Form, MINER_TIERS};
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

    /// The first of a texture layer's three alternate looks, or 0 when it has none (`block::tex`).
    pub fn texture_alternates(&self, layer: u16) -> u16 {
        block::tex::alternates(layer)
    }

    /// The build menu card of the building an item places (`factory/facts.rs`): one row a line, `kind<TAB>label<TAB>value`,
    /// with the kinds size, use, fuel, make, none and stat. "" for anything that is not a machine.
    pub fn item_facts(&self, id: u16) -> String {
        crate::factory::facts(ItemId(id)).map_or_else(String::new, |f| f.rows())
    }

    /// An item's name ("" for unknown ids).
    pub fn item_name(&self, id: u16) -> String {
        item::name(ItemId(id)).to_string()
    }

    /// How to draw an item's icon: texture layers top, side, bottom, then its box proportions x, y, z
    /// (1 = a full cube), then its fit (`item_models::icon_frame`): one cube unit as a share of the box side,
    /// the icon's centre right and down in cube units, and 1 if it is drawn flat as its side picture. Empty for
    /// unknown ids.
    pub fn item_icon(&self, id: u16) -> Vec<f32> {
        item::def(ItemId(id)).map_or_else(Vec::new, |d| {
            let fit = item_models::icon_frame(ItemId(id));
            vec![
                d.tex[0] as f32,
                d.tex[1] as f32,
                d.tex[2] as f32,
                d.size[0],
                d.size[1],
                d.size[2],
                fit.scale(),
                fit.centre[0],
                fit.centre[1],
                f32::from(u8::from(fit.flat)),
            ]
        })
    }

    /// Parts for a manufactured item's icon, `item_model_floats()` each: centre xyz, size xyz (the box it fills), yaw,
    /// top/side/bottom layers, then its form (0 a box; a round body along 1 y, 2 x, 3 z; 4 a leaned box) and the round
    /// body's taper (its far end's width as a share of its near end's) or the leaned box's pitch. Empty means the ordinary `item_icon` box is the model.
    pub fn item_model(&self, id: u16) -> Vec<f32> {
        let mut out = Vec::new();
        for p in item_models::parts(ItemId(id)) {
            let (form, taper) = match p.form {
                Form::Box => (0.0, 1.0),
                Form::Lean(pitch) => (4.0, pitch),
                Form::Round(Axis::Up, t) => (1.0, t),
                Form::Round(Axis::X, t) => (2.0, t),
                Form::Round(Axis::Z, t) => (3.0, t),
            };
            out.extend_from_slice(&[
                p.center[0],
                p.center[1],
                p.center[2],
                p.size[0],
                p.size[1],
                p.size[2],
                p.yaw,
                p.tex[0] as f32,
                p.tex[1] as f32,
                p.tex[2] as f32,
                form,
                taper,
            ]);
        }
        out
    }

    /// Floats per part of `item_model`.
    pub fn item_model_floats(&self) -> usize {
        12
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
        MINER_TIERS[0].recovery
    }

    /// The tier (0 is Mk1) of a tiered machine's item, or -1 for other items.
    pub fn item_tier(&self, item: u16) -> i32 {
        tiers::placed_by(ItemId(item)).map_or(-1, |(_, t)| i32::from(t))
    }

    /// A tier's colour as 0xRRGGBB: Mk1 red, green, blue, violet, Mk5 gold.
    pub fn tier_colour(&self, tier: u32) -> u32 {
        TIER_COLOURS.get(tier as usize).copied().unwrap_or(0)
    }
}

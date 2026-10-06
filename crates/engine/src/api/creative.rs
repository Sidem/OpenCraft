//! Creative worlds for the host (rules and bytes: `mode.rs`): making one, asking the mode, listing every
//! item and taking a stack of one. Every method forwards.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::item::{self, ItemId};
use crate::mode::{creative_items, Mode};
use crate::worldgen::WorldGen;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// A new creative world: every tech done, every item on offer. The mode never changes afterwards.
    pub fn new_creative(seed: u32, view_radius: u32) -> Game {
        let mut game = Game::with_generator(WorldGen::new(seed), view_radius);
        game.sim.set_mode(Mode::Creative);
        game
    }

    pub fn is_creative(&self) -> bool {
        self.sim.mode == Mode::Creative
    }

    /// Every item a creative world offers, as ids (empty in a normal world).
    pub fn creative_items(&self) -> Vec<u16> {
        if self.is_creative() {
            creative_items().iter().map(|i| i.0).collect()
        } else {
            Vec::new()
        }
    }

    /// Takes a stack of `item` (or one) into the inventory, as an action; only in a creative world.
    pub fn creative_give(&mut self, item: u16, whole_stack: bool) {
        let item = ItemId(item);
        if self.is_creative() && item.is_valid() {
            let count = if whole_stack { item::stack_size(item) } else { 1 };
            self.act(Action::Give { item, count });
        }
    }
}

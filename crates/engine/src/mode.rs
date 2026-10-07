//! A world's game mode, picked when the world is made and never changed: `Normal` is the game as
//! designed, `Creative` is for testing everything. It is core state (`Sim::mode`, saved since version 38
//! and in the state hash), so every co-op peer agrees on it.
//!
//! A creative world has every tech done (`Sim::set_mode` completes the tree; loading one completes techs
//! added since it was saved) and offers every item through `creative_items`; the host gives a stack of
//! one on click (`api/creative.rs`). To make creative do more: branch on `Sim::mode` where the rule lives.
//! A new mode is a variant here plus its byte; bytes are saved, so never renumber.

use crate::item::{def, ItemId, COIN};
use crate::sim::Sim;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Normal,
    Creative,
}

impl Mode {
    pub fn byte(self) -> u8 {
        match self {
            Mode::Normal => 0,
            Mode::Creative => 1,
        }
    }

    pub fn from_byte(b: u8) -> Option<Mode> {
        match b {
            0 => Some(Mode::Normal),
            1 => Some(Mode::Creative),
            _ => None,
        }
    }
}

impl Sim {
    /// Sets the mode: a creative world has every tech done (call again after loading, for newer techs).
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        if mode == Mode::Creative {
            self.factory.research.complete_all();
        }
    }
}

/// Every item a creative world offers: each block that can be placed, then every other item (not the recycling coin,
/// which is earned).
pub fn creative_items() -> Vec<ItemId> {
    let blocks = (1..256u16).map(ItemId).filter(|item| item.places().is_some());
    let others = (256u16..).map(ItemId).take_while(|&item| def(item).is_some()).filter(|&item| item != COIN);
    blocks.chain(others).collect()
}

#[cfg(test)]
mod tests;

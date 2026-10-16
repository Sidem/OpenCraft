//! What hand mining wasted this session, for the HUD (`hand_waste.rs`).

use wasm_bindgen::prelude::*;

use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Ore blocks the local player broke by hand this session; it goes up by one with each, so the host can
    /// flash `hand_waste_notice` when it changes.
    pub fn hand_mined_blocks(&self) -> u32 {
        self.waste.blocks
    }

    /// One line about the latest ore block broken by hand: what was kept and what was lost (hint markup).
    pub fn hand_waste_notice(&self) -> String {
        self.waste.notice()
    }

    /// The warning tip about ore wasted by hand (hint markup), or nothing while too few blocks were broken.
    pub fn hand_waste_tip(&self) -> String {
        self.waste.tip()
    }
}

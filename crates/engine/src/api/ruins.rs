//! Ruins for the host: the page of a logbook (`worldgen/logbook.rs`), read by the panel `web/src/ui/logbook.ts` when the
//! player right-clicks one (the request comes through `take_panel_request`, like a machine's), and a vault door's dials
//! (`sim/vault.rs`, panel `web/src/ui/vault.ts`).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block;
use crate::math::IVec3;
use crate::sim::vault;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// The page of the logbook at a block: its title, a newline, its text and, if it points at ground, a last line
    /// `pin`, x, z and the ore's block id split by tabs. Empty when no ruin's logbook stands there.
    pub fn logbook(&self, x: i32, y: i32, z: i32) -> String {
        let Some(page) = self.sim.world.generator().logbook_at(IVec3::new(x, y, z)) else { return String::new() };
        let mut out = format!("{}\n{}", page.title, page.text);
        if let Some((px, pz, ore)) = page.pin {
            out.push_str(&format!("\npin\t{px}\t{pz}\t{ore}"));
        }
        out
    }

    /// How many rings the vault door at a block has (`sim/vault.rs`); 0 when no door stands there.
    pub fn vault_rings(&self, x: i32, y: i32, z: i32) -> u32 {
        let door = self.sim.world.block_anywhere(IVec3::new(x, y, z)) == Some(block::VAULT_DOOR);
        if door {
            vault::RINGS as u32
        } else {
            0
        }
    }

    /// What the dials show (digits 0 to 4, first ring first): how many rings are lit, then the digits of their square
    /// in ring order, for the panel to set against the door's own `4`s.
    pub fn vault_probe(&self, digits: Vec<u8>) -> Vec<u8> {
        let dials = vault::pack(&digits);
        let mut out = vec![vault::lit_rings(dials) as u8];
        out.extend(vault::square_digits(dials));
        out
    }

    /// Turns the door at a block to these digits: it opens (at the next tick) if they fit.
    pub fn vault_open(&mut self, x: i32, y: i32, z: i32, digits: Vec<u8>) {
        self.act(Action::OpenVault { pos: IVec3::new(x, y, z), dials: vault::pack(&digits) });
    }
}

#[cfg(test)]
mod tests;

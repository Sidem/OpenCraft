//! Ruins for the host: the page of a logbook (`worldgen/logbook.rs`), read by the panel `web/src/ui/logbook.ts` when the
//! player right-clicks one (the request comes through `take_panel_request`, like a machine's).

use wasm_bindgen::prelude::*;

use crate::math::IVec3;
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
}

#[cfg(test)]
mod tests;

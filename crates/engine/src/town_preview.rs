//! Placing a town depot (the local player's, presentation only). It is placed like a multi-block machine
//! (`footprint_preview.rs`); this says beforehand whether a town fits there (`Settlements::site`, a pure function of
//! the generator, so it needs no loaded chunks):
//!
//! - the footprint outline is green where a town fits and red where none does (`depot_outline`, from `api/hud.rs`);
//! - the label's second line names the town it would found, or says why none fits (`depot_line`);
//! - once founded, the target readout names the town on its depot (`town_line`, from `api/hud.rs`).
//!
//! The core checks the site again when the depot is placed (`action/multiblock.rs`).

use crate::block::TOWN_DEPOT;
use crate::math::IVec3;
use crate::settlements::{Unfit, KINDS};
use crate::Game;

const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;

impl Game {
    /// The town the held depot would found where it is aimed (an index into `KINDS`), or why none fits; `None` when no
    /// depot is held and aimed.
    fn depot_site(&self) -> Option<Result<u8, Unfit>> {
        let (block, anchor, _, _) = self.footprint_ghost()?;
        (block == TOWN_DEPOT).then(|| self.sim.settlements.site(self.sim.world.generator(), anchor))
    }

    /// The footprint outline's colour: green where the held depot founds a town, red where none fits, else 0 (amber).
    pub(crate) fn depot_outline(&self) -> i32 {
        match self.depot_site() {
            Some(Ok(_)) => GHOST_GREEN,
            Some(Err(_)) => BLOCKED_RED,
            None => 0,
        }
    }

    /// The label's line about the town and whether one fits: "Founds a harbour here, on a sea shore", or why no town
    /// fits; `None` when no depot is held and aimed.
    pub(crate) fn depot_line(&self) -> Option<(String, bool)> {
        Some(match self.depot_site()? {
            Ok(k) => {
                let kind = &KINDS[k as usize];
                (format!("Founds a {} here, {}", kind.name.to_lowercase(), kind.place), true)
            }
            Err(why) => (why.text(), false),
        })
    }

    /// The target readout's line for a depot, from any of its cells: "Harbour: your town, founded 412 blocks from
    /// spawn"; `None` for anything else.
    pub(crate) fn town_line(&self, cell: IVec3) -> Option<String> {
        let anchor = self.sim.factory.footprint_at(cell)?.0;
        let town = self.sim.settlements.at(anchor)?;
        let away = f64::from(anchor.x).hypot(f64::from(anchor.z)) as i32;
        Some(format!("{}: your town, founded {away} blocks from spawn", KINDS[town.kind as usize].name))
    }
}

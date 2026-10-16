//! Placing a town depot (the local player's, presentation only). It is placed like a multi-block machine
//! (`footprint_preview.rs`); this says beforehand whether a town fits there (`Settlements::site`, a pure function of
//! the generator, so it needs no loaded chunks):
//!
//! - the footprint outline is green where a town fits and red where none does (`depot_outline`, from `api/hud.rs`);
//! - the label's second line names the town it would found, or says why none fits (`depot_line`);
//! - once founded, the target readout on its depot names the town and its stage, and shows the open order and the
//!   upkeep (`town_line`, from `api/hud.rs`).
//!
//! The core checks the site again when the depot is placed (`action/multiblock.rs`).

use crate::block::TOWN_DEPOT;
use crate::factory::fmt_int;
use crate::item::name;
use crate::math::IVec3;
use crate::settlements::{upkeep, Unfit, KINDS, STAGE_NAMES};
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

    /// The target readout's lines for a depot, from any of its cells: the town and its stage, the open order's progress,
    /// and its upkeep and whether it is supplied or stalled; `None` for anything else. For example:
    /// "Harbour, a hamlet, 412 blocks from spawn\nFor a village: Concrete 1,200/2,000 · ...\nUses a minute: Planks 24 ·
    /// Glass 24 · Lamp 12 · supplied".
    pub(crate) fn town_line(&self, cell: IVec3) -> Option<String> {
        let anchor = self.sim.factory.footprint_at(cell)?.0;
        let town = self.sim.settlements.at(anchor)?;
        let away = f64::from(anchor.x).hypot(f64::from(anchor.z)) as i32;
        let (kind, built) = (KINDS[town.kind as usize].name, town.built as usize);
        let mut s = match built {
            0 => format!("{kind}: your town, founded {away} blocks from spawn"),
            _ => format!("{kind}, a {}, {away} blocks from spawn", STAGE_NAMES[built]),
        };
        let lines = |l: &[String]| l.join(" · ");
        let order: Vec<String> = (town.order().iter().zip(&town.delivered))
            .map(|(&(item, want), &got)| format!("{} {}/{}", name(item), fmt_int(got.into()), fmt_int(want.into())))
            .collect();
        match STAGE_NAMES.get(built + 1) {
            Some(next) if town.stalled => s += &format!("\nFor a {next} (waits while stalled): {}", lines(&order)),
            Some(next) => s += &format!("\nFor a {next}: {}", lines(&order)),
            None => s += "\nFully grown",
        }
        let upkeep = upkeep(town.kind, town.built.max(1));
        let held = |item| self.sim.factory.depot_held(anchor).map_or(0, |b| b.count(item));
        let goods: Vec<String> = upkeep.iter().map(|&(item, n)| format!("{} {n}", name(item))).collect();
        if built == 0 {
            return Some(s + &format!("\nThe camp will use a minute: {}", lines(&goods)));
        }
        let short: Vec<&str> =
            upkeep.iter().filter(|&&(item, n)| held(item) < n).map(|&(item, _)| name(item)).collect();
        let state = match (town.stalled, short.is_empty()) {
            (true, _) => format!("STALLED, short of {}", short.join(", ")),
            (false, true) => "supplied".to_string(),
            (false, false) => format!("next minute short of {}", short.join(", ")),
        };
        Some(s + &format!("\nUses a minute: {} · {state}", lines(&goods)))
    }
}

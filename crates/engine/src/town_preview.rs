//! Towns as the local player sees them (presentation only). A depot is placed like a multi-block machine
//! (`footprint_preview.rs`); this says beforehand whether a town fits there (`Settlements::site`, a pure function of
//! the generator, so it needs no loaded chunks):
//!
//! - the footprint outline is green where a town fits and red where none does (`depot_outline`, from `api/hud.rs`);
//! - the label's second line names the town it would found, or says why none fits (`depot_line`);
//! - once founded, the target readout on its depot names the town and its stage, and shows the open order and the
//!   upkeep (`town_line`, from `api/hud.rs`);
//! - while it grows, its drones are drawn (`write_town_instances`, from `drone_view.rs`), finished chimneys smoke, and
//!   the buildings still going up are outlined faintly (`town_boxes`, from `ghost_boxes`);
//! - a dormant town's chimneys go cold and its old depot spot is marked amber (`town_boxes`).
//!
//! The core checks the site again when the depot is placed (`action/multiblock.rs`).

use crate::block::TOWN_DEPOT;
use crate::drone_view::push_drone;
use crate::factory::{fmt_int, smoke};
use crate::item::name;
use crate::math::{IVec3, Vec3};
use crate::settlements::{chimney_mouth, fleet, upkeep, Town, Unfit, KINDS, STAGE_NAMES};
use crate::{Game, TICK};

const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;
/// The faint outline of a town building still going up.
const TOWN_PLAN: i32 = 0xd8c8a0;
/// The marked spot where a dormant town's depot stood.
const DORMANT_SPOT: i32 = 0xffb347;
/// Towns further than this from the eye are not drawn.
const DRAW_RANGE: f64 = 160.0;

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
        let town = self.town_at(cell)?;
        let built = town.built as usize;
        let mut s = self.town_heading_of(town);
        let lines = |l: &[String]| l.join(" · ");
        let order: Vec<String> = (town.order().iter().zip(&town.delivered))
            .map(|(&(item, want), &got)| format!("{} {}/{}", name(item), fmt_int(got.into()), fmt_int(want.into())))
            .collect();
        match STAGE_NAMES.get(built + 1) {
            Some(next) if town.stalled => s += &format!("\nFor a {next} (waits while stalled): {}", lines(&order)),
            Some(next) => s += &format!("\nFor a {next}: {}", lines(&order)),
            None => s += "\nFully grown",
        }
        if (town.cursor as usize) < town.plots.len() {
            let (done, all, out) = (town.cursor, town.plots.len(), town.drones.len());
            s += &format!("\nBuilding: {done} of {all} buildings done · {out} of {} drones out", fleet(town.built));
        }
        if let Some(perk) = town.perk_text() {
            s += &format!("\n{perk}");
        }
        let upkeep = upkeep(town.kind, town.built.max(1));
        let goods: Vec<String> = upkeep.iter().map(|&(item, n)| format!("{} {n}", name(item))).collect();
        if built == 0 {
            return Some(s + &format!("\nThe camp will use a minute: {}", lines(&goods)));
        }
        let short = self.short_of(town);
        let state = match (town.stalled, short.is_empty()) {
            (true, _) => format!("STALLED, short of {}", short.join(", ")),
            (false, true) => "supplied".to_string(),
            (false, false) => format!("next minute short of {}", short.join(", ")),
        };
        Some(s + &format!("\nUses a minute: {} · {state}", lines(&goods)))
    }

    /// The awake town whose depot has a cell at `cell`.
    pub(crate) fn town_at(&self, cell: IVec3) -> Option<&Town> {
        self.sim.settlements.at(self.sim.factory.footprint_at(cell)?.0)
    }

    /// "Basalt Forge, a village, 474 blocks from spawn", or "…: your town, founded …" before its camp.
    pub(crate) fn town_heading_of(&self, town: &Town) -> String {
        let away = f64::from(town.pos.x).hypot(f64::from(town.pos.z)) as i32;
        let kind = KINDS[town.kind as usize].name;
        match town.built {
            0 => format!("{kind}: your town, founded {away} blocks from spawn"),
            b => format!("{kind}, a {}, {away} blocks from spawn", STAGE_NAMES[b as usize]),
        }
    }

    /// The upkeep goods its depot holds less than a minute of.
    pub(crate) fn short_of(&self, town: &Town) -> Vec<&'static str> {
        let held = |item| self.sim.factory.depot_held(town.pos).map_or(0, |b| b.count(item));
        upkeep(town.kind, town.built.max(1))
            .iter()
            .filter(|&&(item, n)| held(item) < n)
            .map(|&(item, _)| name(item))
            .collect()
    }

    /// Draws the towns near the eye: their drones (blended between ticks, each with a block slung below) and the smoke
    /// from their finished buildings' chimneys.
    pub(crate) fn write_town_instances(&mut self, eye: Vec3, alpha: f64) {
        let time = (self.sim.tick as f64 + alpha) * TICK;
        for t in &self.sim.settlements.towns {
            if (t.pos.as_vec3() - eye).length() > DRAW_RANGE {
                continue;
            }
            for (k, d) in t.drones.iter().enumerate() {
                let bob = (time * 6.0 + k as f64).sin() * 0.04;
                let at = d.prev + (d.pos - d.prev) * alpha + Vec3::new(0.0, bob, 0.0);
                push_drone(&mut self.instances, at - eye, (time * 0.5) as f32, true);
            }
            for p in t.plots.iter().take(t.cursor as usize).filter(|p| p.chimney(t.kind) && !t.dormant) {
                let mouth = p.corner(t.pos) + chimney_mouth(p.shape());
                let at = mouth.as_vec3() + Vec3::new(0.5, 0.0, 0.5) - eye;
                smoke::puffs(&mut self.instances, at, time, smoke::seed(mouth), 1.0);
            }
        }
    }

    /// Appends faint outlines of the buildings the towns near the eye are still building (see `ghost_boxes`).
    pub(crate) fn town_boxes(&self, out: &mut Vec<i32>) {
        let eye = self.body().eye();
        for t in &self.sim.settlements.towns {
            if (t.pos.as_vec3() - eye).length() > DRAW_RANGE {
                continue;
            }
            if t.dormant {
                // Where its depot stood: put one back there to wake it.
                out.extend([t.pos.x, t.pos.y, t.pos.z, t.pos.x, t.pos.y, t.pos.z, DORMANT_SPOT]);
            }
            for p in t.plots.iter().skip(t.cursor as usize) {
                let (lo, s) = (p.corner(t.pos), p.shape());
                let hi = lo + IVec3::new(s.w - 1, crate::settlements::height(s) - 1, s.d - 1);
                out.extend([lo.x, lo.y - 1, lo.z, hi.x, hi.y, hi.z, TOWN_PLAN]);
            }
        }
    }
}

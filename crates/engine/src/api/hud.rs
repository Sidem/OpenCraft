//! HUD readouts: player state, the targeted block with its mining progress and detail text, a belt
//! line being dragged, the onboarding hints, and the stats overlay counters.

use wasm_bindgen::prelude::*;

use crate::block::{self, AIR, SPENT_ROCK};
use crate::deposits::{owner_of, DepositState, HAND_YIELD};
use crate::factory::{self, Kind, MINER_TIERS};
use crate::hints::{self, HINTS};
use crate::math::IVec3;
use crate::ore_guide;
use crate::research::{self, Unlock};
use crate::upgrade_aim::BLOCKED_RED;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn flying(&self) -> bool {
        self.body().flying
    }

    pub fn on_ground(&self) -> bool {
        self.body().on_ground
    }

    /// The belt line being dragged out (belt_line.rs): 4 numbers per cell (x, y, z, 1 if it will be
    /// built, 0 past the belts in hand); empty when there is none.
    pub fn line_cells(&self) -> Vec<i32> {
        self.planned_cells()
    }

    /// Boxes to outline while placing, 7 numbers each: lowest and highest cells, then a colour (0xRRGGBB;
    /// 0 for amber). The box a held quarry would dig, or a held multi-block machine's footprint with a
    /// red box on each cell in the way, or the machine a held upgrade kit would upgrade; empty otherwise.
    pub fn placement_box(&self) -> Vec<i32> {
        let mut boxes = self.held_boxes();
        boxes.extend(self.ghost_cell_box());
        boxes.extend(self.ghost_boxes());
        boxes.extend(self.blueprint_boxes());
        boxes
    }

    fn held_boxes(&self) -> Vec<i32> {
        let upgrade = self.aim_box();
        if !upgrade.is_empty() {
            return upgrade;
        }
        if let Some(d) = self.quarry_preview() {
            let (lo, hi) = d.bounds();
            return vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, 0];
        }
        let power = self.power_boxes();
        if !power.is_empty() {
            return power;
        }
        let Some((_, _, _, cells)) = self.footprint_ghost() else { return Vec::new() };
        let lo = cells.iter().fold(cells[0].0, |m, c| IVec3::new(m.x.min(c.0.x), m.y.min(c.0.y), m.z.min(c.0.z)));
        let hi = cells.iter().fold(cells[0].0, |m, c| IVec3::new(m.x.max(c.0.x), m.y.max(c.0.y), m.z.max(c.0.z)));
        let mut out = vec![lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, 0];
        for &(c, _) in cells.iter().filter(|c| c.1) {
            out.extend([c.x, c.y, c.z, c.x, c.y, c.z, BLOCKED_RED]);
        }
        out
    }

    /// What the HUD says while a belt line is dragged out or a quarry or multi-block machine is about to
    /// be placed: a title line, then the details ("" otherwise).
    pub fn line_label(&self) -> String {
        let n = self.line.cells.len();
        if n == 0 {
            if self.ghost_mode {
                return self.ghost_label();
            }
            if !self.line.aim.label.is_empty() {
                return self.line.aim.label.clone();
            }
            let quarry = self.quarry_label();
            let footprint = if quarry.is_empty() { self.footprint_label() } else { quarry };
            if !footprint.is_empty() {
                return footprint;
            }
            let power = self.power_label();
            return if power.is_empty() { self.helper_label() } else { power };
        }
        let held = self.inventory().selected_stack().item;
        if let Some(tier) = factory::upgrades::kit_tier(held) {
            let per = factory::tiers::family(block::BELT).map_or(1, |f| f.kits) as usize;
            let (kits, have) = (n * per, self.inventory().count(held));
            let short = if (have as usize) < kits { format!(" (you have {have})") } else { String::new() };
            if let Some(t) = self.sim.factory.research.locked_by(Unlock::Upgrade(block::BELT, tier)) {
                return format!("Belt upgrade to Mk{}\nResearch {} first", tier + 1, research::TECHS[t as usize].name);
            }
            let belts = if n == 1 { "belt" } else { "belts" };
            return format!(
                "Belt upgrade to Mk{}\n{n} {belts} · {kits} kits{short} · release to upgrade, left-click to cancel",
                tier + 1
            );
        }
        let have = self.inventory().count(held) as usize;
        let sloped = self.line.cells.iter().filter(|c| c.shape != factory::Shape::Flat).count();
        let mut s = format!("Belt line\n{n} belt{}", if n == 1 { "" } else { "s" });
        if sloped > 0 {
            s += &format!(", {sloped} on slopes");
        }
        if have < n {
            s += &format!(" · you have {have}");
        }
        s + " · release to build, left-click to cancel"
    }

    pub fn has_target(&self) -> bool {
        self.target.is_some()
    }

    pub fn target_x(&self) -> i32 {
        self.target.map_or(0, |t| t.block.x)
    }

    pub fn target_y(&self) -> i32 {
        self.target.map_or(0, |t| t.block.y)
    }

    pub fn target_z(&self) -> i32 {
        self.target.map_or(0, |t| t.block.z)
    }

    pub fn target_block(&self) -> u8 {
        self.target.map_or(AIR, |t| t.id)
    }

    /// 0..1 while the targeted block is being mined, or drilled for a core sample.
    pub fn mine_progress(&self) -> f32 {
        if let Some(p) = self.prospect.drill_progress() {
            p
        } else if self.mine_block.is_some() {
            self.mine_progress
        } else {
            0.0
        }
    }

    /// Extra lines for the target readout: deposit details for ore, status for machines, what lies
    /// under stained soil.
    /// Lines are separated by `\n`; empty when there is nothing to add. Leaves the core unchanged:
    /// a deposit nobody has touched is surveyed into `self.surveyed`, not tracked.
    pub fn target_detail(&mut self) -> String {
        let Some(hit) = self.target else { return String::new() };
        if let Some(text) = self.sim.factory.describe(hit.block) {
            return text;
        }
        if let Some(text) = ore_guide::stain_reading(self.sim.world.generator(), hit.block, hit.id) {
            return text;
        }
        if !block::is_ore(hit.id) && hit.id != SPENT_ROCK {
            return String::new();
        }
        let Some(d) = owner_of(&mut self.sim.world, hit.block) else { return String::new() };
        let untracked = self.sim.factory.deposits.get(&d.key).is_none();
        if untracked && self.surveyed.as_ref().is_none_or(|s| s.deposit.key != d.key) {
            self.surveyed = Some(DepositState::survey(&mut self.sim.world, d));
        }
        let Some(st) = self.sim.factory.deposits.get(&d.key).or(self.surveyed.as_ref()) else { return String::new() };
        let int = |n: u64| factory::fmt_int(n);
        let left = format!(
            "{} of {} blocks left · {} units",
            int(st.remaining_blocks as u64),
            int(st.initial_blocks as u64),
            int(st.remaining_units() as u64)
        );
        if hit.id == SPENT_ROCK {
            return format!("Worked-out part of a {}\n{left}", st.deposit.name());
        }
        let grade = st.grade();
        format!(
            "{} · {} units per block\n{left}\nBy hand you keep {HAND_YIELD} and lose {}. A Miner Mk1 recovers {}%, a Mk2 {}%.",
            st.deposit.name(),
            int(grade as u64),
            int(grade.saturating_sub(HAND_YIELD) as u64),
            (MINER_TIERS[0].recovery * 100.0).round() as u32,
            (MINER_TIERS[1].recovery * 100.0).round() as u32
        )
    }

    pub fn hint_count(&self) -> u32 {
        HINTS.len() as u32
    }

    pub fn hint_text(&self, i: u32) -> String {
        HINTS.get(i as usize).map_or_else(String::new, |h| h.text.to_string())
    }

    /// The first hint the local player hasn't done yet (`hint_count` when all are done).
    pub fn hint_progress(&self) -> u32 {
        hints::progress(self.inventory(), &self.sim.factory) as u32
    }

    pub fn chunks_loaded(&self) -> u32 {
        self.sim.world.loaded_count() as u32
    }

    pub fn chunks_pending(&self) -> u32 {
        self.sim.world.pending_count() as u32
    }

    pub fn chunks_dirty(&self) -> u32 {
        self.sim.world.dirty_count() as u32
    }

    pub fn item_entities(&self) -> u32 {
        self.items.list.len() as u32
    }

    pub fn belts(&self) -> u32 {
        self.sim.factory.count(Kind::Belt) as u32
    }

    pub fn miners(&self) -> u32 {
        self.sim.factory.count(Kind::Miner) as u32
    }

    pub fn boxes(&self) -> u32 {
        self.sim.factory.count(Kind::Storage) as u32
    }

    pub fn deposits_tracked(&self) -> u32 {
        self.sim.factory.deposits.tracked() as u32
    }
}

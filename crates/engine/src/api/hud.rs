//! HUD readouts: player state, the targeted block with its mining progress and detail text, and
//! the stats overlay counters.

use wasm_bindgen::prelude::*;

use crate::block::{self, AIR, SPENT_ROCK};
use crate::deposits::{owner_of, DepositState, HAND_YIELD};
use crate::factory::{self, MINER_RECOVERY, MK2_RECOVERY};
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn flying(&self) -> bool {
        self.body().flying
    }

    pub fn on_ground(&self) -> bool {
        self.body().on_ground
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

    /// 0..1 while the targeted block is being mined.
    pub fn mine_progress(&self) -> f32 {
        if self.mine_block.is_some() {
            self.mine_progress
        } else {
            0.0
        }
    }

    /// Extra lines for the target readout: deposit details for ore, status for machines.
    /// Lines are separated by `\n`; empty when there is nothing to add. Leaves the core unchanged:
    /// a deposit nobody has touched is surveyed into `self.surveyed`, not tracked.
    pub fn target_detail(&mut self) -> String {
        let Some(hit) = self.target else { return String::new() };
        if let Some(text) = self.sim.factory.describe(hit.block) {
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
            (MINER_RECOVERY * 100.0).round() as u32,
            (MK2_RECOVERY * 100.0).round() as u32
        )
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
        self.sim.factory.belt_count() as u32
    }

    pub fn miners(&self) -> u32 {
        self.sim.factory.miner_count() as u32
    }

    pub fn boxes(&self) -> u32 {
        self.sim.factory.storage_count() as u32
    }

    pub fn deposits_tracked(&self) -> u32 {
        self.sim.factory.deposits.tracked() as u32
    }
}

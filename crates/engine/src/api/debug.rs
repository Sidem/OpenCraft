//! Debug and testing helpers, used from the browser console (`window.opencraft.game`) and tests:
//! give items, teleport, fast-forward the factory, find deposits, read blocks and the position.

use wasm_bindgen::prelude::*;

use crate::block::{AIR, BLOCK_COUNT};
use crate::deposits::Tier;
use crate::math::{IVec3, Vec3};
use crate::sound::Sounds;
use crate::Game;

/// Factory step used when fast-forwarding time.
const SKIP_STEP: f64 = 0.05;

#[wasm_bindgen]
impl Game {
    /// Debug / creative helper: adds items straight to the inventory.
    pub fn give(&mut self, item: u8, count: u32) -> u32 {
        if item != AIR && (item as usize) < BLOCK_COUNT {
            self.inventory.add(item, count)
        } else {
            count
        }
    }

    pub fn teleport(&mut self, x: f64, y: f64, z: f64) {
        self.player.pos = Vec3::new(x, y, z);
        self.player.vel = Vec3::ZERO;
    }

    /// Runs the factory (miners, belts, boxes) for `seconds` of game time at once.
    pub fn skip_time(&mut self, seconds: f64) {
        let eye = self.player.eye();
        let mut quiet = Sounds::default();
        let mut left = seconds.max(0.0);
        while left > 0.0 {
            let dt = left.min(SKIP_STEP);
            self.factory.update(dt, &mut self.world, eye, &mut quiet);
            self.time += dt;
            left -= dt;
        }
    }

    /// Prospecting aid for testing: centre and ore of the nearest deposit of `tier`
    /// (0 = lode, 1 = vein, 2 = outcrop) as `[x, y, z, ore]`, or empty if none is within ~500 blocks.
    pub fn find_deposit(&self, tier: u8) -> Vec<i32> {
        let Some(tier) = Tier::from_u8(tier) else { return Vec::new() };
        self.world
            .generator()
            .find_deposit(self.player.pos.floor(), tier, 16)
            .map_or_else(Vec::new, |d| vec![d.center.x, d.center.y, d.center.z, d.ore() as i32])
    }

    /// Block at a position in a loaded chunk (air otherwise). For testing and the console.
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.world.get_block(IVec3::new(x, y, z)).unwrap_or(AIR)
    }

    pub fn player_x(&self) -> f64 {
        self.player.pos.x
    }

    pub fn player_y(&self) -> f64 {
        self.player.pos.y
    }

    pub fn player_z(&self) -> f64 {
        self.player.pos.z
    }
}

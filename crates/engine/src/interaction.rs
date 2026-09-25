//! The local player's hands and feet: targeting, mining progress, right-click, throwing items, and
//! footstep / landing sounds.
//!
//! Called from `Game::run_tick` (lib.rs), which advances the timers by one `TICK`. The hands only
//! decide *what* to do and queue an `Action` (break, place, take contents); the core applies it
//! (`action.rs`). What a placed block does is decided there, in `Sim::place_block`.

use crate::action::Action;
use crate::block::{self, BlockId, AIR, MINER, STORAGE};
use crate::factory;
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;
use crate::player;
use crate::raycast::raycast;
use crate::sound;
use crate::{Game, LOCAL};

const REACH: f64 = 5.0;
const PLACE_REPEAT_SECONDS: f32 = 0.22;
const BREAK_COOLDOWN_SECONDS: f32 = 0.12;
const DIG_SOUND_INTERVAL: f32 = 0.24;
/// Horizontal distance walked between footstep sounds.
const STEP_STRIDE: f64 = 1.7;
/// Touchdowns slower than this (e.g. walking down a slab edge) make no landing sound.
const LAND_SOUND_MIN_SPEED: f64 = 5.0;

impl Game {
    /// Queues a sound at a world position (stored camera-relative for the host).
    pub(crate) fn play(&mut self, kind: u8, material: u8, at: Vec3, volume: f64) {
        self.sounds.push(kind, material, at - self.player.eye(), volume);
    }

    /// Throws items out in front of the player.
    pub(crate) fn throw(&mut self, item: BlockId, n: u32) {
        let dir = self.player.look_dir();
        let pos = self.player.eye() + dir * 0.4 - Vec3::new(0.0, 0.3, 0.0);
        self.items.spawn(pos, dir * 6.0 + Vec3::new(0.0, 1.5, 0.0), item, n, 1.5);
        self.play(sound::DROP, 0, pos, 1.0);
    }

    pub(crate) fn update_movement_sounds(&mut self, feet_before: Vec3) {
        let landing = std::mem::take(&mut self.player.landing_speed);
        if self.player.flying || !self.player.on_ground {
            return;
        }
        let Some(material) = self.ground_material() else { return };
        let feet = self.player.pos;
        if landing > LAND_SOUND_MIN_SPEED {
            let volume = ((landing - LAND_SOUND_MIN_SPEED) / 15.0).clamp(0.3, 1.0);
            self.play(sound::LAND, material, feet, volume);
            self.step_distance = 0.0;
            return;
        }
        let (dx, dz) = (feet.x - feet_before.x, feet.z - feet_before.z);
        self.step_distance += (dx * dx + dz * dz).sqrt();
        if self.step_distance >= STEP_STRIDE {
            self.step_distance = 0.0;
            let inp = self.player.input;
            let volume = if inp.crouch {
                0.3
            } else if inp.sprint {
                0.8
            } else {
                0.6
            };
            self.play(sound::STEP, material, feet, volume);
        }
    }

    pub(crate) fn update_target(&mut self) {
        let world = &self.sim.world;
        self.target =
            raycast(self.player.eye(), self.player.look_dir(), REACH, |p| world.get_block(p).filter(|&b| b != AIR));
    }

    pub(crate) fn update_mining(&mut self, dt: f32) {
        self.mine_cooldown = (self.mine_cooldown - dt).max(0.0);
        let Some(hit) = self.target.filter(|_| self.mining) else {
            self.mine_block = None;
            self.mine_progress = 0.0;
            self.dig_timer = 0.0;
            return;
        };
        if self.mine_cooldown > 0.0 {
            return;
        }
        if self.mine_block != Some(hit.block) {
            self.mine_block = Some(hit.block);
            self.mine_progress = 0.0;
            self.dig_timer = 0.0;
        }
        let def = block::def(hit.id);
        let center = hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        // Dig ticks play even on unbreakable blocks, so the player hears that they are hitting it.
        self.dig_timer -= dt;
        if self.dig_timer <= 0.0 {
            self.dig_timer = DIG_SOUND_INTERVAL;
            self.play(sound::DIG, def.sound, center, 1.0);
        }
        if def.break_time < 0.0 {
            return;
        }
        self.mine_progress += if def.break_time == 0.0 { 1.0 } else { dt / def.break_time };
        if self.mine_progress < 1.0 {
            return;
        }
        self.act(Action::BreakBlock { pos: hit.block });
        self.dig_timer = 0.0;
        self.mine_block = None;
        self.mine_progress = 0.0;
        self.mine_cooldown = BREAK_COOLDOWN_SECONDS;
    }

    pub(crate) fn update_placing(&mut self, dt: f32) {
        if !self.using {
            return;
        }
        self.use_cooldown -= dt;
        if self.use_cooldown > 0.0 {
            return;
        }
        if let Some(action) = self.right_click_action() {
            self.act(action);
            self.use_cooldown = PLACE_REPEAT_SECONDS;
        }
    }

    /// What right-clicking the target would do: empty a box or miner (unless crouching), or place
    /// the selected block against the targeted face. `None` when it would fail, so holding the
    /// button keeps trying.
    fn right_click_action(&self) -> Option<Action> {
        let hit = self.target?;
        if !self.player.input.crouch && matches!(hit.id, MINER | STORAGE) {
            return Some(Action::TakeContents { pos: hit.block });
        }
        if hit.normal == IVec3::ZERO {
            return None;
        }
        let inv = &self.sim.player(LOCAL).inventory;
        let stack = inv.selected_stack();
        if stack.is_empty() || !block::is_placeable(stack.item) {
            return None;
        }
        let pos = hit.block + hit.normal;
        if self.sim.world.get_block(pos) != Some(AIR) {
            return None;
        }
        // The body is the authority's to check: don't place a solid block where the player stands.
        let cell = Aabb { min: pos.as_vec3(), max: pos.as_vec3() + Vec3::new(1.0, 1.0, 1.0) };
        if block::SOLID[stack.item as usize] && cell.intersects(&self.player.aabb()) {
            return None;
        }
        let facing = factory::dir_from_yaw(self.player.yaw);
        Some(Action::PlaceBlock { pos, slot: inv.selected as u8, facing, against: hit.block })
    }

    /// Sound material of the block the player is standing on (checks the footprint corners so
    /// standing on an edge still finds the supporting block).
    fn ground_material(&self) -> Option<u8> {
        let p = self.player.pos;
        let y = (p.y - 0.05).floor() as i32;
        let hw = player::HALF_WIDTH;
        for (dx, dz) in [(0.0, 0.0), (-hw, -hw), (hw, -hw), (hw, hw), (-hw, hw)] {
            let b = self.sim.world.get_block(IVec3::new((p.x + dx).floor() as i32, y, (p.z + dz).floor() as i32))?;
            if block::SOLID[b as usize] {
                return Some(block::def(b).sound);
            }
        }
        None
    }
}

//! The local player's hands and feet: targeting, mining progress, breaking and placing blocks,
//! emptying machines, throwing items, and footstep / landing sounds.
//!
//! Called from `Game::run_tick` (lib.rs), which advances the timers by one `TICK`, and from a few API
//! methods. DEV_PLAN step 1.3 turns breaking and placing into actions.
//! To make a new block kind do something when placed: add a match arm in `try_place`.

use crate::block::{self, BlockId, AIR, BELT, MINER, STORAGE};
use crate::deposits::HAND_YIELD;
use crate::factory;
use crate::inventory::Stack;
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
        self.break_block(hit.block, hit.id);
        self.dig_timer = 0.0;
        self.mine_block = None;
        self.mine_progress = 0.0;
        self.mine_cooldown = BREAK_COOLDOWN_SECONDS;
        self.update_target();
    }

    /// Breaks a block by hand. Ore keeps only [`HAND_YIELD`] items and costs its deposit a whole
    /// block's share; machines drop themselves plus whatever they held.
    pub(crate) fn break_block(&mut self, p: IVec3, id: BlockId) -> bool {
        let ore = block::is_ore(id);
        if ore {
            self.sim.factory.deposits.hand_mined(&mut self.sim.world, p);
        }
        if !self.sim.world.set_block_anywhere(p, AIR) {
            return false;
        }
        let def = block::def(id);
        let center = p.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        self.play(sound::BREAK, def.sound, center, 1.0);
        let mut drops = self.sim.factory.remove(p);
        if def.drop != AIR {
            drops.insert(0, Stack { item: def.drop, count: if ore { HAND_YIELD } else { 1 } });
        }
        for s in drops {
            let vel = Vec3::new(self.sim.rng.range(-1.5, 1.5), 4.0, self.sim.rng.range(-1.5, 1.5));
            self.items.spawn(center, vel, s.item, s.count, 0.25);
        }
        true
    }

    pub(crate) fn update_placing(&mut self, dt: f32) {
        if !self.using {
            return;
        }
        self.use_cooldown -= dt;
        if self.use_cooldown > 0.0 {
            return;
        }
        if self.try_place() {
            self.use_cooldown = PLACE_REPEAT_SECONDS;
            self.update_target();
        }
    }

    /// Moves a box's or miner's contents into the inventory. False if `pos` is neither.
    pub(crate) fn take_from_machine(&mut self, pos: IVec3) -> bool {
        let inventory = &mut self.sim.players[LOCAL].inventory;
        let pickups = &mut self.pickups;
        let mut got = 0;
        let handled = self.sim.factory.take_contents(pos, |item, n| {
            let taken = n - inventory.add(item, n);
            if taken > 0 {
                pickups.push_back((item, taken));
                got += taken;
            }
            taken
        });
        if got > 0 {
            self.sounds.push(sound::PICKUP, 0, Vec3::new(0.0, -0.6, 0.0), 1.0);
        }
        handled
    }

    /// Right-click: empties a targeted box or miner (unless crouching), otherwise places the
    /// selected block against the targeted face.
    fn try_place(&mut self) -> bool {
        let Some(hit) = self.target else { return false };
        if !self.player.input.crouch && self.take_from_machine(hit.block) {
            return true;
        }
        if hit.normal == IVec3::ZERO {
            return false;
        }
        let stack = self.sim.players[LOCAL].inventory.selected_stack();
        if stack.is_empty() || !block::is_placeable(stack.item) {
            return false;
        }
        let p = hit.block + hit.normal;
        if self.sim.world.block_anywhere_or_generate(p) != AIR {
            return false;
        }
        let cell = Aabb { min: p.as_vec3(), max: p.as_vec3() + Vec3::new(1.0, 1.0, 1.0) };
        if block::SOLID[stack.item as usize] && cell.intersects(&self.player.aabb()) {
            return false;
        }
        if !self.sim.world.set_block_anywhere(p, stack.item) {
            return false;
        }
        match stack.item {
            BELT => self.sim.factory.add_belt(p, factory::dir_from_yaw(self.player.yaw)),
            MINER => {
                let deposit = self.sim.factory.deposits.lookup(&mut self.sim.world, hit.block);
                let drill = factory::face_of(hit.block - p).unwrap_or(block::FACE_BOTTOM as u8);
                self.sim.factory.add_miner(p, drill, deposit);
            }
            STORAGE => self.sim.factory.add_storage(p),
            _ => {}
        }
        self.sim.players[LOCAL].inventory.take_selected(1);
        self.play(sound::PLACE, block::def(stack.item).sound, p.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 1.0);
        true
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

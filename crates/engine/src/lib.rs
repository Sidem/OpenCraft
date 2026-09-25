//! OpenCraft engine: the whole simulation (world streaming, terrain generation, meshing, physics,
//! interaction, inventory, ore deposits and factory machines) compiled to WebAssembly.
//!
//! The JavaScript host only owns the platform: WebGL2, input and DOM UI. Bulk data (chunk meshes,
//! box instances, textures) never gets copied across the boundary; the host reads it straight out
//! of wasm linear memory through `*_ptr` / `*_len` accessors.
//!
//! This file holds the `Game` struct, its constructor and the per-frame `update`. The JS-facing API
//! lives in `api/*.rs` (one `#[wasm_bindgen] impl Game` block per area); every method there only
//! forwards to a module. Mining, placing and movement sounds live in `interaction.rs`.
//! To add a wasm method: put it in the matching `api/` file (see docs/CODEMAP.md).

mod api;
mod block;
mod chunk;
mod deposits;
mod entities;
mod factory;
mod interaction;
mod inventory;
mod math;
mod mesher;
mod noise;
mod physics;
mod player;
mod raycast;
mod recipes;
mod sound;
mod textures;
mod world;
mod worldgen;

use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

use block::{BlockId, AIR};
use entities::Items;
use factory::Factory;
use inventory::Inventory;
use math::{hash2, IVec3, Rng, Vec3};
use player::Player;
use raycast::RayHit;
use sound::Sounds;
use world::{MeshData, World};

const PHYSICS_STEP: f64 = 1.0 / 120.0;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Game {
    world: World,
    player: Player,
    inventory: Inventory,
    items: Items,
    factory: Factory,
    rng: Rng,
    spawn: Vec3,
    time: f64,
    target: Option<RayHit>,
    mining: bool,
    mine_block: Option<IVec3>,
    mine_progress: f32,
    mine_cooldown: f32,
    using: bool,
    use_cooldown: f32,
    dig_timer: f32,
    step_distance: f64,
    sounds: Sounds,
    textures: Vec<u8>,
    /// Box instances (dropped items, belt items, machine parts) for the current frame.
    instances: Vec<f32>,
    pickups: VecDeque<(BlockId, u32)>,
    cur_pickup: (BlockId, u32),
    cur_mesh: Option<MeshData>,
    cur_event_pos: IVec3,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, view_radius: u32) -> Game {
        let world = World::new(seed, view_radius as i32);
        let spawn = Vec3::new(0.5, world.generator().height_at(0, 0) as f64 + 1.0, 0.5);
        Game {
            world,
            player: Player::new(spawn),
            inventory: Inventory::default(),
            items: Items::default(),
            factory: Factory::default(),
            rng: Rng::new(hash2(seed, 17, 42) as u64),
            spawn,
            time: 0.0,
            target: None,
            mining: false,
            mine_block: None,
            mine_progress: 0.0,
            mine_cooldown: 0.0,
            using: false,
            use_cooldown: 0.0,
            dig_timer: 0.0,
            step_distance: 0.0,
            sounds: Sounds::default(),
            textures: textures::generate(),
            instances: Vec::new(),
            pickups: VecDeque::new(),
            cur_pickup: (AIR, 0),
            cur_mesh: None,
            cur_event_pos: IVec3::ZERO,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let dt = dt.clamp(0.0, 0.1);
        self.world.update_streaming(self.player.pos);

        let feet = self.player.pos;
        if self.world.is_loaded(feet) && self.world.is_loaded(feet - Vec3::new(0.0, 1.0, 0.0)) {
            let steps = (dt / PHYSICS_STEP).ceil().max(1.0) as u32;
            let h = dt / steps as f64;
            let world = &self.world;
            let mut solid = |x, y, z| world.is_solid(x, y, z);
            for _ in 0..steps {
                self.player.step(h, &mut solid);
            }
        }
        if self.player.pos.y < -64.0 {
            self.teleport(self.spawn.x, self.spawn.y + 2.0, self.spawn.z);
        }
        self.update_movement_sounds(feet);

        self.update_target();
        self.update_mining(dt as f32);
        self.update_placing(dt as f32);

        let world = &self.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let loaded = |p: Vec3| world.is_loaded(p);
        let pickups = &mut self.pickups;
        let sounds = &mut self.sounds;
        let center = self.player.pos + Vec3::new(0.0, player::HEIGHT * 0.5, 0.0);
        self.items.update(dt, center, &mut solid, &loaded, &mut self.inventory, |item, n| {
            match pickups.back_mut() {
                Some((last, count)) if *last == item => *count += n,
                _ => pickups.push_back((item, n)),
            }
            sounds.push(sound::PICKUP, 0, Vec3::new(0.0, -0.6, 0.0), 1.0);
        });

        let eye = self.player.eye();
        self.factory.update(dt, &mut self.world, eye, &mut self.sounds);
        self.time += dt;
        self.instances.clear();
        self.items.write_instances(&mut self.instances, eye);
        self.factory.write_instances(&mut self.instances, eye, self.time, self.world.view_distance());
    }
}

#[cfg(test)]
mod tests;

//! OpenCraft engine: the whole simulation (world streaming, terrain generation, meshing, physics,
//! interaction, inventory, ore deposits and factory machines) compiled to WebAssembly.
//!
//! The JavaScript host only owns the platform: WebGL2, input and DOM UI. Bulk data (chunk meshes,
//! box instances, textures) never gets copied across the boundary; the host reads it straight out
//! of wasm linear memory through `*_ptr` / `*_len` accessors.
//!
//! This file holds the `Game` struct, its constructor, the per-frame `update` and the fixed tick
//! (`run_tick`). `Game` wraps the deterministic core (`sim.rs`) with the local player's body, loose
//! items and presentation (camera, sounds, instances). The JS-facing API lives in `api/*.rs` (one
//! `#[wasm_bindgen] impl Game` block per area); every method there only forwards to a module. `Game`
//! never edits the core directly: it queues `Action`s (`act`), applied at the next tick. The hands
//! (mining, right-click, footsteps) live in `interaction.rs`, reactions to core events in `events.rs`.
//! To add a wasm method: put it in the matching `api/` file (see docs/CODEMAP.md).
//!
//! Invariant: game state advances only in `run_tick`, by exactly [`TICK`] seconds, so the same inputs
//! give the same results at any frame rate. `update` turns frame time into whole ticks and does the
//! per-frame presentation work (streaming, the interpolated camera, box instances). Look direction is
//! the one input applied per frame, for responsiveness.

mod action;
mod api;
mod block;
mod chunk;
mod deposits;
mod entities;
mod events;
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
mod sim;
mod sound;
mod textures;
mod world;
mod worldgen;

use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

use action::Action;
use block::{BlockId, AIR};
use deposits::DepositState;
use entities::Items;
use math::{IVec3, Vec3};
use player::Player;
use raycast::RayHit;
use sim::{PlayerId, Sim};
use sound::Sounds;
use world::MeshData;

/// Simulation ticks per second.
pub const TICK_RATE: u32 = 60;
/// Length of one tick, in seconds.
pub const TICK: f64 = 1.0 / TICK_RATE as f64;
/// Player physics substeps per tick (1/120 s each).
const PHYSICS_SUBSTEPS: u32 = 2;
/// Most ticks one frame may run. A 0.1 s frame (the host's cap) plus a leftover partial tick needs
/// 7; anything longer (e.g. a hidden tab) drops the excess instead of spiralling.
const MAX_TICKS_PER_FRAME: u32 = 8;
/// Float slack when comparing accumulated frame time with `TICK`, so 60 frames of 1/60 s run 60 ticks.
const TICK_SLACK: f64 = 1e-9;
/// The local player (step 1.4 makes this a `Game` field).
const LOCAL: PlayerId = PlayerId(0);

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct Game {
    /// The deterministic core: world blocks, factory, deposits, inventories, tick.
    sim: Sim,
    player: Player,
    items: Items,
    spawn: Vec3,
    /// Frame time not yet run as ticks, in seconds.
    accumulator: f64,
    /// Player eye at the start of the latest tick, and this frame's camera, interpolated between it
    /// and the current eye (presentation only).
    prev_eye: Vec3,
    render_eye: Vec3,
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
    /// Figures of the last untracked deposit `target_detail` showed, so looking stays cheap
    /// without making the core track it.
    surveyed: Option<DepositState>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, view_radius: u32) -> Game {
        let sim = Sim::new(seed, view_radius as i32);
        let spawn = Vec3::new(0.5, sim.world.generator().height_at(0, 0) as f64 + 1.0, 0.5);
        let player = Player::new(spawn);
        let eye = player.eye();
        Game {
            sim,
            player,
            items: Items::default(),
            spawn,
            accumulator: 0.0,
            prev_eye: eye,
            render_eye: eye,
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
            surveyed: None,
        }
    }

    /// Per frame: runs the whole ticks that `dt` seconds of frame time add up to, then prepares this
    /// frame's camera and box instances.
    pub fn update(&mut self, dt: f64) {
        self.sim.world.update_streaming(self.player.pos);
        self.accumulator += dt.max(0.0);
        let mut ran = 0;
        while self.accumulator >= TICK - TICK_SLACK && ran < MAX_TICKS_PER_FRAME {
            self.run_tick();
            self.accumulator -= TICK;
            ran += 1;
        }
        if self.accumulator >= TICK {
            self.accumulator = 0.0;
        }

        let alpha = (self.accumulator / TICK).clamp(0.0, 1.0);
        self.render_eye = self.prev_eye + (self.player.eye() - self.prev_eye) * alpha;
        self.update_target();
        let (eye, time) = (self.render_eye, (self.sim.tick as f64 + alpha) * TICK);
        self.instances.clear();
        self.items.write_instances(&mut self.instances, eye);
        self.sim.factory.write_instances(&mut self.instances, eye, time, self.sim.world.view_distance());
    }
}

impl Game {
    /// Advances the game by one tick of exactly [`TICK`] seconds: the local player's body and hands,
    /// loose items, then the core (`Sim::step` applies this tick's actions), then its events.
    fn run_tick(&mut self) {
        self.prev_eye = self.player.eye();
        let feet = self.player.pos;
        if self.sim.world.is_loaded(feet) && self.sim.world.is_loaded(feet - Vec3::new(0.0, 1.0, 0.0)) {
            let world = &self.sim.world;
            let mut solid = |x, y, z| world.is_solid(x, y, z);
            for _ in 0..PHYSICS_SUBSTEPS {
                self.player.step(TICK / PHYSICS_SUBSTEPS as f64, &mut solid);
            }
        }
        if self.player.pos.y < -64.0 {
            self.teleport(self.spawn.x, self.spawn.y + 2.0, self.spawn.z);
        }
        self.update_movement_sounds(feet);

        self.update_target();
        self.update_mining(TICK as f32);
        self.update_placing(TICK as f32);

        // Loose items only predict what fits, on a scratch copy of the inventory; the core applies
        // the `PickUp` (and throws back anything that no longer fits).
        let world = &self.sim.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let loaded = |p: Vec3| world.is_loaded(p);
        let center = self.player.pos + Vec3::new(0.0, player::HEIGHT * 0.5, 0.0);
        let mut room = self.sim.player(LOCAL).inventory.clone();
        let mut picked = Vec::new();
        self.items.update(TICK, center, &mut solid, &loaded, &mut room, |item, count| picked.push((item, count)));
        for (item, count) in picked {
            self.act(Action::PickUp { item, count });
        }

        self.sim.step();
        self.handle_sim_events();
    }

    /// Queues an action by the local player for the coming tick.
    fn act(&mut self, action: Action) {
        self.sim.queue(self.sim.tick, LOCAL, action);
    }
}

#[cfg(test)]
mod tests;

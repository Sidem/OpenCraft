//! OpenCraft engine: the whole simulation (world streaming, terrain generation, meshing, physics,
//! interaction, inventory, ore deposits and factory machines) compiled to WebAssembly.
//!
//! The JavaScript host only owns the platform: WebGL2, input and DOM UI. Bulk data (chunk meshes,
//! box instances, textures) never gets copied across the boundary; the host reads it straight out
//! of wasm linear memory through the `*_ptr` / `*_len` accessors below.

mod block;
mod chunk;
mod deposits;
mod entities;
mod factory;
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

use block::{BlockId, AIR, BELT, BLOCK_COUNT, MINER, SPENT_ROCK, STORAGE};
use deposits::{Tier, HAND_YIELD};
use entities::Items;
use factory::{Factory, INSTANCE_FLOATS, MINER_RECOVERY};
use inventory::{Inventory, Stack, HOTBAR_SLOTS, INVENTORY_SLOTS};
use math::{hash2, IVec3, Rng, Vec3};
use physics::Aabb;
use player::Player;
use raycast::{raycast, RayHit};
use recipes::RECIPES;
use sound::Sounds;
use world::{Event, MeshData, World};

const REACH: f64 = 5.0;
const PLACE_REPEAT_SECONDS: f32 = 0.22;
const BREAK_COOLDOWN_SECONDS: f32 = 0.12;
const PHYSICS_STEP: f64 = 1.0 / 120.0;
const DIG_SOUND_INTERVAL: f32 = 0.24;
/// Horizontal distance walked between footstep sounds.
const STEP_STRIDE: f64 = 1.7;
/// Touchdowns slower than this (e.g. walking down a slab edge) make no landing sound.
const LAND_SOUND_MIN_SPEED: f64 = 5.0;
/// Factory step used when fast-forwarding time.
const SKIP_STEP: f64 = 0.05;

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

    // ---------------------------------------------------------------- input

    pub fn set_move(&mut self, forward: f64, strafe: f64, jump: bool, sprint: bool, crouch: bool) {
        self.player.input = player::PlayerInput { forward, strafe, jump, sprint, crouch };
    }

    /// Mouse look, in radians.
    pub fn look(&mut self, d_yaw: f64, d_pitch: f64) {
        self.player.yaw = (self.player.yaw + d_yaw).rem_euclid(std::f64::consts::TAU);
        self.player.pitch = (self.player.pitch - d_pitch).clamp(-1.55, 1.55);
    }

    pub fn set_look(&mut self, yaw: f64, pitch: f64) {
        self.player.yaw = yaw.rem_euclid(std::f64::consts::TAU);
        self.player.pitch = pitch.clamp(-1.55, 1.55);
    }

    pub fn set_mining(&mut self, on: bool) {
        self.mining = on;
    }

    /// Hold to place blocks: places immediately, then repeats while held.
    pub fn set_using(&mut self, on: bool) {
        if on && !self.using {
            self.use_cooldown = 0.0;
        }
        self.using = on;
    }

    pub fn select_slot(&mut self, slot: u32) {
        self.inventory.select(slot as usize);
    }

    pub fn scroll_slot(&mut self, delta: i32) {
        self.inventory.scroll(delta);
    }

    pub fn toggle_fly(&mut self) {
        self.player.flying = !self.player.flying;
        self.player.vel.y = 0.0;
    }

    /// Throws one item from the selected slot.
    pub fn drop_selected(&mut self) {
        if let Some((item, n)) = self.inventory.take_selected(1) {
            self.throw(item, n);
        }
    }

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

    pub fn set_view_radius(&mut self, r: u32) {
        self.world.set_view_radius(r as i32);
    }

    // ---------------------------------------------------------------- simulation

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

    /// Debug helper: runs the factory (miners, belts, boxes) for `seconds` of game time at once.
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

    /// Queues a sound at a world position (stored camera-relative for the host).
    fn play(&mut self, kind: u8, material: u8, at: Vec3, volume: f64) {
        self.sounds.push(kind, material, at - self.player.eye(), volume);
    }

    /// Throws items out in front of the player.
    fn throw(&mut self, item: BlockId, n: u32) {
        let dir = self.player.look_dir();
        let pos = self.player.eye() + dir * 0.4 - Vec3::new(0.0, 0.3, 0.0);
        self.items.spawn(pos, dir * 6.0 + Vec3::new(0.0, 1.5, 0.0), item, n, 1.5);
        self.play(sound::DROP, 0, pos, 1.0);
    }

    /// Sound material of the block the player is standing on (checks the footprint corners so
    /// standing on an edge still finds the supporting block).
    fn ground_material(&self) -> Option<u8> {
        let p = self.player.pos;
        let y = (p.y - 0.05).floor() as i32;
        let hw = player::HALF_WIDTH;
        for (dx, dz) in [(0.0, 0.0), (-hw, -hw), (hw, -hw), (hw, hw), (-hw, hw)] {
            let b = self.world.get_block(IVec3::new((p.x + dx).floor() as i32, y, (p.z + dz).floor() as i32))?;
            if block::SOLID[b as usize] {
                return Some(block::def(b).sound);
            }
        }
        None
    }

    fn update_movement_sounds(&mut self, feet_before: Vec3) {
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

    fn update_target(&mut self) {
        let world = &self.world;
        self.target = raycast(self.player.eye(), self.player.look_dir(), REACH, |p| world.get_block(p).filter(|&b| b != AIR));
    }

    fn update_mining(&mut self, dt: f32) {
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
    fn break_block(&mut self, p: IVec3, id: BlockId) -> bool {
        let ore = block::is_ore(id);
        if ore {
            self.factory.deposits.hand_mined(&mut self.world, p);
        }
        if !self.world.set_block(p, AIR) {
            return false;
        }
        let def = block::def(id);
        let center = p.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        self.play(sound::BREAK, def.sound, center, 1.0);
        let mut drops = self.factory.remove(p);
        if def.drop != AIR {
            drops.insert(0, Stack { item: def.drop, count: if ore { HAND_YIELD } else { 1 } });
        }
        for s in drops {
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.items.spawn(center, vel, s.item, s.count, 0.25);
        }
        true
    }

    fn update_placing(&mut self, dt: f32) {
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
        let stack = self.inventory.selected_stack();
        if stack.is_empty() || !block::is_placeable(stack.item) {
            return false;
        }
        let p = hit.block + hit.normal;
        if self.world.get_block(p) != Some(AIR) {
            return false;
        }
        let cell = Aabb { min: p.as_vec3(), max: p.as_vec3() + Vec3::new(1.0, 1.0, 1.0) };
        if block::SOLID[stack.item as usize] && cell.intersects(&self.player.aabb()) {
            return false;
        }
        if !self.world.set_block(p, stack.item) {
            return false;
        }
        match stack.item {
            BELT => self.factory.add_belt(p, factory::dir_from_yaw(self.player.yaw)),
            MINER => {
                let deposit = self.factory.deposits.lookup(&mut self.world, hit.block);
                let drill = factory::face_of(hit.block - p).unwrap_or(block::FACE_BOTTOM as u8);
                self.factory.add_miner(p, drill, deposit);
            }
            STORAGE => self.factory.add_storage(p),
            _ => {}
        }
        self.inventory.take_selected(1);
        self.play(sound::PLACE, block::def(stack.item).sound, p.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 1.0);
        true
    }

    /// Moves a box's or miner's contents into the inventory. False if `pos` is neither.
    fn take_from_machine(&mut self, pos: IVec3) -> bool {
        let inventory = &mut self.inventory;
        let pickups = &mut self.pickups;
        let mut got = 0;
        let handled = self.factory.take_contents(pos, |item, n| {
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

    // ---------------------------------------------------------------- streaming work

    /// Prepares this frame's work queues. Call once per frame before [`Game::work_step`].
    pub fn begin_work(&mut self) {
        self.world.begin_work();
    }

    /// Generates or meshes one chunk (nearest first). Returns false when there is nothing to do.
    /// The host calls this in a loop until its per-frame time budget is spent.
    pub fn work_step(&mut self) -> bool {
        self.world.work_step()
    }

    pub fn ready(&self) -> bool {
        self.world.area_ready(1)
    }

    /// Pops the next renderer event: 0 = none, 1 = chunk mesh (see `mesh_*`), 2 = chunk unloaded.
    /// The position of either event is available through `event_x/y/z`.
    pub fn next_event(&mut self) -> u32 {
        self.cur_mesh = None;
        match self.world.events.pop_front() {
            None => 0,
            Some(Event::Mesh(m)) => {
                self.cur_event_pos = m.pos;
                self.cur_mesh = Some(m);
                1
            }
            Some(Event::Unload(p)) => {
                self.cur_event_pos = p;
                2
            }
        }
    }

    pub fn event_x(&self) -> i32 {
        self.cur_event_pos.x
    }

    pub fn event_y(&self) -> i32 {
        self.cur_event_pos.y
    }

    pub fn event_z(&self) -> i32 {
        self.cur_event_pos.z
    }

    /// Byte offset of the current mesh's packed vertices in wasm memory.
    pub fn mesh_ptr(&self) -> usize {
        self.cur_mesh.as_ref().map_or(0, |m| m.verts.as_ptr() as usize)
    }

    /// Number of u32 vertices in the current mesh.
    pub fn mesh_len(&self) -> usize {
        self.cur_mesh.as_ref().map_or(0, |m| m.verts.len())
    }

    pub fn mesh_opaque_quads(&self) -> u32 {
        self.cur_mesh.as_ref().map_or(0, |m| m.opaque_quads)
    }

    pub fn mesh_cutout_quads(&self) -> u32 {
        self.cur_mesh.as_ref().map_or(0, |m| m.cutout_quads)
    }

    // ---------------------------------------------------------------- camera & targeting

    pub fn eye_x(&self) -> f64 {
        self.player.eye().x
    }

    pub fn eye_y(&self) -> f64 {
        self.player.eye().y
    }

    pub fn eye_z(&self) -> f64 {
        self.player.eye().z
    }

    pub fn yaw(&self) -> f64 {
        self.player.yaw
    }

    pub fn pitch(&self) -> f64 {
        self.player.pitch
    }

    pub fn flying(&self) -> bool {
        self.player.flying
    }

    pub fn on_ground(&self) -> bool {
        self.player.on_ground
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
    /// Lines are separated by `\n`; empty when there is nothing to add.
    pub fn target_detail(&mut self) -> String {
        let Some(hit) = self.target else { return String::new() };
        if let Some(text) = self.factory.describe(hit.block) {
            return text;
        }
        if !block::is_ore(hit.id) && hit.id != SPENT_ROCK {
            return String::new();
        }
        let Some(key) = self.factory.deposits.lookup(&mut self.world, hit.block) else { return String::new() };
        let Some(st) = self.factory.deposits.get(&key) else { return String::new() };
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
            "{} · {} units per block\n{left}\nBy hand you keep {HAND_YIELD} and lose {}. A miner recovers {}%.",
            st.deposit.name(),
            int(grade as u64),
            int(grade.saturating_sub(HAND_YIELD) as u64),
            (MINER_RECOVERY * 100.0).round() as u32
        )
    }

    // ---------------------------------------------------------------- box instances

    /// Byte offset of this frame's box instances (dropped items, belt items, machine parts):
    /// `instance_count()` records of `factory::INSTANCE_FLOATS` f32 each.
    pub fn instance_ptr(&self) -> usize {
        self.instances.as_ptr() as usize
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len() / INSTANCE_FLOATS
    }

    // ---------------------------------------------------------------- sound

    /// Byte offset of this frame's sound events: `sound_count()` records of
    /// (kind, material, camera-relative x, y, z, volume) as f32.
    pub fn sound_ptr(&self) -> usize {
        self.sounds.as_ptr() as usize
    }

    pub fn sound_count(&self) -> usize {
        self.sounds.count()
    }

    /// Call after the host has played this frame's sounds.
    pub fn clear_sounds(&mut self) {
        self.sounds.clear();
    }

    // ---------------------------------------------------------------- inventory

    pub fn inventory_version(&self) -> u32 {
        self.inventory.version
    }

    pub fn hotbar_size(&self) -> u32 {
        HOTBAR_SLOTS as u32
    }

    /// All slots: the hotbar (0..9) followed by the backpack.
    pub fn inventory_size(&self) -> u32 {
        INVENTORY_SLOTS as u32
    }

    pub fn slot_item(&self, slot: u32) -> u8 {
        self.inventory.slots.get(slot as usize).map_or(AIR, |s| s.item)
    }

    pub fn slot_count(&self, slot: u32) -> u32 {
        self.inventory.slots.get(slot as usize).map_or(0, |s| s.count)
    }

    pub fn selected_slot(&self) -> u32 {
        self.inventory.selected as u32
    }

    /// Inventory screen click. `shift` moves the stack between hotbar and backpack.
    pub fn click_slot(&mut self, slot: u32, shift: bool) {
        if shift {
            self.inventory.quick_move(slot as usize);
        } else {
            self.inventory.click(slot as usize);
        }
    }

    pub fn cursor_item(&self) -> u8 {
        self.inventory.cursor.item
    }

    pub fn cursor_count(&self) -> u32 {
        self.inventory.cursor.count
    }

    /// Closing the inventory screen: the stack on the cursor goes back (or is thrown if full).
    pub fn close_inventory(&mut self) {
        let left = self.inventory.return_cursor();
        if !left.is_empty() {
            self.throw(left.item, left.count);
        }
    }

    pub fn item_total(&self, item: u8) -> u32 {
        self.inventory.count(item)
    }

    /// Pops the next pickup notification; read it with `pickup_item` / `pickup_count`.
    pub fn next_pickup(&mut self) -> bool {
        match self.pickups.pop_front() {
            Some(p) => {
                self.cur_pickup = p;
                true
            }
            None => false,
        }
    }

    pub fn pickup_item(&self) -> u8 {
        self.cur_pickup.0
    }

    pub fn pickup_count(&self) -> u32 {
        self.cur_pickup.1
    }

    // ---------------------------------------------------------------- crafting

    /// Ore kept per block mined by hand.
    pub fn hand_yield(&self) -> u32 {
        HAND_YIELD
    }

    /// Fraction of drilled ore a miner delivers.
    pub fn miner_recovery(&self) -> f64 {
        MINER_RECOVERY
    }

    pub fn recipe_count(&self) -> u32 {
        RECIPES.len() as u32
    }

    pub fn recipe_output(&self, r: u32) -> u8 {
        RECIPES.get(r as usize).map_or(AIR, |x| x.output)
    }

    pub fn recipe_output_count(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| x.count)
    }

    /// Inputs as flat (item, count) pairs.
    pub fn recipe_inputs(&self, r: u32) -> Vec<u32> {
        RECIPES.get(r as usize).map_or_else(Vec::new, |x| x.inputs.iter().flat_map(|&(i, n)| [i as u32, n]).collect())
    }

    pub fn recipe_blurb(&self, r: u32) -> String {
        RECIPES.get(r as usize).map_or_else(String::new, |x| x.blurb.to_string())
    }

    pub fn can_craft(&self, r: u32) -> bool {
        RECIPES.get(r as usize).is_some_and(|x| x.inputs.iter().all(|&(i, n)| self.inventory.count(i) >= n))
    }

    /// Crafts recipe `r` up to `times` times from inventory items. Returns how many times it ran.
    pub fn craft(&mut self, r: u32, times: u32) -> u32 {
        let Some(recipe) = RECIPES.get(r as usize) else { return 0 };
        let mut done = 0;
        while done < times && self.can_craft(r) {
            for &(item, n) in recipe.inputs {
                self.inventory.remove(item, n);
            }
            let left = self.inventory.add(recipe.output, recipe.count);
            if left > 0 {
                self.throw(recipe.output, left);
            }
            done += 1;
        }
        if done > 0 {
            self.pickups.push_back((recipe.output, recipe.count * done));
        }
        done
    }

    // ---------------------------------------------------------------- content

    pub fn block_count(&self) -> u32 {
        BLOCK_COUNT as u32
    }

    pub fn block_name(&self, id: u8) -> String {
        block::def(id).name.to_string()
    }

    /// Sound material a block uses (`block::sound`), i.e. which bank its dig/step/place sounds come from.
    pub fn block_sound(&self, id: u8) -> u8 {
        block::def(id).sound
    }

    /// Texture layer for a block face (0..6 = +X, -X, +Y, -Y, +Z, -Z).
    pub fn block_face_texture(&self, id: u8, face: u32) -> u32 {
        block::def(id).faces[(face as usize).min(5)] as u32
    }

    pub fn texture_ptr(&self) -> usize {
        self.textures.as_ptr() as usize
    }

    pub fn texture_byte_len(&self) -> usize {
        self.textures.len()
    }

    pub fn texture_size(&self) -> u32 {
        textures::TEX_SIZE as u32
    }

    pub fn texture_layers(&self) -> u32 {
        block::tex::COUNT as u32
    }

    // ---------------------------------------------------------------- stats & debugging

    pub fn chunks_loaded(&self) -> u32 {
        self.world.loaded_count() as u32
    }

    pub fn chunks_pending(&self) -> u32 {
        self.world.pending_count() as u32
    }

    pub fn chunks_dirty(&self) -> u32 {
        self.world.dirty_count() as u32
    }

    pub fn item_entities(&self) -> u32 {
        self.items.list.len() as u32
    }

    pub fn belts(&self) -> u32 {
        self.factory.belt_count() as u32
    }

    pub fn miners(&self) -> u32 {
        self.factory.miner_count() as u32
    }

    pub fn boxes(&self) -> u32 {
        self.factory.storage_count() as u32
    }

    pub fn deposits_tracked(&self) -> u32 {
        self.factory.deposits.tracked() as u32
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::DepositKey;
    use crate::factory::MinerStatus;

    fn run_until_ready(g: &mut Game) {
        for _ in 0..10_000 {
            g.update(1.0 / 60.0);
            g.begin_work();
            while g.work_step() {}
            while g.next_event() != 0 {}
            if g.ready() {
                return;
            }
        }
        panic!("world never became ready");
    }

    /// An outcrop ore block near spawn whose deposit has at least `min_blocks` blocks.
    fn find_outcrop_block(g: &mut Game, min_blocks: u32) -> (IVec3, DepositKey) {
        let base = g.player.pos.floor();
        for y in (8..base.y + 4).rev() {
            for z in -40..40 {
                for x in -40..40 {
                    let p = IVec3::new(base.x + x, y, base.z + z);
                    let Some(b) = g.world.get_block(p) else { continue };
                    if !block::is_ore(b) {
                        continue;
                    }
                    if let Some(key) = g.factory.deposits.lookup(&mut g.world, p) {
                        let st = g.factory.deposits.get(&key).unwrap();
                        if key.tier == Tier::Outcrop && st.initial_blocks >= min_blocks {
                            return (p, key);
                        }
                    }
                }
            }
        }
        panic!("no outcrop near spawn");
    }

    #[test]
    fn mine_collect_place_loop() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        for _ in 0..120 {
            g.update(1.0 / 60.0);
        }
        assert!(g.on_ground(), "player should be standing after spawning");
        g.clear_sounds();

        // Look straight down and mine the block underfoot.
        g.set_look(0.0, -1.5);
        g.update(1.0 / 60.0);
        assert!(g.has_target());
        let (tx, ty, tz) = (g.target_x(), g.target_y(), g.target_z());
        let mined = g.target_block();
        g.set_mining(true);
        for _ in 0..200 {
            g.update(1.0 / 60.0);
            if g.world.get_block(IVec3::new(tx, ty, tz)) == Some(AIR) {
                break;
            }
        }
        g.set_mining(false);
        assert_eq!(g.world.get_block(IVec3::new(tx, ty, tz)), Some(AIR), "block should be mined");

        // The drop falls into the hole and gets magnetised into the inventory.
        for _ in 0..300 {
            g.update(1.0 / 60.0);
        }
        let expected = block::def(mined).drop;
        assert_eq!(g.slot_item(0), expected);
        assert_eq!(g.slot_count(0), 1);
        assert!(g.next_pickup());
        assert_eq!(g.pickup_item(), expected);
        let heard = g.sounds.kinds();
        for kind in [sound::DIG, sound::BREAK, sound::LAND, sound::PICKUP] {
            assert!(heard.contains(&kind), "missing sound {kind} in {heard:?}");
        }
        g.clear_sounds();

        // Hover above the hole and put the block back.
        g.toggle_fly();
        g.teleport(tx as f64 + 0.5, ty as f64 + 3.0, tz as f64 + 0.5);
        g.set_look(0.0, -1.55);
        g.update(1.0 / 60.0);
        assert!(g.has_target());
        g.set_using(true);
        g.update(1.0 / 60.0);
        g.set_using(false);
        assert_eq!(g.slot_count(0), 0, "placing consumes the item");
        assert_eq!(g.world.get_block(IVec3::new(tx, ty, tz)), Some(expected));
        assert_eq!(g.sounds.kinds(), vec![sound::PLACE]);
    }

    #[test]
    fn footsteps_and_landing_make_sounds() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        for _ in 0..60 {
            g.update(1.0 / 60.0);
        }

        // Drop from 4 blocks up: one landing thud, positioned below the camera.
        g.clear_sounds();
        let (x, y, z) = (g.player_x(), g.player_y(), g.player_z());
        g.teleport(x, y + 4.0, z);
        for _ in 0..90 {
            g.update(1.0 / 60.0);
        }
        assert_eq!(g.sounds.kinds(), vec![sound::LAND]);
        g.clear_sounds();

        // Walking produces footsteps; flying produces none. Carve a flat corridor towards -Z
        // (the forward direction at yaw 0) so terrain can't block the walk.
        let feet = Vec3::new(g.player_x(), g.player_y(), g.player_z()).floor();
        for dz in 0..12 {
            let p = feet - IVec3::new(0, 0, dz);
            g.world.set_block(p - IVec3::new(0, 1, 0), block::STONE);
            g.world.set_block(p, AIR);
            g.world.set_block(p + IVec3::new(0, 1, 0), AIR);
        }
        g.teleport(feet.x as f64 + 0.5, feet.y as f64, feet.z as f64 + 0.5);
        g.set_look(0.0, 0.0);
        g.update(1.0 / 60.0);
        g.clear_sounds();
        g.set_move(1.0, 0.0, false, false, false);
        for _ in 0..90 {
            g.update(1.0 / 60.0);
        }
        let steps = g.sounds.kinds().iter().filter(|&&k| k == sound::STEP).count();
        assert!(steps >= 2, "expected footsteps, got {:?}", g.sounds.kinds());
        g.clear_sounds();
        g.toggle_fly();
        for _ in 0..90 {
            g.update(1.0 / 60.0);
        }
        assert!(g.sounds.kinds().is_empty());
    }

    #[test]
    fn hand_mining_ore_keeps_a_handful_and_costs_a_block() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        let (p, key) = find_outcrop_block(&mut g, 2);
        let before = g.factory.deposits.get(&key).unwrap().remaining_blocks;
        let ore = g.world.get_block(p).unwrap();
        assert!(g.break_block(p, ore));
        assert_eq!(g.factory.deposits.get(&key).unwrap().remaining_blocks, before - 1);
        let drop = g.items.list.last().unwrap();
        assert_eq!((drop.item, drop.count), (ore, HAND_YIELD));
        assert!(!block::is_placeable(ore), "ore can't be put back");
    }

    /// Miner on top of an outcrop block, a belt leading east, and a box at the end.
    fn build_mine(g: &mut Game, p: IVec3) -> (IVec3, IVec3) {
        let m = p + IVec3::new(0, 1, 0);
        let belt = m + IVec3::new(1, 0, 0);
        let chest = m + IVec3::new(2, 0, 0);
        g.world.set_block(m, MINER);
        g.world.set_block(belt, BELT);
        g.world.set_block(chest, STORAGE);
        let key = g.factory.deposits.lookup(&mut g.world, p);
        assert!(key.is_some());
        g.factory.add_miner(m, block::FACE_BOTTOM as u8, key);
        g.factory.add_belt(belt, 1);
        g.factory.add_storage(chest);
        (m, chest)
    }

    #[test]
    fn miner_drills_the_pool_and_turns_blocks_into_spent_rock() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        let (p, key) = find_outcrop_block(&mut g, 6);
        let ore = key.ore;
        let (m, chest) = build_mine(&mut g, p);
        let st = g.factory.deposits.get(&key).unwrap();
        let (blocks, grade) = (st.remaining_blocks, st.grade() as f64);

        // An outcrop allows 1 unit/s; after 250 s, two blocks' worth (100 units each) are gone.
        g.skip_time(250.0);
        let st = g.factory.deposits.get(&key).unwrap();
        assert_eq!(st.remaining_blocks, blocks - 2);
        assert_eq!(g.world.get_block(p), Some(SPENT_ROCK), "the block under the drill goes first");
        let recovered = (250.0 * MINER_RECOVERY) as u32;
        let stored = g.factory.storage_count_at(chest, ore);
        assert!(stored + 4 >= recovered && stored <= recovered, "stored {stored} of ~{recovered}");
        assert_eq!(g.factory.miner_at(m).status, MinerStatus::Running);
        assert!((st.remaining_units() - (blocks as f64 * grade - 250.0)).abs() < 1.0);

        // The readout names the deposit.
        let detail = g.factory.describe(m).unwrap();
        assert!(detail.contains("outcrop"), "{detail}");
    }

    #[test]
    fn miner_without_output_fills_up_and_stops() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        let (p, key) = find_outcrop_block(&mut g, 2);
        let m = p + IVec3::new(0, 1, 0);
        g.world.set_block(m, MINER);
        let k = g.factory.deposits.lookup(&mut g.world, p);
        g.factory.add_miner(m, block::FACE_BOTTOM as u8, k);
        let units = g.factory.deposits.get(&key).unwrap().remaining_units();
        g.skip_time(300.0);
        let miner = g.factory.miner_at(m);
        assert_eq!(miner.status, MinerStatus::OutputFull);
        assert_eq!(miner.held, factory::MINER_BUFFER);
        let used = units - g.factory.deposits.get(&key).unwrap().remaining_units();
        assert!(used < 110.0, "a full miner stops drawing, used {used}");

        // Right-click empties it into the inventory.
        assert!(g.take_from_machine(m));
        assert_eq!(g.item_total(key.ore), factory::MINER_BUFFER);
    }

    #[test]
    fn crafting_consumes_inputs() {
        let mut g = Game::new(7, 2);
        g.give(block::IRON_ORE, 3);
        g.give(block::STONE, 5);
        let belt = RECIPES.iter().position(|r| r.output == BELT).unwrap() as u32;
        assert_eq!(g.craft(belt, 5), 2);
        assert_eq!(g.item_total(BELT), 8);
        assert_eq!(g.item_total(block::IRON_ORE), 1);
        assert_eq!(g.item_total(block::STONE), 1);
        assert!(!g.can_craft(belt));
    }
}

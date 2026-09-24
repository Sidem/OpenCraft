//! OpenCraft engine: the whole simulation (world streaming, terrain generation, meshing, physics,
//! interaction, inventory) compiled to WebAssembly.
//!
//! The JavaScript host only owns the platform: WebGL2, input and DOM UI. Bulk data (chunk meshes,
//! item instances, textures) never gets copied across the boundary; the host reads it straight out
//! of wasm linear memory through the `*_ptr` / `*_len` accessors below.

mod block;
mod chunk;
mod entities;
mod inventory;
mod math;
mod mesher;
mod noise;
mod physics;
mod player;
mod raycast;
mod textures;
mod world;
mod worldgen;

use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

use block::{BlockId, AIR, BLOCK_COUNT};
use entities::{Items, INSTANCE_FLOATS};
use inventory::{Inventory, HOTBAR_SLOTS};
use math::{hash2, IVec3, Rng, Vec3};
use physics::Aabb;
use player::Player;
use raycast::{raycast, RayHit};
use world::{Event, MeshData, World};

const REACH: f64 = 5.0;
const PLACE_REPEAT_SECONDS: f32 = 0.22;
const BREAK_COOLDOWN_SECONDS: f32 = 0.12;
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
    rng: Rng,
    spawn: Vec3,
    target: Option<RayHit>,
    mining: bool,
    mine_block: Option<IVec3>,
    mine_progress: f32,
    mine_cooldown: f32,
    using: bool,
    use_cooldown: f32,
    textures: Vec<u8>,
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
            rng: Rng::new(hash2(seed, 17, 42) as u64),
            spawn,
            target: None,
            mining: false,
            mine_block: None,
            mine_progress: 0.0,
            mine_cooldown: 0.0,
            using: false,
            use_cooldown: 0.0,
            textures: textures::generate(),
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
            let dir = self.player.look_dir();
            let pos = self.player.eye() + dir * 0.4 - Vec3::new(0.0, 0.3, 0.0);
            self.items.spawn(pos, dir * 6.0 + Vec3::new(0.0, 1.5, 0.0), item, n, 1.5);
        }
    }

    /// Debug / creative helper: adds items straight to the inventory.
    pub fn give(&mut self, item: u8, count: u32) -> u32 {
        if (item as usize) < BLOCK_COUNT {
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

        self.update_target();
        self.update_mining(dt as f32);
        self.update_placing(dt as f32);

        let world = &self.world;
        let mut solid = |x, y, z| world.is_solid(x, y, z);
        let loaded = |p: Vec3| world.is_loaded(p);
        let pickups = &mut self.pickups;
        let center = self.player.pos + Vec3::new(0.0, player::HEIGHT * 0.5, 0.0);
        self.items.update(dt, center, &mut solid, &loaded, &mut self.inventory, |item, n| {
            match pickups.back_mut() {
                Some((last, count)) if *last == item => *count += n,
                _ => pickups.push_back((item, n)),
            }
        });
        self.items.write_instances(self.player.eye());
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
            return;
        };
        if self.mine_cooldown > 0.0 {
            return;
        }
        if self.mine_block != Some(hit.block) {
            self.mine_block = Some(hit.block);
            self.mine_progress = 0.0;
        }
        let def = block::def(hit.id);
        if def.break_time < 0.0 {
            return;
        }
        self.mine_progress += if def.break_time == 0.0 { 1.0 } else { dt / def.break_time };
        if self.mine_progress < 1.0 {
            return;
        }
        if self.world.set_block(hit.block, AIR) && def.drop != AIR {
            let c = hit.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            let vel = Vec3::new(self.rng.range(-1.5, 1.5), 4.0, self.rng.range(-1.5, 1.5));
            self.items.spawn(c, vel, def.drop, 1, 0.25);
        }
        self.mine_block = None;
        self.mine_progress = 0.0;
        self.mine_cooldown = BREAK_COOLDOWN_SECONDS;
        self.update_target();
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

    fn try_place(&mut self) -> bool {
        let Some(hit) = self.target else { return false };
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
        self.inventory.take_selected(1);
        true
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

    // ---------------------------------------------------------------- items & inventory

    pub fn item_instances_ptr(&self) -> usize {
        self.items.instances.as_ptr() as usize
    }

    pub fn item_instance_count(&self) -> usize {
        self.items.instances.len() / INSTANCE_FLOATS
    }

    pub fn inventory_version(&self) -> u32 {
        self.inventory.version
    }

    pub fn hotbar_size(&self) -> u32 {
        HOTBAR_SLOTS as u32
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

    // ---------------------------------------------------------------- content

    pub fn block_count(&self) -> u32 {
        BLOCK_COUNT as u32
    }

    pub fn block_name(&self, id: u8) -> String {
        block::def(id).name.to_string()
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

    // ---------------------------------------------------------------- stats

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

    #[test]
    fn mine_collect_place_loop() {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        for _ in 0..120 {
            g.update(1.0 / 60.0);
        }
        assert!(g.on_ground(), "player should be standing after spawning");

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
    }
}

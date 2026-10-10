//! Debug and testing helpers, used from the browser console (`window.opencraft.game`) and tests:
//! give items, teleport, add and remove players, the state hash, break a co-op core on purpose, run
//! ticks and fast-forward time,
//! find deposits, read blocks and the position.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block::{AIR, STONE};
use crate::deposits::Tier;
use crate::inventory::{add_to_slots, Stack};
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::sim::PlayerId;
use crate::{Game, TICK_RATE};

#[wasm_bindgen]
impl Game {
    /// Debug / creative helper: adds items to the inventory at the next tick (what doesn't fit is lost).
    pub fn give(&mut self, item: u16, count: u32) {
        self.act(Action::Give { item: ItemId(item), count });
    }

    /// Scene building for tests and recordings: gives one `block` and places it at once at `(x, y, z)`
    /// facing `facing`, bypassing reach and co-op routing (like `debug_desync`, solo worlds only).
    pub fn place_at(&mut self, x: i32, y: i32, z: i32, block: u16, facing: u8) {
        let me = self.local;
        let item = ItemId(block);
        self.sim.apply(me, Action::Give { item, count: 1 });
        let Some(slot) = self.sim.player(me).and_then(|p| p.inventory.slots.iter().position(|s| s.item == item)) else {
            return;
        };
        let pos = IVec3::new(x, y, z);
        self.sim.apply(me, Action::PlaceBlock { pos, slot: slot as u8, facing, against: pos - IVec3::new(0, 1, 0) });
    }

    /// Scene building: wires a pole to a machine or pole, or lays track between two rail nodes, at once.
    pub fn connect_at(&mut self, ax: i32, ay: i32, az: i32, bx: i32, by: i32, bz: i32) {
        let (pole, to) = (IVec3::new(ax, ay, az), IVec3::new(bx, by, bz));
        self.sim.apply(self.local, Action::Connect { pole, to });
    }

    /// Scene building: gives one `item` (a locomotive or wagon) and puts it on the rail node at once.
    pub fn train_at(&mut self, x: i32, y: i32, z: i32, item: u16) {
        let me = self.local;
        let item = ItemId(item);
        self.sim.apply(me, Action::Give { item, count: 1 });
        let Some(slot) = self.sim.player(me).and_then(|p| p.inventory.slots.iter().position(|s| s.item == item)) else {
            return;
        };
        self.sim.apply(me, Action::PlaceTrain { pos: IVec3::new(x, y, z), slot: slot as u8 });
    }

    /// Scene building: puts `count` of `item` straight into the box or machine at `(x, y, z)`, bypassing
    /// the inventory; returns how many went in (the rest is dropped).
    pub fn stock_at(&mut self, x: i32, y: i32, z: i32, item: u16, count: u32) -> u32 {
        let (pos, item) = (IVec3::new(x, y, z), ItemId(item));
        match self.sim.factory.box_slots_mut(pos) {
            Some(slots) => count - add_to_slots(slots, item, count),
            None => self.sim.factory.insert(pos, item, count),
        }
    }

    /// Scene building: empties the storage box at `(x, y, z)`, so a backed-up line flows again; returns how many
    /// items it held.
    pub fn empty_box_at(&mut self, x: i32, y: i32, z: i32) -> u32 {
        let Some(slots) = self.sim.factory.box_slots_mut(IVec3::new(x, y, z)) else {
            return 0;
        };
        let held = slots.iter().map(|s| s.count).sum();
        slots.fill(Stack::default());
        held
    }

    /// Scene building: plants a ghost of `block` (for drones to build) at once.
    pub fn ghost_at(&mut self, x: i32, y: i32, z: i32, block: u8, facing: u8) {
        let pos = IVec3::new(x, y, z);
        self.sim.apply(self.local, Action::PlantGhost { pos, block, facing, tier: 0 });
    }

    /// Scene building: sets every cell of the box between the two corners to terrain `block` in this core
    /// only, bypassing actions (no drops; never use it on machines). Clears or levels ground for a set;
    /// remeshing is left to streaming, so big fills stay fast.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_blocks(&mut self, x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32, block: u8) {
        for x in x0.min(x1)..=x0.max(x1) {
            for y in y0.min(y1)..=y0.max(y1) {
                for z in z0.min(z1)..=z0.max(z1) {
                    self.sim.world.set_block_anywhere_later(IVec3::new(x, y, z), block);
                }
            }
        }
    }

    pub fn teleport(&mut self, x: f64, y: f64, z: f64) {
        let body = self.body_mut();
        body.pos = Vec3::new(x, y, z);
        body.vel = Vec3::ZERO;
        // Cut the camera to the new spot instead of gliding there.
        self.prev_eye = self.body().eye();
        self.render_eye = self.prev_eye;
    }

    /// Adds a player without a key (a body at spawn; its inventory from the next tick) and returns its
    /// id, or nothing if the world is full. For tests; co-op joins go through `host_join`.
    pub fn add_player(&mut self) -> Option<u32> {
        self.join(0).map(|id| id.0 as u32)
    }

    /// Removes a player and its inventory (never the local player).
    pub fn remove_player(&mut self, id: u32) {
        if let Ok(id) = u8::try_from(id) {
            self.leave(PlayerId(id));
        }
    }

    /// Fingerprint of the core state (world edits, machines, deposits, inventories, tick), e.g. to
    /// check that a reloaded world matches the one that was saved.
    pub fn state_hash(&self) -> u64 {
        self.sim.state_hash()
    }

    /// Co-op testing: flips the block under the player in this core only, bypassing actions, so this
    /// game's state hash parts from everyone else's, which makes a client resync.
    pub fn debug_desync(&mut self) {
        let p = self.body().pos.floor() - IVec3::new(0, 1, 0);
        let b = self.sim.world.block_anywhere_or_generate(p);
        self.sim.world.set_block_anywhere(p, if b == AIR { STONE } else { AIR });
    }

    /// Runs `n` simulation ticks at once, sounds included (tests and catch-up).
    pub fn run_ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.run_tick();
        }
    }

    /// Fast-forwards `seconds` of game time (rounded to whole ticks) and discards the sounds it makes.
    pub fn skip_time(&mut self, seconds: f64) {
        let pending = std::mem::take(&mut self.sounds);
        self.run_ticks((seconds.max(0.0) * TICK_RATE as f64).round() as u32);
        self.sounds = pending;
    }

    /// Prospecting aid for testing: centre and ore of the nearest deposit of `tier`
    /// (0 = lode, 1 = vein, 2 = outcrop) as `[x, y, z, ore]`, or empty if none is within ~500 blocks.
    pub fn find_deposit(&self, tier: u8) -> Vec<i32> {
        let Some(tier) = Tier::from_u8(tier) else { return Vec::new() };
        self.sim
            .world
            .generator()
            .find_deposit(self.body().pos.floor(), tier, 16)
            .map_or_else(Vec::new, |d| vec![d.center.x, d.center.y, d.center.z, d.ore() as i32])
    }

    /// Block at a position in a loaded chunk (air otherwise). For testing and the console.
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> u8 {
        self.sim.world.get_block(IVec3::new(x, y, z)).unwrap_or(AIR)
    }

    pub fn player_x(&self) -> f64 {
        self.body().pos.x
    }

    pub fn player_y(&self) -> f64 {
        self.body().pos.y
    }

    pub fn player_z(&self) -> f64 {
        self.body().pos.z
    }
}

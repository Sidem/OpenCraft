//! Debug and testing helpers, used from the browser console (`window.opencraft.game`) and tests:
//! give items, teleport, add and remove players, the state hash, break a co-op core on purpose, run
//! ticks and fast-forward time,
//! find deposits, read blocks and the position.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::block::{BlockId, AIR, STONE};
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
        let item = ItemId::block(block);
        self.sim.apply(me, Action::Give { item, count: 1 });
        let Some(slot) = self.sim.player(me).and_then(|p| p.inventory.slots.iter().position(|s| s.item == item)) else {
            return;
        };
        let pos = IVec3::new(x, y, z);
        self.sim.apply(me, Action::PlaceBlock { pos, slot: slot as u8, facing, against: pos - IVec3::new(0, 1, 0) });
    }

    /// Scene building: every machine block of more than one cell that turns (`scripts/port-shots.mjs`).
    pub fn footprint_blocks(&self) -> Vec<BlockId> {
        crate::port_view::footprint_blocks()
    }

    /// The ports of `block` placed `turn` quarter turns from the way the player faces, as the player sees them: pairs
    /// of (side: 0 far, 1 right, 2 near, 3 left; flow: 0 in, 1 out, 2 byproduct), one per face (`port_view.rs`).
    pub fn placement_ports(&self, block: BlockId, turn: u8) -> Vec<u8> {
        self.ports_as_seen(block, turn)
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

    /// Scene building: puts a survey satellite in orbit above the column `(x, z)` at once, as a launch would.
    pub fn launch_satellite_at(&mut self, x: i32, z: i32) {
        let pad = IVec3::new(x, 0, z);
        self.sim.factory.orbit.launch(crate::item::SURVEY_SATELLITE, pad, self.sim.tick);
    }

    /// Scene building: like `launch_satellite_at` for the satellite kind `kind` (an index of `satellites::KINDS`).
    pub fn launch_kind_at(&mut self, x: i32, z: i32, kind: u8) {
        if let Some(k) = crate::satellites::KINDS.get(kind as usize) {
            self.sim.factory.orbit.launch(k.item, IVec3::new(x, 0, z), self.sim.tick);
        }
    }

    /// Scene building: plants a ghost of `block` (for drones to build) at once.
    pub fn ghost_at(&mut self, x: i32, y: i32, z: i32, block: BlockId, facing: u8) {
        let pos = IVec3::new(x, y, z);
        self.sim.apply(self.local, Action::PlantGhost { pos, block, facing, tier: 0 });
    }

    /// Scene building: sets every cell of the box between the two corners to terrain `block` in this core
    /// only, bypassing actions (no drops; never use it on machines). Clears or levels ground for a set;
    /// remeshing is left to streaming, so big fills stay fast.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_blocks(&mut self, x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32, block: BlockId) {
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

    /// The items whose name is `query` (ignoring case), else those whose name contains it; at most 8 ids. For the console's
    /// `give`.
    pub fn find_item(&self, query: &str) -> Vec<u16> {
        let q = query.trim().to_ascii_lowercase();
        let names = |exact: bool| {
            (0..crate::item::ITEM_COUNT as u16)
                .filter(|&i| {
                    let name = crate::item::name(ItemId(i)).to_ascii_lowercase();
                    !name.is_empty() && !q.is_empty() && if exact { name == q } else { name.contains(&q) }
                })
                .take(8)
                .collect::<Vec<u16>>()
        };
        let exact = names(true);
        if exact.is_empty() {
            names(false)
        } else {
            exact
        }
    }

    /// Block at a position in a loaded chunk (air otherwise). For testing and the console.
    pub fn block_at(&self, x: i32, y: i32, z: i32) -> BlockId {
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

    /// Builds a live playable demo of the Telescoping Bucket-Wheel Site Grader right near the player:
    /// a forested hillside with an exposed iron/coal vein, a water-filled hollow, a sandy ridge, powered
    /// generators, a Concrete paving input belt, and a rear spoil/ore/timber discharge belt.
    pub fn setup_grader_demo(&mut self) -> String {
        use crate::block::{
            BELT, BIRCH_LEAVES, BIRCH_LOG, COAL_ORE, CONCRETE, DIRT, GENERATOR, GRASS, IRON_ORE, LEAVES, LOG, PLANKS,
            POLE, SAND, SANDSTONE, SITE_GRADER, STONE, STORAGE, WATER,
        };
        let cx = self.body().pos.x.round() as i32;
        let cz = self.body().pos.z.round() as i32;
        // Find a comfortable ground elevation near `(cx, cz)`.
        let mut grade_y = 84;
        for y in (64..150).rev() {
            let b = self.sim.world.block_anywhere_or_generate(IVec3::new(cx, y, cz));
            if b != AIR && !crate::block::LIQUID[b as usize] && !crate::block::is_leaves(b) && !crate::block::is_log(b)
            {
                grade_y = y.clamp(70, 130);
                break;
            }
        }

        // 0. Advance to crisp clear late-morning sunlight BEFORE placing the grader so the hillside starts untouched.
        let clear_in = self.weather_ahead(0);
        if clear_in > 0.0 {
            self.skip_time(clear_in);
        }
        let tod = self.time_of_day();
        if !(0.34..=0.66).contains(&tod) {
            let ahead = (0.43 - tod + 1.0) % 1.0;
            self.skip_time(ahead * self.day_seconds());
        }

        // 1. Clear airspace and lay solid sub-grade across the staging pad & site (`x: cx-13..=cx+13, z: cz-20..=cz+13`).
        self.fill_blocks(cx - 13, grade_y - 6, cz - 20, cx + 13, grade_y - 1, cz + 13, STONE);
        self.fill_blocks(cx - 13, grade_y, cz - 20, cx + 13, grade_y, cz + 13, GRASS);
        self.fill_blocks(cx - 13, grade_y + 1, cz - 20, cx + 13, grade_y + 22, cz + 13, AIR);

        // Concrete staging apron under the grader and power/belt yard (`z: cz+2..=cz+11`).
        self.fill_blocks(cx - 6, grade_y, cz + 2, cx + 8, grade_y, cz + 10, CONCRETE);

        // 2. Sculpt the 19×18 grading site (`x: cx-9..=cx+9`, `z: cz-16..=cz+1`):
        //    (a) Layered forested dirt/stone hillside on the near-right & centre (`x: cx-2..=cx+9, z: cz-10..=cz+0`).
        for z in (cz - 10)..=cz {
            for x in (cx - 2)..=(cx + 9) {
                let dx = (x - (cx + 4)) as f64;
                let dz = (z - (cz - 4)) as f64;
                let h = (5.4 - (dx * dx * 0.10 + dz * dz * 0.14)).floor() as i32;
                if h > 0 {
                    for dy in 1..=h {
                        let b = if dy == h {
                            GRASS
                        } else if dy >= h - 1 {
                            DIRT
                        } else {
                            STONE
                        };
                        self.sim.world.set_block_anywhere_later(IVec3::new(x, grade_y + dy, z), b);
                    }
                }
            }
        }
        //    Exposed Iron Ore and Coal Ore vein tucked inside the hillside so you see ores sorted straight to output!
        for &(ox, oy, oz, ore) in &[
            (cx + 2, grade_y + 1, cz - 2, IRON_ORE),
            (cx + 3, grade_y + 1, cz - 2, IRON_ORE),
            (cx + 3, grade_y + 2, cz - 3, IRON_ORE),
            (cx + 4, grade_y + 1, cz - 3, IRON_ORE),
            (cx + 1, grade_y + 1, cz - 4, COAL_ORE),
            (cx + 2, grade_y + 1, cz - 4, COAL_ORE),
        ] {
            self.sim.world.set_block_anywhere_later(IVec3::new(ox, oy, oz), ore);
        }

        //    (b) Water-filled hollow / pond on the near-left (`x: cx-8..=cx-3, z: cz-7..=cz+0`) to showcase auto-fill!
        for z in (cz - 7)..=cz {
            for x in (cx - 8)..=(cx - 3) {
                let dx = (x - (cx - 5)) as f64;
                let dz = (z - (cz - 3)) as f64;
                let depth = (3.4 - (dx * dx * 0.25 + dz * dz * 0.20)).floor() as i32;
                if depth > 0 {
                    for dy in 0..depth {
                        self.sim.world.set_block_anywhere_later(IVec3::new(x, grade_y - dy, z), WATER);
                    }
                }
            }
        }

        //    (c) Far sand & sandstone ridge (`z: cz-16..=cz-10`) so the boom telescopes out to full reach!
        for z in (cz - 16)..=(cz - 10) {
            for x in (cx - 8)..=(cx + 8) {
                let ridge =
                    (4.2 - ((z - (cz - 13)).abs() as f64 * 0.85) - ((x - cx).abs() as f64 * 0.22)).floor() as i32;
                if ridge > 0 {
                    for dy in 1..=ridge {
                        let b = if dy == ridge { SAND } else { SANDSTONE };
                        self.sim.world.set_block_anywhere_later(IVec3::new(x, grade_y + dy, z), b);
                    }
                }
            }
        }

        //    (d) Three trees standing right on the hillside & edge (2 oaks + 1 birch) for live whole-tree felling!
        let mut plant_tree = |tx: i32, tz: i32, log: BlockId, leaf: BlockId, h: i32| {
            let mut ty = grade_y + 1;
            while self.sim.world.block_anywhere_or_generate(IVec3::new(tx, ty, tz)) != AIR && ty < grade_y + 10 {
                ty += 1;
            }
            for dy in 0..h {
                self.sim.world.set_block_anywhere_later(IVec3::new(tx, ty + dy, tz), log);
            }
            for dy in (h - 2)..=(h + 1) {
                let r: i32 = if dy == h + 1 { 1 } else { 2 };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() + dz.abs() > r + 1 {
                            continue;
                        }
                        let p = IVec3::new(tx + dx, ty + dy, tz + dz);
                        if self.sim.world.block_anywhere_or_generate(p) == AIR {
                            self.sim.world.set_block_anywhere_later(p, leaf);
                        }
                    }
                }
            }
        };
        plant_tree(cx + 5, cz - 3, LOG, LEAVES, 5);
        plant_tree(cx + 1, cz - 6, BIRCH_LOG, BIRCH_LEAVES, 6);
        plant_tree(cx - 5, cz - 11, LOG, LEAVES, 5);

        // 3. Build the elevated viewing platform and move the player there BEFORE placing machines
        //    (so the player's body never blocks placing the 3×3×2 Site Grader!).
        let deck_y = grade_y + 4;
        self.fill_blocks(cx - 10, deck_y, cz + 6, cx - 8, deck_y, cz + 8, PLANKS);
        for &(px, pz) in &[(cx - 10, cz + 6), (cx - 8, cz + 6), (cx - 10, cz + 8), (cx - 8, cz + 8)] {
            self.fill_blocks(px, grade_y + 1, pz, px, deck_y - 1, pz, STONE);
        }
        let view_pos = Vec3::new((cx - 8) as f64 + 0.1, (deck_y + 1) as f64, (cz + 6) as f64 + 0.6);
        let focus = Vec3::new(cx as f64 + 0.5, (grade_y + 2) as f64, (cz - 1) as f64);
        let eye = view_pos + Vec3::new(0.0, 1.62, 0.0);
        let d = focus - eye;
        {
            let body = self.body_mut();
            body.pos = view_pos;
            body.vel = Vec3::ZERO;
            body.yaw = d.x.atan2(-d.z);
            body.pitch = d.y.atan2(d.x.hypot(d.z));
        }
        self.prev_eye = self.body().eye();
        self.render_eye = self.prev_eye;

        // 4. Place the 3×3×2 Telescoping Bucket-Wheel Site Grader facing North (`dir = 0`, back = -Z).
        //    Anchor at `(cx - 1, grade_y + 1, cz + 4)` -> footprint `x: cx-1..=cx+1, z: cz+2..=cz+4`.
        //    Left paving intake port (`Side::Left, Nth(1)`): `(cx - 1, grade_y + 1, cz + 3)` facing West.
        //    Rear spoil output port (`Side::Front, Nth(1)`): `(cx, grade_y + 1, cz + 4)` facing South.
        //    Grading site (`19×18` ahead along -Z): `x: cx-9..=cx+9, z: cz-16..=cz+1`.
        let grader = IVec3::new(cx - 1, grade_y + 1, cz + 4);
        self.place_at(grader.x, grader.y, grader.z, SITE_GRADER, 0);
        // Configure the demo grader: spare ore deposits so the iron/coal vein stays intact, and pave Concrete!
        self.sim.factory.set_grader(
            grader,
            true,
            CONCRETE,
            crate::factory::GRADER_DEFAULT_WIDTH,
            crate::factory::GRADER_DEFAULT_DEPTH,
            0,
        );

        // 5. Power plant: 2 Coal Generators (80 kW total > 45 kW draw) wired via a Power Pole.
        let gen1 = IVec3::new(cx + 5, grade_y + 1, cz + 4);
        let gen2 = IVec3::new(cx + 7, grade_y + 1, cz + 4);
        let pole = IVec3::new(cx + 3, grade_y + 1, cz + 3);
        self.place_at(gen1.x, gen1.y, gen1.z, GENERATOR, 0);
        self.place_at(gen2.x, gen2.y, gen2.z, GENERATOR, 0);
        self.place_at(pole.x, pole.y, pole.z, POLE, 0);
        self.stock_at(gen1.x, gen1.y, gen1.z, COAL_ORE, 64);
        self.stock_at(gen2.x, gen2.y, gen2.z, COAL_ORE, 64);
        self.connect_at(pole.x, pole.y, pole.z, gen1.x, gen1.y, gen1.z);
        self.connect_at(pole.x, pole.y, pole.z, gen2.x, gen2.y, gen2.z);
        self.connect_at(pole.x, pole.y, pole.z, grader.x, grader.y, grader.z);

        // 6. Left paving intake line: Storage Box full of Concrete -> 3 belts heading East (`dir = 1`) into Left port.
        let in_box = IVec3::new(cx - 5, grade_y + 1, cz + 3);
        self.place_at(in_box.x, in_box.y, in_box.z, STORAGE, 1);
        self.stock_at(in_box.x, in_box.y, in_box.z, CONCRETE, 512);
        for dx in -4..=-2 {
            self.place_at(cx + dx, grade_y + 1, cz + 3, BELT, 1);
        }

        // 7. Rear spoil/ore/timber discharge line: 4 belts heading South (`dir = 2`) from Front port into Storage Box.
        for dz in 5..=8 {
            self.place_at(cx, grade_y + 1, cz + dz, BELT, 2);
        }
        let out_box = IVec3::new(cx, grade_y + 1, cz + 9);
        self.place_at(out_box.x, out_box.y, out_box.z, STORAGE, 2);

        // 8. Give the player a Site Grader, Concrete, Belts, and Power Poles to experiment with right away.
        self.sim.apply(self.local, Action::Give { item: crate::item::SITE_GRADER_ITEM, count: 2 });
        self.sim.apply(self.local, Action::Give { item: ItemId::block(CONCRETE), count: 128 });
        self.sim.apply(self.local, Action::Give { item: ItemId::block(BELT), count: 64 });
        self.sim.apply(self.local, Action::Give { item: ItemId::block(POLE), count: 16 });

        // 9. Run a short 25-tick warm-up (~1.25 s) so power links, belts carry concrete in, and the wheel engages!
        self.run_ticks(25);

        // Keep player grounded cleanly on the viewing deck aimed right across the Site Grader & hillside.
        {
            let body = self.body_mut();
            body.pos = view_pos;
            body.vel = Vec3::ZERO;
            body.yaw = d.x.atan2(-d.z);
            body.pitch = d.y.atan2(d.x.hypot(d.z));
        }
        self.prev_eye = self.body().eye();
        self.render_eye = self.prev_eye;

        format!(
            "Site Grader demo ready at ({}, {}, {}): watching it cut the forested hill, fill the pond, and pave Concrete!",
            grader.x, grader.y, grader.z
        )
    }
}

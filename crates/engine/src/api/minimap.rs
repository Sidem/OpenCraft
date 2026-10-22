//! The maps (minimap.rs): the minimap's redraw on request and image (zero-copy), the world map's
//! images at any scale (key M), other players' marks, the ore, deposit and machine marks with the
//! prospected deposits and explored map the host keeps (minimap/marks.rs, minimap/atlas.rs), where towns fit while a
//! depot is held (minimap/sites.rs), and the
//! ore guide shown beside the world map (ore_guide.rs).

use wasm_bindgen::prelude::*;

use crate::block::TOWN_DEPOT;
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::minimap::{Minimap, MAP_SIZE, MARK_FIELDS, WORLD_MAP_MAX};
use crate::ore_guide;
use crate::settlements::{SPACING, SPAWN_CLEAR};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Redraws the map around the local player if they moved to another column or chunks changed
    /// nearby. Returns whether the image changed. The host calls it a few times a second at most.
    pub fn minimap_redraw(&mut self) -> bool {
        let p = self.body().pos.floor();
        let changed = self.minimap.redraw(&self.sim.world, (p.x, p.z));
        self.refresh_survey();
        changed
    }

    /// Byte offset of the map's RGBA pixels (`minimap_size()`² of them, north up) in wasm memory.
    pub fn minimap_ptr(&self) -> usize {
        self.minimap.pixels.as_ptr() as usize
    }

    pub fn minimap_size(&self) -> usize {
        MAP_SIZE
    }

    /// Other players on the map: (x offset, z offset in blocks, yaw) per player.
    pub fn minimap_players(&self) -> Vec<f32> {
        Minimap::player_marks(&self.bodies, self.local, self.body().pos)
    }

    /// Prospected deposits and machines inside the image, `minimap_mark_fields()` numbers each: x and z
    /// offset in blocks from the image's centre column, colour (0xRRGGBB), shape (0 deposit, 1 machine; the shapes are
    /// `minimap/marks.rs`'s). With a long-distance pole in hand, every one's reach is a ring.
    pub fn minimap_marks(&self) -> Vec<i32> {
        self.minimap.marks(&self.sim.factory, &self.sim.settlements.towns, self.holds_long_pole())
    }

    pub fn minimap_mark_fields(&self) -> u32 {
        MARK_FIELDS as u32
    }

    /// The prospected deposits still holding ore, for the host to keep with the world's record.
    pub fn known_deposits(&self) -> Vec<i32> {
        self.minimap.known.export(&self.sim.factory)
    }

    /// Restores `known_deposits` from an earlier session of this world.
    pub fn set_known_deposits(&mut self, data: &[i32]) {
        self.minimap.known.import(data, self.sim.world.generator());
    }

    /// Draws the world map: `w`×`h` pixels (each side at most 1024), pixel (i, j) showing block column
    /// (x0 + i·scale, z0 + j·scale), north up; unseen columns are transparent. Read it with
    /// `world_map_ptr` (`w`·`h`·4 bytes of RGBA).
    pub fn world_map_draw(&mut self, x0: i32, z0: i32, scale: i32, w: usize, h: usize) {
        self.refresh_survey();
        let (w, h, scale) = (w.min(WORLD_MAP_MAX), h.min(WORLD_MAP_MAX), scale.clamp(1, 64));
        let mut pixels = std::mem::take(&mut self.minimap.world_pixels);
        self.minimap.draw(x0, z0, scale, w, h, &mut pixels);
        self.minimap.world_pixels = pixels;
    }

    pub fn world_map_ptr(&self) -> usize {
        self.minimap.world_pixels.as_ptr() as usize
    }

    /// Changes whenever the explored map does, so the host redraws only then (or when the view moves).
    pub fn world_map_version(&self) -> u32 {
        self.minimap.atlas.changes
    }

    /// Marks on block columns x0..=x1, z0..=z1 (`minimap_mark_fields()` numbers each, as
    /// `minimap_marks`, but x and z are world columns): shape 2 is ore seen at the surface.
    pub fn world_map_marks(&self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<i32> {
        self.minimap.marks_in(
            &self.sim.factory,
            &self.sim.settlements.towns,
            (x0, z0),
            (x1, z1),
            (0, 0),
            self.holds_long_pole(),
        )
    }

    /// What the marks within `reach` blocks of column (x, z) are, nearest first, for the hover box: one line each,
    /// `shape<TAB>colour<TAB>title<TAB>detail` (shape and colour as in `world_map_marks`). Only what the player knows.
    pub fn world_map_describe(&self, x: i32, z: i32, reach: i32) -> String {
        self.minimap.describe(&self.sim.factory, &self.sim.settlements.towns, (x, z), reach, self.holds_long_pole())
    }

    /// While a town depot is held, the explored tiles in block columns x0..=x1, z0..=z1 where a town would fit, as
    /// (x, z of the tile's north-west column, its size in blocks, the kind: `town_kind_name`) per tile; tiles near
    /// spawn or another town are left out. Empty otherwise. Each call works out a few more tiles, so a fresh view fills
    /// in over a few calls.
    pub fn world_map_sites(&mut self, x0: i32, z0: i32, x1: i32, z1: i32) -> Vec<i32> {
        if !self.held_footprint().is_some_and(|f| f.0 == TOWN_DEPOT) {
            return Vec::new();
        }
        let (lo, hi) = ((x0 >> CHUNK_SHIFT, z0 >> CHUNK_SHIFT), (x1 >> CHUNK_SHIFT, z1 >> CHUNK_SHIFT));
        let tiles = self.minimap.sites.tiles_in(&self.minimap.atlas, self.sim.world.generator(), lo, hi);
        let half = CHUNK_SIZE / 2;
        let clear = |x: i32, z: i32| {
            let far = |p: (i32, i32)| f64::from(x - p.0).hypot(f64::from(z - p.1));
            far((0, 0)) >= f64::from(SPAWN_CLEAR)
                && self.sim.settlements.towns.iter().all(|t| far((t.pos.x, t.pos.z)) >= f64::from(SPACING))
        };
        let mut out = Vec::new();
        for (cx, cz, kind) in tiles {
            let (x, z) = (cx << CHUNK_SHIFT, cz << CHUNK_SHIFT);
            if clear(x + half, z + half) {
                out.extend([x, z, CHUNK_SIZE, i32::from(kind)]);
            }
        }
        out
    }

    /// The explored map as bytes, for the host to keep with the world's record.
    pub fn explored_map(&self) -> Vec<u8> {
        self.minimap.atlas.export()
    }

    /// Restores `explored_map` from an earlier session of this world.
    pub fn set_explored_map(&mut self, bytes: &[u8]) {
        self.minimap.atlas.import(bytes);
    }

    /// The ore guide: one line per ore, tab-separated: block id, colour (0xRRGGBB), name, shallowest and
    /// deepest depth below the ground, where it is common, how to spot it.
    pub fn ore_guide(&self) -> String {
        ore_guide::guide_rows(self.sim.world.generator())
    }

    /// General advice for finding ore in this world, one line each.
    pub fn ore_guide_notes(&self) -> String {
        ore_guide::guide_notes(self.sim.world.generator())
    }
}

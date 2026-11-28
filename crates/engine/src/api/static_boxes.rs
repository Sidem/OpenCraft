//! Host API for per-chunk cached static machine and wire box buffers (`static_boxes/`).
//!
//! Exposes zero-copy pointers and counts for each chunk event popped by `next_static_box_event` (an upserted chunk's
//! boxes packed there, `box_pack`), plus the per-tier
//! belt UV scroll uniform (`belt_scroll`) used by `web/src/render/box-glsl.ts`.

use wasm_bindgen::prelude::*;

use crate::box_pack::{pack, PACKED_WORDS};
use crate::factory::BELT_TIERS;
use crate::static_boxes::StaticBoxEvent;
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Fractional UV scroll for belt `tier` (0..4) at this frame's render time (for `boxVert`'s `u_beltScroll`).
    pub fn belt_scroll(&self, tier: u8) -> f32 {
        let speed = BELT_TIERS.get(tier as usize).map_or(1.0, |t| t.speed);
        (self.render_time * f64::from(speed)).fract() as f32
    }

    /// Pops the next static box chunk event: 0 = none, 1 = upsert chunk, 2 = unload chunk.
    pub fn next_static_box_event(&mut self) -> u32 {
        self.static_boxes.cur = None;
        self.packed.static_cubes.clear();
        self.packed.static_round.clear();
        while let Some(ev) = self.static_boxes.events.pop_front() {
            match ev {
                StaticBoxEvent::Upsert(p) => {
                    if let Some(c) = self.static_boxes.chunk(p) {
                        pack(&c.cubes, &mut self.packed.static_cubes);
                        pack(&c.round, &mut self.packed.static_round);
                        self.static_boxes.cur = Some(ev);
                        return 1;
                    }
                }
                StaticBoxEvent::Unload(_) => {
                    self.static_boxes.cur = Some(ev);
                    return 2;
                }
            }
        }
        0
    }

    pub fn static_box_x(&self) -> i32 {
        match self.static_boxes.cur {
            Some(StaticBoxEvent::Upsert(p) | StaticBoxEvent::Unload(p)) => p.x,
            None => 0,
        }
    }

    pub fn static_box_y(&self) -> i32 {
        match self.static_boxes.cur {
            Some(StaticBoxEvent::Upsert(p) | StaticBoxEvent::Unload(p)) => p.y,
            None => 0,
        }
    }

    pub fn static_box_z(&self) -> i32 {
        match self.static_boxes.cur {
            Some(StaticBoxEvent::Upsert(p) | StaticBoxEvent::Unload(p)) => p.z,
            None => 0,
        }
    }

    pub fn static_box_bounds(&self) -> Vec<f32> {
        match self.static_boxes.cur {
            Some(StaticBoxEvent::Upsert(p)) => self.static_boxes.chunk(p).map_or_else(Vec::new, |c| c.bounds.to_vec()),
            _ => Vec::new(),
        }
    }

    /// The upserted chunk's boxes, packed (`static_box_count()` records of `instance_floats()` words; none after an unload).
    pub fn static_box_ptr(&self) -> usize {
        self.packed.static_cubes.as_ptr() as usize
    }

    pub fn static_box_count(&self) -> usize {
        self.packed.static_cubes.len() / PACKED_WORDS
    }

    /// The upserted chunk's round bodies, packed like `static_box_ptr`'s.
    pub fn static_round_ptr(&self) -> usize {
        self.packed.static_round.as_ptr() as usize
    }

    pub fn static_round_count(&self) -> usize {
        self.packed.static_round.len() / PACKED_WORDS
    }
    /// Total cached static box instances currently active across all loaded static chunks.
    pub fn static_box_total(&self) -> usize {
        self.static_boxes.total_instances
    }
}

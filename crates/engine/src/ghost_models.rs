//! Planned buildings drawn as themselves (presentation only, never saved, reads the core): every planted ghost and the
//! blueprint stamp preview show as the real machine's model in the host's blue see-through pass (the pass the held
//! multi-block machine uses: `port_view.rs`, `render/boxes.ts` `drawGhost`), a block the world's mesh draws whole (stone,
//! a storage box) as a see-through cube.
//! The cyan outlines of planted ghosts stay (`ghost_mode.rs` draws them through terrain, so a plan behind a hill is
//! still found); the stamp preview has only the models and its green box.
//!
//! Each set is a scratch `Factory` (`place_preview`: the machines' own `Machine::model`, never ticked) rebuilt only
//! when its source changes: the world's ghosts compared with the list it was built from, the stamp by its key (held
//! blueprint, turns, origin, library version). A frame then costs what drawing those machines costs, within
//! [`DRAW_RANGE`]. Invariants: at most [`crate::ghosts::MAX_GHOSTS`] planted and [`MAX_STAMP`] previewed; a tear-down
//! mark (air) has no model. To change the look: the tint is `GHOST` in `web/src/render/boxes.ts`.

use crate::block::{self, BlockId, Render};
use crate::blueprint::Blueprint;
use crate::factory::{push_box, Factory};
use crate::ghost_mode::DRAW_RANGE;
use crate::ghosts::{Ghost, Ghosts};
use crate::math::{IVec3, Vec3};

/// Machines of a blueprint in hand shown at most.
pub(crate) const MAX_STAMP: usize = 300;

/// What a stamp preview was built from: the blueprint's index in the library, its quarter turns, where it would stand
/// (the cell its lowest corner takes) and the library's version (a rename or edit rebuilds it).
pub(crate) type StampKey = (usize, u8, IVec3, u32);

/// One set of previewed buildings: their machines, and the plain blocks drawn as cubes.
#[derive(Default)]
struct Set {
    factory: Factory,
    cubes: Vec<(BlockId, IVec3)>,
}

#[derive(Default)]
pub(crate) struct GhostModels {
    planted: Set,
    planted_of: Ghosts,
    stamp: Set,
    stamp_of: Option<StampKey>,
}

impl GhostModels {
    /// Appends this frame's see-through boxes into `out` (camera-relative to `eye`).
    pub(crate) fn write(
        &mut self,
        planted: &Ghosts,
        stamp: Option<(StampKey, &Blueprint)>,
        eye: Vec3,
        time: f64,
        out: &mut Vec<f32>,
    ) {
        if self.planted_of != *planted {
            self.planted = Set::of(planted.iter().copied());
            self.planted_of = planted.clone();
        }
        self.planted.write(eye, time, out);
        let key = stamp.map(|s| s.0);
        if self.stamp_of != key {
            self.stamp =
                stamp.map_or_else(Set::default, |(k, bp)| Set::of(bp.ghosts_at(k.2, k.1).into_iter().take(MAX_STAMP)));
            self.stamp_of = key;
        }
        if key.is_some() {
            self.stamp.write(eye, time, out);
        }
    }
}

impl Set {
    fn of(ghosts: impl Iterator<Item = Ghost>) -> Set {
        let mut set = Set::default();
        for g in ghosts.filter(|g| g.block != block::AIR) {
            set.factory.place_preview(g.block, g.pos, g.facing, g.tier);
            // A block the world's mesh draws whole (stone, a storage box) has no model: a cube stands for it.
            if block::def(g.block).render == Render::Opaque {
                set.cubes.push((g.block, g.pos));
            }
        }
        set.factory.settle_preview();
        set
    }

    fn write(&self, eye: Vec3, time: f64, out: &mut Vec<f32>) {
        self.factory.write_instances(out, eye, time, DRAW_RANGE);
        for &(b, pos) in &self.cubes {
            let at = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            if at.length() > DRAW_RANGE {
                continue;
            }
            let f = block::def(b).faces;
            push_box(out, at, 0.0, [1.0, 1.0, 1.0], 0.0, [f[2], f[0], f[3]], false);
        }
    }
}

#[cfg(test)]
mod tests;

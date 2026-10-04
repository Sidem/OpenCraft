//! Tunnels (a kind of terraforming site, `factory/sites.rs`): a straight bore from one block to another, cut by
//! drone ports like a dig.
//!
//! Shape: the path runs along the longer horizontal axis from `from` to `to` (the other horizontal coordinate of `to`
//! is the one of `from`), one step per block, rising or falling at most 1 block in 2 (`new` refuses anything steeper
//! or longer than `MAX_TUNNEL`). The section is centred on the path and stands on it: the path block is the middle of
//! the floor, `SECTIONS` says how wide and tall (1 × 2, 3 × 3, 5 × 5). Cell order: step by step from `from`, in a
//! step the floor row first. All integer maths, so it is deterministic.
//! `survey_tunnel` is a query over loaded chunks for the planner. To add a section: append a row to `SECTIONS`
//! (its index is saved).

use super::{cut_takes, Column, SiteSurvey};
use crate::block::{is_ore, BlockId, LEAVES, LIQUID, LOG};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT;

/// Most blocks a tunnel may run.
pub const MAX_TUNNEL: i64 = 96;
/// Horizontal coordinates beyond this are refused (so no sum can overflow).
const FAR: i32 = 1 << 28;
/// Cross-sections as (width, height); the width is odd except that a 1 × 2 is one block wide.
pub const SECTIONS: [(i32, i32); 3] = [(1, 2), (3, 3), (5, 5)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tunnel {
    pub from: IVec3,
    /// Has the same coordinate as `from` across the path.
    pub to: IVec3,
    /// Index into `SECTIONS`.
    pub size: u8,
}

impl Tunnel {
    /// The tunnel from `a` towards `b` (snapped onto the longer horizontal axis), or `None` if the section is
    /// unknown, the run is too long, too steep, or any cell would leave the world's height.
    pub fn new(a: IVec3, b: IVec3, size: u8) -> Option<Tunnel> {
        let (_, h) = *SECTIONS.get(size as usize)?;
        let ys = 1..WORLD_HEIGHT - 1;
        let sane = |v: IVec3| ys.contains(&v.y) && (-FAR..FAR).contains(&v.x) && (-FAR..FAR).contains(&v.z);
        if !sane(a) || !sane(b) {
            return None;
        }
        let (dx, dz) = ((b.x as i64 - a.x as i64).abs(), (b.z as i64 - a.z as i64).abs());
        let to = if dx >= dz { IVec3::new(b.x, b.y, a.z) } else { IVec3::new(a.x, b.y, b.z) };
        let run = dx.max(dz);
        let rise = (b.y as i64 - a.y as i64).abs();
        let top = a.y.max(b.y) + h - 1;
        (run < MAX_TUNNEL && rise * 2 <= run && top < WORLD_HEIGHT - 1).then_some(Tunnel { from: a, to, size })
    }

    /// Blocks along the path.
    pub fn steps(&self) -> u32 {
        ((self.to.x - self.from.x).abs().max((self.to.z - self.from.z).abs()) + 1) as u32
    }

    /// Cells in each step.
    fn per_step(&self) -> u32 {
        let (w, h) = SECTIONS[self.size as usize];
        (w * h) as u32
    }

    pub fn cells(&self) -> u32 {
        self.steps() * self.per_step()
    }

    /// Cell `i` (below `cells()`).
    pub fn cell(&self, i: u32) -> IVec3 {
        let (step, k) = ((i / self.per_step()) as i32, (i % self.per_step()) as i32);
        let (w, _) = SECTIONS[self.size as usize];
        let (across, up) = (k % w - w / 2, k / w);
        self.at(step, across, up)
    }

    /// Whether `pos` is one of the cells.
    pub fn has_cell(&self, pos: IVec3) -> bool {
        let (w, h) = SECTIONS[self.size as usize];
        let (along, across) = if self.along_x() {
            ((pos.x - self.from.x) * self.sign(), pos.z - self.from.z)
        } else {
            ((pos.z - self.from.z) * self.sign(), pos.x - self.from.x)
        };
        (0..self.steps() as i32).contains(&along)
            && across.abs() <= w / 2
            && (0..h).contains(&(pos.y - self.path_y(along)))
    }

    /// The columns the tunnel's cells fill (lowest first) and the lowest and highest cell's height.
    pub fn bounds(&self) -> (Column, Column, i32, i32) {
        let (_, h) = SECTIONS[self.size as usize];
        let reach = SECTIONS[self.size as usize].0 / 2;
        let lo = (self.from.x.min(self.to.x) - reach, self.from.z.min(self.to.z) - reach);
        let hi = (self.from.x.max(self.to.x) + reach, self.from.z.max(self.to.z) + reach);
        (lo, hi, self.from.y.min(self.to.y), self.from.y.max(self.to.y) + h - 1)
    }

    fn along_x(&self) -> bool {
        self.to.x != self.from.x
    }

    /// Which way the path runs along its axis (1 for a single block).
    fn sign(&self) -> i32 {
        let d = if self.along_x() { self.to.x - self.from.x } else { self.to.z - self.from.z };
        if d < 0 {
            -1
        } else {
            1
        }
    }

    /// The path's height at `step`: a straight line from end to end, rounded half up.
    fn path_y(&self, step: i32) -> i32 {
        let run = self.steps() as i32 - 1;
        if run == 0 {
            return self.from.y;
        }
        self.from.y + (2 * (self.to.y - self.from.y) * step + run).div_euclid(2 * run)
    }

    fn at(&self, step: i32, across: i32, up: i32) -> IVec3 {
        let y = self.path_y(step) + up;
        if self.along_x() {
            IVec3::new(self.from.x + step * self.sign(), y, self.from.z + across)
        } else {
            IVec3::new(self.from.x + across, y, self.from.z + step * self.sign())
        }
    }
}

/// Whether a cut at `pos` would let water in: a water cell touches it.
pub fn touches_water(mut block_at: impl FnMut(IVec3) -> Option<BlockId>, pos: IVec3) -> bool {
    const SIDES: [(i32, i32, i32); 6] = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)];
    SIDES.iter().any(|&(x, y, z)| block_at(pos + IVec3::new(x, y, z)).is_some_and(|b| LIQUID[b as usize]))
}

/// What cutting the tunnel would move, among loaded chunks (a query: never creates core state). Cells a
/// water cell touches are counted as `water`: drones leave them until the water is gone.
pub fn survey_tunnel(world: &World, t: &Tunnel) -> SiteSurvey {
    let mut s = SiteSurvey::default();
    for i in 0..t.cells() {
        let pos = t.cell(i);
        match world.get_block(pos) {
            None => s.unseen += 1,
            Some(b) if cut_takes(b) => {
                s.cut += 1;
                s.ore += is_ore(b) as u32;
                s.trees += (b == LOG || b == LEAVES) as u32;
                s.water += touches_water(|p| world.get_block(p), pos) as u32;
            }
            Some(_) => {}
        }
    }
    s
}

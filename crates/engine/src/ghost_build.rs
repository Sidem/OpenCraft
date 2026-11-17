//! Building a ghost from the pack with one key (the local player's hands, presentation only): aim at a ghost and press
//! X, and the ghost is built for real with its item taken from anywhere in the pack, selected or not
//! (`Action::BuildGhost`, `ghosts.rs` `build_ghost`). Works in and out of ghost mode, within the hands' reach.
//!
//! The ghost is found by a ray through the cells each ghost covers, nearest first and not behind the block the
//! crosshair is on, so a ghost standing in the air (a belt, a pole) is aimed at as it looks. Whether it can be built
//! (free cells, ground under a pole) is the core's call, as for a drone. Never touches the core directly.

use crate::action::Action;
use crate::item;
use crate::math::{IVec3, Vec3};
use crate::Game;

impl Game {
    /// X: builds the ghost under the crosshair from the pack. Returns a line for the player when it cannot be tried
    /// ("" when the build was queued): nothing aimed, or the item is not in the pack.
    pub(crate) fn build_aimed_ghost(&mut self) -> String {
        let Some(anchor) = self.aimed_ghost() else { return "Aim at a ghost to build it".into() };
        let Some(g) = self.sim.ghosts.covering(anchor).copied() else { return String::new() };
        let item = g.item();
        if self.inventory().count(item) == 0 {
            return format!("You have no {} for that ghost", item::name(item));
        }
        self.act(Action::BuildGhost { pos: anchor });
        String::new()
    }

    /// The anchor of the nearest build ghost the crosshair is on within reach, if any.
    pub(crate) fn aimed_ghost(&self) -> Option<IVec3> {
        let eye = self.body().eye();
        let dir = self.body().look_dir();
        let origin = self.third_person.aim_eye(eye, dir);
        let reach = self.reach() + (origin - eye).length();
        // A ghost behind the block the crosshair is on is out of sight.
        let wall = self.target.map_or(reach, |h| (h.block.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - origin).length());
        let mut best: Option<(f64, IVec3)> = None;
        for g in self.sim.ghosts.iter().filter(|g| g.block != crate::block::AIR) {
            if (g.pos.as_vec3() - origin).length() > reach + 6.0 {
                continue;
            }
            for c in g.cells() {
                let hit = ray_cell(origin, dir, c).filter(|&t| t <= reach.min(wall));
                if let Some(t) = hit.filter(|&t| best.is_none_or(|(b, _)| t < b)) {
                    best = Some((t, g.pos));
                }
            }
        }
        best.map(|b| b.1)
    }
}

/// Where a ray from `origin` along `dir` first enters the unit cell `cell` (0 if it starts inside), if it does.
fn ray_cell(origin: Vec3, dir: Vec3, cell: IVec3) -> Option<f64> {
    let (lo, hi) = (cell.as_vec3(), cell.as_vec3() + Vec3::new(1.0, 1.0, 1.0));
    let (mut near, mut far) = (0.0_f64, f64::MAX);
    for (o, d, l, h) in [(origin.x, dir.x, lo.x, hi.x), (origin.y, dir.y, lo.y, hi.y), (origin.z, dir.z, lo.z, hi.z)] {
        if d.abs() < 1e-12 {
            if o < l || o > h {
                return None;
            }
            continue;
        }
        let (a, b) = ((l - o) / d, (h - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

#[cfg(test)]
mod tests;

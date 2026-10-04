//! Drawing the construction drones (presentation only; the flight is core state: `drones/`) and the local
//! player's personal drone (`helpers/`). Each drone is a few boxes, blended between its last two ticks,
//! bobbing a little, with a lamp underneath and the block it carries slung below. To change the look:
//! `push_drone`.

use crate::block::tex;
use crate::factory::push_box;
use crate::item;
use crate::math::Vec3;
use crate::{Game, TICK};

/// Drones further than this from the eye are not drawn.
const DRAW_RANGE: f64 = 160.0;

impl Game {
    /// Appends every drone near the eye to the box instances; `alpha` is how far into the tick the frame is.
    pub(crate) fn write_drone_instances(&mut self, eye: Vec3, alpha: f64) {
        let time = (self.sim.tick as f64 + alpha) * TICK;
        for (i, d) in self.sim.drones.list.iter().enumerate() {
            let pos = d.prev + (d.pos - d.prev) * alpha;
            if (pos - eye).length() > DRAW_RANGE {
                continue;
            }
            let bob = (time * 6.0 + i as f64).sin() as f32 * 0.04;
            let carried = d.load.count > 0;
            let yaw = (time * 0.5) as f32;
            push_drone(&mut self.instances, pos - eye + Vec3::new(0.0, f64::from(bob), 0.0), yaw, carried);
        }
        self.write_companion(eye, alpha, time);
    }

    /// The local player's personal drone: hovering at their right shoulder, or out on an errand to its box.
    fn write_companion(&mut self, eye: Vec3, alpha: f64, time: f64) {
        let Some(p) = self.sim.player(self.local).filter(|p| p.inventory.count(item::PERSONAL_DRONE) > 0) else {
            return;
        };
        let body = self.body();
        let (sy, cy) = body.yaw.sin_cos();
        let home = self.prev_eye
            + (body.eye() - self.prev_eye) * alpha
            + Vec3::new(cy, 0.0, sy)
            + Vec3::new(-sy, 0.0, cy) * 0.7
            + Vec3::new(0.0, 0.1 + 0.06 * (time * 3.0).sin(), 0.0);
        let mut at = home;
        if let Some(f) = p.helpers.fetch {
            let done = 1.0 - (f64::from(f.left) - alpha) / f64::from(f.total.max(1));
            let leg = if done < 0.5 { 2.0 * done } else { 2.0 - 2.0 * done }.clamp(0.0, 1.0);
            let goal = f.from.as_vec3() + Vec3::new(0.5, 1.6, 0.5);
            at = home + (goal - home) * leg;
        }
        push_drone(&mut self.instances, at - eye, (time * 1.5) as f32, false);
    }
}

fn push_drone(out: &mut Vec<f32>, at: Vec3, yaw: f32, carried: bool) {
    let body = [tex::DRONE; 3];
    push_box(out, at, yaw, [0.34, 0.12, 0.34], 0.0, body, false);
    // Two crossed arms with a rotor lamp at each end.
    for (dx, dz) in [(0.0, 1.0), (1.0, 0.0)] {
        let size = [0.1 + 0.6 * dx as f32, 0.04, 0.1 + 0.6 * dz as f32];
        push_box(out, at + Vec3::new(0.0, 0.06, 0.0), yaw, size, 0.0, [tex::FRAME; 3], false);
    }
    push_box(out, at + Vec3::new(0.0, -0.09, 0.0), yaw, [0.1, 0.06, 0.1], 0.0, [tex::DRONE_PORT_TOP; 3], false);
    if carried {
        push_box(out, at + Vec3::new(0.0, -0.3, 0.0), yaw, [0.26, 0.26, 0.26], 0.0, [tex::FRAME; 3], false);
    }
}

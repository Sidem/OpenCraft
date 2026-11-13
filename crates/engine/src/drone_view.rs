//! Drawing the construction and cargo drones (presentation only; the flight is core state: `drones/`); the personal
//! drones are `companion_view.rs`. Each drone is a few boxes, blended between its last two ticks,
//! bobbing a little, with a lamp underneath and the block it carries slung below. To change the look:
//! `push_drone`.

use crate::block::tex;
use crate::factory::{push_box, push_capsule};
use crate::math::Vec3;
use crate::{Game, TICK};

/// Drones further than this from the eye are not drawn.
const DRAW_RANGE: f64 = 160.0;
/// Drop pods further than this from the eye are not drawn (they fall from high up and are seen from afar).
const POD_RANGE: f64 = 400.0;

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
        for (i, c) in self.sim.cargo.list.iter().enumerate() {
            let pos = c.prev + (c.pos - c.prev) * alpha;
            if (pos - eye).length() > DRAW_RANGE {
                continue;
            }
            let bob = (time * 5.0 + i as f64).sin() as f32 * 0.05;
            let at = pos - eye + Vec3::new(0.0, f64::from(bob), 0.0);
            let yaw = (time * 0.4) as f32;
            push_drone(&mut self.instances, at, yaw, false);
            // A cargo drone is a drone with a crate slung under it (amber-lidded while it carries a load).
            let lid = if c.load.count > 0 { tex::CARGO_DRONE } else { tex::FRAME };
            push_box(&mut self.instances, at + Vec3::new(0.0, -0.32, 0.0), yaw, [0.5, 0.3, 0.5], 0.0, [lid; 3], false);
        }
        self.write_town_instances(eye, alpha);
        self.write_pod_instances(eye, alpha);
        self.write_companions(eye, alpha, time);
    }

    /// The drop pods in their first seconds, rising out of their launchers, and in their last, braking onto their pads: a
    /// capsule on its thrusters (`factory/capsule.rs`). Then the bursts of dust where they left or landed (`orbit_fx.rs`).
    fn write_pod_instances(&mut self, eye: Vec3, alpha: f64) {
        let time = self.sim.tick as f64 + alpha;
        let (factory, out) = (&self.sim.factory, &mut self.instances);
        for (i, at) in factory.falling_pods(time).into_iter().chain(factory.rising_pods(time)).enumerate() {
            let at = at - eye;
            if at.length() > POD_RANGE {
                continue;
            }
            push_capsule(out, at, 1.0, time * TICK, (i as f64 * 0.37).fract());
        }
        self.orbit_fx.write(&mut self.instances, eye, time * TICK);
    }
}

pub(crate) fn push_drone(out: &mut Vec<f32>, at: Vec3, yaw: f32, carried: bool) {
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

//! The water wheel's dynamic 3D model and visual water interaction (presentation only): a steel axle, iron-banded
//! wooden hub, dual circular oak shroud rims (12 felloes each), closed inner sole-board drum, 8 radial spokes per
//! face, and 12 pitched wooden paddles turning smoothly in the direction pushed by falling water.
//!
//! When falling water hits one side (`Store::hydro_mask`), the descending buckets on that side visibly carry blue
//! water (`tex::WATER`) down and spill it at the bottom, while animated water droplets and white spray foam burst
//! where the waterfall strikes the paddles and pours into the tailrace.
//!
//! Geometry: model units are blocks, origin the 3×1×3 footprint's centre, axle along Z (`+z` front), `+x` right,
//! `+y` up. Every rotating box uses `[axle, radial, tangential]` as its `push_framed` axes so `axle.y == 0` always,
//! avoiding gimbal lock at every angle and facing.

use std::f64::consts::{PI, TAU};

use crate::block::tex;
use crate::math::Vec3;
use crate::TICK;

use super::super::super::render::{push_box, push_framed, push_shape, Axis, Form, Framed};
use super::super::super::smoke;
use super::super::Processor;

/// Number of rim segments, sole-board planks, and paddles around the wheel.
const SEGMENTS: usize = 12;
/// Radial positions (in blocks) of the sole-board drum, shroud rims, paddle centres and water pockets.
const R_SOLE: f64 = 0.99;
const R_RIM: f64 = 1.16;
const R_PADDLE: f64 = 1.21;
const R_WATER: f64 = 1.18;
/// Slight forward pitch of each wooden paddle (radians) so it reads as a real bucket blade.
const PADDLE_CANT: f64 = 0.16;
/// Splash droplet lifetime (seconds).
const SPLASH_LIFE: f64 = 0.55;
const SPILL_LIFE: f64 = 0.65;

/// Draws the spinning wheel and its water interaction about `centre` (footprint centre, camera-relative), turned by
/// `yaw`, at render `time`.
pub fn draw(p: &Processor, out: &mut Vec<f32>, centre: Vec3, yaw: f32, time: f64) {
    let turn = p.store.wheel_angle + (time / TICK).fract() * p.store.wheel_spin;
    let theta = turn * TAU;
    let left_mask = p.store.hydro_mask & 0x07;
    let right_mask = (p.store.hydro_mask >> 4) & 0x07;
    let driven = (left_mask > 0) ^ (right_mask > 0);
    let mut m = Model { out, centre, yaw: f64::from(yaw) };
    m.axle_and_hub();
    m.spokes(theta);
    m.rims_and_drum(theta);
    m.paddles_and_water(theta, left_mask, right_mask, driven);
    if left_mask != 0 || right_mask != 0 {
        let seed = smoke::seed(p.pos);
        m.water_effects(left_mask, right_mask, driven, time, seed);
    }
}

struct Model<'a> {
    out: &'a mut Vec<f32>,
    centre: Vec3,
    yaw: f64,
}

impl Model<'_> {
    /// Central steel axle passing through both pillow-block bearings, wooden nave (hub), and twin iron binding rings.
    fn axle_and_hub(&mut self) {
        let axle = Form::Round(Axis::Z, 1.0);
        push_shape(self.out, axle, self.centre, self.yaw as f32, [0.18, 0.18, 0.98], [tex::STEEL; 3], false);
        let hub_tex = [tex::LOG_TOP, tex::LOG_SIDE, tex::LOG_TOP];
        push_shape(self.out, axle, self.centre, self.yaw as f32, [0.54, 0.54, 0.60], hub_tex, false);
        for z in [-0.23, 0.23] {
            let at = self.point([0.0, 0.0, z]);
            push_shape(self.out, axle, at, self.yaw as f32, [0.58, 0.58, 0.07], [tex::MAT_GUNMETAL; 3], false);
        }
    }

    /// Four full-diameter oak spokes on each face (8 radial arms per side), slightly staggered in Z inside the hub.
    fn spokes(&mut self, theta: f64) {
        for &face_z in &[-0.20, 0.20] {
            for k in 0..4 {
                let a = theta + k as f64 * (PI / 4.0);
                let z = face_z + (k as f64 - 1.5) * 0.004;
                self.wheel_box([0.0, 0.0, z], a, [0.06, 2.26, 0.09], [tex::HANDLE; 3]);
            }
        }
    }

    /// Dual circular oak shroud rims (12 felloes per side) and the closed wooden sole-board drum between them.
    fn rims_and_drum(&mut self, theta: f64) {
        let step = TAU / SEGMENTS as f64;
        for k in 0..SEGMENTS {
            let a = theta + (k as f64 + 0.5) * step;
            let (c, s) = (a.cos(), a.sin());
            let stagger = if k % 2 == 0 { 0.002 } else { -0.002 };
            // Front and back circular oak shroud felloes.
            for &face_z in &[-0.27, 0.27] {
                self.wheel_box([R_RIM * c, R_RIM * s, face_z + stagger], a, [0.06, 0.19, 0.63], [tex::HANDLE; 3]);
            }
            // Inner sole-board plank forming the water-tight backing drum behind the buckets.
            let drum_w = if k % 2 == 0 { 0.48 } else { 0.474 };
            self.wheel_box([R_SOLE * c, R_SOLE * s, 0.0], a, [drum_w, 0.06, 0.54], [tex::PLANKS; 3]);
        }
    }

    /// Twelve pitched wooden paddles around the circumference, plus water fills inside the descending wet-side buckets.
    fn paddles_and_water(&mut self, theta: f64, left_mask: u8, right_mask: u8, driving: bool) {
        let step = TAU / SEGMENTS as f64;
        let drive_x: f64 = match (driving, left_mask > 0, right_mask > 0) {
            (true, true, false) => -1.0,
            (true, false, true) => 1.0,
            _ => 0.0,
        };
        let active_mask = if drive_x < 0.0 { left_mask } else { right_mask };
        let top_tier = highest_tier(active_mask);
        let catch_y = top_tier + 0.48;
        // Cant the paddles slightly toward the falling water when driven; keep neutral cant at rest.
        let cant = if drive_x > 0.0 { -PADDLE_CANT } else { PADDLE_CANT };

        for k in 0..SEGMENTS {
            let a = theta + k as f64 * step;
            let (c, s) = (a.cos(), a.sin());
            self.wheel_box([R_PADDLE * c, R_PADDLE * s, 0.0], a + cant, [0.49, 0.42, 0.06], [tex::PLANKS; 3]);

            if drive_x != 0.0 {
                // Position the water pocket just on the upper side of the descending paddle.
                let pocket_a = a - drive_x * (step * 0.28);
                let (px, py) = (R_WATER * pocket_a.cos(), R_WATER * pocket_a.sin());
                if px * drive_x > 0.15 && py <= catch_y && py > -1.25 {
                    let spill = ((py + 1.25) / 0.42).clamp(0.0, 1.0);
                    let fill = spill.min(((catch_y - py) / 0.25).clamp(0.35, 1.0)) as f32;
                    self.wheel_box([px, py, 0.0], pocket_a, [0.45, 0.24 * fill, 0.19 * fill], [tex::WATER; 3]);
                }
            }
        }
    }

    /// Animated water impact splashes where waterfalls strike the paddles and tailrace spill foam underneath.
    fn water_effects(&mut self, left_mask: u8, right_mask: u8, driving: bool, time: f64, seed: f64) {
        // Impact spray on each side that has falling water (even when stalled on both sides).
        for &(side_x, mask) in &[(-1.0, left_mask), (1.0, right_mask)] {
            if mask == 0 {
                continue;
            }
            for tier in 0..3u8 {
                if mask & (1 << tier) == 0 {
                    continue;
                }
                let base_y = f64::from(tier) - 1.0;
                for k in 0..3usize {
                    let phase = (time / SPLASH_LIFE + seed + tier as f64 * 0.31 + k as f64 / 3.0).fract();
                    let spread = (k as f64 - 1.0) * 0.22;
                    let out_x = side_x * (1.36 + 0.28 * phase + 0.06 * (phase * PI).sin());
                    let drop_y = base_y + 0.22 + 0.24 * phase - 1.15 * phase * phase;
                    let jitter_z = spread + 0.08 * (time * 5.0 + k as f64 * 2.3).sin();
                    let sz = (0.14 * (1.0 - 0.65 * phase)) as f32;
                    let layer = if k == 1 || phase > 0.45 { tex::SMOKE_THIN } else { tex::WATER };
                    let pos = self.point([out_x, drop_y, jitter_z]);
                    push_box(self.out, pos, pos.x.atan2(pos.z) as f32, [sz; 3], 0.0, [layer; 3], false);
                }
            }
        }

        // Tailrace spill & foam pouring off the bottom buckets while the wheel is driven.
        if driving {
            let drive_x = if left_mask > 0 { -1.0 } else { 1.0 };
            for k in 0..5usize {
                let phase = (time / SPILL_LIFE + seed * 1.7 + k as f64 / 5.0).fract();
                let x = drive_x * (0.85 - 0.95 * phase);
                let y = -1.18 - 0.30 * phase * phase;
                let z = ((k as f64 * 2.1 + seed * 7.0).sin()) * 0.22;
                let sz = (0.15 * (1.0 - 0.55 * phase)) as f32;
                let layer = if k % 2 == 0 { tex::WATER } else { tex::SMOKE_THIN };
                let pos = self.point([x, y, z]);
                push_box(self.out, pos, pos.x.atan2(pos.z) as f32, [sz; 3], 0.0, [layer; 3], false);
            }
        }
    }

    /// Pushes a box in the rotating wheel frame at angle `angle`: `size` is `[along_axle_z, radial, tangential]`.
    /// Uses `[axle, radial, tangential]` so `axle.y == 0` always (no gimbal lock at any angle or facing).
    fn wheel_box(&mut self, at: [f64; 3], angle: f64, size: [f32; 3], layers: [u16; 3]) {
        let (s, c) = angle.sin_cos();
        let axes = [self.dir([0.0, 0.0, 1.0]), self.dir([c, s, 0.0]), self.dir([-s, c, 0.0])];
        let pos = self.point(at);
        push_framed(self.out, pos, axes, size, layers, Framed { taper: None, slide: None });
    }

    /// Model point `[x, y, z]` turned with the machine, camera-relative.
    fn point(&self, p: [f64; 3]) -> Vec3 {
        self.centre + self.dir(p)
    }

    /// Model direction `[x, y, z]` turned by `yaw` (same frame as `model::local`).
    fn dir(&self, [x, y, z]: [f64; 3]) -> Vec3 {
        let (s, c) = self.yaw.sin_cos();
        Vec3::new(c * x - s * z, y, s * x + c * z)
    }
}

/// Highest active vertical tier (`-1.0`, `0.0`, or `+1.0`) in a 3-bit side mask.
fn highest_tier(mask: u8) -> f64 {
    if mask & 0b100 != 0 {
        1.0
    } else if mask & 0b010 != 0 {
        0.0
    } else {
        -1.0
    }
}

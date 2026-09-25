//! Machine models as box instances: belts with their items, and miners with a pumping drill and a
//! status lamp. Also defines the box instance format, which dropped items (`entities.rs`) share.
//! Presentation only: nothing here changes factory state.

use std::f32::consts::FRAC_PI_2;

use crate::block::{tex, BlockId, FACE_BOTTOM, FACE_SIDE, FACE_TEX, FACE_TOP};
use crate::math::Vec3;

use super::belt::{BELT_HEIGHT, BELT_SPEED, ITEM_SIZE};
use super::{Factory, MinerStatus, FACES};

/// Floats per box instance: centre xyz (camera-relative), yaw, size xyz, uv scroll,
/// texture layers top/side/bottom, uv mode (0 = whole texture per face, 1 = world-scaled).
pub const INSTANCE_FLOATS: usize = 12;

/// Pushes one box instance (see [`INSTANCE_FLOATS`]).
#[allow(clippy::too_many_arguments)]
pub fn push_box(
    out: &mut Vec<f32>,
    center: Vec3,
    yaw: f32,
    size: [f32; 3],
    scroll: f32,
    tex: [u16; 3],
    world_uv: bool,
) {
    out.extend_from_slice(&[
        center.x as f32,
        center.y as f32,
        center.z as f32,
        yaw,
        size[0],
        size[1],
        size[2],
        scroll,
        tex[0] as f32,
        tex[1] as f32,
        tex[2] as f32,
        if world_uv { 1.0 } else { 0.0 },
    ]);
}

impl Factory {
    /// Writes box instances for every machine within `range` of the camera.
    pub fn write_instances(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        let r2 = range * range;
        let near = |c: Vec3| {
            let d = c - eye;
            d.x * d.x + d.y * d.y + d.z * d.z <= r2
        };
        let scroll = (time * BELT_SPEED as f64).fract() as f32;
        for b in &self.belts {
            let base = b.pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
            if !near(base) {
                continue;
            }
            let rel = base - eye;
            let yaw = b.dir as f32 * FRAC_PI_2;
            let (s, c) = yaw.sin_cos();
            let at = |x: f32, y: f32, z: f32| rel + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
            push_box(
                out,
                at(0.0, BELT_HEIGHT * 0.5, 0.0),
                yaw,
                [0.84, BELT_HEIGHT, 1.0],
                scroll,
                [tex::BELT_TOP, tex::FRAME, tex::FRAME],
                true,
            );
            for side in [-0.46, 0.46] {
                push_box(out, at(side, 0.13, 0.0), yaw, [0.08, 0.26, 1.0], 0.0, [tex::FRAME; 3], true);
            }
            for it in &b.items {
                let (x, z) = b.offset(it.p);
                let pos = rel + Vec3::new(x as f64, (BELT_HEIGHT + ITEM_SIZE * 0.5) as f64, z as f64);
                push_box(out, pos, yaw, [ITEM_SIZE; 3], 0.0, item_tex(it.item), false);
            }
        }

        for m in &self.miners {
            let center = m.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
            if !near(center) {
                continue;
            }
            let rel = center - eye;
            let f = FACES[m.drill as usize].as_vec3();
            let axis = m.drill as usize / 2;
            let size = |along: f32, across: f32| {
                let mut s = [across; 3];
                s[axis] = along;
                s
            };
            let running = m.status == MinerStatus::Running && m.draw_rate > 0.01;
            let pump = if running { 0.05 * (0.5 + 0.5 * (time * 10.0).sin()) } else { 0.0 };
            push_box(
                out,
                rel + f * -0.13,
                0.0,
                size(0.7, 0.86),
                0.0,
                [tex::MINER_TOP, tex::MINER_SIDE, tex::FRAME],
                true,
            );
            push_box(out, rel + f * 0.27, 0.0, size(0.1, 0.6), 0.0, [tex::FRAME; 3], true);
            push_box(out, rel + f * (0.44 + pump), 0.0, size(0.36, 0.22), 0.0, [tex::DRILL; 3], true);
            let lamp = match m.status {
                MinerStatus::Running => tex::LAMP_GREEN,
                MinerStatus::OutputFull => tex::LAMP_YELLOW,
                MinerStatus::NoDeposit | MinerStatus::Exhausted => tex::LAMP_RED,
            };
            push_box(out, rel + f * -0.5, 0.0, size(0.04, 0.2), 0.0, [lamp; 3], false);
        }
    }
}

fn item_tex(item: BlockId) -> [u16; 3] {
    let f = FACE_TEX[item as usize];
    [f[FACE_TOP], f[FACE_SIDE], f[FACE_BOTTOM]]
}

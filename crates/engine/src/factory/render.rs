//! Machine models as box instances: the instance format, which dropped items (`entities.rs`) share,
//! and `write_instances`, which asks every machine near the camera for its model (`Machine::model`,
//! in each kind's file). Presentation only: nothing here changes factory state.

use crate::math::Vec3;

use super::{Factory, Machine};

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
        models(&self.belts, out, eye, time, range);
        models(&self.miners, out, eye, time, range);
        models(&self.storages, out, eye, time, range);
        models(&self.smelters, out, eye, time, range);
        models(&self.constructors, out, eye, time, range);
        models(&self.routers, out, eye, time, range);
    }
}

/// Models of the machines in `list` whose cell centre is within `range` of `eye`.
fn models<T: Machine>(list: &[T], out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
    for m in list {
        let rel = m.pos().as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
        if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range * range {
            m.model(out, rel, time);
        }
    }
}

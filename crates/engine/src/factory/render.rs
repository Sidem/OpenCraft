//! Machine models as box instances: the instance format, which dropped items (`entities.rs`) share,
//! and `write_instances`, which asks every machine near the camera for its model (`Machine::model`,
//! in each kind's file); `map_machines` for the minimap's marks. Presentation only: nothing here
//! changes factory state.

use crate::math::{IVec3, Vec3};

use super::{Factory, Kind, Machine};

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
        models(&self.generators, out, eye, time, range);
        models(&self.poles, out, eye, time, range);
        models(&self.labs, out, eye, time, range);
        models(&self.pipework, out, eye, time, range);
        self.write_wires(out, eye, range);
    }

    /// Calls `f` with the kind and position of every machine a map marks (not belts, routers or poles,
    /// which are many and show as terrain) whose column lies in `lo..=hi` (x, z).
    pub fn map_machines(&self, lo: (i32, i32), hi: (i32, i32), f: &mut impl FnMut(Kind, IVec3)) {
        let mut each = |kind: Kind, p: IVec3| {
            if (lo.0..=hi.0).contains(&p.x) && (lo.1..=hi.1).contains(&p.z) {
                f(kind, p);
            }
        };
        self.miners.iter().for_each(|m| each(Kind::Miner, m.pos()));
        self.storages.iter().for_each(|m| each(Kind::Storage, m.pos()));
        self.smelters.iter().for_each(|m| each(Kind::Smelter, m.pos()));
        self.constructors.iter().for_each(|m| each(Kind::Constructor, m.pos()));
        self.generators.iter().for_each(|m| each(Kind::Generator, m.pos()));
        self.labs.iter().for_each(|m| each(Kind::Lab, m.pos()));
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

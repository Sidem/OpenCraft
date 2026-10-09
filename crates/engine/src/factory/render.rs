//! Machine models as box instances: the instance format, which dropped items (`entities.rs`) share,
//! and `write_instances`, which asks every machine near the camera for its model (`Machine::model`,
//! in each kind's file); `map_machines` for the minimap's marks. Presentation only: nothing here
//! changes factory state.

use crate::math::{IVec3, Vec3};

use crate::block::{BlockId, GENERATOR, LAB, MINER, QUARRY, STORAGE};
use crate::world::DAYLIGHT;

use super::{Factory, Machine};

/// Floats per box instance: centre xyz (camera-relative), yaw, size xyz, uv scroll,
/// texture layers top/side/bottom, and the last float: uv mode (0 = whole texture per face, 1 =
/// world-scaled) plus twice the light byte (sky in the low nibble, block light in the high one, as
/// `light.rs` writes it), then pitch, roll, top-width taper and a reserved float. `push_box` starts
/// every box at [`DAYLIGHT`]; `light_boxes` sets the real light. Ordinary boxes have no tilt or taper.
pub const INSTANCE_FLOATS: usize = 16;
const MODE_LIGHT: usize = 11;

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
        packed(world_uv, DAYLIGHT),
        0.0,
        0.0,
        1.0,
        0.0,
    ]);
}

/// The instance's last float: the uv mode plus twice the light byte.
fn packed(world_uv: bool, light: u8) -> f32 {
    f32::from(world_uv) + 2.0 * f32::from(light)
}

/// Lights every box in `boxes` (records of [`INSTANCE_FLOATS`], centres relative to `eye`) with the light
/// of the cell its centre is in, from `light` (`World::light_at`). A cell with no light at all (a wire
/// or an item sunk into the ground, whose centre is inside solid rock) borrows the best of its
/// neighbours, so only a box really in the dark is dark.
pub fn light_boxes(boxes: &mut [f32], eye: Vec3, mut light: impl FnMut(IVec3) -> u8) {
    const AROUND: [(i32, i32, i32); 6] = [(0, 1, 0), (1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1), (0, -1, 0)];
    let mut last: Option<(IVec3, u8)> = None;
    for rec in boxes.chunks_exact_mut(INSTANCE_FLOATS) {
        let cell = (eye + Vec3::new(rec[0] as f64, rec[1] as f64, rec[2] as f64)).floor();
        let lit = match last {
            Some((c, l)) if c == cell => l,
            _ => match light(cell) {
                0 => AROUND.iter().map(|&(x, y, z)| light(cell + IVec3::new(x, y, z))).fold(0, brighter),
                l => l,
            },
        };
        last = Some((cell, lit));
        rec[MODE_LIGHT] = packed(rec[MODE_LIGHT] as u32 & 1 == 1, lit);
    }
}

/// The brighter of two light bytes, sky and block light each.
fn brighter(a: u8, b: u8) -> u8 {
    (a & 15).max(b & 15) | (a >> 4).max(b >> 4) << 4
}

impl Factory {
    /// Writes box instances for every machine within `range` of the camera.
    pub fn write_instances(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        models(&self.belts, out, eye, time, range);
        models(&self.miners, out, eye, time, range);
        models(&self.storages, out, eye, time, range);
        models(&self.processors, out, eye, time, range);
        models(&self.routers, out, eye, time, range);
        models(&self.generators, out, eye, time, range);
        models(&self.poles, out, eye, time, range);
        self.write_accents(out, eye, range);
        models(&self.labs, out, eye, time, range);
        models(&self.pipework, out, eye, time, range);
        models(&self.quarries, out, eye, time, range);
        models(&self.sensors, out, eye, time, range);
        models(&self.nodes, out, eye, time, range);
        self.write_fibre(out, eye, range);
        self.write_beams(out, eye, time, range);
        models(&self.rails, out, eye, time, range);
        self.write_tracks(out, eye, range);
        self.write_train_models(out, eye, range);
        self.write_wires(out, eye, range);
        self.write_cables(out, eye, range);
    }

    /// Calls `f` with the block and position of every machine a map marks (not belts, routers or poles,
    /// which are many and show as terrain) whose column lies in `lo..=hi` (x, z).
    pub fn map_machines(&self, lo: (i32, i32), hi: (i32, i32), f: &mut impl FnMut(BlockId, IVec3)) {
        let mut each = |block: BlockId, p: IVec3| {
            if (lo.0..=hi.0).contains(&p.x) && (lo.1..=hi.1).contains(&p.z) {
                f(block, p);
            }
        };
        self.miners.iter().for_each(|m| each(MINER, m.pos()));
        self.storages.iter().for_each(|m| each(STORAGE, m.pos()));
        self.processors.iter().for_each(|m| each(m.spec.block, m.pos()));
        self.generators.iter().for_each(|m| each(GENERATOR, m.pos()));
        self.labs.iter().for_each(|m| each(LAB, m.pos()));
        self.quarries.iter().for_each(|m| each(QUARRY, m.pos()));
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

#[cfg(test)]
mod tests;

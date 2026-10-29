//! Machine models as box instances: the instance format, which dropped items (`entities.rs`) share,
//! and `write_instances`, which asks every machine near the camera for its model (`Machine::model`,
//! in each kind's file); `map_machines` for the minimap's marks. Presentation only: nothing here
//! changes factory state.

use std::f32::consts::FRAC_PI_2;

use crate::math::{IVec3, Vec3};

use crate::block::{tex, BlockId, GENERATOR, LAB, QUARRY, STORAGE};
use crate::world::DAYLIGHT;

use super::{Factory, Machine};

/// Floats per box instance: centre xyz (camera-relative), yaw, size xyz, uv scroll,
/// texture layers top/side/bottom, then the mode and light: the uv mode bit (0 = whole texture per face, 1 =
/// world-scaled), the [`ROUND`] bit, plus four times the light byte (sky in the low nibble, block light in the high
/// one, as `light.rs` writes it), then pitch, roll, top-width taper and the glow flag. `push_box` starts
/// every box at [`DAYLIGHT`]; `light_boxes` sets the real light and the glow. Ordinary boxes have no
/// tilt or taper. A glow flag of 2 is a beacon (`mark_beacon`): it glows and the fog doesn't hide it; 3 or more is a beacon
/// of one colour (`mark_beacon_tinted`).
pub const INSTANCE_FLOATS: usize = 16;
/// The mode bit of a round body (`push_round`): the host draws it as a cylinder, not a box (`render/boxes.ts`).
pub const ROUND: u32 = 2;
const MODE_LIGHT: usize = 11;
const PITCH: usize = 12;
const TAPER: usize = 14;
const GLOW: usize = 15;
const BEACON: f32 = 2.0;
/// A tinted beacon's flag is this plus its 12-bit colour (`mark_beacon_tinted`).
const TINTED_BEACON: f32 = 3.0;
/// Texture layers that give light rather than take it (status lights, lamps, laser beams, the jetpack's
/// flame): the shader draws a box with one of them at full brightness, day or night.
const GLOWING: [u16; 6] = [tex::LAMP_GREEN, tex::LAMP_YELLOW, tex::LAMP_RED, tex::LAMP_BLUE, tex::LAMP, tex::BEAM];

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
        packed(u32::from(world_uv), DAYLIGHT),
        0.0,
        0.0,
        1.0,
        0.0,
    ]);
}

/// The way a round body's axis runs, in its own frame before `yaw` turns it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Axis {
    Up,
    X,
    Z,
}

/// What a model part is drawn as: a box, a box leaned by a pitch about its own x after its yaw turns it (its up goes
/// to (-sin yaw sin pitch, cos pitch, cos yaw sin pitch): a tower's leg), or a round body along `axis` whose far end
/// (+y, +x or +z) is `taper` times as wide as its near end (1 a cylinder, under 1 a cone's frustum).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Form {
    Box,
    Lean(f32),
    Round(Axis, f32),
}

/// A plain cylinder standing upright.
pub const UPRIGHT: Form = Form::Round(Axis::Up, 1.0);

/// Pushes a round body (see [`Form`]) filling the box of `size` (x, y, z before `yaw`, as `push_box`), its axis along
/// `axis`. The host draws it from a cylinder mesh (`render/boxes.ts`); the engine sees it as that box (hit boxes, light).
#[allow(clippy::too_many_arguments)]
#[inline(never)]
pub fn push_round(
    out: &mut Vec<f32>,
    center: Vec3,
    yaw: f32,
    axis: Axis,
    size: [f32; 3],
    taper: f32,
    tex: [u16; 3],
    world_uv: bool,
) {
    push_box(out, center, yaw, size, 0.0, tex, world_uv);
    make_round(out, axis, taper);
}

/// Makes the box just pushed onto `out` (perhaps already tilted by a pitch) the round body filling it, its axis along
/// `axis` in the box's own frame (the held items: `avatars/model.rs`).
pub fn make_round(out: &mut [f32], axis: Axis, taper: f32) {
    let rec = out.len() - INSTANCE_FLOATS;
    let r = &mut out[rec..];
    // A cylinder's own axis is y; a quarter more pitch lays it along z, and a quarter turn less of yaw then along x
    // (where a pitch only spins it about its axis). Its own size is across, along and across; the pitch takes its z
    // to -y.
    let [x, y, z] = [r[4], r[5], r[6]];
    match axis {
        Axis::Up => {}
        Axis::Z => {
            r[PITCH] += FRAC_PI_2;
            r[4..7].copy_from_slice(&[x, z, y]);
        }
        Axis::X => {
            r[3] -= FRAC_PI_2;
            r[PITCH] = FRAC_PI_2;
            r[4..7].copy_from_slice(&[z, x, y]);
        }
    }
    r[MODE_LIGHT] = packed(r[MODE_LIGHT] as u32 & 1 | ROUND, (r[MODE_LIGHT] as u32 >> 2) as u8);
    r[TAPER] = taper;
}

/// Pushes a part of `form` (a box or [`push_round`]). Always inlined: processors push every box through it, and a call per
/// box made their models a fifth slower to write (`bench_machine_marks`).
#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub fn push_shape(
    out: &mut Vec<f32>,
    form: Form,
    center: Vec3,
    yaw: f32,
    size: [f32; 3],
    tex: [u16; 3],
    world_uv: bool,
) {
    match form {
        Form::Box => push_box(out, center, yaw, size, 0.0, tex, world_uv),
        Form::Lean(pitch) => {
            push_box(out, center, yaw, size, 0.0, tex, world_uv);
            let rec = out.len() - INSTANCE_FLOATS;
            out[rec + PITCH] = pitch;
        }
        Form::Round(axis, taper) => push_round(out, center, yaw, axis, size, taper, tex, world_uv),
    }
}

/// Copies the round bodies among `boxes` into `round` (cleared first); the host draws `boxes` as boxes, skipping
/// these, and `round` with its cylinder mesh.
pub fn split_round(boxes: &[f32], round: &mut Vec<f32>) {
    round.clear();
    for rec in boxes.chunks_exact(INSTANCE_FLOATS) {
        if rec[MODE_LIGHT] as u32 & ROUND != 0 {
            round.extend_from_slice(rec);
        }
    }
}

/// A pair of boxes among `boxes` that fake one round body (the standing check that models use [`push_round`]: the
/// tests of `factory/tests/fake_round.rs`, `item_models` and `trains`): two untilted boxes with one centre that are
/// the same box turned part of a quarter (an octagon, a star of strips) or the same box with two sides swapped (a
/// cross). Thin bars crossing (a plus, an X) are not round and pass.
#[cfg(test)]
pub fn fake_round(boxes: &[f32]) -> Option<(usize, usize)> {
    // Each untilted box as its yaw within a quarter turn and its size, x and z swapped for an odd quarter.
    let norm = |r: &[f32]| {
        let q = ((r[3] + 0.005) / FRAC_PI_2).floor();
        let s = if q.rem_euclid(2.0) == 1.0 { [r[6], r[5], r[4]] } else { [r[4], r[5], r[6]] };
        (r[3] - q * FRAC_PI_2, s)
    };
    let plain = |r: &&[f32]| r[MODE_LIGHT] as u32 & ROUND == 0 && r[PITCH].abs() < 1e-3 && r[13].abs() < 1e-3;
    let recs: Vec<(usize, &[f32])> =
        boxes.chunks_exact(INSTANCE_FLOATS).enumerate().filter(|(_, r)| plain(r)).collect();
    // Within a tenth: fakes stagger their copies a little so their faces don't fight.
    let near = |a: f32, b: f32| (a - b).abs() <= 0.1 * a.max(b) + 0.005;
    let squat = |a: f32, b: f32, share: f32| a.min(b) >= share * a.max(b);
    for (n, &(i, a)) in recs.iter().enumerate() {
        for &(j, b) in &recs[n + 1..] {
            if !(0..3).all(|k| (a[k] - b[k]).abs() <= 0.005) {
                continue;
            }
            let ((ya, sa), (yb, sb)) = (norm(a), norm(b));
            let turn = (ya - yb).abs();
            let same = near(sa[0], sb[0]) && near(sa[2], sb[2]) || near(sa[0], sb[2]) && near(sa[2], sb[0]);
            if same && near(sa[1], sb[1]) && turn > 0.05 && turn < FRAC_PI_2 - 0.05 && squat(sa[0], sa[2], 0.2) {
                return Some((i, j));
            }
            // Crossed about one axis: that axis's side shared, each box the wider on one of the other two, neither
            // a thin bar.
            let crossed = |k: usize| {
                let (u, v) = ((k + 1) % 3, (k + 2) % 3);
                let wider = |p: &[f32; 3], q: &[f32; 3]| p[u] > q[u] + 0.01 && q[v] > p[v] + 0.01;
                near(sa[k], sb[k])
                    && (wider(&sa, &sb) || wider(&sb, &sa))
                    && squat(sa[u], sa[v], 0.5)
                    && squat(sb[u], sb[v], 0.5)
            };
            if turn < 0.01 && (0..3).any(crossed) {
                return Some((i, j));
            }
        }
    }
    None
}
/// Makes the box just pushed onto `out` a beacon: it lights itself and shows through the fog, to mark something far
/// off (the long-distance pole a held one is measured from: `power_tools/long_pole.rs`).
pub fn mark_beacon(out: &mut [f32]) {
    if let Some(last) = out.last_mut() {
        *last = BEACON;
    }
}

/// A beacon painted one flat colour (the telescope's pins: `telescope/beacons.rs`). `rgb` is 0xRRGGBB, scaled up so its
/// brightest channel is full (a light, not a pigment) and kept at 4 bits a channel in the glow flag, which is
/// [`TINTED_BEACON`] plus the colour; the box shader decodes it (`boxVert` in `render/shaders.ts`).
pub fn mark_beacon_tinted(out: &mut [f32], rgb: u32) {
    let [r, g, b] = [rgb >> 16 & 255, rgb >> 8 & 255, rgb & 255];
    let top = r.max(g).max(b);
    let level = |c: u32| (c * 15 + top / 2).checked_div(top).unwrap_or(15);
    if let Some(last) = out.last_mut() {
        *last = TINTED_BEACON + (level(r) | level(g) << 4 | level(b) << 8) as f32;
    }
}

/// The instance's mode-and-light float: the mode bits (uv mode, [`ROUND`]) plus four times the light byte.
fn packed(mode: u32, light: u8) -> f32 {
    mode as f32 + 4.0 * f32::from(light)
}

/// Lights every box in `boxes` (records of [`INSTANCE_FLOATS`], centres relative to `eye`) with the light
/// of the cell its centre is in, from `light` (`World::light_at`), and flags the [`GLOWING`] ones. A cell with no light at all (a wire
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
        rec[MODE_LIGHT] = packed(rec[MODE_LIGHT] as u32 & 3, lit);
        if rec[GLOW] < BEACON {
            rec[GLOW] = f32::from(u8::from(rec[8..11].iter().any(|&l| GLOWING.contains(&(l as u16)))));
        }
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
        self.write_train_models(out, eye, time, range);
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
        self.miners.iter().for_each(|m| each(m.block(), m.pos()));
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

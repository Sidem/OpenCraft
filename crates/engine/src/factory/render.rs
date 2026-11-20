//! Machine models as box instances: the instance format, which dropped items (`entities.rs`) share,
//! and `write_instances`, which asks every machine near the camera for its model (`Machine::model`,
//! in each kind's file); `map_machines` for the minimap's marks. Presentation only: nothing here
//! changes factory state.

use std::f32::consts::{FRAC_PI_2, PI};

use crate::math::{IVec3, Vec3};

use crate::block::{tex, BlockId, GENERATOR, LAB, QUARRY, STORAGE};
use crate::deposits::DepositKey;
use crate::world::DAYLIGHT;

use super::miner::MinerStatus;
use super::{Factory, Machine};

/// Floats per box instance: centre xyz, yaw, size xyz, uv scroll, texture layers top/side/bottom, mode+light (bit 0
/// world uv, bit 1 `ROUND`, bits 2..=9 light byte, bits 10..=12 belt scroll tier+1), pitch, roll, taper, glow (2 beacon,
/// >=3 tinted beacon). `push_box` starts at [`DAYLIGHT`]; `light_boxes` sets real light and glow.
pub const INSTANCE_FLOATS: usize = 16;
/// The mode bit of a round body (`push_round`): the host draws it as a cylinder, not a box (`render/boxes.ts`).
pub const ROUND: u32 = 2;
/// Bit shift in `MODE_LIGHT` for a belt top's shader-driven scroll speed (`tier + 1` in bits 10..=12; 0 = static scroll).
const BELT_SCROLL_SHIFT: u32 = 10;
const LIGHT_MASK: u32 = 0xff << 2;
const MODE_LIGHT: usize = 11;
pub(super) const PITCH: usize = 12;
const TAPER: usize = 14;
const GLOW: usize = 15;
const BEACON: f32 = 2.0;
const TINTED_BEACON: f32 = 3.0;
/// Texture layers that give light rather than take it (status lights, lamps, laser beams, jetpack/rocket plumes).
#[inline]
fn is_glowing(l: u16) -> bool {
    matches!(
        l,
        tex::LAMP_GREEN
            | tex::LAMP_YELLOW
            | tex::LAMP_RED
            | tex::LAMP_BLUE
            | tex::LAMP
            | tex::BEAM
            | tex::PLUME_CORE
            | tex::PLUME_FLAME
            | tex::PLUME_TAIL
            | tex::PLUME_DIAMOND
    )
}

/// Pushes one box instance (see [`INSTANCE_FLOATS`]).
#[allow(clippy::too_many_arguments)]
pub fn push_box(out: &mut Vec<f32>, c: Vec3, yaw: f32, s: [f32; 3], scroll: f32, t: [u16; 3], world_uv: bool) {
    let rec = [
        c.x as f32,
        c.y as f32,
        c.z as f32,
        yaw,
        s[0],
        s[1],
        s[2],
        scroll,
        t[0] as f32,
        t[1] as f32,
        t[2] as f32,
        packed(u32::from(world_uv), DAYLIGHT),
        0.0,
        0.0,
        1.0,
        0.0,
    ];
    out.extend_from_slice(&rec);
}

/// Round body axis in its own frame before `yaw`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Axis {
    Up,
    X,
    Z,
}

/// Model part form: box, pitched box (`Lean`), or round body along `Axis` with top-width `taper`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Form {
    Box,
    Lean(f32),
    Round(Axis, f32),
}

pub const UPRIGHT: Form = Form::Round(Axis::Up, 1.0);

/// Pushes a round body filling `size` along `axis` (rendered as a cylinder in `render/boxes.ts`).
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

/// Pushes a tapered cone frustum `len` long, `top` wide at upper end and `bottom` at lower, leaned by `lean`.
#[allow(clippy::too_many_arguments)]
pub fn push_cone(
    out: &mut Vec<f32>,
    center: Vec3,
    yaw: f32,
    lean: f32,
    len: f32,
    top: f32,
    bottom: f32,
    tex: [u16; 3],
) {
    let (wide, taper, pitch) = if top <= bottom { (bottom, top, lean) } else { (top, bottom, lean + PI) };
    push_round(out, center, yaw, Axis::Up, [wide, len, wide], taper / wide.max(1e-6), tex, false);
    let rec = out.len() - INSTANCE_FLOATS;
    out[rec + PITCH] = pitch;
}

/// How `push_framed` draws its box: plain, or a round body along its y whose far end is `taper` times as wide; `slide`
/// gives the round body world-scale texels slid that far along its axis (a stem fed through a harvester head).
#[derive(Clone, Copy)]
pub struct Framed {
    pub taper: Option<f32>,
    pub slide: Option<f32>,
}

/// Pushes a box whose own x, y and z lie along the unit vectors `axes` (right-handed), `size` along each, drawn as
/// `look`; yaw · roll · pitch solved as the shader composes them (`hitbox.rs`). For `forester/crane.rs`.
pub fn push_framed(
    out: &mut Vec<f32>,
    center: Vec3,
    [x, y, z]: [Vec3; 3],
    size: [f32; 3],
    tex: [u16; 3],
    look: Framed,
) {
    let yaw = x.z.atan2(x.x) as f32;
    let roll = x.y.clamp(-1.0, 1.0).asin() as f32;
    let pitch = (-z.y).atan2(y.y) as f32;
    push_box(out, center, yaw, size, look.slide.unwrap_or(0.0), tex, look.slide.is_some());
    let rec = out.len() - INSTANCE_FLOATS;
    (out[rec + PITCH], out[rec + PITCH + 1]) = (pitch, roll);
    if let Some(taper) = look.taper {
        make_round(out, Axis::Up, taper);
    }
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
    r[MODE_LIGHT] = (r[MODE_LIGHT] as u32 | ROUND) as f32;
    r[TAPER] = taper;
}

/// Tags the box just pushed onto `out` as a scrolling belt top of `tier`: the box shader adds `u_beltScroll[tier]`
/// to its uv scroll so belt frames stay static in per-chunk GPU buffers (`static_boxes`).
pub fn mark_belt_scroll(out: &mut [f32], tier: u8) {
    let rec = out.len() - INSTANCE_FLOATS;
    out[rec + MODE_LIGHT] = (out[rec + MODE_LIGHT] as u32 | ((u32::from(tier) + 1) << BELT_SCROLL_SHIFT)) as f32;
}

/// Whether an instance record is a round body (`ROUND` bit set).
#[inline]
pub fn is_round(rec: &[f32]) -> bool {
    rec[MODE_LIGHT] as u32 & ROUND != 0
}

/// Inverse of `push_shape` for icons drawn from a placed model (`item_models/placed.rs`).
pub fn shape_of(r: &[f32]) -> (Form, [f32; 3], f32, [f32; 3], [u16; 3]) {
    let centre = [r[0], r[1], r[2]];
    let tex = [r[8] as u16, r[9] as u16, r[10] as u16];
    let (yaw, pitch, size) = (r[3], r[PITCH], [r[4], r[5], r[6]]);
    if r[MODE_LIGHT] as u32 & ROUND == 0 {
        let form = if pitch.abs() < 1e-4 { Form::Box } else { Form::Lean(pitch) };
        return (form, centre, yaw, size, tex);
    }
    // `make_round` laid a body along z by a quarter of pitch, its size then across, along, across.
    let taper = r[TAPER];
    if (pitch - FRAC_PI_2).abs() < 1e-3 {
        (Form::Round(Axis::Z, taper), centre, yaw, [size[0], size[2], size[1]], tex)
    } else if (pitch + FRAC_PI_2).abs() < 1e-3 {
        (Form::Round(Axis::Z, taper), centre, yaw + PI, [size[0], size[2], size[1]], tex)
    } else {
        (Form::Round(Axis::Up, taper), centre, yaw, size, tex)
    }
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

/// Copies records within `radius` of the camera into `out` (at most `max`) and round bodies into `round`.
pub fn copy_near(boxes: &[f32], radius: f32, max: usize, out: &mut Vec<f32>, round: &mut Vec<f32>) -> bool {
    out.clear();
    round.clear();
    for rec in boxes.chunks_exact(INSTANCE_FLOATS) {
        if rec[0] * rec[0] + rec[1] * rec[1] + rec[2] * rec[2] > radius * radius {
            continue;
        }
        if out.len() >= max * INSTANCE_FLOATS {
            return true;
        }
        out.extend_from_slice(rec);
        if rec[MODE_LIGHT] as u32 & ROUND != 0 {
            round.extend_from_slice(rec);
        }
    }
    false
}

#[cfg(test)]
mod fake_round;
#[cfg(test)]
pub use fake_round::fake_round;

/// Marks the last box as a self-lit beacon shown through fog (`power_tools/long_pole.rs`).
pub fn mark_beacon(out: &mut [f32]) {
    if let Some(last) = out.last_mut() {
        *last = BEACON;
    }
}

/// Marks the last box as a flat-colour tinted beacon (`TINTED_BEACON` + 12-bit RGB, `telescope/beacons.rs`).
pub fn mark_beacon_tinted(out: &mut [f32], rgb: u32) {
    let [r, g, b] = [rgb >> 16 & 255, rgb >> 8 & 255, rgb & 255];
    let top = r.max(g).max(b);
    let level = |c: u32| (c * 15 + top / 2).checked_div(top).unwrap_or(15);
    if let Some(last) = out.last_mut() {
        *last = TINTED_BEACON + (level(r) | level(g) << 4 | level(b) << 8) as f32;
    }
}

/// Marks the last box pure black regardless of light (`process/port_mouth.rs`).
pub fn mark_dark(out: &mut [f32]) {
    if let Some(last) = out.last_mut() {
        *last = TINTED_BEACON;
    }
}

/// The instance's mode-and-light float: the mode bits (uv mode, [`ROUND`]) plus four times the light byte.
fn packed(mode: u32, light: u8) -> f32 {
    mode as f32 + 4.0 * f32::from(light)
}

/// Applies cell light `lit` and the glowing flag to one instance record `rec`.
#[inline]
fn apply_light(rec: &mut [f32], lit: u8) {
    rec[MODE_LIGHT] = ((rec[MODE_LIGHT] as u32 & !LIGHT_MASK) | (u32::from(lit) << 2)) as f32;
    if rec[GLOW] < BEACON {
        let (t0, t1, t2) = (rec[8] as u16, rec[9] as u16, rec[10] as u16);
        let g = is_glowing(t0) || (t1 != t0 && is_glowing(t1)) || (t2 != t0 && t2 != t1 && is_glowing(t2));
        rec[GLOW] = f32::from(u8::from(g));
    }
}

/// Lights every box in `boxes` (records of [`INSTANCE_FLOATS`], centres relative to `eye`) with the light
/// of the cell its centre is in, from `light` (`World::light_at`), and flags glowing layers. A cell with no light at
/// all (inside solid rock) borrows the best of its neighbours, so only a box really in the dark is dark.
#[inline(never)]
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
        apply_light(rec, lit);
    }
}

/// The brighter of two light bytes, sky and block light each.
fn brighter(a: u8, b: u8) -> u8 {
    (a & 15).max(b & 15) | (a >> 4).max(b >> 4) << 4
}

impl Factory {
    /// Writes box instances for every machine within `range` of the camera. Power wires are drawn by the host's view
    /// (`wire_props.rs`, from `for_each_wire`), which lifts them over the ground.
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
        self.write_cables(out, eye, range);
    }

    /// Calls `f` with the block, tier (0 for a machine without tiers) and position of every machine a map marks
    /// (not belts, routers or poles, which are many and show as terrain) whose column lies in `lo..=hi` (x, z).
    pub fn map_machines(&self, lo: (i32, i32), hi: (i32, i32), f: &mut impl FnMut(BlockId, u8, IVec3)) {
        let mut each = |block: BlockId, tier: u8, p: IVec3| {
            if (lo.0..=hi.0).contains(&p.x) && (lo.1..=hi.1).contains(&p.z) {
                f(block, tier, p);
            }
        };
        self.miners.iter().for_each(|m| each(m.block(), if m.is_coal() { 0 } else { m.tier }, m.pos()));
        self.storages.iter().for_each(|m| each(STORAGE, m.tier, m.pos()));
        self.processors.iter().for_each(|m| each(m.spec.block, m.tier, m.pos()));
        self.generators.iter().for_each(|m| each(GENERATOR, m.tier, m.pos()));
        self.labs.iter().for_each(|m| each(LAB, m.tier, m.pos()));
        self.quarries.iter().for_each(|m| each(QUARRY, m.tier, m.pos()));
    }

    /// Every miner standing on a deposit: where, which deposit, and whether it has worked it out (for the maps).
    pub fn miner_deposits(&self) -> impl Iterator<Item = (IVec3, DepositKey, bool)> + '_ {
        self.miners.iter().filter_map(|m| m.deposit.map(|k| (m.pos(), k, m.status == MinerStatus::Exhausted)))
    }
}

/// Models of the machines in `list` whose cell centre is within `range` of `eye`.
pub(super) fn models<T: Machine>(list: &[T], out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
    for m in list {
        let rel = m.pos().as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
        if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= range * range {
            m.model(out, rel, time);
        }
    }
}

#[cfg(test)]
mod tests;

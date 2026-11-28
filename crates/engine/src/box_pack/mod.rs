//! The box instance records the host uploads, packed from the engine's working records (`factory::INSTANCE_FLOATS`
//! f32s, 64 bytes) into [`PACKED_WORDS`] u32s (32 bytes): every frame's dynamic boxes go to the GPU, so their size is
//! upload time. The engine builds, lights and tests the wide records; only what leaves for the host is packed, at the
//! `api` getters (and `static_boxes`, once a chunk changes). Presentation only.
//!
//! Layout (the host's `render/box-glsl.ts` decodes the same; change both together):
//! - words 0..=2: camera- or region-relative centre xyz, the f32 bits unchanged.
//! - word 3: size x | size y << 16, half floats. Word 4: size z (half float) | yaw << 16 (a turn in 16 bits).
//! - word 5: pitch (13 bits of a turn) | roll << 13 (12 bits) | belt scroll tier+1 << 25 (3) | glow kind << 28 (2: none,
//!   glowing, beacon, tinted beacon) | world uv << 30 | `ROUND` << 31.
//! - word 6: texture layers top | side << 10 | bottom << 20, 10 bits each (`tex::COUNT` stays below 1024).
//! - word 7: light byte, or a tinted beacon's 12-bit colour | taper << 12 (12 bits, 2048 is 1.0) | uv scroll's
//!   fraction << 24 (8 bits; textures repeat, so the whole part changes nothing).
//!
//! Invariant: packing loses only what no pixel shows: sizes to half-float precision (1 part in 2048), angles to the
//! steps above (yaw 0.006°, pitch 0.04°, roll 0.09°; quarter turns exact), taper to 1/2048, scroll to 1/256 of a texture.

use std::f64::consts::TAU;

use crate::factory::{INSTANCE_FLOATS, ROUND};
use crate::Game;

/// u32 words per packed box record (32 bytes).
pub const PACKED_WORDS: usize = 8;

const _: () = assert!(crate::block::tex::COUNT <= 1024, "word 6 holds a texture layer in 10 bits: widen it");

/// This frame's packed box lists for the host, each split into boxes and round bodies (`api/render.rs`,
/// `api/xray.rs`), and the static chunk last popped (`api/static_boxes.rs`).
#[derive(Default)]
pub struct Packed {
    pub boxes: Vec<u32>,
    pub round: Vec<u32>,
    pub ghost: Vec<u32>,
    pub ghost_round: Vec<u32>,
    pub marks: Vec<u32>,
    pub marks_round: Vec<u32>,
    pub xray: Vec<u32>,
    pub xray_round: Vec<u32>,
    pub static_cubes: Vec<u32>,
    pub static_round: Vec<u32>,
}

impl Game {
    /// Packs this frame's box lists for the host (the end of `Game::update`).
    pub(crate) fn pack_boxes(&mut self) {
        let p = &mut self.packed;
        pack_split(&self.instances, &mut p.boxes, &mut p.round);
        pack_split(&self.ghosts.boxes, &mut p.ghost, &mut p.ghost_round);
        pack_split(&self.ghosts.marks, &mut p.marks, &mut p.marks_round);
        pack_split(&self.xray.boxes, &mut p.xray, &mut p.xray_round);
    }
}

/// Packs `records` into `boxes` and its round bodies (`ROUND`) into `round`, each cleared first: the host draws
/// each list over its own mesh, so neither carries the other's records.
pub fn pack_split(records: &[f32], boxes: &mut Vec<u32>, round: &mut Vec<u32>) {
    boxes.clear();
    round.clear();
    for rec in records.chunks_exact(INSTANCE_FLOATS) {
        let r: &[f32; INSTANCE_FLOATS] = rec.try_into().expect("chunks_exact");
        let to = if whole(r[11]) & ROUND != 0 { &mut *round } else { &mut *boxes };
        to.extend_from_slice(&pack_one(r));
    }
}

/// Packs every record of `records` (wide, [`INSTANCE_FLOATS`] each) into `out`, which is cleared first.
pub fn pack(records: &[f32], out: &mut Vec<u32>) {
    out.clear();
    out.reserve(records.len() / INSTANCE_FLOATS * PACKED_WORDS);
    for rec in records.chunks_exact(INSTANCE_FLOATS) {
        out.extend_from_slice(&pack_one(rec.try_into().expect("chunks_exact")));
    }
}

/// One wide record packed (see the module header for the layout). Branch-light on purpose: it runs for every box
/// every frame, and in wasm a float to int cast is a checked sequence, so whole numbers are read by the add trick
/// (`whole`, `steps`) instead.
#[inline]
pub fn pack_one(r: &[f32; INSTANCE_FLOATS]) -> [u32; PACKED_WORDS] {
    let mode = whole(r[11]);
    let (light, belt) = ((mode >> 2) & 0xff, (mode >> 10) & 7);
    // Glow 0, 1, 2 or 3 + a 12-bit tint: the kind is the value up to 3, the payload the tint or else the light.
    let glow = whole(r[15]);
    let kind = glow.min(3);
    let payload = if glow >= 3 { (glow - 3) & 0xfff } else { light };
    debug_assert!(r[8] < 1024.0 && r[9] < 1024.0 && r[10] < 1024.0, "texture layer past 10 bits");
    let tex = |i: usize| whole(r[i]) & 0x3ff;
    let taper = whole(r[14].max(0.0) * 2048.0).min(4095);
    let scroll = steps(r[7], 256.0) & 0xff;
    [
        r[0].to_bits(),
        r[1].to_bits(),
        r[2].to_bits(),
        u32::from(half(r[4])) | u32::from(half(r[5])) << 16,
        u32::from(half(r[6])) | turn(r[3], 16) << 16,
        turn(r[12], 13) | turn(r[13], 12) << 13 | belt << 25 | kind << 28 | (mode & 3) << 30,
        tex(8) | tex(9) << 10 | tex(10) << 20,
        payload | taper << 12 | scroll << 24,
    ]
}

/// Angle `a` (radians) as a share of a whole turn in `bits` bits, wrapped into one turn.
#[inline]
fn turn(a: f32, bits: u32) -> u32 {
    steps(a, f64::from(1u32 << bits) / TAU) & ((1 << bits) - 1)
}

/// `x * per` rounded to the nearest whole step (ties to even), as the low 32 bits of a two's-complement count (the
/// caller masks them). Adding 1.5 × 2^52 leaves the rounded whole number in the f64's low mantissa bits, so no cast
/// or `floor` runs; exact while |x × per| < 2^51, far past any angle a box carries.
#[inline]
fn steps(x: f32, per: f64) -> u32 {
    const MAGIC: f64 = 6_755_399_441_055_744.0; // 1.5 × 2^52
    (f64::from(x) * per + MAGIC).to_bits() as u32
}

/// A small non-negative float (below 2^22) rounded to the nearest whole number, by the same add trick in f32.
#[inline]
fn whole(x: f32) -> u32 {
    const MAGIC: f32 = 12_582_912.0; // 1.5 × 2^23
    (x + MAGIC).to_bits() & 0x3f_ffff
}

/// `x` as IEEE half-float bits, rounded to nearest (ties to even); past the largest half it saturates, below the
/// smallest it goes to zero. GLSL's `unpackHalf2x16` reads it back.
#[inline]
pub fn half(x: f32) -> u16 {
    let b = x.to_bits();
    // The usual case, a normal half (2^-14 up to below 65,504): drop 13 mantissa bits, ties to even, and rebias the
    // exponent; a carry out of the mantissa rolls into the exponent as it should.
    let abs = b & 0x7fff_ffff;
    if (0x3880_0000..0x477f_f000).contains(&abs) {
        let rounded = (abs + 0x0fff + ((abs >> 13) & 1)) >> 13;
        return ((b >> 16) & 0x8000) as u16 | (rounded - ((127 - 15) << 10)) as u16;
    }
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7f_ffff;
    if exp >= 31 {
        return sign | 0x7bff; // saturate (no box is 65,504 blocks long)
    }
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        // A subnormal half: the implicit bit joins the mantissa, shifted down.
        let m = man | 0x80_0000;
        let shift = (14 - exp) as u32;
        let half_m = m >> shift;
        let rest = m & ((1 << shift) - 1);
        let mid = 1 << (shift - 1);
        let up = rest > mid || (rest == mid && half_m & 1 == 1);
        return sign | (half_m + u32::from(up)) as u16;
    }
    let half_m = man >> 13;
    let rest = man & 0x1fff;
    let up = rest > 0x1000 || (rest == 0x1000 && half_m & 1 == 1);
    // A carry out of the mantissa rolls into the exponent, which is how rounding up to the next power of two works.
    let v = ((exp as u32) << 10 | half_m) + u32::from(up);
    if v >= 0x7c00 {
        return sign | 0x7bff;
    }
    sign | v as u16
}

#[cfg(test)]
mod tests;

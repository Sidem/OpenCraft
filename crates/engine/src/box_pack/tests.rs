use std::f32::consts::{FRAC_PI_2, PI, TAU};

use super::*;
use crate::factory::{make_round, mark_beacon, mark_beacon_tinted, push_box, Axis};
use crate::math::Vec3;

/// The host's decode (`render/box-glsl.ts`), field by field, for checking a packed record against its wide one.
struct Decoded {
    pos: [f32; 3],
    size: [f32; 3],
    yaw: f32,
    pitch: f32,
    roll: f32,
    belt: u32,
    kind: u32,
    uv: u32,
    round: u32,
    tex: [u32; 3],
    payload: u32,
    taper: f32,
    scroll: f32,
}

fn unhalf(h: u32) -> f32 {
    let (sign, exp, man) = (h >> 15 & 1, (h >> 10 & 0x1f) as i32, (h & 0x3ff) as f32);
    let v = if exp == 0 { man / 1024.0 * 2f32.powi(-14) } else { (1.0 + man / 1024.0) * 2f32.powi(exp - 15) };
    if sign == 1 {
        -v
    } else {
        v
    }
}

/// The first record of out, as pack_one takes it.
fn rec(out: &[f32]) -> &[f32; INSTANCE_FLOATS] {
    out[..INSTANCE_FLOATS].try_into().unwrap()
}

fn decode(w: [u32; PACKED_WORDS]) -> Decoded {
    let turn = |v: u32, bits: u32| v as f32 / (1u32 << bits) as f32 * TAU;
    Decoded {
        pos: [f32::from_bits(w[0]), f32::from_bits(w[1]), f32::from_bits(w[2])],
        size: [unhalf(w[3] & 0xffff), unhalf(w[3] >> 16), unhalf(w[4] & 0xffff)],
        yaw: turn(w[4] >> 16, 16),
        pitch: turn(w[5] & 0x1fff, 13),
        roll: turn(w[5] >> 13 & 0xfff, 12),
        belt: w[5] >> 25 & 7,
        kind: w[5] >> 28 & 3,
        uv: w[5] >> 30 & 1,
        round: w[5] >> 31,
        tex: [w[6] & 0x3ff, w[6] >> 10 & 0x3ff, w[6] >> 20 & 0x3ff],
        payload: w[7] & 0xfff,
        taper: (w[7] >> 12 & 0xfff) as f32 / 2048.0,
        scroll: (w[7] >> 24) as f32 / 256.0,
    }
}

/// The same angle, a whole number of turns apart, within `tol` radians.
fn same_angle(a: f32, b: f32, tol: f32) -> bool {
    let d = (a - b).rem_euclid(TAU);
    d.min(TAU - d) <= tol
}

#[test]
fn a_box_keeps_every_field_within_its_steps() {
    let mut out = Vec::new();
    push_box(&mut out, Vec3::new(-123.456, 7.25, 300.5), -2.7, [0.83, 0.07, 71.45], 3.3, [185, 17, 1023], true);
    out[11] = (out[11] as u32 | 3 << 10) as f32; // a Mk2 belt top (mark_belt_scroll)
    let d = decode(pack_one(rec(&out)));
    assert_eq!(d.pos, [out[0], out[1], out[2]], "the centre is exact");
    for (got, want) in d.size.iter().zip([0.83, 0.07, 71.45]) {
        assert!((got - want).abs() <= want / 2048.0, "size {got} for {want}");
    }
    assert!(same_angle(d.yaw, -2.7, TAU / 65536.0));
    assert_eq!((d.belt, d.kind, d.uv, d.round), (3, 0, 1, 0));
    assert_eq!(d.tex, [185, 17, 1023]);
    assert_eq!(d.payload, u32::from(crate::world::DAYLIGHT), "the light byte");
    assert_eq!(d.taper, 1.0);
    assert!((d.scroll - 0.3).abs() <= 1.0 / 512.0, "only the fraction of the scroll: {}", d.scroll);
}

#[test]
fn quarter_turns_and_round_bodies_pack_exactly() {
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, FRAC_PI_2, [1.0, 2.0, 0.5], 0.0, [1, 2, 3], false);
    make_round(&mut out, Axis::Z, 0.625);
    let d = decode(pack_one(rec(&out)));
    assert_eq!((d.yaw, d.pitch, d.roll), (FRAC_PI_2, FRAC_PI_2, 0.0));
    assert_eq!((d.round, d.taper), (1, 0.625));
    assert_eq!(d.size, [1.0, 0.5, 2.0], "make_round swapped the sizes; halves hold them exactly");
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, -PI, [1.0; 3], 0.0, [0; 3], false);
    assert_eq!(decode(pack_one(rec(&out))).yaw, PI, "a negative angle wraps into the turn");
}

#[test]
fn beacons_keep_their_kind_and_colour() {
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, 0.0, [1.0; 3], 0.0, [0; 3], false);
    mark_beacon(&mut out);
    assert_eq!(decode(pack_one(rec(&out))).kind, 2);
    mark_beacon_tinted(&mut out, 0xff8000);
    let d = decode(pack_one(rec(&out)));
    let tint = out[15] as u32 - 3;
    assert_eq!((d.kind, d.payload), (3, tint));
    out[15] = 1.0;
    assert_eq!(decode(pack_one(rec(&out))).kind, 1, "glowing layers");
}

#[test]
fn halves_round_to_nearest_and_saturate() {
    for x in [0.0f32, 1.0, 0.5, 2.0, 65504.0, 1e-5, 0.1, 3.3, 1234.5678, 6.1e-5] {
        let back = unhalf(u32::from(half(x)));
        assert!((back - x).abs() <= x.abs() / 2048.0 + 6e-8, "{x} came back {back}");
    }
    assert_eq!(unhalf(u32::from(half(1e9))), 65504.0);
    assert_eq!(half(-2.0), 0xc000);
}

#[test]
fn pack_writes_eight_words_a_record() {
    let mut out = Vec::new();
    for i in 0..3 {
        push_box(&mut out, Vec3::new(i as f64, 0.0, 0.0), 0.0, [1.0; 3], 0.0, [0; 3], false);
    }
    let mut packed = vec![7; 5];
    pack(&out, &mut packed);
    assert_eq!(packed.len(), 3 * PACKED_WORDS);
    assert_eq!(f32::from_bits(packed[2 * PACKED_WORDS]), 2.0);
}

#[test]
fn a_runaway_angle_still_lands_in_its_turn() {
    // A spinning part's yaw grew past five million radians in a real save; it packs to the same place in the turn.
    let yaw = 5_227_504.0f32;
    let mut out = Vec::new();
    push_box(&mut out, Vec3::ZERO, yaw, [1.0; 3], 0.0, [0; 3], false);
    let want = f64::from(yaw).rem_euclid(std::f64::consts::TAU) as f32;
    assert!(same_angle(decode(pack_one(rec(&out))).yaw, want, TAU / 65536.0));
}

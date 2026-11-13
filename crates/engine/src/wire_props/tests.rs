//! Tests of the props under power wires: none over open ground, standing on the hill they clear, and every wire over
//! random rolling ground clear of it once propped.

use super::*;
use crate::math::Rng;

/// A wire end on a pole standing on ground of height `h` at column (x, z).
fn pole_top(x: i32, h: i32, z: i32) -> Vec3 {
    Vec3::new(f64::from(x) + 0.5, f64::from(h) + POLE_WIRE_Y, f64::from(z) + 0.5)
}

/// Whether the wire `a`..`b` touches the ground anywhere but next to its ends (as `worst_dip` judges it).
fn touches(a: Vec3, b: Vec3, ground: &impl Fn(IVec3) -> bool) -> bool {
    worst_dip(a, b, ground).is_some()
}

#[test]
fn a_wire_over_flat_ground_needs_no_props() {
    let flat = |c: IVec3| c.y < 10;
    assert!(route(pole_top(0, 10, 0), pole_top(30, 10, 4), &flat).is_empty());
}

#[test]
fn a_hill_between_two_poles_gets_a_prop_standing_on_it() {
    // A ridge 6 high across x = 14..=17; the poles stand on flat ground either side.
    let hill = |c: IVec3| c.y < if (14..=17).contains(&c.x) { 16 } else { 10 };
    let (a, b) = (pole_top(0, 10, 0), pole_top(30, 10, 0));
    assert!(touches(a, b, &hill), "the bare wire runs through the ridge");
    let props = route(a, b, &hill);
    assert!(!props.is_empty());
    for top in &props {
        let foot = (*top - Vec3::new(0.0, POLE_WIRE_Y, 0.0)).floor();
        assert!(!hill(foot) && hill(foot - IVec3::new(0, 1, 0)), "stands on the surface: {top:?}");
    }
    let mut from = a;
    for &top in props.iter().chain([&b]) {
        assert!(!touches(from, top, &hill), "{from:?} to {top:?} clears the ground");
        from = top;
    }
}

#[test]
fn every_wire_over_random_rolling_ground_clears_it_once_propped() {
    let mut rng = Rng::new(7);
    let mut propped = 0;
    for _ in 0..300 {
        let (bumps, scale, tilt) = (rng.range(0.05, 0.4), rng.range(1.0, 9.0), rng.range(-0.4, 0.4));
        let phase = rng.range(0.0, 6.0);
        let height = move |x: i32, z: i32| {
            let (x, z) = (f64::from(x), f64::from(z));
            (20.0 + scale * ((x * bumps + phase).sin() + (z * bumps * 0.7).cos()) + tilt * x).floor() as i32
        };
        let ground = |c: IVec3| c.y < height(c.x, c.z);
        let (x1, z1) = (rng.below(30) as i32 + 3, rng.below(30) as i32 - 15);
        let (a, b) = (pole_top(0, height(0, 0), 0), pole_top(x1, height(x1, z1), z1));
        let props = route(a, b, &ground);
        propped += usize::from(!props.is_empty());
        let mut from = a;
        for &top in props.iter().chain([&b]) {
            assert!(!touches(from, top, &ground), "{a:?} to {b:?}: {from:?} to {top:?} still touches");
            from = top;
        }
    }
    assert!(propped > 30, "the cases exercise props: {propped} of 300");
}

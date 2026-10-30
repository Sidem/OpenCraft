use std::collections::HashSet;

use super::*;
use crate::block::{STONE, WATER};
use crate::factory::rail::curve::{dir_of, fit, yaw_of, Fit, MAX_SPAN};
use crate::math::{Rng, Vec3};
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

const E: u8 = 64;

/// A world of solid cells: everything listed is stone, the rest air.
fn world(solid: &HashSet<IVec3>) -> impl FnMut(IVec3) -> BlockId + '_ {
    move |p| if solid.contains(&p) { STONE } else { AIR }
}

/// Flat ground two blocks deep under y = 0, from x = -4 to 40.
fn flat() -> HashSet<IVec3> {
    let mut s = HashSet::new();
    for x in -4..=40 {
        for z in -4..=4 {
            s.extend([v(x, -1, z), v(x, -2, z)]);
        }
    }
    s
}

fn east(len: i32) -> Curve {
    Curve::new(v(0, 0, 0), E, v(len, 0, 0), E)
}

#[test]
fn level_track_on_level_ground_lies_and_a_node_needs_ground() {
    let ground = flat();
    assert_eq!(faults(&east(16), world(&ground)), vec![]);
    assert!(node_stands(world(&ground), v(3, 0, 0)));
    assert!(!node_stands(world(&ground), v(3, 1, 0)), "a block up, in the air");
}

#[test]
fn a_dip_one_block_deep_is_carried_by_ballast_a_deeper_gap_is_refused() {
    let mut ground = flat();
    for x in 5..=8 {
        ground.remove(&v(x, -1, 0));
    }
    assert_eq!(faults(&east(16), world(&ground)), vec![], "one block down is within reach");
    for x in 5..=8 {
        ground.remove(&v(x, -2, 0));
    }
    let want: Vec<(IVec3, Fault)> = (5..=8).map(|x| (v(x, -1, 0), Fault::Floating)).collect();
    assert_eq!(faults(&east(16), world(&ground)), want, "the cells to fill, in order");
}

#[test]
fn ground_or_a_block_where_a_car_runs_is_in_the_way_two_blocks_up_is_clear() {
    for (block, hit) in [(v(10, 0, 0), true), (v(10, 1, 0), true), (v(10, 2, 0), false), (v(10, 0, 2), false)] {
        let mut ground = flat();
        ground.insert(block);
        let got = faults(&east(16), world(&ground));
        let want = if hit { vec![(block, Fault::Buried)] } else { vec![] };
        assert_eq!(got, want, "a block at {block:?}");
    }
    // Water is no ground and no room either.
    let ground = flat();
    let wet = |p: IVec3| if p == v(6, 0, 0) { WATER } else { world(&ground)(p) };
    assert_eq!(faults(&east(16), wet), vec![(v(6, 0, 0), Fault::Buried)]);
}

#[test]
fn a_ramp_climbs_a_staircase_of_whole_blocks_but_not_open_air() {
    // 12 blocks east, 4 up: a third. Under it, each column's top is a whole block at or under the bed.
    let ramp = Curve::new(v(0, 0, 0), E, v(12, 4, 0), E);
    let mut stairs = flat();
    for x in 0..=16 {
        for y in 0..x / 3 {
            stairs.insert(v(x, y, 0));
        }
    }
    assert_eq!(faults(&ramp, world(&stairs)), vec![]);
    // The same ramp over level ground leaves it a third of the way up.
    let got = faults(&ramp, world(&flat()));
    assert!(!got.is_empty() && got.iter().all(|f| f.1 == Fault::Floating), "{got:?}");
    // A staircase a block too high buries it.
    let mut high = flat();
    for x in 0..=16 {
        for y in 0..x / 3 + 1 {
            high.insert(v(x, y, 0));
        }
    }
    assert!(faults(&ramp, world(&high)).iter().any(|f| f.1 == Fault::Buried));
}

/// The outside check of the fuzz below, written apart from `faults`: walking the track a fiftieth of a block at a
/// time, the highest solid block of each column must top out between `MAX_DROP` under the bed and `MAX_SINK` over
/// it, with no solid block over it up to a car's roof. Returns the first point where that fails.
fn oracle(curve: &Curve, world: &mut World) -> Option<Vec3> {
    let n = (curve.length() * 50.0).ceil() as usize;
    for i in 0..=n {
        let p = curve.at(i as f64 / n as f64);
        let bed = p.y - 0.5;
        let (x, z) = (p.x.floor() as i32, p.z.floor() as i32);
        let first_solid_down = ((bed - 4.0).floor() as i32..=(bed + 1.75).floor() as i32)
            .rev()
            .find(|&y| SOLID[world.block_anywhere_or_generate(v(x, y, z)) as usize]);
        let Some(y) = first_solid_down else { return Some(p) };
        let top = f64::from(y + 1);
        if top > bed + MAX_SINK + 1e-9 || top < bed - MAX_DROP - 1e-9 {
            return Some(p);
        }
    }
    None
}

/// 2,000 random pairs of nodes standing on generated terrain. Of those whose shape fits, `faults` must refuse every
/// one the oracle finds off the ground. Prints how many the shape check alone (the game before 2026-10-30) let float
/// or run through the ground.
#[test]
fn fuzz_no_accepted_track_floats_or_runs_through_the_ground() {
    let mut world = World::new(1337, 2);
    let mut rng = Rng::new(7);
    let surface = |world: &mut World, x: i32, z: i32| {
        let y = world.generator().height_at(x, z) + 1;
        let p = v(x, y, z);
        (world.block_anywhere_or_generate(p) == AIR && node_stands(|q| world.block_anywhere_or_generate(q), p))
            .then_some(p)
    };
    let (mut shaped, mut accepted, mut bad_before) = (0, 0, 0);
    for _ in 0..2000 {
        let (x, z) = (rng.below(160) as i32 - 80, rng.below(160) as i32 - 80);
        let (dist, turn) = (rng.range(4.0, f64::from(MAX_SPAN)), rng.range(0.0, std::f64::consts::TAU));
        let (bx, bz) = (x + (dist * turn.sin()).round() as i32, z - (dist * turn.cos()).round() as i32);
        let (Some(a), Some(b)) = (surface(&mut world, x, z), surface(&mut world, bx, bz)) else { continue };
        let chord = yaw_of(f64::from(b.x - a.x), f64::from(b.z - a.z));
        let ya = chord.wrapping_add(rng.below(24) as u8).wrapping_sub(12);
        // B faces the mirror of A's heading in the chord, as the hand lays it (`Game::follow_yaw`).
        let (dx, dz) = dir_of(ya);
        let (cx, cz) = dir_of(chord);
        let dot = dx * cx + dz * cz;
        let yb = yaw_of(2.0 * dot * cx - dx, 2.0 * dot * cz - dz);
        if fit(a, ya, b, yb) != Fit::Ok {
            continue;
        }
        shaped += 1;
        let curve = Curve::new(a, ya, b, yb);
        let found = faults(&curve, |p| world.block_anywhere_or_generate(p));
        let truth = oracle(&curve, &mut world);
        if found.is_empty() {
            accepted += 1;
            assert!(truth.is_none(), "accepted {a:?} {ya} → {b:?} {yb}, off the ground at {truth:?}");
        } else {
            bad_before += 1;
        }
    }
    eprintln!("ground fuzz: {shaped} pairs fit the shape, {accepted} lie on the ground, {bad_before} were let float or run through it before");
    assert!(shaped > 400 && accepted > 100, "the fuzz reaches enough tracks: {shaped} shaped, {accepted} accepted");
}

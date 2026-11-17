use super::super::tests::place_block;
use super::*;
use crate::block::{GENERATOR, LONG_POLE, MINER, POLE};
use crate::world::World;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn by_hand() -> Factory {
    Factory { by_hand: true, ..Factory::default() }
}

fn settle(f: &mut Factory) {
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
}

/// Every wire as (end a, end b, between two real poles).
fn wires(f: &Factory, eye: Vec3, range: f64) -> Vec<(Vec3, Vec3, bool)> {
    let mut out = Vec::new();
    f.for_each_wire(eye, range, |a, b, _, poles| out.push((a, b, poles)));
    out
}

fn near(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-9
}

/// Every place a wire may leave pole `p`: each side's first eight ranks.
fn hang_points(p: &Pole) -> Vec<Vec3> {
    (0..2).flat_map(|side| (0..8).map(move |rank| p.hang_point(side, rank))).collect()
}

/// A Mk1 pole with four wires, a Mk4 pole with six wires and a machine lead, and two towers with four wires each.
fn grid() -> Factory {
    let mut f = by_hand();
    let link = |f: &mut Factory, a: IVec3, others: &[IVec3]| {
        for &o in others {
            place_block(f, POLE, o);
            f.connect(a, o);
        }
    };
    place_block(&mut f, POLE, v(0, 3, 0));
    link(&mut f, v(0, 3, 0), &[v(6, 3, 0), v(-6, 3, 0), v(0, 3, 6), v(0, 3, -6)]);
    place_block(&mut f, POLE, v(40, 3, 0));
    let mk4 = f.poles.iter().position(|p| p.pos == v(40, 3, 0)).unwrap();
    f.poles[mk4].tier = 3;
    link(&mut f, v(40, 3, 0), &[v(46, 3, 0), v(34, 3, 3), v(40, 3, 8), v(40, 3, -8), v(46, 3, 5)]);
    place_block(&mut f, GENERATOR, v(44, 3, -3));
    f.connect(v(40, 3, 0), v(44, 3, -3));
    for x in [0, 120] {
        place_block(&mut f, LONG_POLE, v(x, 3, 60));
    }
    place_block(&mut f, LONG_POLE, v(60, 3, 110));
    f.connect(v(0, 3, 60), v(120, 3, 60));
    f.connect(v(0, 3, 60), v(60, 3, 110));
    link(&mut f, v(0, 3, 60), &[v(8, 3, 60), v(-8, 3, 60)]);
    place_block(&mut f, MINER, v(4, 3, 66));
    f.connect(v(0, 3, 60), v(4, 3, 66));
    settle(&mut f);
    f
}

#[test]
fn both_ends_of_every_wire_in_a_built_grid_are_hang_points_of_their_poles() {
    let f = grid();
    let all = wires(&f, Vec3::ZERO, 1e9);
    assert!(all.len() >= 14, "a real grid: {}", all.len());
    let points: Vec<Vec3> = f.poles.iter().flat_map(hang_points).collect();
    let middles: Vec<Vec3> = f.poles.iter().map(Pole::wire_point).collect();
    for (a, b, poles) in all {
        assert!(points.iter().any(|&p| near(p, a)), "{a:?} is no insulator");
        assert!(!middles.iter().any(|&m| near(m, a) || near(m, b)), "a wire still leaves a pole's middle");
        // A lead to a machine ends on the machine; between poles both ends are insulators.
        assert!(!poles || points.iter().any(|&p| near(p, b)), "{b:?} is no insulator");
    }
}

#[test]
fn a_wire_leaves_the_side_its_other_end_lies_towards_and_a_line_along_z_runs_parallel() {
    let mut f = by_hand();
    // One pair along x, and a separate pair along z (so no pole has a second wire on its side to fan).
    for p in [v(0, 3, 0), v(10, 3, 0), v(50, 3, 0), v(50, 3, 8)] {
        place_block(&mut f, POLE, p);
    }
    f.connect(v(0, 3, 0), v(10, 3, 0));
    f.connect(v(50, 3, 0), v(50, 3, 8));
    settle(&mut f);
    let centre = |p: IVec3| p.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
    let w = wires(&f, Vec3::ZERO, 1e9);
    let east = w.iter().find(|(a, _, _)| a.x < 20.0).expect("the wire to the east pole");
    assert!(east.0.x > centre(v(0, 3, 0)).x, "leaves the east insulator");
    assert!(east.1.x < centre(v(10, 3, 0)).x, "and arrives at the west one");
    let north = w.iter().find(|(a, _, _)| a.x > 20.0).expect("the wire along z");
    let (dx_a, dx_b) = (north.0.x - centre(v(50, 3, 0)).x, north.1.x - centre(v(50, 3, 8)).x);
    assert!((dx_a - dx_b).abs() < 1e-9, "the same insulator at both ends: parallel, {dx_a} and {dx_b}");
}

#[test]
fn a_towers_wires_on_one_side_take_its_arms_in_turn_and_never_share_a_clamp() {
    let mut f = by_hand();
    place_block(&mut f, LONG_POLE, v(0, 3, 0));
    for z in [0, 12] {
        place_block(&mut f, LONG_POLE, v(100, 3, z));
        f.connect(v(0, 3, 0), v(100, 3, z));
    }
    settle(&mut f);
    let w = wires(&f, Vec3::ZERO, 1e9);
    assert_eq!(w.len(), 2);
    let (first, second) = (w[0].0, w[1].0);
    let tower = f.poles.iter().find(|p| p.pos == v(0, 3, 0)).unwrap();
    assert!(near(first, tower.hang_point(1, 0)) && near(second, tower.hang_point(1, 1)));
    assert!(first.y < second.y, "the first wire takes the lower arm");
    assert!(first.x > second.x, "the lower arm reaches further");
    // Each far tower holds its wire on the lower arm of its west side: no two clamps coincide.
    assert!(near(w[0].1, w[1].1 - Vec3::new(0.0, 0.0, 12.0)), "parallel at the far towers");
}

#[test]
fn a_wire_never_leaves_a_towers_lightning_peak() {
    let f = grid();
    for p in f.poles.iter().filter(|p| p.is_long()) {
        let peak = p.hang_centre() + Vec3::new(0.0, super::pole_tower::TOP, 0.0);
        assert!(hang_points(p).iter().all(|&h| !near(h, peak)));
    }
}

#[test]
fn wires_past_the_last_level_share_it_fanned_along_the_arm_and_the_fan_wraps() {
    let p = Pole { pos: v(0, 3, 0), tier: 0, wall: 0 };
    let x = |rank| p.hang_point(1, rank).x - p.hang_point(1, 0).x;
    assert_eq!(x(0), 0.0);
    assert!(x(1) < 0.0 && x(2) < x(1), "towards the post");
    assert!(x(1) > -0.1, "a few centimetres, not a new arm");
    assert_eq!(x(FAN_STEPS), 0.0, "the fan wraps");
    assert!(near(p.hang_point(0, 0), p.hang_point(1, 0) - Vec3::new(2.0 * INSULATOR_X, 0.0, 0.0)), "mirrored");
}

#[test]
fn the_ends_do_not_depend_on_where_the_camera_is() {
    let f = grid();
    let all = wires(&f, Vec3::ZERO, 1e9);
    let eye = Vec3::new(40.0, 5.0, 0.0);
    let shown = wires(&f, eye, 20.0);
    assert!(!shown.is_empty() && shown.len() < all.len(), "some wires are culled");
    for s in shown {
        assert!(all.iter().any(|a| near(a.0, s.0) && near(a.1, s.1)), "a wire moved when others were culled");
    }
}

#[test]
fn the_wiring_preview_names_the_next_free_insulator() {
    let mut f = by_hand();
    for p in [v(0, 3, 0), v(10, 3, 0)] {
        place_block(&mut f, POLE, p);
    }
    settle(&mut f);
    let east = Vec3::new(20.0, 4.5, 0.5);
    let pole = f.poles.iter().find(|p| p.pos == v(0, 3, 0)).unwrap();
    assert_eq!(f.pole_hang(v(0, 3, 0), east), Some(pole.hang_point(1, 0)), "nothing wired yet");
    f.connect(v(0, 3, 0), v(10, 3, 0));
    settle(&mut f);
    let pole = f.poles.iter().find(|p| p.pos == v(0, 3, 0)).unwrap();
    assert_eq!(f.pole_hang(v(0, 3, 0), east), Some(pole.hang_point(1, 1)), "the second east wire fans");
    assert_eq!(
        f.pole_hang(v(0, 3, 0), Vec3::new(-20.0, 4.5, 0.5)),
        Some(pole.hang_point(0, 0)),
        "the west side is free"
    );
    assert_eq!(f.pole_hang(v(5, 3, 5), east), None, "no pole there");
    assert!(near(ghost_hang(v(0, 3, 0), 0, 0, east), pole.hang_point(1, 0)));
}

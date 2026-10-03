use super::*;
use crate::block::{AIR, WATER};

fn flat(_x: i32, y: i32, _z: i32) -> bool {
    y < 10
}

fn dry(_x: i32, _y: i32, _z: i32) -> BlockId {
    AIR
}

fn settle(p: &mut Player, seconds: f64) {
    let steps = (seconds * 120.0) as usize;
    for _ in 0..steps {
        p.step(1.0 / 120.0, &mut flat, &mut dry);
    }
}

#[test]
fn lands_and_walks() {
    let mut p = Player::new(Vec3::new(0.5, 14.0, 0.5));
    settle(&mut p, 2.0);
    assert!(p.on_ground);
    assert!((p.pos.y - 10.0).abs() < 1e-6, "y = {}", p.pos.y);
    p.input.forward = 1.0;
    settle(&mut p, 1.0);
    assert!(p.pos.z < -3.0, "should walk towards -Z, z = {}", p.pos.z);
}

#[test]
fn a_belt_carries_a_body_on_the_ground_but_not_in_the_air() {
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    settle(&mut p, 0.2);
    p.conveyor = Vec3::new(2.0, 0.0, 0.0);
    settle(&mut p, 1.0);
    assert!((p.pos.x - 2.5).abs() < 0.05, "two blocks a second, x = {}", p.pos.x);
    assert!((p.carried.x - 2.0).abs() < 0.05 && p.carried.z == 0.0);

    // Walking against it goes by the sum: walking speed minus the belt's.
    let mut q = Player::new(Vec3::new(0.5, 10.0, 0.5));
    settle(&mut q, 0.2);
    q.conveyor = Vec3::new(0.0, 0.0, -1.0);
    q.yaw = std::f64::consts::PI; // facing +Z
    q.input.forward = 1.0;
    settle(&mut q, 1.0);
    assert!(q.pos.z - 0.5 > 3.0 && q.pos.z - 0.5 < 3.6, "4.3 walking against 1.0, z = {}", q.pos.z);

    // Not on the ground (falling, flying): nothing carries it.
    let mut a = Player::new(Vec3::new(0.5, 14.0, 0.5));
    a.conveyor = Vec3::new(2.0, 0.0, 0.0);
    a.step(1.0 / 120.0, &mut flat, &mut dry);
    assert!(!a.on_ground && a.pos.x == 0.5);
    let mut f = Player::new(Vec3::new(0.5, 20.0, 0.5));
    (f.flying, f.conveyor) = (true, Vec3::new(2.0, 0.0, 0.0));
    settle(&mut f, 0.5);
    assert!((f.pos.x - 0.5).abs() < 1e-6 && f.carried.x == 0.0);
}

#[test]
fn a_crouching_body_is_not_carried_off_the_edge() {
    // Ground only west of x = 3.
    let edge = |x: i32, y: i32, _z: i32| x < 3 && y < 10;
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    p.input.crouch = true;
    for _ in 0..30 {
        p.step(1.0 / 120.0, &mut |x, y, z| edge(x, y, z), &mut dry);
    }
    p.conveyor = Vec3::new(4.0, 0.0, 0.0);
    for _ in 0..240 {
        p.step(1.0 / 120.0, &mut |x, y, z| edge(x, y, z), &mut dry);
    }
    assert!(p.on_ground && p.pos.x < 3.3, "stopped at the edge, x = {}", p.pos.x);
}

#[test]
fn jump_clears_one_block() {
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    settle(&mut p, 0.2);
    p.input.jump = true;
    let mut apex: f64 = 0.0;
    for _ in 0..120 {
        p.step(1.0 / 120.0, &mut flat, &mut dry);
        apex = apex.max(p.pos.y - 10.0);
    }
    assert!(apex > 1.05 && apex < 1.6, "apex {apex}");
}

/// A sea (water from y 50 up to the surface at 62) west of x = 0, and a beach whose top is at y 63,
/// a block above the water, east of it.
fn seaside(x: i32, y: i32, _z: i32) -> bool {
    y < if x < 0 { 50 } else { 63 }
}

fn sea(x: i32, y: i32, _z: i32) -> BlockId {
    if x < 0 && (50..62).contains(&y) {
        WATER
    } else {
        AIR
    }
}

fn swim(p: &mut Player, seconds: f64) {
    for _ in 0..(seconds * 120.0) as usize {
        p.step(1.0 / 120.0, &mut seaside, &mut sea);
    }
}

#[test]
fn a_swimmer_sinks_slowly_rises_with_jump_and_climbs_out() {
    let mut p = Player::new(Vec3::new(-6.5, 70.0, 0.5));
    swim(&mut p, 2.5);
    assert!(p.in_water && p.splash_speed > 10.0, "fell in with a splash: {}", p.splash_speed);
    assert!(p.pos.y > 52.0 && p.pos.y < 60.0, "water slowed the dive: y = {}", p.pos.y);
    assert!(p.vel.y < 0.0 && p.vel.y > -1.0, "sinks slowly: {}", p.vel.y);

    p.input.jump = true;
    swim(&mut p, 8.0);
    let (mut low, mut high) = (f64::MAX, f64::MIN);
    for _ in 0..240 {
        swim(&mut p, 1.0 / 120.0);
        (low, high) = (low.min(p.pos.y), high.max(p.pos.y));
    }
    assert!(low > 60.5 && high < 61.3, "bobs with its chest at the surface: {low}..{high}");
    assert!(low + EYE_HEIGHT > 62.0, "head above the water");

    // Face east (+X) and swim at the beach: a leap at the edge lands on it.
    p.yaw = std::f64::consts::FRAC_PI_2;
    p.input.forward = 1.0;
    swim(&mut p, 4.0);
    p.input = PlayerInput::default();
    swim(&mut p, 1.0);
    assert!(p.on_ground && !p.in_water, "out of the water");
    assert!((p.pos.y - 63.0).abs() < 1e-6 && p.pos.x > 0.3, "on the beach: {:?}", (p.pos.x, p.pos.y));
}

#[test]
fn swimming_is_half_speed_and_flying_ignores_water() {
    let mut p = Player::new(Vec3::new(-40.5, 55.0, 0.5));
    p.input.forward = 1.0;
    swim(&mut p, 2.0);
    let before = p.pos.z;
    swim(&mut p, 1.0);
    let speed = before - p.pos.z;
    assert!((speed - WALK_SPEED * SWIM_SPEED_FACTOR).abs() < 0.1, "swims at {speed}");

    p.flying = true;
    p.input.forward = 0.0;
    swim(&mut p, 1.0);
    let y = p.pos.y;
    swim(&mut p, 1.0);
    assert!(!p.in_water && (p.pos.y - y).abs() < 1e-3, "hovers in the water");
}

/// Ground at y 10 with a cliff east of x = 1 whose top is at y 15, and ladders at x = 0 from y 10 to 14.
fn cliff(x: i32, y: i32, _z: i32) -> bool {
    y < if x >= 1 { 15 } else { 10 }
}

fn ladders(x: i32, y: i32, _z: i32) -> BlockId {
    if x == 0 && (10..15).contains(&y) {
        LADDER
    } else {
        AIR
    }
}

fn climb(p: &mut Player, seconds: f64) {
    for _ in 0..(seconds * 120.0) as usize {
        p.step(1.0 / 120.0, &mut cliff, &mut ladders);
    }
}

#[test]
fn a_ladder_holds_a_climber_and_leads_onto_the_ledge() {
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    p.yaw = std::f64::consts::FRAC_PI_2;
    p.input.jump = true;
    climb(&mut p, 0.8);
    let y = p.pos.y;
    assert!(y > 12.0 && y < 13.0 && !p.on_ground, "climbs at 3 blocks a second: y = {y}");
    p.input.jump = false;
    climb(&mut p, 1.0);
    assert!((p.pos.y - y).abs() < 1e-9, "holds on");
    p.input.crouch = true;
    climb(&mut p, 0.4);
    assert!(p.pos.y < y - 1.0, "goes down");
    p.input.crouch = false;
    p.input.jump = true;
    p.input.forward = 1.0;
    climb(&mut p, 1.5);
    p.input.jump = false;
    climb(&mut p, 1.0);
    assert!(p.on_ground && (p.pos.y - 15.0).abs() < 1e-6 && p.pos.x > 1.2, "on the ledge: {:?}", p.pos);
}

#[test]
fn a_belt_lift_stack_climbs_like_a_ladder() {
    let mut lifts = |x: i32, y: i32, _z: i32| if x == 0 && (10..15).contains(&y) { LIFT } else { AIR };
    let mut p = Player::new(Vec3::new(0.5, 10.0, 0.5));
    p.yaw = std::f64::consts::FRAC_PI_2;
    p.input.jump = true;
    p.input.forward = 1.0;
    for _ in 0..240 {
        p.step(1.0 / 120.0, &mut cliff, &mut lifts);
    }
    p.input.jump = false;
    for _ in 0..120 {
        p.step(1.0 / 120.0, &mut cliff, &mut lifts);
    }
    assert!(p.on_ground && (p.pos.y - 15.0).abs() < 1e-6 && p.pos.x > 1.2, "up the lifts onto the ledge: {:?}", p.pos);
}

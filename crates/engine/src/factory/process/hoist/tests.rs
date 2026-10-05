//! The hoist: a winch beside the top of a shaft makes riders climb at up to `HOIST_SPEED`, only with power, and a body
//! really rises that fast (the shaft is a stack of `HOIST` blocks, which the tests answer for with a closure).

use super::*;
use crate::block::{AIR, COAL_ORE, GENERATOR, POLE};
use crate::factory::Factory;
use crate::player::Player;
use crate::world::World;

const TOP: i32 = 99;

/// The shaft: hoist blocks in the column at (0, z 0) from y 80 to `TOP`.
fn shaft(x: i32, y: i32, z: i32) -> BlockId {
    if (x, z) == (0, 0) && (80..=TOP).contains(&y) {
        HOIST
    } else {
        AIR
    }
}

/// A winch at `winch`, a pole and a coal generator (60 kW), the winch on the grid if `wired`.
fn plant(winch: IVec3, wired: bool) -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, WINCH, winch, 0);
    if wired {
        put(&mut f, POLE, winch + IVec3::new(3, 0, 0), 3);
        let gen = winch + IVec3::new(4, 0, 0);
        put(&mut f, GENERATOR, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 32), 32);
    }
    f
}

fn run(f: &mut Factory, ticks: u64) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for t in 0..ticks {
        f.update(&mut world, t, &mut events);
    }
}

fn rate(f: &Factory, y: f64) -> f64 {
    f.hoist_rate(Vec3::new(0.5, y, 0.5), &mut |x, y, z| shaft(x, y, z))
}

#[test]
fn a_powered_winch_above_the_top_carries_riders_anywhere_in_the_shaft() {
    let mut f = plant(IVec3::new(0, TOP + 1, 0), true);
    run(&mut f, 5);
    for y in [80.0, 90.5, 99.0] {
        assert_eq!(rate(&f, y), HOIST_SPEED, "at {y}");
    }
    assert_eq!(rate(&f, 70.0), 0.0, "below the shaft");
    let text = f.describe(IVec3::new(0, TOP + 1, 0)).unwrap();
    assert!(text.starts_with("Shaft running at 9 blocks a second"), "{text}");
}

#[test]
fn a_winch_beside_the_top_cell_works_too() {
    let mut f = plant(IVec3::new(1, TOP, 0), true);
    run(&mut f, 5);
    assert_eq!(rate(&f, 90.0), HOIST_SPEED);
}

#[test]
fn without_power_a_wire_or_a_winch_at_the_top_the_shaft_is_a_ladder() {
    let mut dark = plant(IVec3::new(0, TOP + 1, 0), false);
    run(&mut dark, 5);
    assert_eq!(rate(&dark, 90.0), 0.0);
    assert!(dark.describe(IVec3::new(0, TOP + 1, 0)).unwrap().contains("only a ladder"));
    let mut low = plant(IVec3::new(0, 90, 1), true);
    run(&mut low, 5);
    assert_eq!(rate(&low, 90.0), 0.0, "a winch halfway down is not beside the top cell");
    let mut f = plant(IVec3::new(0, TOP + 1, 0), true);
    assert_eq!(rate(&f, 90.0), 0.0, "before the first tick it has had no power");
    run(&mut f, 1);
    assert!(rate(&f, 90.0) > 0.0);
}

#[test]
fn a_body_rises_in_a_powered_shaft_three_times_faster_than_on_a_ladder() {
    let rise = |hoist: f64| {
        let mut p = Player::new(Vec3::new(0.5, 82.0, 0.5));
        p.hoist = hoist;
        p.input.jump = true;
        for _ in 0..20 {
            p.step(0.05, &mut |_, _, _| false, &mut |x, y, z| shaft(x, y, z));
        }
        p.pos.y - 82.0
    };
    let (ladder, hoisted) = (rise(0.0), rise(HOIST_SPEED));
    assert!((ladder - 3.0).abs() < 0.2 && (hoisted - 9.0).abs() < 0.2, "{ladder} and {hoisted}");
}

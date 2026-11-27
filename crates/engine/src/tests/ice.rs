//! FREEZE through `Game`: a body walks on a frozen pool in winter and swims in it in summer; under the ice it is not
//! trapped. Ice is a pure function of place and date, so the test only builds a pool and sets the clock.

use super::*;
use crate::block::{STONE, WATER};
use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};
use crate::weather::ice::{cold_at, IceTable, Water, BEARS_LEVEL};
use crate::worldgen::SEA_LEVEL;

/// A 3 × 3 pool of still water one deep, walled and floored with stone, high above the sea (so it is a lake and cold):
/// returns the centre cell of its water.
fn build_pool(g: &mut Game) -> IVec3 {
    let p = g.body().pos.floor();
    let c = IVec3::new(p.x + 4, SEA_LEVEL + 80, p.z + 4);
    for dx in -2..=2 {
        for dz in -2..=2 {
            let at = |dy| IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            g.sim.world.set_block(at(-1), STONE);
            let inner = dx.abs() <= 1 && dz.abs() <= 1;
            g.sim.world.set_block(at(0), if inner { WATER } else { STONE });
        }
    }
    c
}

/// Puts the body just above the pool's centre and runs its physics for a second; returns where its feet end.
fn drop_onto(g: &mut Game, c: IVec3, tick: u64) -> f64 {
    g.sim.tick = tick;
    let local = g.local.0 as usize;
    let body = g.bodies[local].as_mut().expect("the local body");
    body.pos = Vec3::new(f64::from(c.x) + 0.5, f64::from(c.y) + 1.2, f64::from(c.z) + 0.5);
    body.vel = Vec3::ZERO;
    body.flying = false;
    for _ in 0..60 {
        g.step_bodies();
    }
    g.body().pos.y
}

/// Day `day` of the second year at noon.
fn noon(day: u64) -> u64 {
    (YEAR_DAYS + day) * DAY_TICKS + DAY_TICKS / 2 - START
}

#[test]
fn a_body_stands_on_a_frozen_pool_in_winter_and_swims_in_it_in_summer() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let c = build_pool(&mut g);
    let cold = cold_at(g.sim.world.generator(), c.x, c.y, c.z);
    let (winter, summer) = (noon(33), noon(13));
    assert!(IceTable::at(winter).level(Water::Lake, cold) >= BEARS_LEVEL, "the pool is cold enough (cold {cold})");
    assert_eq!(IceTable::at(summer).level(Water::Lake, cold), 0, "open in summer (cold {cold})");

    let feet = drop_onto(&mut g, c, winter);
    assert!((feet - (f64::from(c.y) + 15.0 / 16.0)).abs() < 0.02, "stands on the ice: feet at {feet}, pool at {}", c.y);
    assert!(!g.body().in_water);

    let feet = drop_onto(&mut g, c, summer);
    assert!(feet < f64::from(c.y) + 0.5, "sinks into open water: feet at {feet}");
    assert!(g.body().in_water);
}

#[test]
fn a_swimmer_under_ice_is_not_held_by_it() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let c = build_pool(&mut g);
    // Feet below the ice's top: the slab bears only from above, so it never boxes in a body already under it.
    let ice = IceTable::at(noon(33));
    let below = crate::weather::ice::ice_boxes(&g.sim.world, &ice, f64::from(c.y) + 0.5, c.x, c.y, c.z);
    let above = crate::weather::ice::ice_boxes(&g.sim.world, &ice, f64::from(c.y) + 1.0, c.x, c.y, c.z);
    assert!(below.is_empty());
    assert_eq!(above.slice().len(), 1);
    let slab = above.slice()[0];
    assert!((slab.max.y - (f64::from(c.y) + 15.0 / 16.0)).abs() < 1e-9);
}

//! The washer: crushed ore only, water from a pipe, tailings out of the side hatch, and the ingot yields of the
//! three routes the plan promises (raw 1.0, crushed 1.5, washed 2.0).

use crate::block::{COAL_ORE, GENERATOR, IRON_ORE, POLE, PUMP, TAILINGS};
use crate::factory::{Factory, DIRS};
use crate::item::{ItemId, CRUSHED_IRON, WASHED_IRON};
use crate::math::IVec3;
use crate::recipes::{water_use, CRUSH_RECIPES, MACHINE_RECIPES, WASH_RECIPES};
use crate::research::TECHS;
use crate::world::World;

use super::super::{Processor, Status};
use super::*;
use crate::factory::footprint::Role;

/// A washer at the origin, a pole and two generators (120 kW) with coal, Ore Washing researched, and, if `water`, a
/// pump piece on its water inlet holding ten units.
fn rig(water: bool) -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let tech = TECHS.iter().position(|t| t.name == "Ore Washing").unwrap() as u8;
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
    f.place(&mut world, WASHER, IVec3::ZERO, 0, IVec3::ZERO, 0);
    let pole = IVec3::new(4, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    for dz in 0..2 {
        let gen = pole + IVec3::new(1, 0, dz);
        f.place(&mut world, GENERATOR, gen, 0, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    if water {
        let tap = f.processors[0].pipe_ports().into_iter().find(|p| p.2 == Role::Water).expect("a water inlet");
        let at = tap.0 + DIRS[tap.1 as usize];
        f.place(&mut world, PUMP, at, 0, at, 0);
        f.pipework.iter_mut().for_each(|p| p.held = 10);
    }
    f
}

fn run(f: &mut Factory, seconds: u32) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..seconds * crate::TICK_RATE {
        f.update(&mut world, tick as u64, &mut events);
    }
}

fn washer(f: &Factory) -> &Processor {
    &f.processors[0]
}

#[test]
fn it_takes_crushed_ore_only() {
    let mut f = rig(true);
    assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 4), 0, "raw ore does not fit");
    assert_eq!(f.insert(IVec3::ZERO, CRUSHED_IRON, 3), 3);
}

#[test]
fn three_crushed_ore_and_water_make_four_washed_and_a_tailings_block() {
    let mut f = rig(true);
    f.insert(IVec3::ZERO, CRUSHED_IRON, 3);
    run(&mut f, 5);
    let w = washer(&f);
    assert_eq!(w.out.count(WASHED_IRON), 4);
    assert_eq!(w.side.count(ItemId::block(TAILINGS)), 1);
    assert_eq!(w.power(), 60);
}

#[test]
fn without_water_it_waits() {
    let mut f = rig(false);
    f.insert(IVec3::ZERO, CRUSHED_IRON, 3);
    run(&mut f, 3);
    assert_eq!(washer(&f).status, Status::NoWater);
    assert_eq!(washer(&f).out.total(), 0);
}

#[test]
fn ingots_per_ore_by_route() {
    // Raw smelts 1 for 1; the crusher makes 3 from 2; the washer makes 4 from 3 of those, each smelting 1 for 1.
    let (crush, wash) = (&MACHINE_RECIPES[CRUSH_RECIPES[0] as usize], &MACHINE_RECIPES[WASH_RECIPES[0] as usize]);
    let crushed_per_ore = crush.outputs[0].1 as f64 / crush.inputs[0].1 as f64;
    let washed_per_crushed = wash.outputs[0].1 as f64 / wash.inputs[0].1 as f64;
    assert_eq!(crush.outputs[0].0, CRUSHED_IRON);
    assert_eq!((crushed_per_ore, crushed_per_ore * washed_per_crushed), (1.5, 2.0));
    assert_eq!(water_use(WASH_RECIPES[0]), 1);
}

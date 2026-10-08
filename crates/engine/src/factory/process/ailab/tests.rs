//! The AI lab: a data consumer that researches like a center (twice a lab's speed, every 2nd unit free), the only
//! lab for techs that cost compute, and idle without a datacenter's compute on its grid. The compute producer here is
//! a winch row with a `compute` number, leaked into a `'static` spec (as in the data grid's tests).

use crate::block::{BlockId, AI_LAB, COAL_ORE, FIBRE_NODE, GENERATOR, LAB, POLE, RESEARCH_CENTER, WINCH};
use crate::factory::footprint::SINGLE;
use crate::factory::links::Slot;
use crate::factory::process::ProcessTier;
use crate::factory::{add_to, Factory};
use crate::math::IVec3;
use crate::research::{needs_ai_lab, Research, PACKS, TECHS};
use crate::world::World;

use super::*;

/// "AI Research", the one tech so far that costs compute.
const AI_TECH: u8 = 62;

fn producer(compute: i32) -> &'static ProcessSpec {
    Box::leak(Box::new(ProcessSpec {
        block: WINCH,
        categories: &[],
        pick: Pick::ByInput,
        buffers: [0, 0, 0],
        side: 0,
        tiers: &[ProcessTier { energy: Energy::Hoist, speed: 1000, fuel: 0, power: 1 }],
        footprint: SINGLE,
        verb: "Testing",
        products: "nothing",
        waiting: "Idle",
        map_colour: 0,
        compute,
        parts: &[],
    }))
}

/// Finishes `tech` and everything it needs.
fn finish(r: &mut Research, tech: u8) {
    for &n in TECHS[tech as usize].needs {
        finish(r, n);
    }
    for _ in 0..TECHS[tech as usize].units {
        r.add_unit(tech);
    }
}

/// `block` at the origin, a pole and two coal generators (120 kW), a fibre node, and, if `datacenter`, a TF producer.
fn rig(block: BlockId, datacenter: bool) -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, block, IVec3::ZERO, 0, IVec3::ZERO, 0);
    let pole = IVec3::new(4, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    for x in 5..7 {
        let gen = IVec3::new(x, 0, 0);
        f.place(&mut world, GENERATOR, gen, 0, gen, 0);
        f.insert(gen, COAL_ORE.into(), 64);
    }
    let node = IVec3::new(2, 0, 2);
    f.place(&mut world, FIBRE_NODE, node, 0, node, 0);
    if datacenter {
        let at = IVec3::new(4, 0, 2);
        add_to(&mut f.processors, Processor::new(at, producer(100), 0), &mut f.at, Slot::Process);
    }
    f.dirty = true;
    finish(&mut f.research, AI_TECH - 1);
    f.research.set_current(Some(AI_TECH));
    assert_eq!(f.research.current, Some(AI_TECH), "the tech is available");
    for p in PACKS {
        f.insert(IVec3::ZERO, p, 64);
    }
    f
}

fn run(f: &mut Factory, seconds: f64) {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    for tick in 0..(seconds * crate::TICK_RATE as f64) as u64 {
        f.update(&mut world, tick, &mut events);
    }
}

fn lab(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.is_ai_lab()).unwrap()
}

#[test]
fn the_spec_is_a_2x2x2_data_consumer_that_works_like_a_center() {
    let p = Processor::new(IVec3::ZERO, &AI_LAB_SPEC, 0);
    assert!(p.is_ai_lab() && p.is_center());
    assert_eq!((AI_LAB_SPEC.footprint.size, AI_LAB_SPEC.compute), ([2, 2, 2], -20));
    assert_eq!((p.free_every(&[0; 5]), p.stats().speed, p.power()), (2, 2000, 100));
}

#[test]
fn only_ai_research_costs_compute() {
    assert!(needs_ai_lab(AI_TECH) && !needs_ai_lab(0));
    assert_eq!(TECHS[AI_TECH as usize].name, "AI Research");
}

#[test]
fn with_a_datacenter_on_the_grid_it_researches_and_every_second_unit_is_free() {
    let mut f = rig(AI_LAB, true);
    run(&mut f, 65.0); // 60 s a unit at twice a lab's speed: 30 s
    assert_eq!(f.research.progress(AI_TECH), 2);
    assert_eq!(lab(&f).input.count(PACKS[0]), 62, "units 1 and 3 paid, unit 2 was free");
    assert_eq!(f.power.demand[0], 101, "20 TF is on the data grid, the lab's 100 kW and the winch's 1");
    assert_eq!(lab(&f).center_text().unwrap(), "Researching AI Research");
    let i = f.processors.iter().position(|p| p.is_ai_lab()).unwrap();
    assert_eq!(f.compute_line(i).unwrap(), "Data grid: 20 TF wanted of 100 TF supplied · 100%");
}

#[test]
fn without_compute_on_its_grid_it_does_not_work() {
    let mut f = rig(AI_LAB, false);
    run(&mut f, 40.0);
    assert_eq!(f.research.progress(AI_TECH), 0);
    assert!(lab(&f).center_text().unwrap().starts_with("No power or no compute"), "{:?}", lab(&f).center_text());
    assert_eq!(lab(&f).input.count(PACKS[0]), 64, "nothing was taken");
}

#[test]
fn a_center_and_a_small_lab_say_the_tech_needs_an_ai_lab() {
    let mut f = rig(RESEARCH_CENTER, true);
    let mut world = World::new(1, 2);
    let at = IVec3::new(8, 0, 4);
    f.place(&mut world, LAB, at, 0, at, 0);
    run(&mut f, 5.0);
    assert_eq!(f.research.progress(AI_TECH), 0);
    let center = f.processors.iter().find(|p| p.is_center()).unwrap();
    assert_eq!(center.center_text().unwrap(), "AI Research needs an AI lab");
    assert_eq!(f.labs[0].status_text(&f.research), "AI Research needs an AI lab");
}

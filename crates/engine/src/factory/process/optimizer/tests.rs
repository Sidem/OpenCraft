//! The optimizer: +25% on machines in range while it gets power and compute, no stacking, nothing without a datacenter.
//! The compute producer is a winch row with a `compute` number, leaked into a `'static` spec (as in the AI lab's tests).

use crate::block::{BlockId, COAL_ORE, CONSTRUCTOR, FIBRE_NODE, GENERATOR, OPTIMIZER, POLE, WINCH};
use crate::factory::footprint::SINGLE;
use crate::factory::links::Slot;
use crate::factory::process::ProcessTier;
use crate::factory::tests::{powered, recipe_for, run};
use crate::factory::{add_to, Factory};
use crate::item::{IRON_INGOT, IRON_PLATE};
use crate::math::IVec3;
use crate::world::World;

use super::*;

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

fn place(f: &mut Factory, block: BlockId, pos: IVec3) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos, 0);
}

/// A constructor pressing iron plates at the origin on a 360 kW grid, with an optimizer at each of `optimizers`, a
/// fibre node beside them and, if `datacenter`, a 100 TF producer.
fn rig(optimizers: &[IVec3], datacenter: bool) -> Factory {
    let mut f = Factory::default();
    powered(&mut f);
    for z in 1..6 {
        let gen = IVec3::new(3, 3, z);
        place(&mut f, GENERATOR, gen);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    place(&mut f, CONSTRUCTOR, IVec3::ZERO);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(IRON_PLATE))).is_some());
    f.insert(IVec3::ZERO, IRON_INGOT, 40);
    for &at in optimizers {
        place(&mut f, OPTIMIZER, at);
    }
    place(&mut f, POLE, IVec3::new(6, 3, 3));
    place(&mut f, FIBRE_NODE, IVec3::new(5, 0, 3));
    if datacenter {
        add_to(&mut f.processors, Processor::new(IVec3::new(7, 0, 3), producer(100), 0), &mut f.at, Slot::Process);
    }
    f.dirty = true;
    f
}

const NEAR: IVec3 = IVec3::new(5, 0, 0);

fn plates(f: &mut Factory) -> u32 {
    run(f, 8.1, |_| {}); // 2 s a plate: 4 at full speed
    f.constructor_at(IVec3::ZERO).out.count(IRON_PLATE)
}

#[test]
fn a_machine_in_range_works_a_quarter_faster_while_the_optimizer_gets_its_compute() {
    assert_eq!(plates(&mut rig(&[], true)), 4, "without an optimizer");
    let mut f = rig(&[NEAR], true);
    assert_eq!(plates(&mut f), 5, "1.25 x: 1.6 s a plate");
    assert_eq!(f.constructor_at(IVec3::ZERO).boost, 1250);
    let i = f.processors.iter().position(|p| p.energy() == Energy::Optimizer).unwrap();
    assert_eq!(f.processors[i].optimizer_text().unwrap(), "Machines within 16 blocks run 25% faster");
}

#[test]
fn without_compute_it_gives_nothing_and_says_why() {
    let mut f = rig(&[NEAR], false);
    assert_eq!(plates(&mut f), 4);
    assert_eq!(f.constructor_at(IVec3::ZERO).boost, 1000);
    let p = f.processors.iter().find(|p| p.energy() == Energy::Optimizer).unwrap();
    assert!(p.optimizer_text().unwrap().starts_with("No power or no compute"));
}

#[test]
fn optimizers_do_not_stack() {
    let mut f = rig(&[NEAR, IVec3::new(5, 0, 6)], true);
    assert_eq!(plates(&mut f), 5);
    assert_eq!(f.constructor_at(IVec3::ZERO).boost, 1250);
}

#[test]
fn the_best_optimizer_in_range_counts_and_the_range_is_sixteen_blocks() {
    let list = [(IVec3::new(0, 0, 0), 250), (IVec3::new(10, 0, 0), 100)];
    assert_eq!(bonus_at(&list, IVec3::new(5, 0, 0)), 1250);
    assert_eq!(bonus_at(&list, IVec3::new(26, 0, 0)), 1100, "16 blocks from the weaker one");
    assert_eq!(bonus_at(&list, IVec3::new(27, 0, 0)), 1000, "17 is out");
    assert_eq!(bonus_at(&[], IVec3::ZERO), 1000);
}

#[test]
fn it_multiplies_with_the_machine_speed_research() {
    let mut f = rig(&[NEAR], true);
    (0..10).for_each(|_| f.research.add_unit(64)); // Machine Speed level 10: x1.3
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.constructor_at(IVec3::ZERO).boost, 1625);
}

#[test]
fn it_speeds_up_miners_too() {
    let mut f = rig(&[NEAR], true);
    place(&mut f, crate::block::MINER, IVec3::new(0, 0, 2));
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.miner_at(IVec3::new(0, 0, 2)).boost, 1250);
}

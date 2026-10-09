//! The swarm hub: +50% fleet for drone ports in range while it gets power and compute, no stacking.
//! The compute producer is a winch row with a `compute` number, leaked into a `'static` spec (as in the optimizer's tests).

use crate::block::{BlockId, COAL_ORE, DRONE_PORT, FIBRE_NODE, GENERATOR, POLE, SWARM_HUB, WINCH};
use crate::factory::footprint::SINGLE;
use crate::factory::links::Slot;
use crate::factory::process::ProcessTier;
use crate::factory::tests::{powered, run};
use crate::factory::{add_to, Factory};
use crate::math::IVec3;
use crate::research::TECHS;
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

fn place(f: &mut Factory, block: BlockId, pos: IVec3, tier: u8) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos, tier);
}

/// A Mk4 drone port at the origin on a 360 kW grid, with a hub at each of `hubs`, a fibre node beside them and, if
/// `datacenter`, a 100 TF producer.
fn rig(hubs: &[IVec3], datacenter: bool) -> Factory {
    let mut f = Factory::default();
    powered(&mut f);
    for z in 1..6 {
        let gen = IVec3::new(3, 3, z);
        place(&mut f, GENERATOR, gen, 0);
        assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    }
    place(&mut f, DRONE_PORT, IVec3::ZERO, 3);
    for &at in hubs {
        place(&mut f, SWARM_HUB, at, 0);
    }
    place(&mut f, POLE, IVec3::new(6, 3, 3), 0);
    place(&mut f, FIBRE_NODE, IVec3::new(5, 0, 3), 0);
    if datacenter {
        add_to(&mut f.processors, Processor::new(IVec3::new(7, 0, 3), producer(100), 0), &mut f.at, Slot::Process);
    }
    f.dirty = true;
    f
}

const NEAR: IVec3 = IVec3::new(5, 0, 0);

fn fleet(f: &mut Factory) -> u32 {
    run(f, 0.2, |_| {});
    f.processors.iter().find(|p| p.spec.pick == Pick::Hangar).unwrap().fleet()
}

#[test]
fn a_port_in_range_keeps_half_as_many_drones_again_while_the_hub_gets_its_compute() {
    assert_eq!(fleet(&mut rig(&[], true)), 16, "without a hub");
    let mut f = rig(&[NEAR], true);
    assert_eq!(fleet(&mut f), 24);
    let port = f.processors.iter().find(|p| p.spec.pick == Pick::Hangar).unwrap();
    assert!(port.hangar_text().unwrap().contains("fleet 24 (swarm hub)"));
    let hub = f.processors.iter().find(|p| p.spec.block == SWARM_HUB).unwrap();
    assert!(hub.hub_text().unwrap().contains("50%"));
}

#[test]
fn without_compute_it_gives_nothing_and_says_why() {
    let mut f = rig(&[NEAR], false);
    assert_eq!(fleet(&mut f), 16);
    let hub = f.processors.iter().find(|p| p.spec.block == SWARM_HUB).unwrap();
    assert!(hub.hub_text().unwrap().starts_with("No power or no compute"));
}

#[test]
fn hubs_do_not_stack() {
    let mut f = rig(&[NEAR, IVec3::new(5, 0, 6)], true);
    assert_eq!(fleet(&mut f), 24);
}

#[test]
fn a_port_takes_the_best_hub_in_range_and_the_range_is_twelve_blocks() {
    let list = [(IVec3::new(0, 0, 0), 500), (IVec3::new(10, 0, 0), 200)];
    assert_eq!(bonus_at(&list, IVec3::new(5, 0, 0)), 500);
    assert_eq!(bonus_at(&list, IVec3::new(22, 0, 0)), 200, "12 blocks from the weaker one");
    assert_eq!(bonus_at(&list, IVec3::new(23, 0, 0)), 0, "13 is out");
    assert_eq!(bonus_at(&[], IVec3::ZERO), 0);
}

#[test]
fn the_tech_needs_ai_research_and_swarm_logistics() {
    let by_name = |name: &str| TECHS.iter().position(|t| t.name == name).unwrap() as u8;
    let tech = &TECHS[by_name("Drone Swarms") as usize];
    assert!(tech.needs.contains(&by_name("AI Research")));
    assert!(tech.needs.contains(&by_name("Swarm Logistics")));
}

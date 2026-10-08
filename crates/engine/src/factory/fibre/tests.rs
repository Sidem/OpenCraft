//! The data grid: which nodes join, what hangs on which node, how a shortage is shared, the save, and the lines drawn.
//! The test machines are winches (they always want power) with a `compute` number, leaked into `'static` specs.

use crate::block::{COAL_ORE, FIBRE_NODE, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::footprint::SINGLE;
use crate::factory::links::Slot;
use crate::factory::power::FULL_SPEED;
use crate::factory::process::{Energy, Pick, ProcessSpec, ProcessTier, Processor};
use crate::factory::{add_to, Factory};
use crate::math::{IVec3, Vec3};
use crate::world::World;

use super::*;

fn spec(compute: i32) -> &'static ProcessSpec {
    Box::leak(Box::new(ProcessSpec {
        block: crate::block::WINCH,
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

fn at(x: i32, z: i32) -> IVec3 {
    IVec3::new(x, 3, z)
}

/// A powered factory (a pole and a coal generator) with one machine of `compute` at each of `machines`.
fn plant(nodes: &[IVec3], machines: &[(IVec3, i32)]) -> Factory {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    f.place(&mut world, POLE, at(3, 1), 0, at(3, 1), 0);
    f.place(&mut world, GENERATOR, at(3, 2), 0, at(3, 2), 0);
    f.insert(at(3, 2), COAL_ORE.into(), 64);
    for &n in nodes {
        f.place(&mut world, FIBRE_NODE, n, 0, n, 0);
    }
    for &(pos, compute) in machines {
        add_to(&mut f.processors, Processor::new(pos, spec(compute), 0), &mut f.at, Slot::Process);
    }
    f.dirty = true;
    f.update(&mut world, 1, &mut Vec::new());
    f
}

fn run(f: &mut Factory, ticks: u64) {
    let mut world = World::new(1, 2);
    for t in 0..ticks {
        f.update(&mut world, 2 + t, &mut Vec::new());
    }
}

#[test]
fn nodes_within_twelve_blocks_join_and_a_far_one_stands_alone() {
    let f = plant(&[at(0, 5), at(12, 5), at(24, 5), at(60, 5)], &[]);
    assert_eq!(f.data.node_grid, [0, 0, 0, 1], "a chain of nodes 12 apart is one grid");
    assert_eq!(f.data.links, [(0, 1), (1, 2)]);
    let f = plant(&[at(0, 5), at(13, 5)], &[]);
    assert_eq!(f.data.node_grid, [0, 1], "13 apart is too far");
}

#[test]
fn a_machine_hangs_on_the_nearest_node_within_five_blocks() {
    let f = plant(&[at(0, 5), at(4, 5)], &[(at(5, 2), 50), (at(30, 2), 50), (at(0, 1), 0)]);
    let on = |i: usize| f.data.process_node[i];
    // Node 1 at (4, 5) is 3.2 blocks from (5, 2); node 0 is 4.2.
    assert_eq!(on(0), Some(1));
    assert_eq!(on(1), None, "no node within reach");
    assert_eq!(on(2), None, "a machine with no compute takes no part");
}

#[test]
fn a_shortage_slows_every_consumer_alike_and_a_loner_stops() {
    let producer = (at(0, 2), 100);
    let users = [(at(1, 2), -40), (at(2, 2), -40), (at(4, 2), -40)];
    let mut machines = vec![producer];
    machines.extend(users);
    let mut f = plant(&[at(2, 5)], &machines);
    run(&mut f, 3);
    let speeds: Vec<u32> = f.processors.iter().map(|p| p.speed).collect();
    // 100 TF against 120 wanted: 833 thousandths each (the producer has no compute draw).
    assert_eq!((f.data.supply[0], f.data.demand[0]), (100, 120));
    assert_eq!(speeds[1..], [833, 833, 833]);
    // Only two consumers: all of them at full speed.
    let mut f = plant(&[at(2, 5)], &machines[..3]);
    run(&mut f, 3);
    assert_eq!(f.data.satisfaction(Some(0)), FULL_SPEED);
    assert_eq!([f.processors[1].speed, f.processors[2].speed], [FULL_SPEED; 2]);
    // A consumer with no node at all gets nothing, and says why.
    let mut f = plant(&[], &[(at(1, 2), -40)]);
    run(&mut f, 3);
    assert_eq!(f.processors[0].speed, 0);
    assert_eq!(f.compute_line(0).as_deref(), Some(NO_NODE));
}

#[test]
fn a_producer_gives_what_its_power_allows() {
    let mut f = plant(&[at(2, 5)], &[(at(1, 2), 100), (at(2, 2), -10)]);
    run(&mut f, 3);
    assert_eq!(f.data.supply[0], 100, "fully powered: all its TF");
    f.generators[0].fuel.remove(COAL_ORE.into(), 64);
    f.generators[0].energy = 0;
    run(&mut f, 3);
    assert_eq!(f.data.supply[0], 0, "no power, no compute");
    assert_eq!(f.processors[1].speed, 0);
}

#[test]
fn nodes_save_and_load_and_the_readout_names_the_grid() {
    let f = plant(&[at(0, 5), at(10, 5)], &[(at(1, 2), 100)]);
    assert!(f.describe(at(0, 5)).unwrap().contains("2 nodes, 1 producers, 0 consumers"));
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut world = World::new(1, 2);
    let mut g = Factory::read_state(&mut world, &mut ByteReader::new(&w.bytes)).unwrap();
    g.update(&mut world, 1, &mut Vec::new());
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.data.node_grid, [0, 0]);
    let text = g.describe(at(0, 5)).unwrap();
    assert!(text.contains("2 nodes"), "{text}");
    // A real winch has no compute, so a loaded test machine (a leaked spec) is an ordinary winch again.
    assert_eq!(g.compute_line(0), None);
}

#[test]
fn lines_join_linked_nodes_and_machines_to_their_node() {
    let f = plant(&[at(0, 5), at(10, 5)], &[(at(1, 2), 100)]);
    let mut out = Vec::new();
    f.write_fibre(&mut out, Vec3::ZERO, 100.0);
    let boxes = out.len() / crate::factory::render::INSTANCE_FLOATS;
    assert!(boxes >= 10, "a 10-block line and a 3-block line are many short boxes, got {boxes}");
    let layer = out[8] as u16;
    assert_eq!(layer, tex::FIBRE);
    let mut none = Vec::new();
    f.write_fibre(&mut none, Vec3::new(500.0, 0.0, 0.0), 10.0);
    assert!(none.is_empty(), "nothing drawn out of range");
}

//! Laser links: two power islands 20 blocks apart in open air, an emitter on one shooting along +x at a receiver on the
//! other. Blocks go in and out through player actions, so the `block_changed` hook is the one under test.

use crate::action::Action;
use crate::block::{
    BlockId, COAL_ORE, CONSTRUCTOR, DATA_RECEIVER, FIBRE_NODE, GENERATOR, GLASS, LASER_EMITTER, LASER_MIRROR,
    LASER_RECEIVER, POLE, STONE,
};
use crate::factory::footprint::SINGLE;
use crate::factory::process::{Energy, Pick, ProcessSpec, ProcessTier};
use crate::factory::tests::recipe_for;
use crate::item::{IRON_INGOT, IRON_PLATE};
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};

use super::*;

const P: PlayerId = PlayerId(0);
/// Facing that makes an emitter shoot towards +x.
const EAST: u8 = 3;

fn at(x: i32) -> IVec3 {
    IVec3::new(900 + x, 200, -900)
}

fn put(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing, against: pos });
    assert_eq!(sim.world.block_anywhere_or_generate(pos), block, "placed at {pos:?}");
}

fn run(sim: &mut Sim, ticks: u32) {
    (0..ticks).for_each(|_| sim.step());
}

/// A generator and an emitter on one pole at x 0..6, and a receiver and a working constructor on another at x 20..26.
fn islands(receiver_x: i32, receiver_facing: u8) -> Sim {
    let mut sim = Sim::new(7, 2);
    put(&mut sim, GENERATOR, at(0), 0);
    sim.factory.insert(at(0), COAL_ORE.into(), 64);
    put(&mut sim, POLE, at(3), 0);
    put(&mut sim, LASER_EMITTER, at(6), EAST);
    put(&mut sim, POLE, at(receiver_x + 3), 0);
    put(&mut sim, LASER_RECEIVER, at(receiver_x), receiver_facing);
    put(&mut sim, CONSTRUCTOR, at(receiver_x + 6), 0);
    assert!(sim.factory.set_recipe(at(receiver_x + 6), Some(recipe_for(IRON_PLATE))).is_some());
    sim.factory.insert(at(receiver_x + 6), IRON_INGOT, 40);
    run(&mut sim, 2);
    sim
}

fn grid_of(f: &Factory, pole: IVec3) -> usize {
    let i = f.poles.iter().position(|p| p.pos == pole).unwrap();
    f.power.pole_grid[i] as usize
}

fn demand(f: &Factory, pole: IVec3) -> u32 {
    f.power.demand[grid_of(f, pole)]
}

#[test]
fn a_clear_beam_joins_the_two_grids_and_a_block_in_the_way_cuts_them_apart() {
    let mut sim = islands(20, 0);
    let f = &sim.factory;
    assert_eq!(f.beams.links.len(), 1);
    assert_eq!(f.beams.rays[0].end, End::Linked(f.beams.links[0].1));
    assert_eq!(grid_of(f, at(3)), grid_of(f, at(23)), "one grid through the beam");
    put(&mut sim, STONE, at(12), 0);
    run(&mut sim, 1);
    let f = &sim.factory;
    assert!(f.beams.links.is_empty());
    assert!(matches!(f.beams.rays[0].end, End::Blocked(p, STONE) if p == at(12)));
    assert_ne!(grid_of(f, at(3)), grid_of(f, at(23)), "two grids again");
    sim.apply(P, Action::BreakBlock { pos: at(12) });
    run(&mut sim, 1);
    assert_eq!(grid_of(&sim.factory, at(3)), grid_of(&sim.factory, at(23)), "breaking the block restores the link");
}

#[test]
fn glass_lets_the_beam_through() {
    let mut sim = islands(20, 0);
    put(&mut sim, GLASS, at(12), 0);
    run(&mut sim, 1);
    assert_eq!(sim.factory.beams.links.len(), 1);
}

#[test]
fn the_receiver_takes_the_beam_from_any_way_it_faces() {
    for facing in 0..4 {
        let sim = islands(20, facing);
        assert_eq!(sim.factory.beams.links.len(), 1, "receiver facing {facing}");
    }
}

#[test]
fn the_receiving_side_costs_a_ninth_more_on_the_emitting_grid() {
    let mut sim = islands(20, 0);
    let joined = demand(&sim.factory, at(3));
    put(&mut sim, STONE, at(12), 0);
    run(&mut sim, 1);
    let alone = demand(&sim.factory, at(23));
    assert!(alone > 0 && alone < joined);
    assert_eq!(joined, alone + alone / LOSS_DIVISOR, "10% of what is delivered is lost on the way");
}

#[test]
fn a_block_change_off_the_line_does_not_recast_and_one_on_it_does() {
    let mut sim = islands(20, 0);
    assert!(!sim.factory.beams.stale);
    put(&mut sim, STONE, at(12) + IVec3::new(0, 1, 0), 0);
    assert!(!sim.factory.beams.stale, "a block beside the beam");
    put(&mut sim, STONE, at(-1), 0);
    assert!(!sim.factory.beams.stale, "a block behind the emitter");
    put(&mut sim, GLASS, at(12), 0);
    assert!(sim.factory.beams.stale, "a block on the beam");
    run(&mut sim, 1);
    assert!(!sim.factory.beams.stale);
}

#[test]
fn a_receiver_out_of_range_is_not_reached() {
    let mut sim = islands(RANGE + 7, 0); // the emitter is at x 6
    assert!(sim.factory.beams.links.is_empty());
    assert_eq!(sim.factory.beams.rays[0].end, End::Open);
    let text = sim.factory.beam_line(emitter_index(&sim.factory)).unwrap();
    assert!(text.starts_with("Nothing in line"), "{text}");
    sim.apply(P, Action::BreakBlock { pos: at(RANGE + 7) });
    put(&mut sim, LASER_RECEIVER, at(RANGE + 6), 0);
    run(&mut sim, 1);
    assert_eq!(sim.factory.beams.links.len(), 1, "the last block in range counts");
}

/// The emitter's beam goes east along z -900, a mirror at x 16 sends it south to z -892, and a second one sends it east
/// again to a receiver at x 26: `corners` are the mirrors' facings (even turns east into south and south into east).
fn bent_islands(corners: [u8; 2]) -> Sim {
    let mut sim = Sim::new(7, 2);
    let z = |dz: i32| at(0) + IVec3::new(0, 0, dz);
    let x = |x: i32, dz: i32| z(dz) + IVec3::new(x, 0, 0);
    put(&mut sim, GENERATOR, at(0), 0);
    sim.factory.insert(at(0), COAL_ORE.into(), 64);
    put(&mut sim, POLE, at(3), 0);
    put(&mut sim, LASER_EMITTER, at(6), EAST);
    put(&mut sim, LASER_MIRROR, at(16), corners[0]);
    put(&mut sim, LASER_MIRROR, x(16, 8), corners[1]);
    put(&mut sim, POLE, x(29, 8), 0);
    put(&mut sim, LASER_RECEIVER, x(26, 8), 0);
    put(&mut sim, CONSTRUCTOR, x(32, 8), 0);
    assert!(sim.factory.set_recipe(x(32, 8), Some(recipe_for(IRON_PLATE))).is_some());
    sim.factory.insert(x(32, 8), IRON_INGOT, 40);
    run(&mut sim, 2);
    sim
}

#[test]
fn a_beam_goes_round_two_mirrors_and_joins_the_grids_at_the_end() {
    let mut sim = bent_islands([0, 0]);
    let f = &sim.factory;
    let ends: Vec<End> = f.beams.rays.iter().map(|r| r.end).collect();
    assert!(matches!(ends[..], [End::Turned(_), End::Turned(_), End::Linked(_)]), "{ends:?}");
    assert_eq!(f.beams.links.len(), 1);
    let (e, r) = f.beams.links[0];
    assert_eq!(grid_of(f, at(3)), grid_of(f, at(29) + IVec3::new(0, 0, 8)), "one grid through two corners");
    assert_eq!(
        f.beam_line(e as usize).unwrap().split(':').next().unwrap(),
        "Beam to a receiver 28 blocks away (round 2 mirrors)"
    );
    assert_eq!(f.beam_line(r as usize).unwrap(), "Fed by an emitter 28 blocks away (round 2 mirrors)");
    let mirror = f.processors.iter().position(|p| p.spec.block == LASER_MIRROR).unwrap();
    assert!(f.beam_line(mirror).unwrap().starts_with("Turning 1 beam. Joins its west and south sides"));
    assert_eq!(f.processors[mirror].status, Status::Working);

    // Breaking the second mirror cuts the beam where it stands; the first still lights.
    sim.apply(P, Action::BreakBlock { pos: at(16) + IVec3::new(0, 0, 8) });
    run(&mut sim, 1);
    let f = &sim.factory;
    assert!(f.beams.links.is_empty());
    assert_eq!(f.beams.rays.len(), 2);
    assert!(matches!(f.beams.rays[1].end, End::Open));
}

#[test]
fn a_mirror_turned_the_other_way_sends_the_beam_the_other_way() {
    assert_eq!(reflect(IVec3::new(1, 0, 0), 0), Some(IVec3::new(0, 0, 1)));
    assert_eq!(reflect(IVec3::new(1, 0, 0), 1), Some(IVec3::new(0, 0, -1)));
    assert_eq!(reflect(IVec3::new(0, 0, 1), 2), Some(IVec3::new(1, 0, 0)));
    assert_eq!(reflect(IVec3::new(0, 0, -1), 3), Some(IVec3::new(1, 0, 0)));
    assert_eq!(reflect(IVec3::new(0, 1, 0), 0), None, "a beam going up passes no mirror");
    let sim = bent_islands([1, 0]);
    assert!(sim.factory.beams.links.is_empty(), "the first mirror sent the beam north, away from the receiver");
    assert_eq!(sim.factory.beams.rays.len(), 2);
}

#[test]
fn a_mirror_turns_with_r_and_the_beam_follows() {
    let mut sim = bent_islands([1, 0]);
    sim.apply(P, Action::Rotate { pos: at(16) });
    run(&mut sim, 1);
    assert_eq!(sim.factory.beams.links.len(), 1, "turned to the other diagonal, it sends the beam south");
}

/// A producer of 100 TF and a consumer of 40 on their own power, as test machines with a `compute` number.
fn test_machine(sim: &mut Sim, pos: IVec3, compute: i32) {
    let spec: &'static ProcessSpec = Box::leak(Box::new(ProcessSpec {
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
    }));
    let f = &mut sim.factory;
    crate::factory::add_to(&mut f.processors, Processor::new(pos, spec, 0), &mut f.at, Slot::Process);
    f.dirty = true;
}

#[test]
fn a_data_beam_through_two_mirrors_carries_compute_between_two_sites() {
    let mut sim = Sim::new(7, 2);
    let site = |dz: i32, x: i32| at(x) + IVec3::new(0, 0, dz);
    for (dx, dz) in [(0, 0), (24, 12)] {
        put(&mut sim, GENERATOR, site(dz, dx), 0);
        sim.factory.insert(site(dz, dx), COAL_ORE.into(), 64);
        put(&mut sim, POLE, site(dz, dx + 3), 0);
    }
    put(&mut sim, LASER_EMITTER, at(6), EAST);
    put(&mut sim, FIBRE_NODE, at(6) + IVec3::new(0, 0, 3), 0);
    put(&mut sim, LASER_MIRROR, at(16), 0);
    put(&mut sim, LASER_MIRROR, site(8, 16), 0);
    put(&mut sim, DATA_RECEIVER, site(8, 26), 0);
    put(&mut sim, FIBRE_NODE, site(11, 26), 0);
    test_machine(&mut sim, site(2, 4), 100);
    test_machine(&mut sim, site(10, 29), -40);
    run(&mut sim, 3);
    let f = &sim.factory;
    let speed_of = |compute: i32| f.processors.iter().find(|p| p.spec.compute == compute).unwrap().speed;
    assert_eq!(f.beams.links.len(), 1);
    assert_eq!(f.data.joined.len(), 1, "both ends have a fibre node in reach");
    assert_eq!(f.data.node_grid[0], f.data.node_grid[1], "the two sites are one data grid");
    assert!(speed_of(-40) > 0, "the consumer runs on the producer's compute across the beam");
    let (e, r) = f.beams.links[0];
    assert!(f.beam_line(e as usize).unwrap().contains("(round 2 mirrors): it joins the two data grids"));
    assert!(f.beam_line(r as usize).unwrap().starts_with("Fed by an emitter 28 blocks away"));
    assert_eq!(f.power.pole_grid.len(), 2);
    assert_ne!(f.power.pole_grid[0], f.power.pole_grid[1], "a data beam joins no power grids");

    // A block in the beam cuts the sites apart, and the consumer stops.
    put(&mut sim, STONE, at(11), 0);
    run(&mut sim, 3);
    let f = &sim.factory;
    assert_ne!(f.data.node_grid[0], f.data.node_grid[1]);
    assert_eq!(f.processors.iter().find(|p| p.spec.compute == -40).unwrap().speed, 0);
}

#[test]
fn a_data_receiver_with_no_fibre_node_beside_it_joins_nothing() {
    let mut sim = Sim::new(7, 2);
    put(&mut sim, LASER_EMITTER, at(6), EAST);
    put(&mut sim, FIBRE_NODE, at(6) + IVec3::new(0, 0, 3), 0);
    put(&mut sim, DATA_RECEIVER, at(20), 0);
    run(&mut sim, 2);
    let f = &sim.factory;
    assert_eq!(f.beams.links.len(), 1);
    assert!(f.data.joined.is_empty());
    let (e, _) = f.beams.links[0];
    assert!(f.beam_line(e as usize).unwrap().contains("each need a fibre node within 5 blocks"));
}

fn emitter_index(f: &Factory) -> usize {
    f.processors.iter().position(|p| p.spec.block == LASER_EMITTER).unwrap()
}

#[test]
fn the_readouts_say_what_the_beam_does() {
    let mut sim = islands(20, 0);
    let (e, r) = sim.factory.beams.links[0];
    assert!(sim.factory.beam_line(e as usize).unwrap().starts_with("Beam to a receiver 14 blocks away"));
    assert_eq!(sim.factory.beam_line(r as usize).unwrap(), "Fed by an emitter 14 blocks away");
    put(&mut sim, STONE, at(12), 0);
    run(&mut sim, 1);
    assert!(sim.factory.beam_line(e as usize).unwrap().starts_with("Beam blocked by"));
    assert!(sim.factory.beam_line(r as usize).unwrap().starts_with("No beam reaches it"));
    let other = sim.factory.processors.iter().position(|p| p.spec.block == CONSTRUCTOR).unwrap();
    assert_eq!(sim.factory.beam_line(other), None, "other machines have no beam line");
}

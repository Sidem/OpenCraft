//! Laser links: two power islands 20 blocks apart in open air, an emitter on one shooting along +x at a receiver on the
//! other. Blocks go in and out through player actions, so the `block_changed` hook is the one under test.

use crate::action::Action;
use crate::block::{BlockId, COAL_ORE, CONSTRUCTOR, GENERATOR, GLASS, LASER_EMITTER, LASER_RECEIVER, POLE, STONE};
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

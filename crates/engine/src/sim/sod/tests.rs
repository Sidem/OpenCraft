//! Sod: digging bares the turf faces it uncovers, grass grows back over them, covering forgets them; on bare cores.

use super::*;
use crate::action::Action;
use crate::block::{DIRT, GRASS, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::sim::tests::SEED;
use crate::sim::PlayerId;
use crate::TICK_RATE;

const A: PlayerId = PlayerId(0);
/// High enough to be open sky everywhere near the origin.
const SKY: i32 = 230;
/// The `-x` face's bit: the face of the cell at `+x` that looks at the origin.
const WEST: u8 = 1 << 1;

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn run(sim: &mut Sim, seconds: u32) {
    for _ in 0..seconds * TICK_RATE {
        sim.step();
        sim.events.clear();
    }
}

/// A 7 × 7 lawn on a stone slab in the sky: stone, dirt, grass.
fn lawn(sim: &mut Sim) {
    for z in -3..=3 {
        for x in -3..=3 {
            sim.world.set_block_anywhere(at(x, SKY - 2, z), STONE);
            sim.world.set_block_anywhere(at(x, SKY - 1, z), DIRT);
            sim.world.set_block_anywhere(at(x, SKY, z), GRASS);
        }
    }
}

#[test]
fn a_dug_hole_shows_soil_walls_that_grass_grows_back_over_on_every_core() {
    let (mut a, mut b) = (Sim::new(SEED, 2), Sim::new(SEED, 2));
    for sim in [&mut a, &mut b] {
        lawn(sim);
        sim.apply(A, Action::BreakBlock { pos: at(0, SKY, 0) });
    }
    let wall = at(1, SKY, 0);
    assert_eq!(a.world.bare_anywhere(wall), WEST, "only the face toward the hole");
    assert_eq!(a.world.bare_anywhere(at(-1, SKY, 0)), 1 << 0);
    assert_eq!(a.world.bare_anywhere(at(1, SKY, 1)), 0, "a diagonal neighbour keeps its faces");
    assert!(a.timers.pending(wall, TimerKind::SodGrow));
    run(&mut a, 80);
    assert_eq!(a.world.bare_anywhere(wall), WEST, "still soil while the hole is fresh");
    run(&mut a, 520);
    run(&mut b, 600);
    assert_eq!(a.state_hash(), b.state_hash());
    let walls = [at(1, SKY, 0), at(-1, SKY, 0), at(0, SKY, 1), at(0, SKY, -1)];
    assert!(walls.iter().all(|&p| a.world.bare_anywhere(p) == 0), "grown back within ten minutes");
}

#[test]
fn filling_the_hole_forgets_the_bare_faces() {
    let mut sim = Sim::new(SEED, 2);
    lawn(&mut sim);
    sim.apply(A, Action::BreakBlock { pos: at(0, SKY, 0) });
    sim.apply(A, Action::Give { item: STONE.into(), count: 1 });
    let slot = sim.player(A).unwrap().inventory.slots.iter().position(|s| s.item == STONE.into()).unwrap();
    sim.apply(A, Action::PlaceBlock { pos: at(0, SKY, 0), slot: slot as u8, facing: 0, against: at(0, SKY - 1, 0) });
    assert_eq!(sim.block(at(0, SKY, 0)), STONE);
    assert_eq!(sim.world.bare_anywhere(at(1, SKY, 0)), 0);
}

#[test]
fn breaking_a_bare_block_clears_its_faces() {
    let mut sim = Sim::new(SEED, 2);
    lawn(&mut sim);
    sim.apply(A, Action::BreakBlock { pos: at(0, SKY, 0) });
    sim.apply(A, Action::BreakBlock { pos: at(1, SKY, 0) });
    assert_eq!(sim.world.bare_anywhere(at(1, SKY, 0)), 0, "air keeps no sod");
    assert!(!sim.world.set_bare_anywhere(at(1, SKY, 0), WEST), "only turf can be bare");
}

#[test]
fn dirt_turning_to_grass_greens_one_open_face_first() {
    let mut sim = Sim::new(SEED, 2);
    let p = at(0, SKY, 0);
    sim.world.set_block_anywhere(p - at(0, 1, 0), STONE);
    sim.world.set_block_anywhere(p, GRASS);
    sim.sod_sprouted(p);
    assert_eq!(sim.world.bare_anywhere(p).count_ones(), 4, "five open faces, one grown");
    assert!(sim.timers.pending(p, TimerKind::SodGrow));
    run(&mut sim, 900);
    assert_eq!(sim.world.bare_anywhere(p), 0);
}

#[test]
fn bare_faces_survive_a_save() {
    let mut a = Sim::new(SEED, 2);
    lawn(&mut a);
    a.apply(A, Action::BreakBlock { pos: at(0, SKY, 0) });
    let mut w = ByteWriter::default();
    a.write_state(&mut w);
    let mut b = Sim::new(SEED, 2);
    b.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(b.world.bare_anywhere(at(1, SKY, 0)), WEST);
    assert_eq!(a.state_hash(), b.state_hash());
}

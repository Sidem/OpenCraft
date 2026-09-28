//! Footprints: their geometry and ports, an assembler fed from three belts, and placing and breaking
//! one in a whole core (`Sim`) as a player does.

use super::*;
use crate::action::Action;
use crate::block::{ASSEMBLER, MACHINE_PART, STONE};
use crate::factory::links::Link;
use crate::factory::process::spec;
use crate::factory::tests::{powered, recipe_for, run};
use crate::item::{ItemId, COPPER_WIRE, GEAR, IRON_ROD, MOTOR};
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::world::World;

const NORTH: u8 = 0;
const EAST: u8 = 1;
const SOUTH: u8 = 2;
const WEST: u8 = 3;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn assembler() -> &'static Footprint {
    &spec(ASSEMBLER).unwrap().footprint
}

#[test]
fn a_footprint_runs_right_and_away_from_its_placer_and_turns_with_them() {
    // Facing north: right is +x, away is -z.
    let north = assembler().cells(IVec3::ZERO, NORTH);
    assert_eq!(north.len(), 8);
    assert_eq!(&north[..4], &[v(0, 0, 0), v(1, 0, 0), v(0, 0, -1), v(1, 0, -1)]);
    assert_eq!(north[4], v(0, 1, 0));
    // Facing east: right is +z, away is +x.
    assert_eq!(&assembler().cells(IVec3::ZERO, EAST)[..4], &[v(0, 0, 0), v(0, 0, 1), v(1, 0, 0), v(1, 0, 1)]);
    let c = assembler().centre(NORTH);
    assert_eq!((c.x, c.y, c.z), (0.5, 0.5, -0.5));
    assert_eq!(SINGLE.cells(v(3, 4, 5), WEST), vec![v(3, 4, 5)]);
}

#[test]
fn ports_sit_on_the_bottom_layer_of_their_side() {
    let fp = assembler();
    // Placed facing north its front faces south, back to the placer's view.
    let mut out = fp.faces(IVec3::ZERO, NORTH, Role::Out);
    out.sort_by_key(|f| f.0.x);
    assert_eq!(out, vec![(v(0, 0, 0), SOUTH), (v(1, 0, 0), SOUTH)]);
    let inlets = fp.faces(IVec3::ZERO, NORTH, Role::In);
    assert_eq!(inlets.len(), 6, "back, left and right: two cells each");
    assert!(fp.takes(IVec3::ZERO, NORTH, v(0, 0, -1), v(0, 0, -2)), "from behind");
    assert!(fp.takes(IVec3::ZERO, NORTH, v(0, 0, 0), v(-1, 0, 0)), "from the left");
    assert!(!fp.takes(IVec3::ZERO, NORTH, v(0, 0, 0), v(0, 0, 1)), "not into the outlet");
    assert!(!fp.takes(IVec3::ZERO, NORTH, v(0, 1, 0), v(-1, 1, 0)), "not into the upper layer");
    assert!(SINGLE.takes(IVec3::ZERO, NORTH, IVec3::ZERO, v(0, 1, 0)));
}

/// A box of `n` `item` at `from` and a belt from it into the cell ahead.
fn feed(f: &mut Factory, from: IVec3, dir: u8, item: ItemId, n: u32) {
    f.add_storage(from);
    f.stock(from, item, n);
    f.add_belt(from + DIRS[dir as usize], dir);
}

#[test]
fn an_assembler_takes_parts_from_three_belts_and_gives_motors_at_its_front() {
    let mut f = Factory::default();
    powered(&mut f);
    f.place(&mut World::new(1, 2), ASSEMBLER, IVec3::ZERO, NORTH, IVec3::ZERO, 0);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(MOTOR))).is_some());
    feed(&mut f, v(0, 0, -3), SOUTH, IRON_ROD, 3);
    feed(&mut f, v(-2, 0, 0), EAST, GEAR, 6);
    feed(&mut f, v(3, 0, -1), WEST, COPPER_WIRE, 12);
    f.add_belt(v(0, 0, 1), SOUTH);
    f.add_storage(v(0, 0, 2));
    // A belt pointing into the front (an outlet) is not linked.
    f.add_belt(v(1, 0, 1), NORTH);
    run(&mut f, 20.0, |_| {});
    assert_eq!(f.storage_count_at(v(0, 0, 2), MOTOR), 3, "three motors at 5 s each");
    assert_eq!(f.belt_at(v(1, 0, 1)).out, Link::None);
    // Every cell names the machine; power reached it through its cells (20 kW while working).
    assert!(f.describe(v(1, 1, -1)).is_some_and(|t| t.contains("Waiting for")));
    assert_eq!(f.block_at(v(1, 1, -1)), Some(ASSEMBLER));
}

#[test]
fn an_assembler_holds_at_most_a_stack_of_each_input() {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), ASSEMBLER, IVec3::ZERO, NORTH, IVec3::ZERO, 0);
    f.set_recipe(IVec3::ZERO, Some(recipe_for(MOTOR)));
    assert_eq!(f.insert(IVec3::ZERO, COPPER_WIRE, 200), 64, "wire can't take the other inputs' slots");
    assert_eq!(f.insert(v(1, 1, -1), GEAR, 10), 10, "any cell takes items by hand");
}

const P: PlayerId = PlayerId(0);
/// On hand-built ground far from spawn: stone to y 100, air above.
const A: IVec3 = IVec3::new(-4000, 101, 4000);

fn flat_ground() -> Sim {
    let mut sim = Sim::new(1337, 2);
    for x in -4..6 {
        for z in -6..4 {
            for y in 96..=108 {
                sim.world.set_block_anywhere(A + v(x, y - 101, z), if y <= 100 { STONE } else { crate::block::AIR });
            }
        }
    }
    sim.apply(P, Action::Give { item: ASSEMBLER.into(), count: 1 });
    sim
}

fn place(sim: &mut Sim) {
    sim.apply(P, Action::PlaceBlock { pos: A, slot: 0, facing: NORTH, against: A - v(0, 1, 0) });
}

#[test]
fn placing_needs_every_cell_free_and_fills_the_rest_with_parts() {
    let mut sim = flat_ground();
    sim.world.set_block_anywhere(A + v(1, 1, -1), STONE);
    place(&mut sim);
    assert_eq!(sim.world.block_anywhere_or_generate(A), crate::block::AIR, "a blocked cell refuses");
    assert_eq!(sim.factory.count(crate::factory::Kind::Process), 0);
    sim.world.set_block_anywhere(A + v(1, 1, -1), crate::block::AIR);
    place(&mut sim);
    let cells = assembler().cells(A, NORTH);
    assert_eq!(sim.world.block_anywhere_or_generate(cells[0]), ASSEMBLER);
    assert!(cells[1..].iter().all(|&c| sim.world.block_anywhere_or_generate(c) == MACHINE_PART));
    assert_eq!(sim.players[0].as_ref().unwrap().inventory.count(ASSEMBLER.into()), 0);
}

#[test]
fn breaking_any_cell_returns_the_assembler_with_what_it_held() {
    let mut sim = flat_ground();
    place(&mut sim);
    sim.factory.set_recipe(A, Some(recipe_for(MOTOR)));
    assert_eq!(sim.factory.insert(A, GEAR, 5), 5);
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos: A + v(1, 1, -1) });
    for c in assembler().cells(A, NORTH) {
        assert_eq!(sim.world.block_anywhere_or_generate(c), crate::block::AIR, "{c:?} cleared");
    }
    let dropped: Vec<(ItemId, u32)> = sim
        .events
        .iter()
        .filter_map(|e| match *e {
            SimEvent::Dropped { item, count, .. } => Some((item, count)),
            _ => None,
        })
        .collect();
    assert_eq!(dropped, vec![(ASSEMBLER.into(), 1), (GEAR, 5)]);
    assert_eq!(sim.factory.count(crate::factory::Kind::Process), 0);
    assert_eq!(sim.factory.block_at(A), None);
}

#[test]
fn a_placed_assembler_survives_a_save_round_trip_facing_its_way() {
    use crate::bytes::{ByteReader, ByteWriter};
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), ASSEMBLER, IVec3::ZERO, EAST, IVec3::ZERO, 0);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.footprint_at(v(1, 1, 1)).map(|f| (f.0, f.1)), Some((IVec3::ZERO, ASSEMBLER)));
}

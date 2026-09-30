use super::*;
use crate::block::{GENERATOR, LAB, MINER, POLE};
use crate::world::World;

fn put(f: &mut Factory, block: u8, pos: IVec3) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos, 0);
}

/// A factory whose wires are all made by hand.
fn fresh() -> Factory {
    Factory { by_hand: true, ..Factory::default() }
}

fn at(x: i32) -> IVec3 {
    IVec3::new(x, 3, 0)
}

/// A generator with fuel at `gen`, so its grid counts as powered.
fn source(f: &mut Factory, gen: IVec3) {
    put(f, GENERATOR, gen);
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
}

#[test]
fn nothing_is_wired_by_itself() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    put(&mut f, GENERATOR, at(1));
    put(&mut f, MINER, at(2));
    put(&mut f, POLE, at(4));
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
    assert_eq!(f.power.pole_grid, [0, 1], "poles in range stay apart");
    assert_eq!(f.power.gen_pole, [None]);
    assert_eq!(f.power.miner_pole, [None]);
}

#[test]
fn a_click_wires_a_machine_and_a_full_pole_refuses() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    for x in 1..=5 {
        put(&mut f, MINER, at(x));
    }
    for x in 1..=4 {
        assert_eq!(f.hookup(at(0), at(x)), Hookup::Connect);
        f.connect(at(0), at(x));
    }
    assert_eq!((f.slots_used(at(0)), f.pole_slots(at(0))), (4, Some(4)));
    assert_eq!(f.hookup(at(0), at(5)), Hookup::PoleFull);
    f.connect(at(0), at(5));
    assert_eq!(f.slots_used(at(0)), 4);
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
    assert_eq!(f.power.miner_pole, [Some(0), Some(0), Some(0), Some(0), None]);
    assert_eq!(f.hookup(at(0), at(1)), Hookup::Disconnect);
    f.disconnect(at(0), at(1));
    assert_eq!(f.slots_used(at(0)), 3);
}

#[test]
fn wiring_needs_reach_and_a_machine_that_takes_power() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    put(&mut f, MINER, at(6));
    put(&mut f, crate::block::BELT, at(2));
    assert_eq!(f.hookup(at(0), at(6)), Hookup::TooFar, "a Mk1 pole reaches 5");
    assert_eq!(f.hookup(at(0), at(2)), Hookup::Nothing, "belts need no power");
}

#[test]
fn a_machine_moves_to_another_pole_and_wires_follow_removals() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    put(&mut f, POLE, at(4));
    put(&mut f, LAB, at(2));
    f.connect(at(0), at(2));
    assert_eq!(f.hookup(at(4), at(2)), Hookup::Move(at(0)));
    f.connect(at(4), at(2));
    assert_eq!((f.slots_used(at(0)), f.slots_used(at(4))), (0, 1));
    f.remove(at(2));
    assert_eq!(f.slots_used(at(4)), 0, "a removed machine frees its slot");
    f.connect(at(0), at(4));
    assert_eq!(f.slots_used(at(0)), 1);
    f.remove(at(4));
    assert_eq!(f.slots_used(at(0)), 0, "so does a removed pole");
}

#[test]
fn a_new_pole_wires_itself_to_the_nearest_powered_pole_only() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    put(&mut f, POLE, at(3));
    assert_eq!(f.slots_used(at(0)), 0, "no power yet: nothing to wire to");
    // A generator on the pole at 0 makes its grid powered; the pole at 3 is on no grid.
    put(&mut f, GENERATOR, at(-1));
    f.connect(at(0), at(-1));
    f.update(&mut World::new(1, 2), 1, &mut Vec::new());
    put(&mut f, POLE, at(6));
    assert_eq!(f.slots_used(at(3)), 0, "an unpowered pole is passed over");
    assert_eq!(f.slots_used(at(6)), 1, "the powered one is in range");
    put(&mut f, POLE, at(2));
    assert_eq!(f.slots_used(at(2)), 1, "wired once, to the nearest powered pole (at 0)");
    assert_eq!(f.slots_used(at(0)), 3, "the generator, the pole at 6 and the one at 2");
    f.update(&mut World::new(1, 2), 2, &mut Vec::new());
    let grid = |x| f.poles.iter().position(|p| p.pos == at(x)).map(|i| f.power.pole_grid[i]);
    assert_eq!((grid(0), grid(2), grid(3)), (grid(0), grid(0), grid(3)));
    assert_ne!(grid(3), grid(0));
}

#[test]
fn old_saves_are_wired_by_range() {
    let mut f = fresh();
    source(&mut f, at(1));
    put(&mut f, POLE, at(0));
    put(&mut f, POLE, at(9));
    put(&mut f, MINER, at(12));
    put(&mut f, MINER, at(30));
    f.hook_by_reach();
    f.update(&mut World::new(1, 2), 2, &mut Vec::new());
    assert_eq!(f.power.pole_grid[0], f.power.pole_grid[1]);
    assert_eq!(f.power.gen_pole[0], Some(0));
    assert_eq!(f.power.miner_pole, [Some(1), None]);
}

#[test]
fn wires_survive_a_save() {
    let mut f = fresh();
    put(&mut f, POLE, at(0));
    put(&mut f, MINER, at(2));
    f.connect(at(0), at(2));
    let mut w = crate::bytes::ByteWriter::default();
    f.write_state(&mut w);
    let mut r = crate::bytes::ByteReader::new(&w.bytes);
    let back = Factory::read_state(&mut World::new(1, 2), &mut r).expect("reads");
    assert_eq!(back.slots_used(at(0)), 1);
}

use super::*;
use crate::action::Action;
use crate::block::STONE;

const P: PlayerId = PlayerId(0);

/// A stone ledge high in the air at (0, 200, 0) with a floor 12 blocks below it, out of any loaded chunk.
fn ledge(sim: &mut Sim) -> IVec3 {
    let top = IVec3::new(0, 200, 0);
    sim.world.set_block_anywhere(top, STONE);
    for x in -1..=2 {
        sim.world.set_block_anywhere(IVec3::new(x, 188, 0), STONE);
    }
    top
}

fn coils(sim: &Sim) -> u32 {
    sim.player(P).unwrap().inventory.count(ROPE_ITEM)
}

#[test]
fn a_coil_hangs_eight_blocks_from_a_ledge_and_a_second_reaches_the_floor() {
    let mut sim = Sim::new(7, 2);
    let top = ledge(&mut sim);
    sim.apply(P, Action::Give { item: ROPE_ITEM, count: 3 });
    let side = top + IVec3::new(1, 0, 0);
    sim.apply(P, Action::HangRope { pos: side, slot: 0 });
    let rope_at = |sim: &mut Sim, y: i32| sim.world.block_anywhere_or_generate(IVec3::new(1, y, 0)) == ROPE;
    assert!((193..=200).all(|y| rope_at(&mut sim, y)) && !rope_at(&mut sim, 192), "eight cells from the ledge");
    assert_eq!(coils(&sim), 2);
    // Aimed at the strand, the next coil carries on from its bottom and stops at the floor (y 188).
    sim.apply(P, Action::HangRope { pos: side, slot: 0 });
    assert!((189..=200).all(|y| rope_at(&mut sim, y)), "down to the floor");
    assert_eq!(coils(&sim), 1);
}

#[test]
fn a_rope_needs_something_to_hang_from_and_rope_in_hand() {
    let mut sim = Sim::new(7, 2);
    let air = IVec3::new(40, 220, 40);
    sim.apply(P, Action::HangRope { pos: air, slot: 0 });
    assert_eq!(sim.world.block_anywhere_or_generate(air), AIR, "no coil");
    sim.apply(P, Action::Give { item: ROPE_ITEM, count: 1 });
    sim.apply(P, Action::HangRope { pos: air, slot: 0 });
    assert_eq!((sim.world.block_anywhere_or_generate(air), coils(&sim)), (AIR, 1), "nothing holds it up");
}

#[test]
fn breaking_any_cell_takes_the_strand_up_whole_and_never_gives_more_than_went_in() {
    let mut sim = Sim::new(7, 2);
    let top = ledge(&mut sim);
    sim.apply(P, Action::Give { item: ROPE_ITEM, count: 2 });
    let side = top + IVec3::new(1, 0, 0);
    sim.apply(P, Action::HangRope { pos: side, slot: 0 });
    sim.apply(P, Action::HangRope { pos: side, slot: 0 });
    sim.events.clear();
    sim.apply(P, Action::BreakBlock { pos: IVec3::new(1, 192, 0) });
    assert!((189..=200).all(|y| sim.world.block_anywhere_or_generate(IVec3::new(1, y, 0)) == AIR), "all gone");
    let dropped: u32 = sim
        .events
        .iter()
        .filter_map(|e| match e {
            SimEvent::Dropped { item, count, .. } if *item == ROPE_ITEM => Some(*count),
            _ => None,
        })
        .sum();
    assert_eq!(dropped, 2, "12 cells come back as 2 coils");
}

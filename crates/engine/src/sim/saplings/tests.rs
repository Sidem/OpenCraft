//! Saplings: dropped by leaves, planted on soil, grown into trees; on bare cores, compared by hash.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, LEAVES, LOG, STONE};
use crate::item::ItemId;
use crate::sim::tests::SEED;
use crate::sim::PlayerId;
use crate::TICK_RATE;

const A: PlayerId = PlayerId(0);
/// High enough to be open sky everywhere near the origin.
const SKY: i32 = 200;

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// Runs `seconds`, counting the trees that grew.
fn run(sim: &mut Sim, seconds: u32) -> usize {
    let mut grew = 0;
    for _ in 0..seconds * TICK_RATE {
        sim.step();
        grew += sim.events.iter().filter(|e| matches!(e, SimEvent::TreeGrew { .. })).count();
        sim.events.clear();
    }
    grew
}

/// Tries to place `block` at `pos` from player A's inventory; whether it is there afterwards.
fn place(sim: &mut Sim, pos: IVec3, block: BlockId) -> bool {
    sim.apply(A, Action::Give { item: block.into(), count: 1 });
    let slot = sim.player(A).unwrap().inventory.slots.iter().position(|s| s.item == block.into()).unwrap();
    sim.apply(A, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos - at(0, 1, 0) });
    sim.block(pos) == block
}

#[test]
fn leaves_sometimes_drop_a_sapling() {
    let mut sim = Sim::new(SEED, 2);
    let n = 1000;
    let mut saplings = 0;
    for i in 0..n {
        let p = at(i % 40, SKY, i / 40);
        sim.world.set_block_anywhere(p, LEAVES);
        sim.apply(A, Action::BreakBlock { pos: p });
        let sapling = |e: &SimEvent| matches!(e, SimEvent::Dropped { item, .. } if *item == ItemId::from(SAPLING));
        saplings += sim.events.iter().filter(|e| sapling(e)).count();
        sim.events.clear();
    }
    let expected = (n as f64 * SAPLING_CHANCE) as usize;
    assert!(saplings > expected / 2 && saplings < expected * 2, "{saplings} saplings from {n} leaves");
}

#[test]
fn a_planted_sapling_grows_into_a_tree_on_every_core() {
    let (mut a, mut b) = (Sim::new(SEED, 2), Sim::new(SEED, 2));
    for sim in [&mut a, &mut b] {
        sim.world.set_block_anywhere(at(0, SKY - 1, 0), GRASS);
        assert!(place(sim, at(0, SKY, 0), SAPLING));
        assert!(sim.timers.pending(at(0, SKY, 0), TimerKind::SaplingGrow));
    }
    assert_eq!(run(&mut a, 30), 0, "not before a minute");
    let grew = run(&mut a, 600);
    run(&mut b, 630);
    assert_eq!(grew, 1, "grew within ten minutes");
    assert_eq!(a.state_hash(), b.state_hash());
    let trunk = (SKY..SKY + 8).take_while(|&y| a.block(at(0, y, 0)) == LOG).count();
    assert!((4..=6).contains(&trunk), "trunk {trunk}");
    assert_eq!(a.block(at(0, SKY - 1, 0)), DIRT);
    // The lower wide canopy layer: 5 × 5 around the trunk, some corners trimmed.
    let y = SKY + trunk as i32 - 3;
    let leaves = (-2..=2).flat_map(|z| (-2..=2).map(move |x| (x, z))).filter(|&(x, z)| a.block(at(x, y, z)) == LEAVES);
    assert!(leaves.count() >= 20, "a full canopy");
}

#[test]
fn saplings_need_soil_and_room() {
    let mut sim = Sim::new(SEED, 2);
    sim.world.set_block_anywhere(at(5, SKY - 1, 0), STONE);
    assert!(!place(&mut sim, at(5, SKY, 0), SAPLING), "not on stone");

    // A block over the sapling holds it back; once it's gone the sapling grows.
    sim.world.set_block_anywhere(at(0, SKY - 1, 0), DIRT);
    assert!(place(&mut sim, at(0, SKY, 0), SAPLING));
    sim.world.set_block_anywhere(at(0, SKY + 2, 0), STONE);
    assert_eq!(run(&mut sim, 900), 0);
    assert_eq!(sim.block(at(0, SKY, 0)), SAPLING);
    assert!(sim.timers.pending(at(0, SKY, 0), TimerKind::SaplingGrow), "still trying");
    sim.world.set_block_anywhere(at(0, SKY + 2, 0), crate::block::AIR);
    assert_eq!(run(&mut sim, 900), 1);
    assert_eq!(sim.block(at(0, SKY, 0)), LOG);
}

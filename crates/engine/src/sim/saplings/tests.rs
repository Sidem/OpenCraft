//! Saplings: dropped by leaves, planted on soil, grown into trees; on bare cores, compared by hash.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, LEAVES, OAK_TREE_LOG, SAPLING, STONE};
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

/// The blocks within `r` of `c` in each axis.
fn cube_around(c: IVec3, r: i32) -> Vec<IVec3> {
    (-r..=r).flat_map(|y| (-r..=r).flat_map(move |z| (-r..=r).map(move |x| c + at(x, y, z)))).collect()
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
    // A young oak: a trunk no taller than a sapling grows, limbs and a crown of leaves round its top.
    let trunk = (SKY..SKY + 30).take_while(|&y| a.block(at(0, y, 0)) == OAK_TREE_LOG).count() as i32;
    assert!((4..=crate::worldgen::YOUNG_MAX).contains(&trunk), "trunk {trunk}");
    assert_eq!(a.block(at(0, SKY - 1, 0)), DIRT);
    let crown = cube_around(at(0, SKY + trunk - 1, 0), 5);
    assert!(crown.iter().filter(|&&p| a.block(p) == LEAVES).count() >= 40, "a full crown");
    assert!(crown.iter().filter(|&&p| a.block(p) == OAK_TREE_LOG && (p.x != 0 || p.z != 0)).count() >= 3, "limbs");
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
    assert_eq!(sim.block(at(0, SKY, 0)), OAK_TREE_LOG);
}

#[test]
fn each_species_leaves_and_grows_its_own_sapling() {
    use crate::block::{BIRCH_LEAVES, BIRCH_LOG, BIRCH_SAPLING, PINE_LEAVES, PINE_LOG, PINE_SAPLING};
    let mut sim = Sim::new(SEED, 2);
    for (leaves, sapling) in [(BIRCH_LEAVES, BIRCH_SAPLING), (PINE_LEAVES, PINE_SAPLING), (LEAVES, SAPLING)] {
        assert_eq!(crate::block::sapling_of(leaves), sapling);
        let mut got = 0;
        for i in 0..300 {
            let p = at(i % 40, SKY, i / 40);
            sim.world.set_block_anywhere(p, leaves);
            sim.apply(A, Action::BreakBlock { pos: p });
            got += sim
                .events
                .iter()
                .filter(|e| matches!(e, SimEvent::Dropped { item, .. } if *item == sapling.into()))
                .count();
            sim.events.clear();
        }
        assert!(got > 0, "{leaves} leaves left no sapling");
    }
    // A planted pine sapling grows a pine (its own log, needles round its top), a birch a birch.
    sim.world.set_block_anywhere(at(0, SKY - 1, 100), GRASS);
    sim.world.set_block_anywhere(at(10, SKY - 1, 100), GRASS);
    assert!(place(&mut sim, at(0, SKY, 100), PINE_SAPLING) && place(&mut sim, at(10, SKY, 100), BIRCH_SAPLING));
    assert_eq!(run(&mut sim, 900), 2);
    let trunk = (SKY..SKY + 30).take_while(|&y| sim.block(at(0, y, 100)) == PINE_LOG).count() as i32;
    assert!((4..=crate::worldgen::YOUNG_MAX).contains(&trunk), "pine trunk {trunk}");
    let top = cube_around(at(0, SKY + trunk - 1, 100), 2);
    assert!(top.iter().any(|&p| sim.block(p) == PINE_LEAVES), "needles at the top");
    assert_eq!(sim.block(at(10, SKY, 100)), BIRCH_LOG);
}

//! Torches: where they stand and what happens when their block goes; on a bare core.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, STONE, WATER};
use crate::item::ItemId;
use crate::sim::tests::SEED;
use crate::sim::PlayerId;

const A: PlayerId = PlayerId(0);
/// High enough to be open sky everywhere near the origin.
const SKY: i32 = 200;

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// Tries to place a torch at `pos` from player A's inventory; whether it is there afterwards.
fn place_torch(sim: &mut Sim, pos: IVec3) -> bool {
    sim.apply(A, Action::Give { item: TORCH.into(), count: 1 });
    let slot = sim.player(A).unwrap().inventory.slots.iter().position(|s| s.item == TORCH.into()).unwrap();
    sim.apply(A, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos - UP });
    sim.block(pos) == TORCH
}

fn put(sim: &mut Sim, pos: IVec3, b: BlockId) {
    sim.world.set_block_anywhere(pos, b);
}

#[test]
fn a_torch_stands_only_on_a_solid_block_in_an_empty_cell() {
    let mut sim = Sim::new(SEED, 2);
    assert!(!place_torch(&mut sim, at(0, SKY, 0)), "not in the air");
    put(&mut sim, at(2, SKY - 1, 0), STONE);
    put(&mut sim, at(2, SKY, 0), WATER);
    assert!(!place_torch(&mut sim, at(2, SKY, 0)), "not in water");
    put(&mut sim, at(0, SKY - 1, 0), STONE);
    assert!(place_torch(&mut sim, at(0, SKY, 0)));
}

#[test]
fn a_torch_drops_when_its_block_is_broken() {
    let mut sim = Sim::new(SEED, 2);
    put(&mut sim, at(0, SKY - 1, 0), STONE);
    assert!(place_torch(&mut sim, at(0, SKY, 0)));
    sim.events.clear();
    sim.apply(A, Action::BreakBlock { pos: at(0, SKY - 1, 0) });
    assert_eq!(sim.block(at(0, SKY, 0)), AIR);
    let torch = |e: &SimEvent| matches!(e, SimEvent::Dropped { item, .. } if *item == ItemId::from(TORCH));
    assert_eq!(sim.events.iter().filter(|e| torch(e)).count(), 1, "the torch comes off as an item");
}

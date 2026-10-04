use super::*;
use crate::action::Action;
use crate::block::{BEDROCK, BELT, STORAGE};
use crate::bytes::{ByteReader, ByteWriter};

const P: PlayerId = PlayerId(0);

fn spot() -> IVec3 {
    IVec3::new(500, 200, -500)
}

fn plant(sim: &mut Sim, pos: IVec3, facing: u8) {
    sim.apply(P, Action::PlaceGhost { pos, slot: 0, facing });
}

#[test]
fn a_ghost_costs_nothing_and_leaves_the_world_alone() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    plant(&mut sim, spot(), 2);
    assert_eq!(sim.ghosts.len(), 1);
    assert_eq!(sim.ghosts.covering(spot()).map(|g| (g.block, g.facing)), Some((STORAGE, 2)));
    assert_eq!(sim.player(P).unwrap().inventory.count(STORAGE.into()), 1, "item kept");
    assert!(block::replaceable(sim.world.block_anywhere_or_generate(spot())), "world untouched");
    assert_eq!(sim.ghosts.needs(), vec![(ItemId::block(STORAGE), 1)]);
}

#[test]
fn ghosts_need_free_cells_a_held_block_and_stay_sorted() {
    let mut sim = Sim::new(7, 2);
    plant(&mut sim, spot(), 0);
    assert_eq!(sim.ghosts.len(), 0, "an empty hand plants nothing");
    sim.apply(P, Action::Give { item: BELT.into(), count: 1 });
    plant(&mut sim, IVec3::new(0, 0, 0), 0);
    assert_eq!(sim.ghosts.len(), 0, "bedrock is not free");
    assert_eq!(sim.world.block_anywhere_or_generate(IVec3::new(0, 0, 0)), BEDROCK);
    plant(&mut sim, spot() + IVec3::new(3, 0, 0), 1);
    plant(&mut sim, spot(), 1);
    plant(&mut sim, spot(), 3); // same anchor: replaced
    let order: Vec<_> = sim.ghosts.iter().map(|g| (g.pos.x, g.facing)).collect();
    assert_eq!(order, vec![(500, 3), (503, 1)]);
}

#[test]
fn building_on_a_ghost_takes_its_facing_and_clears_it() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: BELT.into(), count: 2 });
    plant(&mut sim, spot(), 3);
    let against = spot() - IVec3::new(0, 1, 0);
    sim.apply(P, Action::PlaceBlock { pos: spot(), slot: 0, facing: 0, against });
    assert_eq!(sim.world.block_anywhere(spot()), Some(BELT));
    assert_eq!(sim.ghosts.len(), 0, "the ghost is built");
    // The same belt placed straight with the ghost's facing is the same world.
    let mut direct = Sim::new(7, 2);
    direct.apply(P, Action::Give { item: BELT.into(), count: 2 });
    direct.apply(P, Action::PlaceBlock { pos: spot(), slot: 0, facing: 3, against });
    assert_eq!(sim.state_hash(), direct.state_hash(), "ghost facing used");
}

#[test]
fn removing_and_saving_round_trip() {
    let mut sim = Sim::new(7, 2);
    sim.apply(P, Action::Give { item: BELT.into(), count: 1 });
    plant(&mut sim, spot(), 1);
    plant(&mut sim, spot() + IVec3::new(0, 0, 2), 2);
    let mut w = ByteWriter::default();
    sim.ghosts.write_state(&mut w);
    assert_eq!(Ghosts::read_state(&mut ByteReader::new(&w.bytes)), Some(sim.ghosts.clone()));
    sim.apply(P, Action::RemoveGhost { pos: spot() });
    assert_eq!(sim.ghosts.len(), 1);
    sim.apply(P, Action::RemoveGhost { pos: spot() });
    assert_eq!(sim.ghosts.len(), 1, "nothing there");
}

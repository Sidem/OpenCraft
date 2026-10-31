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
fn a_dock_ghost_goes_only_onto_a_rail_node_and_builds_there() {
    use crate::block::{LOADING_DOCK, RAIL, STONE};
    let mut sim = Sim::new(7, 2);
    let node = spot();
    sim.world.set_block_anywhere(node - IVec3::new(0, 1, 0), STONE);
    assert!(sim.put_block(P, node, RAIL.into(), 64, node - IVec3::new(0, 1, 0)));
    sim.apply(P, Action::Give { item: LOADING_DOCK.into(), count: 1 });
    plant(&mut sim, node + IVec3::new(2, 0, 0), 0);
    sim.plant_ghost(node + IVec3::new(0, 1, 0), LOADING_DOCK, 0, 0);
    assert_eq!(sim.ghosts.len(), 0, "no dock ghost off the rails, from the hand or a blueprint");
    plant(&mut sim, node, 0);
    assert_eq!(sim.ghosts.covering(node).map(|g| g.block), Some(LOADING_DOCK), "onto the node");
    sim.apply(P, Action::PlaceBlock { pos: node, slot: 0, facing: 0, against: node - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere(node), Some(LOADING_DOCK));
    assert_eq!(sim.factory.rail_yaw(node), Some(64));
    assert_eq!(sim.ghosts.len(), 0, "the ghost is built");
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

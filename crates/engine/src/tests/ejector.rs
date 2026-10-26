//! A belt ejector in a running game: its drops become loose items that land in front of it and merge into a few stacks,
//! near the player and in chunks nobody has loaded, so an ejector can run all day without filling the world with items.

use super::run_until_ready;
use crate::block::{self, AIR, STONE};
use crate::item::{self, ItemId};
use crate::math::IVec3;
use crate::Game;

const DIRT: ItemId = ItemId::block(block::DIRT);
const EAST: u8 = 1;

/// Runs `seconds` of game with an ejector at `pos` facing east, kept fed, and returns the dirt lying about (items,
/// entities, the eastmost x).
fn lying_after(g: &mut Game, pos: IVec3, seconds: u32) -> (u32, usize, f64) {
    g.sim.world.set_block_anywhere(pos, block::BELT_EJECTOR);
    g.sim.factory.place(&mut g.sim.world, block::BELT_EJECTOR, pos, EAST, pos, 0);
    for _ in 0..seconds * 60 {
        g.sim.factory.insert(pos, DIRT, 4);
        g.update(1.0 / 60.0);
    }
    let dirt = || g.items.list.iter().filter(|e| e.item == DIRT);
    (dirt().map(|e| e.count).sum(), dirt().count(), dirt().map(|e| e.pos.x).fold(f64::MIN, f64::max))
}

#[test]
fn an_ejector_near_the_player_lands_its_items_in_front_and_stacks_them() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in 36..=48 {
        for z in -3..=3 {
            for h in y - 2..=y + 6 {
                g.sim.world.set_block_anywhere(IVec3::new(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    let pos = IVec3::new(40, y + 1, 0);
    let (items, entities, east) = lying_after(&mut g, pos, 40);
    assert!((395..=401).contains(&items), "ten a second for 40 s: {items}");
    assert!(entities <= items.div_ceil(item::stack_size(DIRT)) as usize + 6, "{entities} entities for {items} items");
    assert!(east > 40.5, "they landed in front of it, to the east: {east}");
    assert_eq!(g.item_entities() as usize, entities, "nothing else is lying about");
}

#[test]
fn an_ejector_in_an_unloaded_chunk_does_not_pile_up_either() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let pos = IVec3::new(5000, 100, 5000);
    assert!(!g.sim.world.is_loaded(crate::math::Vec3::new(5000.5, 101.2, 5000.5)));
    let (items, entities, _) = lying_after(&mut g, pos, 60);
    assert!((595..=601).contains(&items), "ten a second for 60 s: {items}");
    assert!(entities <= items.div_ceil(item::stack_size(DIRT)) as usize + 6, "{entities} entities for {items} items");
}

use crate::action::Action;
use crate::block::{self, BELT, CONSTRUCTOR, STONE_BRICKS};
use crate::item::{ItemId, CONSTRUCTOR_MK2};
use crate::math::IVec3;
use crate::raycast::RayHit;
use crate::shape;
use crate::sim::PlayerId;
use crate::tests::run_until_ready;
use crate::Game;

fn aim(g: &mut Game, pos: IVec3, id: block::BlockId) {
    g.target = Some(RayHit { block: pos, normal: IVec3::new(0, 1, 0), id });
}

#[test]
fn pipette_selects_from_hotbar_or_pulls_from_backpack_and_matches_shape() {
    let mut g = Game::new_normal(2024, 2);
    run_until_ready(&mut g);
    let pos = g.body().eye().floor() + IVec3::new(2, 0, 2);
    // Put Stone Bricks in backpack slot 12 and fill hotbar slot 0 with Dirt.
    g.sim.apply(PlayerId(0), Action::Give { item: block::DIRT.into(), count: 5 });
    g.sim.players[0].as_mut().unwrap().inventory.slots[12] =
        crate::inventory::Stack { item: STONE_BRICKS.into(), count: 10 };
    g.sim.world.set_block_anywhere(pos, STONE_BRICKS);
    g.sim.world.set_shape_anywhere(pos, shape::STAIR_BASE + 2);
    aim(&mut g, pos, STONE_BRICKS);

    assert_eq!(g.pipette(), "");
    assert_eq!(g.held_shape_mode(), shape::MODE_STAIR);
    g.run_ticks(1);
    let sel = g.inventory().selected_stack();
    assert_eq!(sel.item, ItemId::block(STONE_BRICKS));
}

#[test]
fn pipette_falls_back_to_held_tier_and_reports_missing_items() {
    let mut g = Game::new_normal(2024, 2);
    run_until_ready(&mut g);
    let pos = g.body().eye().floor() + IVec3::new(2, 0, 2);
    g.sim.world.set_block_anywhere(pos, CONSTRUCTOR);
    g.sim.factory.place(&mut g.sim.world, CONSTRUCTOR, pos, 1, pos, 1); // Constructor Mk2
    aim(&mut g, pos, CONSTRUCTOR);

    // Missing completely: reports notice.
    assert_eq!(g.pipette(), "No Constructor Mk2 in your pack");

    // Holding Mk1 Constructor falls back to Mk1 when Mk2 isn't held.
    g.give(CONSTRUCTOR, 1);
    g.give(BELT, 4);
    g.run_ticks(1);
    g.select_slot(1);
    g.run_ticks(1);
    aim(&mut g, pos, CONSTRUCTOR);
    assert_eq!(g.pipette(), "");
    g.run_ticks(1);
    assert_eq!(g.inventory().selected_stack().item, ItemId::block(CONSTRUCTOR));

    // Holding Mk2 picks exact Mk2.
    g.give(CONSTRUCTOR_MK2.0, 1);
    g.run_ticks(1);
    aim(&mut g, pos, CONSTRUCTOR);
    assert_eq!(g.pipette(), "");
    g.run_ticks(1);
    assert_eq!(g.inventory().selected_stack().item, CONSTRUCTOR_MK2);
}

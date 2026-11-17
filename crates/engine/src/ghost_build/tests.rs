use super::ray_cell;
use crate::action::Action;
use crate::block;
use crate::math::{IVec3, Vec3};
use crate::tests::run_until_ready;
use crate::Game;

/// The scene stands on a stone pad high in the air, out of the terrain's way.
const PAD_Y: i32 = 200;

fn ready() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    g.fill_blocks(-16, PAD_Y - 1, -16, 24, PAD_Y - 1, 16, block::STONE);
    g
}

fn at(x: i32, z: i32) -> IVec3 {
    IVec3::new(x, PAD_Y, z)
}

/// A smelter ghost at `cell`, planted for real through the core.
fn plant(g: &mut Game, cell: IVec3) {
    g.act(Action::PlantGhost { pos: cell, block: block::SMELTER, facing: 1, tier: 0 });
    g.run_ticks(2);
    assert!(g.sim.ghosts.covering(cell).is_some());
}

/// Stands the player at `from` with the camera turned at the middle of `cell`.
fn aim(g: &mut Game, from: IVec3, cell: IVec3) {
    g.body_mut().pos = Vec3::new(from.x as f64 + 0.5, from.y as f64, from.z as f64 + 0.5);
    g.render_eye = g.body().pos;
    let to = Vec3::new(cell.x as f64 + 0.5, cell.y as f64 + 0.5, cell.z as f64 + 0.5) - g.body().eye();
    let body = g.body_mut();
    body.yaw = to.x.atan2(-to.z).rem_euclid(std::f64::consts::TAU);
    body.pitch = to.y.atan2(to.x.hypot(to.z));
    g.run_ticks(1);
}

#[test]
fn x_builds_the_aimed_ghost_from_a_slot_that_is_not_selected() {
    let mut g = ready();
    let cell = at(0, 0);
    plant(&mut g, cell);
    g.give(block::SMELTER as u16, 3);
    g.select_slot(8); // an empty hand
    g.run_ticks(2);
    assert!(g.inventory().selected_stack().is_empty());
    aim(&mut g, at(2, 4), cell);
    assert_eq!(g.build_aimed_ghost(), "");
    g.run_ticks(2);
    assert_eq!(g.sim.world.block_anywhere(cell), Some(block::SMELTER), "built");
    assert!(g.sim.ghosts.covering(cell).is_none(), "the ghost is cleared");
    assert_eq!(g.inventory().count(crate::item::ItemId::block(block::SMELTER)), 2, "one smelter paid");
    assert_eq!(g.sim.factory.placed_as(cell).map(|p| p.0), Some(1), "it faces as the ghost did");
}

#[test]
fn without_the_item_nothing_is_built_and_the_player_is_told() {
    let mut g = ready();
    let cell = at(0, 0);
    plant(&mut g, cell);
    aim(&mut g, at(2, 4), cell);
    let said = g.build_aimed_ghost();
    assert!(said.contains("Smelter"), "{said}");
    g.run_ticks(2);
    assert!(g.sim.ghosts.covering(cell).is_some());
    assert_ne!(g.sim.world.block_anywhere(cell), Some(block::SMELTER));
}

#[test]
fn aiming_at_nothing_says_so() {
    let mut g = ready();
    plant(&mut g, at(0, 0));
    g.give(block::SMELTER as u16, 1);
    aim(&mut g, at(2, 4), at(10, 10));
    assert!(g.build_aimed_ghost().contains("Aim at a ghost"));
    g.run_ticks(2);
    assert_eq!(g.inventory().count(crate::item::ItemId::block(block::SMELTER)), 1);
}

#[test]
fn a_ghost_behind_a_wall_is_not_the_one_aimed_at() {
    let mut g = ready();
    let cell = at(0, 0);
    plant(&mut g, cell);
    g.give(block::SMELTER as u16, 1);
    // A stone wall between the eye and the ghost, on the line of sight.
    g.fill_blocks(1, PAD_Y, 1, 3, PAD_Y + 3, 3, block::STONE);
    aim(&mut g, at(4, 5), cell);
    assert!(g.build_aimed_ghost().contains("Aim at a ghost"), "the wall hides it");
}

#[test]
fn the_nearest_of_two_ghosts_in_line_is_built() {
    let mut g = ready();
    plant(&mut g, at(0, 0));
    plant(&mut g, at(0, -2));
    g.give(block::SMELTER as u16, 1);
    aim(&mut g, at(0, 3), at(0, -2)); // both within the hands' reach
    assert_eq!(g.aimed_ghost(), Some(at(0, 0)), "the near one is in front");
}

#[test]
fn a_ghost_built_in_a_world_with_the_item_through_the_core_alone() {
    // The same action a co-op peer sends: no hands, no selection.
    let mut sim = crate::sim::Sim::new(7, 2);
    let p = crate::sim::PlayerId(0);
    let cell = IVec3::new(500, 200, -500);
    sim.apply(p, Action::PlantGhost { pos: cell, block: block::SMELTER, facing: 2, tier: 0 });
    sim.apply(p, Action::BuildGhost { pos: cell });
    assert!(sim.ghosts.covering(cell).is_some(), "no item, no build");
    sim.apply(p, Action::Give { item: block::SMELTER.into(), count: 1 });
    sim.apply(p, Action::BuildGhost { pos: cell });
    assert_eq!(sim.world.block_anywhere(cell), Some(block::SMELTER));
    assert!(sim.ghosts.covering(cell).is_none());
}

#[test]
fn a_tear_down_mark_is_not_built() {
    let mut sim = crate::sim::Sim::new(7, 2);
    let p = crate::sim::PlayerId(0);
    let cell = IVec3::new(500, 200, -500);
    sim.world.set_block_anywhere(cell, block::STONE);
    sim.apply(p, Action::MarkRemoval { pos: cell });
    sim.apply(p, Action::BuildGhost { pos: cell });
    assert_eq!(sim.world.block_anywhere(cell), Some(block::STONE));
}

#[test]
fn the_ray_enters_the_cell_it_points_at_and_not_one_beside_it() {
    let origin = Vec3::new(0.5, 0.5, 5.5);
    let dir = Vec3::new(0.0, 0.0, -1.0);
    assert_eq!(ray_cell(origin, dir, IVec3::new(0, 0, 0)), Some(4.5));
    assert_eq!(ray_cell(origin, dir, IVec3::new(1, 0, 0)), None);
    assert_eq!(ray_cell(origin, dir, IVec3::new(0, 0, 8)), None, "behind the eye");
}

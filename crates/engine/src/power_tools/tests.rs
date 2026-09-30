use super::*;
use crate::block::{GENERATOR, MINER, STONE, STORAGE};
use crate::factory::Kind;
use crate::item::ItemId;
use crate::raycast::RayHit;
use crate::tests::run_until_ready;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// Flat stone ground up to height `y`, air above; blocks below `y` are stone.
fn ground(y: i32) -> impl Fn(IVec3) -> Option<BlockId> {
    move |p| Some(if p.y <= y { STONE } else { AIR })
}

#[test]
fn a_snapped_pole_stands_at_the_full_reach_along_the_view() {
    let east = Vec3::new(1.0, 0.0, 0.0);
    let anchor = v(0, 5, 0);
    // Aimed nearby, but the view runs east: ten blocks on, on the ground.
    assert_eq!(plan_pole(ground(4), anchor, v(3, 5, 1), east, 10, true), Some((v(10, 5, 0), true)));
    // The view's climb is ignored: only its direction along the ground counts.
    assert_eq!(
        plan_pole(ground(4), anchor, v(3, 5, 1), Vec3::new(1.0, -0.9, 0.0), 10, true),
        Some((v(10, 5, 0), true))
    );
    // Looking straight down there is no direction: it turns towards the aim.
    assert_eq!(
        plan_pole(ground(4), anchor, v(0, 5, 6), Vec3::new(0.0, -1.0, 0.0), 10, true),
        Some((v(0, 5, 10), true))
    );
}

#[test]
fn a_free_pole_goes_where_aimed_and_is_clamped_beyond_its_reach() {
    let anchor = v(0, 5, 0);
    let east = Vec3::new(1.0, 0.0, 0.0);
    assert_eq!(plan_pole(ground(4), anchor, v(4, 5, 0), east, 10, false), Some((v(4, 5, 0), false)));
    assert_eq!(plan_pole(ground(4), anchor, v(30, 5, 0), east, 10, false), Some((v(10, 5, 0), true)));
    // A longer pole reaches further.
    assert_eq!(plan_pole(ground(4), anchor, v(30, 5, 0), east, 16, false), Some((v(16, 5, 0), true)));
}

#[test]
fn a_full_reach_pole_follows_the_ground_up_and_down_and_backs_off_over_gaps() {
    let anchor = v(0, 5, 0);
    let east = Vec3::new(1.0, 0.0, 0.0);
    // A slope rising a block every two blocks: the pole stands on it, still within ten blocks of the anchor.
    let slope = |p: IVec3| Some(if p.y <= 4 + p.x / 2 { STONE } else { AIR });
    let (pos, full) = plan_pole(slope, anchor, v(3, 5, 0), east, 10, true).unwrap();
    assert!(full && pos.x >= 8 && dist2(pos, anchor) <= 100 && pos.y == 5 + pos.x / 2, "{pos:?}");
    // A gap in the ground at x = 10: the next standing cell short of it.
    let gap = |p: IVec3| Some(if p.y <= 4 && p.x != 10 && p.x != 9 { STONE } else { AIR });
    assert_eq!(plan_pole(gap, anchor, v(3, 5, 0), east, 10, true), Some((v(8, 5, 0), true)));
    // No ground at all: nothing to stand on.
    assert_eq!(plan_pole(|_| Some(AIR), anchor, v(3, 5, 0), east, 10, true), None);
}

/// A game on a flat stone platform (ground level returned), poles in hand, the player near the origin
/// facing east (yaw a quarter turn).
fn on_a_platform(item: ItemId, count: u32) -> (Game, i32) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -12..=30 {
        for z in -6..=6 {
            for h in y - 14..=y + 8 {
                let shaft = x == 6 && z == 0 && h < y;
                let id = if h <= y && !shaft { STONE } else { AIR };
                g.sim.world.set_block_anywhere(v(x, h, z), id);
            }
        }
    }
    g.sim.factory.by_hand = true;
    g.give(item.0, count);
    g.run_ticks(3);
    let slot = g.inventory().slots.iter().position(|s| s.item == item).expect("given") as u32;
    g.select_slot(slot);
    g.run_ticks(2);
    let body = g.body_mut();
    body.pos = Vec3::new(3.5, (y + 1) as f64, 0.5);
    body.yaw = std::f64::consts::FRAC_PI_2;
    body.pitch = 0.0;
    (g, y)
}

fn aim_at(g: &mut Game, block: IVec3, normal: IVec3) {
    let id = g.sim.world.get_block(block).unwrap_or(AIR);
    g.target = Some(RayHit { block, normal, id });
}

/// One tick of holding or releasing the use button: the tool's input, then the core applies what it sent.
fn tick_using(g: &mut Game, on: bool) {
    g.using = on;
    g.update_power_tools(1.0 / 60.0);
    g.sim.step();
}

fn poles(g: &Game) -> Vec<IVec3> {
    let mut all: Vec<IVec3> = g.sim.factory.poles().map(|p| p.0).collect();
    all.sort_by_key(|p| (p.x, p.y, p.z));
    all
}

/// Puts a machine or pole straight into the core and the world, as a placed block.
fn put(g: &mut Game, block: BlockId, pos: IVec3) {
    g.sim.world.set_block(pos, block);
    g.sim.factory.place(&mut g.sim.world, block, pos, 0, pos - UP, 0);
}

#[test]
fn the_ghost_pole_goes_where_aimed_and_shift_snaps_it_to_full_reach() {
    let (mut g, y) = on_a_platform(POLE.into(), 8);
    // No pole to place from yet: the first one goes where you point, with no ghost.
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    assert_eq!(g.pole_ghost(), None);
    assert_eq!(g.power_label(), "");

    let anchor = v(0, y + 1, 0);
    put(&mut g, POLE, anchor);
    // By default it stands where aimed, when that is within reach of the last pole.
    let ghost = g.pole_ghost().expect("a pole is near");
    assert_eq!((ghost.pos, ghost.anchor, ghost.full, ghost.free), (v(4, y + 1, 0), anchor, false, true));
    assert!(g.power_label().contains("4 of 10 blocks"), "{}", g.power_label());
    assert!(g.power_label().contains("no powered pole in range"), "{}", g.power_label());
    assert_eq!(g.power_boxes(), [4, y + 1, 0, 4, y + 1, 0, GHOST_GREEN]);
    // Aimed beyond its reach, it is clamped to the widest spacing.
    aim_at(&mut g, v(25, y, 0), v(0, 1, 0));
    assert_eq!(g.pole_ghost().map(|p| (p.pos, p.full)), Some((v(10, y + 1, 0), true)));

    // Holding Shift snaps to the full reach along the view even when aimed nearer.
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    g.body_mut().input.sprint = true;
    assert_eq!(g.pole_ghost().map(|p| (p.pos, p.full)), Some((v(10, y + 1, 0), true)));
    g.body_mut().input.sprint = false;

    // A pole in the way is shown red.
    g.sim.world.set_block(v(4, y + 1, 0), STORAGE);
    assert!(!g.pole_ghost().unwrap().free);
    assert!(g.power_label().contains("in the way"));
    // Something else in hand: no ghost.
    g.select_slot(8);
    g.run_ticks(2);
    assert_eq!(g.pole_ghost(), None);
}

#[test]
fn the_ghost_pole_shows_the_powered_pole_it_will_wire_itself_to() {
    let (mut g, y) = on_a_platform(POLE.into(), 8);
    let (pole, gen) = (v(0, y + 1, 0), v(0, y + 1, 1));
    put(&mut g, POLE, pole);
    put(&mut g, GENERATOR, gen);
    g.sim.factory.connect(pole, gen);
    g.sim.factory.update(&mut g.sim.world, 1, &mut Vec::new());
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    assert_eq!(g.power_boxes(), [0, y + 1, 0, 0, y + 1, 0, ANCHOR_BLUE, 4, y + 1, 0, 4, y + 1, 0, GHOST_GREEN]);
    assert!(g.power_label().contains("wires itself to the powered pole 4 blocks away"), "{}", g.power_label());
}

#[test]
fn a_placed_pole_is_selected_and_a_click_wires_the_machine_aimed_at() {
    let (mut g, y) = on_a_platform(POLE.into(), 8);
    // The first pole goes where pointed and is selected at once.
    aim_at(&mut g, v(2, y, 0), v(0, 1, 0));
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    let pole = v(2, y + 1, 0);
    assert_eq!(poles(&g), [pole]);
    assert_eq!(g.selected_pole(), Some(pole));

    let miner = v(4, y + 1, 0);
    put(&mut g, MINER, miner);
    aim_at(&mut g, miner, v(0, 1, 0));
    assert!(g.power_label().contains("click to connect"), "{}", g.power_label());
    assert_eq!(g.power_boxes(), [2, y + 1, 0, 2, y + 1, 0, ANCHOR_BLUE, 4, y + 1, 0, 4, y + 1, 0, GHOST_GREEN]);
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.sim.factory.wired_pole(miner), Some(pole));
    assert_eq!(g.sim.factory.slots_used(pole), 1);
    assert!(g.power_label().contains("disconnect"), "{}", g.power_label());

    // A plain click now leaves a wired machine alone; crouch-click cuts the wire.
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.sim.factory.slots_used(pole), 1);
    g.body_mut().input.crouch = true;
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.sim.factory.slots_used(pole), 0);
}

#[test]
fn right_clicking_a_pole_selects_it_and_again_deselects() {
    let (mut g, y) = on_a_platform(POLE.into(), 8);
    let (a, b) = (v(2, y + 1, 0), v(8, y + 1, 0));
    put(&mut g, POLE, a);
    put(&mut g, POLE, b);
    aim_at(&mut g, a, v(0, 1, 0));
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.selected_pole(), Some(a));
    aim_at(&mut g, b, v(0, 1, 0));
    assert!(g.power_label().contains("crouch-click to link"), "{}", g.power_label());
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.selected_pole(), Some(b), "a plain click selects the other pole");
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    assert_eq!(g.selected_pole(), None);
    // Crouch-click links two poles.
    aim_at(&mut g, a, v(0, 1, 0));
    tick_using(&mut g, true);
    tick_using(&mut g, false);
    aim_at(&mut g, b, v(0, 1, 0));
    g.body_mut().input.crouch = true;
    tick_using(&mut g, true);
    assert_eq!(g.sim.factory.slots_used(a), 1);
}

#[test]
fn holding_places_one_pole_then_a_line_as_the_player_walks() {
    let (mut g, y) = on_a_platform(POLE.into(), 8);
    let first = v(1, y + 1, 0);
    // Pressed with nothing near: plain placing puts the first pole in front of the face aimed at.
    g.sim.world.set_block(first, POLE);
    g.sim.factory.place(&mut g.sim.world, POLE, first, 0, first, 0);

    // Shift held: the next pole goes ten blocks on, at once.
    g.body_mut().input.sprint = true;
    aim_at(&mut g, v(5, y, 0), v(0, 1, 0));
    tick_using(&mut g, true);
    assert_eq!(poles(&g), [first, v(11, y + 1, 0)]);
    // Held on the spot: nothing more, the next spot (21) is far away.
    for _ in 0..30 {
        g.use_cooldown = 0.0;
        tick_using(&mut g, true);
    }
    assert_eq!(poles(&g).len(), 2);
    // Walk on: when the spot after the last pole comes within reach, the next is placed there.
    g.body_mut().pos.x = 16.5;
    aim_at(&mut g, v(20, y, 0), v(0, 1, 0));
    g.use_cooldown = 0.0;
    tick_using(&mut g, true);
    assert_eq!(poles(&g), [first, v(11, y + 1, 0), v(21, y + 1, 0)]);
    tick_using(&mut g, false);
    g.sim.factory.update(&mut g.sim.world, 1, &mut Vec::new());
    assert_eq!(g.sim.factory.count(Kind::Pole), 3);
    let held = g.inventory().slots.iter().map(|s| if s.item == ItemId::block(POLE) { s.count } else { 0 }).sum::<u32>();
    assert_eq!(held, 6, "the hand-placed first pole cost nothing");
}

#[test]
fn a_cable_click_hangs_cables_to_the_ground_and_crouching_places_one() {
    let (mut g, y) = on_a_platform(CABLE.into(), 30);
    // Aim at the shaft's wall (x = 7) from inside the shaft column: the drop starts there and runs to the floor.
    aim_at(&mut g, v(7, y - 1, 0), v(-1, 0, 0));
    let cells = g.cable_drop();
    assert_eq!(
        (cells.first().copied(), cells.last().copied(), cells.len()),
        (Some(v(6, y - 1, 0)), Some(v(6, y - 14, 0)), 14)
    );
    assert_eq!(g.power_boxes(), [6, y - 14, 0, 6, y - 1, 0, GHOST_GREEN]);
    assert!(g.power_label().contains("14 hang from here"), "{}", g.power_label());
    tick_using(&mut g, true);
    for _ in 0..8 {
        tick_using(&mut g, false);
    }
    assert_eq!(g.sim.factory.count(Kind::Pole), 14);
    let held =
        g.inventory().slots.iter().map(|s| if s.item == ItemId::block(CABLE) { s.count } else { 0 }).sum::<u32>();
    assert_eq!(held, 16);

    // Crouching places just the one, by the ordinary path.
    aim_at(&mut g, v(7, y, 1), v(0, 1, 0));
    g.body_mut().input.crouch = true;
    assert!(g.power_boxes().is_empty());
    assert!(!g.update_power_tools(1.0 / 60.0) || !g.using);
}

#[test]
fn a_drop_stops_at_the_stack_and_at_things_in_the_way() {
    let (mut g, y) = on_a_platform(CABLE.into(), 5);
    aim_at(&mut g, v(7, y - 1, 0), v(-1, 0, 0));
    assert_eq!(g.cable_drop().len(), 5, "five cables make five cells");
    // On flat ground it is one cell, which the plain path places.
    aim_at(&mut g, v(2, y, 2), v(0, 1, 0));
    assert_eq!(g.cable_drop(), [v(2, y + 1, 2)]);
    g.using = true;
    assert!(!g.update_power_tools(1.0 / 60.0), "one cell is left to plain placing");
}

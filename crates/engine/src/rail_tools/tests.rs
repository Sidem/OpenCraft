use super::*;
use crate::block::{AIR, STONE};
use crate::factory::Kind;
use crate::raycast::RayHit;
use crate::tests::run_until_ready;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A game on a flat stone platform (ground level returned) with rails in hand, the player near the origin
/// facing east.
fn on_a_platform() -> (Game, i32) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -12..=40 {
        for z in -20..=20 {
            for h in y - 14..=y + 8 {
                g.sim.world.set_block_anywhere(v(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    g.give(RAIL.into(), 16);
    g.run_ticks(3);
    let slot = g.inventory().slots.iter().position(|s| s.item == RAIL.into()).expect("given") as u32;
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

/// One click: the button goes down for a tick (the tool sends its actions, the core applies them) and up again.
fn click(g: &mut Game) {
    for on in [true, false] {
        g.using = on;
        g.update_rail_tools();
        g.sim.step();
    }
}

#[test]
fn the_first_node_faces_the_view_and_the_next_one_lays_a_curve_from_it() {
    let (mut g, y) = on_a_platform();
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    assert!(g.rail_label().contains("first node"), "{}", g.rail_label());
    click(&mut g);
    assert_eq!(g.sim.factory.rail_yaw(v(4, y + 1, 0)), Some(64), "faces east, the way the player looks");
    assert_eq!(g.rails.sel, Some(v(4, y + 1, 0)), "and is selected");

    // The ghost of the next one, 12 blocks on: it joins, straight.
    aim_at(&mut g, v(16, y, 0), v(0, 1, 0));
    assert_eq!(g.rail_plan(), Some(Plan::Place { pos: v(16, y + 1, 0), yaw: 64, fit: Some(Fit::Ok), free: true }));
    assert!(g.rail_label().contains("12 blocks of track"), "{}", g.rail_label());
    let boxes = g.rail_boxes();
    assert_eq!(boxes[6], ANCHOR_BLUE);
    assert_eq!(boxes[13], GHOST_GREEN);
    click(&mut g);
    assert_eq!(g.sim.factory.count(Kind::Rail), 2);
    assert!(g.sim.factory.track_between(v(4, y + 1, 0), v(16, y + 1, 0)));

    // Off to the side it bends: the new node faces the mirror of the heading in the chord.
    aim_at(&mut g, v(28, y, 8), v(0, 1, 0));
    click(&mut g);
    assert_eq!(g.sim.factory.rail_yaw(v(28, y + 1, 8)), Some(112));
    assert_eq!(g.sim.factory.tracks().len(), 2);
    let mut drawn = Vec::new();
    g.sim.factory.write_instances(&mut drawn, Vec3::new(16.0, y as f64, 4.0), 0.0, 80.0);
    assert!(drawn.len() > 20 * crate::factory::INSTANCE_FLOATS, "the curves are drawn");
}

#[test]
fn a_node_that_does_not_fit_is_not_placed_and_crouching_starts_a_new_line() {
    let (mut g, y) = on_a_platform();
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    click(&mut g);
    // Two blocks on: too close. Far to the side: too sharp. Neither places anything.
    aim_at(&mut g, v(6, y, 0), v(0, 1, 0));
    assert!(matches!(g.rail_plan(), Some(Plan::Place { fit: Some(Fit::TooClose), .. })));
    assert_eq!(g.rail_boxes()[13], BLOCKED_RED);
    click(&mut g);
    aim_at(&mut g, v(6, y, 12), v(0, 1, 0));
    assert!(matches!(g.rail_plan(), Some(Plan::Place { fit: Some(Fit::TooSharp), .. })));
    assert!(g.rail_label().contains("too sharp") || g.rail_label().contains("Bend too sharp"), "{}", g.rail_label());
    click(&mut g);
    assert_eq!(g.sim.factory.count(Kind::Rail), 1);

    // Crouching starts a new line: the node goes down alone.
    g.body_mut().input.crouch = true;
    aim_at(&mut g, v(6, y, 12), v(0, 1, 0));
    click(&mut g);
    g.body_mut().input.crouch = false;
    assert_eq!(g.sim.factory.count(Kind::Rail), 2);
    assert!(g.sim.factory.tracks().is_empty());
}

#[test]
fn clicking_nodes_selects_joins_and_cuts() {
    let (mut g, y) = on_a_platform();
    let (a, b, c) = (v(4, y + 1, 0), v(16, y + 1, 0), v(28, y + 1, 0));
    for p in [a, b, c] {
        g.sim.world.set_block_anywhere(p, RAIL);
        g.sim.factory.place(&mut g.sim.world, RAIL, p, 64, p - IVec3::new(0, 1, 0), 0);
    }
    // Select the first, then click the second: they join and the second is selected.
    aim_at(&mut g, a, IVec3::ZERO);
    click(&mut g);
    assert_eq!(g.rails.sel, Some(a));
    aim_at(&mut g, b, IVec3::ZERO);
    assert!(g.rail_label().contains("click to lay 12 blocks"), "{}", g.rail_label());
    click(&mut g);
    assert!(g.sim.factory.track_between(a, b));
    assert_eq!(g.rails.sel, Some(b));
    // Clicking the selected node deselects it.
    aim_at(&mut g, b, IVec3::ZERO);
    click(&mut g);
    assert_eq!(g.rails.sel, None);
    // Select the third and click the second: joined to it (a plain click selects the other); crouch-click cuts.
    aim_at(&mut g, c, IVec3::ZERO);
    click(&mut g);
    aim_at(&mut g, b, IVec3::ZERO);
    click(&mut g);
    assert!(g.sim.factory.track_between(b, c));
    g.rails.sel = Some(c);
    aim_at(&mut g, b, IVec3::ZERO);
    assert!(g.rail_label().contains("crouch-click to cut"), "{}", g.rail_label());
    g.body_mut().input.crouch = true;
    click(&mut g);
    g.body_mut().input.crouch = false;
    assert!(!g.sim.factory.track_between(b, c) && g.sim.factory.track_between(a, b));
}

#[test]
fn only_rails_in_hand_use_the_track_hand() {
    let (mut g, y) = on_a_platform();
    g.select_slot(8);
    g.run_ticks(2);
    aim_at(&mut g, v(4, y, 0), v(0, 1, 0));
    assert_eq!(g.rail_plan(), None);
    assert!(g.rail_boxes().is_empty() && g.rail_label().is_empty());
    assert!(!g.update_rail_tools());
}

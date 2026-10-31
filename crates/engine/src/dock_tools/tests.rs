use super::*;
use crate::block::{AIR, RAIL, STONE, UNLOADING_DOCK};
use crate::factory::Fit;
use crate::raycast::RayHit;
use crate::sim::PlayerId;
use crate::tests::run_until_ready;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A flat stone platform with rail nodes on it: a, b, c along x (joined), and d on its own facing a diagonal. A
/// loading dock is in hand. Returns the game, a, b, c, d.
fn scene() -> (Game, [IVec3; 4]) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -12..=40 {
        for z in -12..=12 {
            for h in y - 3..=y + 6 {
                g.sim.world.set_block_anywhere_later(v(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    let nodes = [v(4, y + 1, 0), v(16, y + 1, 0), v(28, y + 1, 0), v(16, y + 1, 8)];
    for (i, &p) in nodes.iter().enumerate() {
        g.sim.world.set_block_anywhere(p, RAIL);
        g.sim.factory.place(&mut g.sim.world, RAIL, p, if i == 3 { 32 } else { 64 }, p - UP, 0);
    }
    g.sim.factory.lay_track(nodes[0], nodes[1]);
    g.sim.factory.lay_track(nodes[1], nodes[2]);
    g.give(LOADING_DOCK.into(), 4);
    g.run_ticks(3);
    let slot = g.inventory().slots.iter().position(|s| s.item == LOADING_DOCK.into()).expect("given") as u32;
    g.select_slot(slot);
    g.run_ticks(2);
    g.body_mut().pos = Vec3::new(16.5, (y + 1) as f64, 4.5);
    (g, nodes)
}

fn aim_at(g: &mut Game, block: IVec3, normal: IVec3) {
    let id = g.sim.world.get_block(block).unwrap_or(AIR);
    g.target = Some(RayHit { block, normal, id });
}

fn click(g: &mut Game) {
    for on in [true, false] {
        g.using = on;
        g.update_dock_tools();
        g.sim.step();
    }
}

fn outlined(g: &Game, colour: i32) -> Vec<IVec3> {
    g.dock_boxes().chunks_exact(7).filter(|b| b[6] == colour).map(|b| v(b[0], b[1], b[2])).collect()
}

#[test]
fn a_dock_goes_onto_a_straight_node_keeping_its_tracks_and_says_why_not_elsewhere() {
    let (mut g, [a, b, c, d]) = scene();
    aim_at(&mut g, v(10, a.y - 1, 6), UP);
    assert!(g.dock_label().contains("aim at a rail node on straight track"), "{}", g.dock_label());
    let mut can = outlined(&g, CAN_TEAL);
    can.sort_by_key(|p| p.x);
    assert_eq!(can, vec![a, b, c], "the straight nodes, not the diagonal one");
    // Aiming at the ground beside a node takes the node.
    aim_at(&mut g, b - UP + IVec3::new(1, 0, 0), UP);
    assert_eq!(outlined(&g, GHOST_GREEN), vec![b]);
    assert!(g.dock_label().contains("click to build it here"), "{}", g.dock_label());
    click(&mut g);
    let f = &g.sim.factory;
    assert_eq!(f.dock_anchor(b), Some(b), "the node is a dock now");
    assert_eq!(f.rail_yaw(b), Some(64), "and still a rail node, heading the same way");
    assert!(f.track_between(a, b) && f.track_between(b, c), "with its tracks");
    assert_eq!(g.sim.world.get_block(b), Some(LOADING_DOCK));
    // The diagonal node is red and says why.
    aim_at(&mut g, d, IVec3::ZERO);
    assert_eq!(outlined(&g, BLOCKED_RED), vec![d]);
    assert!(g.dock_label().contains("diagonal"), "{}", g.dock_label());
    click(&mut g);
    assert_eq!(g.sim.factory.dock_anchor(d), None);
    // A dock end takes one track: a fourth node joined to the dock's west end too is refused.
    let e = v(10, a.y, 6);
    g.sim.world.set_block_anywhere(e, RAIL);
    g.sim.factory.place(&mut g.sim.world, RAIL, e, 64, e - UP, 0);
    assert_eq!(g.sim.factory.track_fit(b, e), Some(Fit::DockEnd));
}

#[test]
fn the_core_refuses_a_dock_off_the_rails_by_every_way_a_block_is_placed() {
    let (mut g, [a, ..]) = scene();
    let ground = v(10, a.y, 4);
    // A placing action, as a hand or a co-op client sends.
    let slot = g.inventory().selected as u8;
    g.act(Action::PlaceBlock { pos: ground, slot, facing: 0, against: ground - UP });
    g.run_ticks(1);
    assert_eq!(g.sim.world.get_block(ground), Some(AIR));
    // `put_block`, which drones building ghosts and blueprints use.
    let me = PlayerId(0);
    assert!(!g.sim.put_block(me, ground, LOADING_DOCK.into(), 0, ground - UP));
    assert!(!g.sim.put_block(me, a + UP, LOADING_DOCK.into(), 0, a), "nor above a node");
    assert!(g.sim.put_block(me, a, LOADING_DOCK.into(), 0, a - UP), "onto a node it goes");
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Process), 1);
}

#[test]
fn a_dock_is_laid_in_line_after_the_selected_node_joined_to_it_and_rails_go_on_from_it() {
    let (mut g, [_, _, c, _]) = scene();
    g.select_rail_piece(c, 64);
    // Aim at the ground a little off the line, 8 blocks on: the dock lands on the line itself.
    let at = c + IVec3::new(8, 0, 1);
    aim_at(&mut g, at - UP, UP);
    assert!(g.dock_label().contains("click to lay it in line: 8 blocks"), "{}", g.dock_label());
    assert!(outlined(&g, ANCHOR_BLUE) == vec![c] && outlined(&g, GHOST_GREEN) == vec![c + IVec3::new(8, 0, 0)]);
    click(&mut g);
    let d = c + IVec3::new(8, 0, 0);
    let f = &g.sim.factory;
    assert_eq!(f.dock_anchor(d), Some(d), "a dock");
    assert!(f.track_between(c, d), "joined in the same action");
    assert_eq!(g.rail_anchor(), Some((d, 64)), "and selected");
    // A hole in the ground under the next stretch is outlined amber, and a click does nothing.
    let hole = d + IVec3::new(4, -1, 0);
    for h in hole.y - 2..=hole.y {
        g.sim.world.set_block_anywhere(IVec3::new(hole.x, h, 0), AIR);
    }
    g.give(UNLOADING_DOCK.into(), 1);
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == UNLOADING_DOCK.into()).unwrap() as u32;
    g.select_slot(slot);
    g.run_ticks(1);
    aim_at(&mut g, d + IVec3::new(8, -1, 0), UP);
    assert!(g.dock_label().contains("fill 1 block (amber)"), "{}", g.dock_label());
    assert_eq!(outlined(&g, FILL_AMBER), vec![hole]);
    click(&mut g);
    assert_eq!(g.sim.factory.dock_anchor(d + IVec3::new(8, 0, 0)), None);
}

#[test]
fn the_core_lays_a_dock_in_line_only_joined_on_the_ground_along_an_axis() {
    let (mut g, [_, b, c, d]) = scene();
    let me = PlayerId(0);
    let dock = LOADING_DOCK.into();
    let on = c + IVec3::new(4, 0, 0);
    assert!(!g.sim.put_block(me, on, dock, 64, on - UP), "against the ground: nothing to join");
    assert!(!g.sim.put_block(me, on, dock, 32, c), "a diagonal heading");
    assert!(!g.sim.put_block(me, on + UP * 2, dock, 64, c), "in the air");
    assert!(!g.sim.put_block(me, d + IVec3::new(0, 0, 10), dock, 0, d), "after a diagonal node: a bend");
    assert!(!g.sim.put_block(me, b + IVec3::new(2, 0, 0), dock, 64, b), "too close to join");
    assert!(g.sim.put_block(me, on, dock, 64, c));
    assert!(g.sim.factory.track_between(c, on));
    // A dock is an end of the line too: rails join on from it, and cut from it (`Action::Connect` from a dock).
    let e = on + IVec3::new(6, 0, 0);
    g.sim.world.set_block_anywhere(e, RAIL);
    g.sim.factory.place(&mut g.sim.world, RAIL, e, 64, e - UP, 0);
    g.act(Action::Connect { pole: on, to: e });
    g.run_ticks(1);
    assert!(g.sim.factory.track_between(on, e), "joined from the dock");
    g.act(Action::Disconnect { pole: on, to: e });
    g.run_ticks(1);
    assert!(!g.sim.factory.track_between(on, e), "and cut from it");
}

#[test]
fn breaking_a_dock_puts_the_rail_node_back_with_its_tracks() {
    let (mut g, [a, b, c, _]) = scene();
    aim_at(&mut g, b, IVec3::ZERO);
    click(&mut g);
    g.act(Action::BreakBlock { pos: b });
    g.run_ticks(1);
    let f = &g.sim.factory;
    assert_eq!(f.dock_anchor(b), None);
    assert_eq!(f.rail_yaw(b), Some(64));
    assert!(f.track_between(a, b) && f.track_between(b, c));
    assert_eq!(g.sim.world.get_block(b), Some(RAIL));
}

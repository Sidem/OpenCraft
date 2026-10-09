use super::*;
use crate::belt_line::Route;
use crate::block::{AIR, STONE, STORAGE};
use crate::inventory::INVENTORY_SLOTS;
use crate::tests::{belt_test_slab, run_until_ready};

const EAST: u8 = 1;

/// Stone below the ground height `top(x, z)`, air above; `extra` overrides single cells.
fn terrain<'a>(
    top: impl Fn(i32, i32) -> i32 + 'a,
    extra: &'a [(IVec3, BlockId)],
) -> impl Fn(IVec3) -> Option<BlockId> + 'a {
    move |p| {
        if let Some(&(_, b)) = extra.iter().find(|(q, _)| *q == p) {
            return Some(b);
        }
        Some(if p.y < top(p.x, p.z) { STONE } else { AIR })
    }
}

fn flat(_: i32, _: i32) -> i32 {
    10
}

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A wall of stone: x = `x`, z from `z0` to `z1`, y from 10 to 12.
fn wall(x: i32, z0: i32, z1: i32) -> Vec<(IVec3, BlockId)> {
    (z0..=z1).flat_map(|z| (10..=12).map(move |y| (at(x, y, z), STONE))).collect()
}

/// A route is one belt at the start, one at the end, each next to the one before (a step up or down keeps its
/// heading on both belts), and no belt twice.
fn assert_valid(cells: &[LineCell], start: IVec3, end: IVec3) {
    assert_eq!((cells[0].pos, cells.last().unwrap().pos), (start, end));
    for w in cells.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = b.pos - a.pos;
        assert_eq!(d.x.abs() + d.z.abs(), 1, "{:?} to {:?} is one step", a.pos, b.pos);
        assert!(d.y.abs() <= 1);
        assert_eq!(DIRS[a.dir as usize], IVec3::new(d.x, 0, d.z), "{:?} leads to the next", a.pos);
        if d.y != 0 {
            assert_eq!(a.dir, b.dir, "a step keeps its heading");
        }
    }
    let mut seen: Vec<IVec3> = cells.iter().map(|c| c.pos).collect();
    seen.sort_by_key(|p| (p.x, p.y, p.z));
    seen.dedup();
    assert_eq!(seen.len(), cells.len(), "no belt twice");
}

#[test]
fn open_ground_gives_a_short_route_with_few_corners_that_feeds_the_machine() {
    let (start, end) = (at(0, 10, 0), at(6, 10, 3));
    let cells = find(terrain(flat, &[]), start, end, Some(EAST)).unwrap();
    assert_valid(&cells, start, end);
    assert_eq!(cells.len(), 10, "the shortest way");
    assert_eq!(cells.last().unwrap().dir, EAST, "the last belt feeds the machine");
    let corners = cells.windows(2).filter(|w| w[0].dir != w[1].dir).count();
    assert!(corners <= 3, "{corners} corners");
}

#[test]
fn a_wall_is_walked_round() {
    let walls = wall(3, -3, 3);
    let (start, end) = (at(0, 10, 0), at(6, 10, 0));
    let cells = find(terrain(flat, &walls), start, end, None).unwrap();
    assert_valid(&cells, start, end);
    assert!(cells.len() > 13, "round the end of the wall: {}", cells.len());
    assert!(cells.iter().all(|c| !walls.iter().any(|(p, _)| *p == c.pos)));
}

#[test]
fn a_step_up_is_climbed_by_a_pair_of_belts_running_the_same_way() {
    let step = |x: i32, _: i32| if x >= 4 { 11 } else { 10 };
    let (start, end) = (at(0, 10, 0), at(8, 11, 0));
    let cells = find(terrain(step, &[]), start, end, Some(EAST)).unwrap();
    assert_valid(&cells, start, end);
    assert_eq!(cells.len(), 9);
    assert_eq!(cells[3].shape, Shape::Up, "the belt before the step is the ramp");
}

#[test]
fn a_gap_is_crossed_by_belts_over_it() {
    let gap = |x: i32, _: i32| if (3..=5).contains(&x) { 2 } else { 10 };
    let (start, end) = (at(0, 10, 0), at(9, 10, 0));
    let cells = find(terrain(gap, &[]), start, end, Some(EAST)).unwrap();
    assert_valid(&cells, start, end);
    assert_eq!(cells.len(), 10, "straight across");
    assert!(cells.iter().all(|c| c.pos.y == 10 && c.dir == EAST));
}

#[test]
fn a_target_that_cannot_be_reached_has_no_route() {
    // The end cell is boxed in on every side and (up to the search's height) the cells beside it too.
    let mut box_in = Vec::new();
    for y in 8..=20 {
        for d in DIRS {
            box_in.push((at(6, 10, 0) + d + at(0, y - 10, 0), STONE));
        }
    }
    let start = at(0, 10, 0);
    assert!(find(terrain(flat, &box_in), start, at(6, 10, 0), None).is_none());
    // Neither end may be inside a block.
    assert!(find(terrain(flat, &[]), at(0, 5, 0), at(6, 10, 0), None).is_none());
    assert!(find(terrain(flat, &[]), start, at(6, 5, 0), None).is_none());
}

#[test]
fn one_cell_when_the_ends_are_the_same() {
    let cells = find(terrain(flat, &[]), at(2, 10, 2), at(2, 10, 2), Some(EAST)).unwrap();
    assert_eq!(cells, vec![LineCell::belt(at(2, 10, 2), EAST, Shape::Flat)]);
}

/// A game on the belt slab with Auto-Routing known, a storage box at the far end and belts selected.
fn routed_game() -> (Game, u32) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    g.sim.world.set_block(at(13, 202, 0), STORAGE);
    g.sim.factory.research.complete_all();
    g.give(crate::block::BELT.into(), 20);
    g.run_ticks(2);
    let slot = (0..INVENTORY_SLOTS as u32).find(|&s| g.slot_item(s) == crate::block::BELT as u16).unwrap();
    g.select_slot(slot);
    g.run_ticks(2);
    (g, slot)
}

/// Presses on the slab ahead and points at the box.
fn drag_to_the_box(g: &mut Game) {
    let east = std::f64::consts::FRAC_PI_2;
    g.set_look(east, -1.3);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    g.set_look(east, -0.09);
    for _ in 0..3 {
        g.update(1.0 / 60.0);
    }
}

#[test]
fn a_line_dragged_onto_a_machine_routes_itself_and_builds_belts() {
    let (mut g, slot) = routed_game();
    drag_to_the_box(&mut g);
    assert_eq!(g.line.route, Route::Found);
    let last = *g.line.cells.last().unwrap();
    assert_eq!((last.pos, last.dir), (at(12, 202, 0), EAST), "ends in front of the box, feeding it");
    assert!(g.line_label().starts_with("Auto-route\n"), "{}", g.line_label());
    let planned = g.line.cells.len();
    g.set_using(false);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sim.factory.count(factory::Kind::Belt), planned);
    assert_eq!(g.sim.ghosts.len(), 0);
    assert_eq!(g.slot_count(slot), 20 - planned as u32);
}

#[test]
fn in_ghost_mode_the_route_is_planted_as_ghosts_and_the_world_stays_as_it_was() {
    let (mut g, slot) = routed_game();
    g.toggle_ghost_mode();
    drag_to_the_box(&mut g);
    assert_eq!(g.line.route, Route::Found);
    let cells = g.line.cells.clone();
    assert!(g.line_label().contains("release to plant ghosts"), "{}", g.line_label());
    g.set_using(false);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    let ghosts: Vec<IVec3> = g.sim.ghosts.iter().map(|gh| gh.pos).collect();
    assert_eq!(ghosts.len(), cells.len());
    assert!(cells.iter().all(|c| ghosts.contains(&c.pos)));
    assert_eq!(g.sim.factory.count(factory::Kind::Belt), 0, "nothing was built");
    assert_eq!(g.slot_count(slot), 20, "ghosts cost nothing");
    assert!(cells.iter().all(|c| g.sim.world.get_block(c.pos) == Some(AIR)));
}

#[test]
fn without_the_research_a_line_stays_plain() {
    let (mut g, _) = routed_game();
    g.sim.factory.research = crate::research::Research::default();
    drag_to_the_box(&mut g);
    assert_eq!(g.line.route, Route::Off);
}

#[test]
fn a_machine_that_cannot_be_reached_says_so_and_builds_nothing() {
    let (mut g, slot) = routed_game();
    let east = std::f64::consts::FRAC_PI_2;
    g.set_look(east, -1.3);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    // Wall the start cell in, two blocks high, once the line has started.
    let start = g.line.start.expect("a line started").0;
    for d in factory::DIRS {
        for dy in 0..2 {
            g.sim.world.set_block(start + d + at(0, dy, 0), STONE);
        }
    }
    g.teleport(0.5, 206.0, 0.5);
    g.set_look(east, -0.39);
    for _ in 0..3 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.line.route, Route::Blocked);
    assert!(g.line.cells.is_empty());
    assert!(g.line_label().starts_with("Auto-route\nNo route"), "{}", g.line_label());
    assert!(!g.line_cells().is_empty(), "the target is marked red");
    g.set_using(false);
    for _ in 0..30 {
        g.update(1.0 / 60.0);
    }
    assert_eq!((g.sim.factory.count(factory::Kind::Belt), g.sim.ghosts.len(), g.slot_count(slot)), (0, 0, 20));
}

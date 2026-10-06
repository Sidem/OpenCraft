use super::*;
use crate::block::{AIR, STONE, STORAGE, UNDERPASS_IN};
use crate::inventory::Stack;
use crate::item::GREEN_KIT;
use crate::raycast::RayHit;
use crate::tests::{belt_test_slab, run_until_ready};

const EAST: u8 = 1;
const NORTH: u8 = 0;
const SOUTH: u8 = 2;

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

/// The planned cells only (the tests of the underpass behaviour look at where the line was blocked too).
fn plan(block: impl Fn(IVec3) -> Option<BlockId>, start: IVec3, end: IVec3, facing: u8) -> Vec<LineCell> {
    super::plan(block, start, end, facing).0
}

fn positions(cells: &[LineCell]) -> Vec<(i32, i32, i32)> {
    cells.iter().map(|c| (c.pos.x, c.pos.y, c.pos.z)).collect()
}

#[test]
fn a_straight_line_on_flat_ground() {
    let cells = plan(terrain(flat, &[]), IVec3::new(0, 10, 0), IVec3::new(5, 10, 1), NORTH);
    let mut expected: Vec<_> = (0..6).map(|x| (x, 10, 0)).collect();
    expected.push((5, 10, 1));
    assert_eq!(positions(&cells), expected, "the longer axis first, then the turn");
    assert!(cells[..5].iter().all(|c| c.dir == EAST && c.shape == Shape::Flat));
    assert_eq!((cells[5].dir, cells[6].dir), (SOUTH, SOUTH));
}

#[test]
fn a_click_without_dragging_lays_one_belt_the_way_you_face() {
    let cells = plan(terrain(flat, &[]), IVec3::new(3, 10, 3), IVec3::new(3, 10, 3), NORTH);
    assert_eq!(cells, vec![LineCell::belt(IVec3::new(3, 10, 3), NORTH, Shape::Flat)]);
}

#[test]
fn a_line_turns_once_and_the_corner_runs_the_new_way() {
    let cells = plan(terrain(flat, &[]), IVec3::new(0, 10, 0), IVec3::new(4, 10, -2), NORTH);
    assert_eq!(cells.len(), 7);
    assert_eq!((cells[4].pos, cells[4].dir), (IVec3::new(4, 10, 0), NORTH));
    assert_eq!((cells[3].dir, cells[6].pos), (EAST, IVec3::new(4, 10, -2)));
}

#[test]
fn a_line_follows_steps_up_and_down() {
    // A one-block hill from x = 3 to x = 5.
    let hill = |x: i32, _: i32| if (3..=5).contains(&x) { 11 } else { 10 };
    let cells = plan(terrain(hill, &[]), IVec3::new(0, 10, 0), IVec3::new(8, 10, 0), NORTH);
    let ys: Vec<i32> = cells.iter().map(|c| c.pos.y).collect();
    assert_eq!(ys, [10, 10, 10, 11, 11, 11, 10, 10, 10]);
    let shapes: Vec<Shape> = cells.iter().map(|c| c.shape).collect();
    use Shape::{Down, Flat, Up};
    assert_eq!(shapes, [Flat, Flat, Up, Flat, Flat, Flat, Down, Flat, Flat]);
}

#[test]
fn a_line_stops_at_a_wall_or_a_machine() {
    let wall = |x: i32, _: i32| if x >= 4 { 12 } else { 10 };
    let (cells, blocked) = super::plan(terrain(wall, &[]), IVec3::new(0, 10, 0), IVec3::new(8, 10, 0), NORTH);
    assert_eq!(cells.len(), 4, "a two-block wall too wide to go under ends the line");
    assert_eq!(blocked, Some(IVec3::new(4, 10, 0)), "and is marked");
    let with_box = [(IVec3::new(3, 10, 0), STORAGE)];
    let cells = plan(terrain(flat, &with_box), IVec3::new(0, 10, 0), IVec3::new(2, 10, 0), NORTH);
    assert_eq!(cells.len(), 3, "a line ending before the box feeds it");
    assert!(cells.iter().all(|c| c.shape == Shape::Flat));
}

#[test]
fn a_line_dives_under_a_box_in_the_way_and_comes_back_up() {
    let with_box = [(IVec3::new(3, 10, 0), STORAGE)];
    let (cells, blocked) = super::plan(terrain(flat, &with_box), IVec3::new(0, 10, 0), IVec3::new(6, 10, 0), NORTH);
    assert_eq!(blocked, None);
    assert_eq!(positions(&cells), (0..=6).filter(|&x| x != 3).map(|x| (x, 10, 0)).collect::<Vec<_>>());
    let pieces: Vec<Piece> = cells.iter().map(|c| c.piece).collect();
    use Piece::{Belt, Entry, Exit};
    assert_eq!(pieces, [Belt, Belt, Entry(1), Exit, Belt, Belt]);
    assert_eq!((cells[2].shape, cells[3].shape), (Shape::Entry, Shape::Exit));
}

#[test]
fn the_width_it_can_go_under_is_the_best_underpass_reach() {
    // A row of boxes x = 3..=(2 + width) with ground beyond: ten blocks are fine, eleven stop the line.
    let boxes = |width: i32| (3..3 + width).map(|x| (IVec3::new(x, 10, 0), STORAGE)).collect::<Vec<_>>();
    for (width, ok) in [(4, true), (10, true), (11, false)] {
        let extra = boxes(width);
        let (cells, blocked) = super::plan(terrain(flat, &extra), IVec3::new(0, 10, 0), IVec3::new(20, 10, 0), NORTH);
        assert_eq!(blocked.is_none(), ok, "{width} wide");
        assert_eq!(cells.iter().filter(|c| c.piece == Piece::Exit).count(), usize::from(ok));
        if ok {
            assert_eq!(cells[2].piece, Piece::Entry(width as u8));
        }
    }
}

#[test]
fn a_line_that_ends_inside_the_obstacle_is_blocked() {
    let with_box = [(IVec3::new(3, 10, 0), STORAGE), (IVec3::new(4, 10, 0), STORAGE)];
    let (cells, blocked) = super::plan(terrain(flat, &with_box), IVec3::new(0, 10, 0), IVec3::new(4, 10, 0), NORTH);
    assert_eq!((cells.len(), blocked), (3, Some(IVec3::new(3, 10, 0))), "no free cell before the path ends");
}

#[test]
fn the_cheapest_stocked_underpass_that_covers_the_width_is_chosen() {
    let with_boxes: Vec<_> = (3..8).map(|x| (IVec3::new(x, 10, 0), STORAGE)).collect(); // five wide: Mk2 or better
    let plan5 = || super::plan(terrain(flat, &with_boxes), IVec3::new(0, 10, 0), IVec3::new(10, 10, 0), NORTH).0;
    let pick = |belt_tier: u8, have: [u32; 4]| {
        let mut cells = plan5();
        pass::assign(&mut cells, belt_tier, have);
        let e = cells.iter().find(|c| matches!(c.piece, Piece::Entry(_))).unwrap();
        let x = cells.iter().find(|c| c.piece == Piece::Exit).unwrap();
        assert_eq!((e.tier, e.short), (x.tier, x.short));
        (e.tier, e.short)
    };
    assert_eq!(pick(0, [9, 2, 2, 2]), (1, false), "a Mk1 is too short for five blocks");
    assert_eq!(pick(0, [9, 0, 0, 2]), (3, false), "any longer one will do");
    assert_eq!(pick(2, [9, 9, 0, 2]), (3, false), "and never one slower than the belt");
    assert_eq!(pick(0, [9, 1, 1, 1]), (1, true), "one of a pair is none: red, showing the cheapest that would do");
    assert_eq!(pick(0, [2, 0, 0, 0]), (1, true), "a Mk1 pair doesn't reach");
}

#[test]
fn only_the_underpass_pairs_and_belts_in_stock_are_built() {
    let cells = |short: bool| {
        let mut c = vec![LineCell::belt(IVec3::ZERO, 1, Shape::Flat); 5];
        c[1] = LineCell { piece: Piece::Entry(2), shape: Shape::Entry, short, ..c[1] };
        c[2] = LineCell { piece: Piece::Exit, shape: Shape::Exit, short, ..c[2] };
        c
    };
    assert_eq!(pass::affordable(&cells(false), 9), [true; 5]);
    assert_eq!(pass::affordable(&cells(true), 9), [true, false, false, true, true], "no pair: left out");
    assert_eq!(pass::affordable(&cells(false), 2), [true, true, true, true, false], "belts run out");
}

#[test]
fn a_line_is_at_most_max_line_long_and_needs_a_free_start() {
    let cells = plan(terrain(flat, &[]), IVec3::new(0, 10, 0), IVec3::new(500, 10, 0), NORTH);
    assert_eq!(cells.len(), MAX_LINE);
    assert!(plan(terrain(flat, &[]), IVec3::new(0, 9, 0), IVec3::new(5, 9, 0), NORTH).is_empty());
}

#[test]
fn an_upgrade_line_follows_belts_and_picks_the_tier_below_the_kit() {
    // Belts along x = 0..=6 at y 10, stepping up to y 11 from x 4; x 2 is already Mk2; x 7 has none.
    let tier_at = |p: IVec3| match (p.x, p.y, p.z) {
        (2, 10, 0) => Some(1),
        (0..=3, 10, 0) | (4..=6, 11, 0) | (9, 11, 0) => Some(0),
        _ => None,
    };
    let cells = plan_upgrade(tier_at, IVec3::new(0, 10, 0), IVec3::new(9, 11, 0), 0);
    assert_eq!(positions(&cells), [(0, 10, 0), (1, 10, 0), (3, 10, 0), (4, 11, 0), (5, 11, 0), (6, 11, 0)]);
    assert!(cells.iter().all(|c| c.dir == EAST));
    // Nothing to upgrade from a Mk2 kit's point of view but the Mk2 belt (tier 1 to 2).
    assert_eq!(positions(&plan_upgrade(tier_at, IVec3::new(0, 10, 0), IVec3::new(9, 11, 0), 1)), [(2, 10, 0)]);
}

/// Replaces the local player's slots with these stacks (slot, item, count) and selects `selected`.
fn stock(g: &mut Game, stacks: &[(usize, ItemId, u32)], selected: usize) {
    let inv = &mut g.sim.players[0].as_mut().unwrap().inventory;
    inv.slots = crate::inventory::Inventory::EMPTY.slots;
    for &(slot, item, count) in stacks {
        inv.slots[slot] = Stack { item, count };
    }
    inv.selected = selected;
}

#[test]
fn a_line_longer_than_the_held_stack_takes_belts_from_the_other_stacks() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    stock(&mut g, &[(2, BELT.into(), 3), (0, BELT.into(), 4), (20, BELT.into(), 7)], 2);
    let east = std::f64::consts::FRAC_PI_2;
    g.set_look(east, -1.3);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    g.set_look(east, -0.15);
    for _ in 0..3 {
        g.update(1.0 / 60.0);
    }
    let planned = g.line.cells.len();
    assert!(planned > 7, "the line is longer than the held stack and the next: {planned}");
    assert!(g.line_label().contains(&format!("{planned} belts")) && !g.line_label().contains("you have"));
    g.set_using(false);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    let built = planned.min(14);
    assert_eq!(g.sim.factory.count(factory::Kind::Belt), built, "every cell the stacks pay for");
    assert_eq!(g.item_total(BELT.into()), 14 - built as u32);
    assert_eq!(g.slot_count(2), 0, "the held stack went first");
}

/// Drags a line east along the test slab (y 201) over a box at x = 1, with the inventory as given, builds it
/// and returns the game after the placements landed.
fn line_over_a_box(stacks: &[(usize, ItemId, u32)]) -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    g.sim.world.set_block(IVec3::new(1, 201, 0), STORAGE);
    stock(&mut g, stacks, 0);
    let (start, end) = (IVec3::new(-2, 201, 0), IVec3::new(4, 201, 0));
    g.line.start = Some((start, EAST));
    g.line.cells = super::plan(|p| g.sim.world.get_block(p), start, end, EAST).0;
    g.assign_passes();
    let planned = g.line.cells.clone();
    g.queue_build(&planned, false);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    g
}

#[test]
fn a_line_over_a_box_builds_an_underpass_pair_from_the_inventory() {
    use crate::factory::Kind;
    let pass = ItemId::block(UNDERPASS_IN);
    let g = line_over_a_box(&[(0, BELT.into(), 10), (1, pass, 3)]);
    let f = &g.sim.factory;
    assert_eq!(f.count(Kind::Belt), 6, "four belts and the two ends of the tunnel");
    let at = |x| IVec3::new(x, 201, 0);
    assert_eq!((f.belt_at(at(0)).shape, f.belt_at(at(2)).shape), (Shape::Entry, Shape::Exit));
    assert!(!f.describe(at(0)).unwrap().contains("needs another underpass"), "the entry found its exit");
    assert_eq!((g.item_total(BELT.into()), g.item_total(pass.0)), (6, 1), "belts for belts, underpasses for the pair");
}

#[test]
fn without_underpasses_the_line_is_built_but_the_tunnel_is_left_out() {
    use crate::factory::Kind;
    let g = line_over_a_box(&[(0, BELT.into(), 10)]);
    assert_eq!(g.sim.factory.count(Kind::Belt), 4, "the belts either side, no tunnel");
    assert_eq!(g.item_total(BELT.into()), 6);
    // The preview marks the pair red (flag 0) and the belts ok (1) before building.
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    g.sim.world.set_block(IVec3::new(1, 201, 0), STORAGE);
    stock(&mut g, &[(0, BELT.into(), 10)], 0);
    g.line.cells = super::plan(|p| g.sim.world.get_block(p), IVec3::new(-2, 201, 0), IVec3::new(4, 201, 0), EAST).0;
    g.assign_passes();
    let flags: Vec<i32> = g.planned_cells().chunks(4).map(|c| c[3]).collect();
    assert_eq!(flags, [1, 1, 0, 0, 1, 1]);
    assert!(g.belt_line_label().contains("2 underpasses") && g.belt_line_label().contains("2 underpasses missing"));
}

/// Belts `0..n` along x at y 70, a lone Mk2 at the far end, all registered in the factory.
fn kit_line(g: &mut Game, n: i32) {
    for (t, tech) in crate::research::TECHS.iter().enumerate() {
        (0..tech.units).for_each(|_| g.sim.factory.research.add_unit(t as u8));
    }
    for x in 0..n {
        g.sim.factory.add_belt(IVec3::new(x, 70, 0), 1);
    }
    g.run_ticks(1);
}

#[test]
fn shift_clicking_a_belt_upgrades_its_whole_line_with_kits_from_every_stack() {
    let mut g = Game::new(7, 2);
    kit_line(&mut g, 6);
    // The held stack is in the higher slot, which the core empties first.
    stock(&mut g, &[(5, GREEN_KIT, 3), (0, GREEN_KIT, 2)], 5);
    g.target = Some(RayHit { block: IVec3::new(0, 70, 0), normal: IVec3::ZERO, id: BELT });
    g.body_mut().input.sprint = true;
    g.using = true;
    assert!(g.update_belt_line());
    assert!(!g.using, "one click, one line");
    g.run_ticks(60);
    let tiers: Vec<u8> = (0..6).map(|x| g.sim.factory.belt_at(IVec3::new(x, 70, 0)).tier).collect();
    assert_eq!(tiers, [1, 1, 1, 1, 1, 0], "five kits upgrade the five belts nearest the click");
    assert_eq!(g.item_total(GREEN_KIT.0), 0);
}

#[test]
fn without_shift_a_click_upgrades_only_the_dragged_belts() {
    let mut g = Game::new(7, 2);
    kit_line(&mut g, 4);
    stock(&mut g, &[(0, GREEN_KIT, 9)], 0);
    g.target = Some(RayHit { block: IVec3::new(1, 70, 0), normal: IVec3::ZERO, id: BELT });
    g.using = true;
    g.update_belt_line();
    g.using = false;
    g.update_belt_line();
    g.run_ticks(30);
    let tiers: Vec<u8> = (0..4).map(|x| g.sim.factory.belt_at(IVec3::new(x, 70, 0)).tier).collect();
    assert_eq!(tiers, [0, 1, 0, 0]);
}

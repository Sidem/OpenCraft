use super::*;
use crate::action::Action;
use crate::block::{is_leaves, is_log, is_tree_wood, AIR, GRASS, LOG, SAND, STONE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{IRON_AXE, STEEL_AXE, STONE_AXE};
use crate::worldgen::{grow, height_of, KINDS};

/// The bamboo clump's row in `KINDS`.
const BAMBOO_CLUMP: usize = 10;

const P: PlayerId = PlayerId(0);
/// High in the sky, so nothing generated is near.
const Y: i32 = 180;

/// A sim with a stone floor at `Y` round `(0, 0)` and a grown tree of `kind` standing on it at `at`.
fn with_tree(kinds: &[(usize, IVec3)]) -> Sim {
    let mut sim = Sim::new(1337, 2);
    for z in -24..=24 {
        for x in -24..=24 {
            for y in Y - 4..Y {
                sim.world.set_block_anywhere_later(IVec3::new(x, y, z), STONE);
            }
            sim.world.set_block_anywhere_later(IVec3::new(x, Y, z), GRASS);
        }
    }
    for &(kind, at) in kinds {
        if KINDS[kind].ground == SAND {
            sim.world.set_block_anywhere_later(at, SAND);
        }
        let world = &mut sim.world;
        grow(kind, at, height_of(kind, 3), 1337, |p, b, replace| {
            if replace || world.block_anywhere_or_generate(p) == AIR {
                world.set_block_anywhere_later(p, b);
            }
        });
    }
    sim
}

fn give(sim: &mut Sim, item: ItemId, n: u32) {
    let inv = &mut sim.players[0].as_mut().unwrap().inventory;
    inv.add(item, n);
    inv.selected = inv.slots.iter().position(|s| s.item == item).unwrap_or(0);
}

/// The trunk standing at `at` (the tree's foot cell is one above the ground block).
fn trunk_at(sim: &mut Sim, at: IVec3) -> Trunk {
    let world = &mut sim.world;
    (0..4)
        .find_map(|up| trunk(|p| world.block_anywhere_or_generate(p), at + IVec3::new(0, 1 + up, 0)))
        .expect("a trunk")
}

/// Swings at the trunk until it falls (on its mark each time, or always beside it); how many it took.
fn swings_to_fell(sim: &mut Sim, t: &Trunk, on_mark: bool) -> u32 {
    let seed = sim.world.generator().seed();
    for n in 1..=5000 {
        let mark = mark_of(seed, t, sim.felling.marks(t.foot).unwrap_or(0));
        // Beside the mark: the far side of the trunk.
        let at = if on_mark {
            mark
        } else {
            Vec3::new(2.0 * t.centre[0] as f64 - mark.x, mark.y, 2.0 * t.centre[1] as f64 - mark.z)
        };
        sim.apply(P, Action::ChopTree { pos: t.foot, at, from: 0 });
        if sim.felling.marks(t.foot).is_none() {
            return n;
        }
    }
    panic!("never fell");
}

#[test]
fn every_kind_falls_in_about_the_time_its_trunk_takes_to_mine() {
    for kind in 0..KINDS.len() {
        for (tool, speed) in [(None, 1.0), (Some(STONE_AXE), 2.0), (Some(IRON_AXE), 4.0), (Some(STEEL_AXE), 6.0)] {
            let at = IVec3::new(0, Y, 0);
            let mut sim = with_tree(&[(kind, at)]);
            if let Some(t) = tool {
                give(&mut sim, t, 1000);
            }
            let t = trunk_at(&mut sim, at);
            let log = sim.world.block_anywhere_or_generate(t.foot);
            let mining = t.logs as f32 * block::def(log).break_time / speed;
            let felling = swings_to_fell(&mut sim, &t, true) as f32 * SWING_SECONDS;
            // Within 15%, or a swing for the smallest trees.
            assert!(
                (felling - mining).abs() <= (mining * 0.15).max(SWING_SECONDS),
                "kind {kind} with {tool:?}: {felling} s felling, {mining} s mining {} logs",
                t.logs
            );
        }
    }
}

#[test]
fn swings_beside_the_mark_take_about_twice_as_long() {
    let at = IVec3::new(0, Y, 0);
    let (mut a, mut b) = (with_tree(&[(0, at)]), with_tree(&[(0, at)]));
    let t = trunk_at(&mut a, at);
    let on = swings_to_fell(&mut a, &t, true);
    let off = swings_to_fell(&mut b, &t, false);
    assert!(off >= on * 2 - 2 && off <= on * 2 + 1, "on the mark {on}, beside it {off}");
}

#[test]
fn a_felled_tree_leaves_nothing_standing_and_lands_its_logs_where_its_trunk_lies() {
    for kind in 0..KINDS.len() {
        let at = IVec3::new(0, Y, 0);
        let mut sim = with_tree(&[(kind, at)]);
        let t = trunk_at(&mut sim, at);
        let wood = wood::of_tree(sim.world.block_anywhere_or_generate(t.foot)).unwrap();
        swings_to_fell(&mut sim, &t, true);
        let fallen = sim.fallen.pop().expect("drawn falling");
        assert!(!fallen.blocks.is_empty());
        // Nothing of it stands: no tree wood, no leaf without a log within reach. A bamboo clump's other culms are
        // trees of their own and stay, with the leaves they hold.
        let mut logs_left = Vec::new();
        for z in -14..=14 {
            for y in Y - 3..Y + 60 {
                for x in -14..=14 {
                    let p = IVec3::new(x, y, z);
                    let b = sim.world.block_anywhere_or_generate(p);
                    if is_tree_wood(b) {
                        assert_eq!(kind, BAMBOO_CLUMP, "kind {kind}: a log left at {p:?}");
                        logs_left.push(p);
                    }
                }
            }
        }
        assert!(t.base.iter().all(|&p| sim.world.block_anywhere_or_generate(p) == AIR));
        for z in -14..=14 {
            for y in Y - 3..Y + 60 {
                for x in -14..=14 {
                    let p = IVec3::new(x, y, z);
                    if is_leaves(sim.world.block_anywhere_or_generate(p)) {
                        let near = logs_left.iter().any(|l| { let d = *l - p; d.x.abs() + d.y.abs() + d.z.abs() } <= crate::sim::LEAF_SUPPORT_STEPS);
                        assert!(near, "kind {kind}: a leaf no log holds at {p:?}");
                    }
                }
            }
        }
        // Nothing lands until it has fallen, then the trunk's logs, all of them, along where it lies.
        sim.events.clear();
        for _ in 0..FALL_TICKS - 1 {
            sim.step();
        }
        assert!(!sim.events.iter().any(|e| matches!(e, SimEvent::Dropped { .. })), "kind {kind}: lands early");
        sim.step();
        let mut logs = 0;
        for e in &sim.events {
            if let SimEvent::Dropped { pos, item, count, .. } = *e {
                let b = sim.world.block_anywhere_or_generate(pos.floor());
                assert!(
                    !block::def(b).solid || is_leaves(b),
                    "kind {kind}: {} lands inside {}",
                    crate::item::name(item),
                    block::def(b).name
                );
                if item == wood::item(wood, Form::Log) {
                    logs += count;
                    // Falling away from +x (the side `from` 0): along -x, within a block and a half of the line.
                    assert!(
                        (pos.z as f32 - t.centre[1]).abs() <= 1.5 && (pos.x as f32) < t.centre[0],
                        "kind {kind}: {pos:?}"
                    );
                }
            }
        }
        assert!(logs >= t.logs && logs > 0, "kind {kind}: {logs} logs landed, the trunk had {}", t.logs);
    }
}

#[test]
fn a_grown_tree_does_not_mine_a_block_at_a_time_but_a_placed_log_does() {
    let at = IVec3::new(0, Y, 0);
    let mut sim = with_tree(&[(3, at)]);
    let t = trunk_at(&mut sim, at);
    sim.apply(P, Action::BreakBlock { pos: t.foot });
    assert!(is_tree_wood(sim.world.block_anywhere_or_generate(t.foot)));
    let wall = IVec3::new(10, Y + 1, 10);
    sim.world.set_block_anywhere_later(wall, LOG);
    sim.apply(P, Action::BreakBlock { pos: wall });
    assert_eq!(sim.world.block_anywhere_or_generate(wall), AIR);
    // Swinging at a placed log is nothing.
    sim.world.set_block_anywhere_later(wall, LOG);
    sim.apply(P, Action::ChopTree { pos: wall, at: wall.as_vec3(), from: 0 });
    assert!(sim.felling.chops.is_empty());
}

#[test]
fn a_swing_high_up_the_trunk_does_nothing() {
    let at = IVec3::new(0, Y, 0);
    let mut sim = with_tree(&[(3, at)]);
    let t = trunk_at(&mut sim, at);
    let high = IVec3::new(t.foot.x, t.low + LOW, t.foot.z);
    assert!(is_tree_wood(sim.world.block_anywhere_or_generate(high)));
    sim.apply(P, Action::ChopTree { pos: high, at: high.as_vec3(), from: 0 });
    assert_eq!(sim.felling.marks(t.foot), None);
}

#[test]
fn a_neighbour_keeps_its_wood_and_crown() {
    let (a, b) = (IVec3::new(0, Y, 0), IVec3::new(7, Y, 0));
    let mut sim = with_tree(&[(0, a), (0, b)]);
    let count = |sim: &mut Sim| {
        let mut n = (0, 0);
        for z in -12..=12 {
            for y in Y..Y + 30 {
                for x in 4..=20 {
                    let blk = sim.world.block_anywhere_or_generate(IVec3::new(x, y, z));
                    n.0 += u32::from(is_log(blk) && x >= 7);
                    n.1 += u32::from(is_leaves(blk) && x >= 8);
                }
            }
        }
        n
    };
    let before = count(&mut sim);
    let t = trunk_at(&mut sim, a);
    swings_to_fell(&mut sim, &t, true);
    let after = count(&mut sim);
    assert_eq!(after.0, before.0, "the neighbour's logs");
    assert!(after.1 * 10 >= before.1 * 9, "the neighbour keeps its crown: {after:?} of {before:?}");
}

#[test]
fn chops_and_falls_survive_a_save() {
    let at = IVec3::new(0, Y, 0);
    let mut sim = with_tree(&[(0, at), (3, IVec3::new(12, Y, 12))]);
    let t = trunk_at(&mut sim, at);
    sim.apply(P, Action::ChopTree { pos: t.foot, at: t.foot.as_vec3(), from: 3 });
    let other = trunk_at(&mut sim, IVec3::new(12, Y, 12));
    swings_to_fell(&mut sim, &other, true);
    let mut w = ByteWriter::default();
    sim.felling.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    assert_eq!(Felling::read_state(&mut r).as_ref(), Some(&sim.felling));
    assert!(sim.felling.work(t.foot) > 0 && !sim.felling.falls.is_empty());
}

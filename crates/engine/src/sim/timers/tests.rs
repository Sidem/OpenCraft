//! Block timers: leaf decay and grass spreading, on bare cores (no loaded chunks), compared by hash.

use super::*;
use crate::action::Action;
use crate::block::STONE;
use crate::sim::tests::SEED;
use crate::sim::PlayerId;

const A: PlayerId = PlayerId(0);
/// High enough to be open sky everywhere near the origin.
const SKY: i32 = 230;

fn run(sim: &mut Sim, seconds: u32) -> usize {
    let mut decayed = 0;
    for _ in 0..seconds * TICK_RATE {
        sim.step();
        decayed += sim.events.iter().filter(|e| matches!(e, SimEvent::LeafDecayed { .. })).count();
        sim.events.clear();
    }
    decayed
}

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn place(sim: &mut Sim, pos: IVec3, block: BlockId) {
    sim.apply(A, Action::Give { item: block.into(), count: 1 });
    let slot = sim.player(A).unwrap().inventory.slots.iter().position(|s| s.item == block.into()).unwrap();
    sim.apply(A, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos - at(0, 1, 0) });
    assert_eq!(sim.block(pos), block, "placed");
}

/// A generated tree near the origin: its log positions, bottom first.
fn tree(sim: &mut Sim) -> Vec<IVec3> {
    for z in -48..48 {
        for x in -48..48 {
            let h = sim.world.generator().height_at(x, z);
            let Some(y) = (h - 3..h + 4).find(|&y| sim.block(at(x, y, z)) == LOG && sim.block(at(x, y - 1, z)) == DIRT)
            else {
                continue;
            };
            return (y..).take_while(|&y| sim.block(at(x, y, z)) == LOG).map(|y| at(x, y, z)).collect();
        }
    }
    panic!("no tree near the origin");
}

fn leaves_near(sim: &mut Sim, top: IVec3) -> Vec<IVec3> {
    cube(top, 3).filter(|&p| sim.block(p) == LEAVES).collect()
}

#[test]
fn a_felled_tree_loses_its_leaves_within_a_minute_on_every_core() {
    let (mut a, mut b) = (Sim::new(SEED, 2), Sim::new(SEED, 2));
    let logs = tree(&mut a);
    let top = *logs.last().unwrap();
    let before = leaves_near(&mut a, top).len();
    assert!(before > 15, "{before} leaves");
    for sim in [&mut a, &mut b] {
        for &p in &logs {
            sim.queue(0, A, Action::BreakBlock { pos: p });
        }
    }
    let decayed = run(&mut a, 60);
    run(&mut b, 60);
    assert_eq!(a.state_hash(), b.state_hash());
    // Whatever is left belongs to a neighbouring tree.
    let left = leaves_near(&mut a, top);
    assert!(left.iter().all(|&p| a.leaf_supported(p)), "{} unsupported leaves left", left.len());
    assert!(decayed + left.len() >= before && decayed > 15, "{decayed} decayed of {before}");
    assert_eq!(a.timers.list.iter().filter(|t| t.kind == TimerKind::LeafDecay).count(), 0);
}

#[test]
fn leaves_held_by_another_log_stay() {
    let mut sim = Sim::new(SEED, 2);
    // Logs at x = 0 and x = 3, leaves bridging them, and a leaf only the first log holds.
    for (x, b) in [(0, LOG), (1, LEAVES), (2, LEAVES), (3, LOG), (-1, LEAVES)] {
        sim.world.set_block_anywhere(at(x, SKY, 0), b);
    }
    sim.apply(A, Action::BreakBlock { pos: at(0, SKY, 0) });
    assert!(sim.timers.pending(at(-1, SKY, 0), TimerKind::LeafDecay));
    assert!(!sim.timers.pending(at(1, SKY, 0), TimerKind::LeafDecay));
    assert_eq!(run(&mut sim, 60), 1);
    assert_eq!(sim.block(at(-1, SKY, 0)), AIR);
    assert_eq!((sim.block(at(1, SKY, 0)), sim.block(at(2, SKY, 0))), (LEAVES, LEAVES));

    // A log placed before a leaf's timer fires saves it.
    for (x, b) in [(10, LOG), (11, LEAVES)] {
        sim.world.set_block_anywhere(at(x, SKY, 0), b);
    }
    sim.apply(A, Action::BreakBlock { pos: at(10, SKY, 0) });
    place(&mut sim, at(12, SKY, 0), LOG);
    run(&mut sim, 60);
    assert_eq!(sim.block(at(11, SKY, 0)), LEAVES);
}

#[test]
fn a_save_mid_decay_resumes() {
    let mut a = Sim::new(SEED, 2);
    for &p in &tree(&mut a) {
        a.queue(0, A, Action::BreakBlock { pos: p });
    }
    run(&mut a, 2);
    assert!(a.timers.len() > 5, "decay under way");
    let mut w = ByteWriter::default();
    a.write_state(&mut w);
    let mut b = Sim::new(SEED, 2);
    b.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    assert_eq!(a.state_hash(), b.state_hash());
    for t in 0..60 * TICK_RATE {
        a.step();
        b.step();
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
    }
    assert_eq!(b.timers.len(), a.timers.len());
}

/// A two-layer platform in the sky: dirt under grass, `r` blocks around the origin.
fn meadow(sim: &mut Sim, r: i32) {
    for z in -r..=r {
        for x in -r..=r {
            sim.world.set_block_anywhere(at(x, SKY - 1, z), DIRT);
            sim.world.set_block_anywhere(at(x, SKY, z), GRASS);
        }
    }
}

#[test]
fn bare_dirt_beside_grass_greens_edge_inwards_on_every_core() {
    let (mut a, mut b) = (Sim::new(SEED, 2), Sim::new(SEED, 2));
    for sim in [&mut a, &mut b] {
        meadow(sim, 5);
        // Dig the grass off a 5 × 5 patch, baring the dirt below.
        for z in -2..=2 {
            for x in -2..=2 {
                sim.apply(A, Action::BreakBlock { pos: at(x, SKY, z) });
            }
        }
    }
    let ring = |x: i32, z: i32| 2 - x.abs().max(z.abs()); // 0 = the patch's edge, 2 = its middle
    assert!(a.timers.pending(at(2, SKY - 1, 0), TimerKind::GrassGrow));
    // Inner cells got timers while the grass next to them was still there; they recheck when fired.
    assert!(!a.grass_can_grow(at(1, SKY - 1, 0)), "only the edge touches grass");
    let mut greened = [[0u32; 5]; 5];
    for s in 1..=600 {
        run(&mut a, 1);
        run(&mut b, 1);
        for z in -2..=2 {
            for x in -2..=2 {
                let cell = &mut greened[(z + 2) as usize][(x + 2) as usize];
                if *cell == 0 && a.block(at(x, SKY - 1, z)) == GRASS {
                    *cell = s;
                }
            }
        }
    }
    assert_eq!(a.state_hash(), b.state_hash());
    let when = |x: i32, z: i32| greened[(z + 2) as usize][(x + 2) as usize];
    assert!(greened.iter().flatten().all(|&s| s > 0 && s <= 300), "all green within five minutes: {greened:?}");
    // Edge inwards: a cell inside the patch greens only once a cell next to it has.
    for (x, z) in (-1..=1).flat_map(|z| (-1..=1).map(move |x| (x, z))) {
        let first_neighbour = RING.iter().map(|&(dx, dz)| when(x + dx, z + dz)).min().unwrap();
        assert!(when(x, z) >= first_neighbour, "{x}, {z}: {greened:?}");
    }
    let edge_first = (-2..=2).flat_map(|z| (-2..=2).map(move |x| (x, z))).filter(|&(x, z)| ring(x, z) == 0);
    assert!(edge_first.map(|(x, z)| when(x, z)).min() < Some(when(0, 0)), "{greened:?}");
}

#[test]
fn covered_grass_turns_to_dirt_and_grows_back_when_uncovered() {
    let mut sim = Sim::new(SEED, 2);
    meadow(&mut sim, 2);
    place(&mut sim, at(0, SKY + 1, 0), STONE);
    assert!(sim.timers.pending(at(0, SKY, 0), TimerKind::GrassDie));
    run(&mut sim, 180);
    assert_eq!(sim.block(at(0, SKY, 0)), DIRT);
    sim.apply(A, Action::BreakBlock { pos: at(0, SKY + 1, 0) });
    run(&mut sim, 300);
    assert_eq!(sim.block(at(0, SKY, 0)), GRASS);
}

#[test]
fn timers_are_capped_and_ordered() {
    let mut timers = BlockTimers::default();
    for i in 0..MAX_TIMERS as i32 + 10 {
        timers.schedule(Timer { due: (i % 7) as u64, pos: at(i, 0, 0), kind: TimerKind::GrassGrow });
    }
    assert_eq!(timers.len(), MAX_TIMERS);
    timers.schedule(Timer { due: 0, pos: at(0, 0, 0), kind: TimerKind::GrassGrow });
    assert_eq!(timers.len(), MAX_TIMERS, "no duplicates");
    assert!(timers.list.windows(2).all(|w| (w[0].due, w[0].pos.x) < (w[1].due, w[1].pos.x)));
}

//! Determinism (DEV_PLAN step 1.5): cores fed the same actions keep equal state hashes, whatever
//! chunks happen to be loaded, and a core read back from its bytes carries on identically.

use super::*;
use crate::block::{AIR, BELT, IRON_ORE, MINER, SPENT_ROCK, STONE, STORAGE};
use crate::deposits::{owner_of, DepositKey, Tier};
use crate::recipes::RECIPES;

const SEED: u32 = 1337;
const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);

/// Generates the chunks around `p` (no meshing), as streaming around a player would.
fn load_around(sim: &mut Sim, p: IVec3) {
    sim.world.update_streaming(p.as_vec3());
    while sim.world.work_step() {}
}

/// The outcrop nearest the origin: its top ore block, another of its ore blocks, and its key.
fn outcrop() -> (IVec3, IVec3, DepositKey) {
    let mut sim = Sim::new(SEED, 2);
    let d = sim.world.generator().find_deposit(IVec3::ZERO, Tier::Outcrop, 16).expect("an outcrop");
    load_around(&mut sim, d.center);
    let (lo, hi) = d.bounds();
    let mut ore = Vec::new();
    for y in (lo.y..=hi.y).rev() {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                let p = IVec3::new(x, y, z);
                if owner_of(&mut sim.world, p).is_some_and(|o| o.key == d.key) {
                    ore.push(p);
                }
            }
        }
    }
    assert!(ore.len() >= 2, "{} ore blocks", ore.len());
    (ore[0], ore[ore.len() - 1], d.key)
}

/// The action log: A builds a miner on the outcrop's top block feeding two belts into a box, crafts
/// belts and hand-mines another ore block; B joins and puts stone on the box.
fn script(top: IVec3, other: IVec3) -> Vec<(u64, PlayerId, Action)> {
    let cell = |dx| top + IVec3::new(dx, 1, 0);
    let above_box = cell(3) + IVec3::new(0, 1, 0);
    let belts = RECIPES.iter().position(|r| r.output == BELT).unwrap() as u16;
    let give = |item, count| Action::Give { item, count };
    let place = |pos, slot, facing, against| Action::PlaceBlock { pos, slot, facing, against };
    // A's slots: 0 miner, 1 belts, 2 box.
    let mut log = vec![
        (0, A, give(MINER, 1)),
        (0, A, give(BELT, 1)),
        (0, A, give(STORAGE, 1)),
        (0, A, give(IRON_ORE, 1)),
        (0, A, give(STONE, 2)),
        (0, A, Action::Craft { recipe: belts, times: 1 }),
        (0, B, Action::Join),
        (0, B, give(STONE, 3)),
        (1, B, Action::BreakBlock { pos: above_box }),
    ];
    // Clear the cells first (breaking air does nothing).
    log.extend((0..4).map(|dx| (1, A, Action::BreakBlock { pos: cell(dx) })));
    log.extend([
        (2, A, place(cell(0), 0, 0, top)),
        (2, A, place(cell(1), 1, 1, cell(1))),
        (2, A, place(cell(2), 1, 1, cell(2))),
        (2, A, place(cell(3), 2, 0, cell(3))),
        (3, B, place(above_box, 0, 0, cell(3))),
        (5, A, Action::BreakBlock { pos: other }),
    ]);
    log
}

/// A fresh core with the whole log queued.
fn scripted(log: &[(u64, PlayerId, Action)]) -> Sim {
    let mut sim = Sim::new(SEED, 2);
    for &(tick, player, action) in log {
        sim.queue(tick, player, action);
    }
    sim
}

fn step(sim: &mut Sim) {
    sim.step();
    sim.events.clear();
}

#[test]
fn same_actions_give_the_same_state_every_tick() {
    let (top, other, key) = outcrop();
    let log = script(top, other);
    let (mut a, mut b) = (scripted(&log), scripted(&log));
    let start = a.state_hash();
    assert_eq!(start, b.state_hash());
    // 105 s: past the 100 units that turn the drilled block into spent rock.
    for t in 0..6300 {
        step(&mut a);
        step(&mut b);
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
    }
    assert_ne!(a.state_hash(), start);

    // The scenario really ran.
    assert_eq!(a.world.block_anywhere_or_generate(top), SPENT_ROCK);
    assert_eq!(a.world.block_anywhere_or_generate(other), AIR);
    let st = a.factory.deposits.get(&key).unwrap();
    assert_eq!(st.remaining_blocks, st.initial_blocks - 2, "one hand-mined, one drilled");
    assert!(a.factory.storage_count_at(top + IVec3::new(3, 1, 0), key.ore) > 30);
    assert_eq!(a.player(B).unwrap().inventory.count(STONE), 2, "B placed one");
}

#[test]
fn a_reloaded_core_carries_on_identically() {
    let (top, other, key) = outcrop();
    let mut a = scripted(&script(top, other));
    for _ in 0..3000 {
        step(&mut a);
    }
    let mut w = ByteWriter::default();
    a.write_state(&mut w);
    let mut b = Sim::new(SEED, 2);
    b.read_state(&mut ByteReader::new(&w.bytes)).expect("reads back");
    let mut again = ByteWriter::default();
    b.write_state(&mut again);
    assert!(again.bytes == w.bytes, "the same bytes after a round trip");

    // Past the first spent rock, which happens after the reload.
    for t in 3000..6300 {
        step(&mut a);
        step(&mut b);
        assert_eq!(a.state_hash(), b.state_hash(), "tick {t}");
    }
    assert_eq!(b.world.block_anywhere_or_generate(top), SPENT_ROCK);
    assert_eq!(
        b.factory.deposits.get(&key).unwrap().remaining_blocks,
        a.factory.deposits.get(&key).unwrap().remaining_blocks
    );
}

#[test]
fn loaded_chunks_do_not_change_the_state() {
    let (top, other, _) = outcrop();
    let log = script(top, other);
    let (mut bare, mut loaded) = (scripted(&log), scripted(&log));
    load_around(&mut loaded, top);
    assert_eq!(bare.state_hash(), loaded.state_hash(), "loading chunks is not an edit");
    // The loaded core edits loaded chunks, streams them out (they move to storage) and back in.
    for t in 0..900 {
        match t {
            300 => load_around(&mut loaded, top + IVec3::new(4000, 0, 0)),
            600 => load_around(&mut loaded, top),
            _ => {}
        }
        step(&mut bare);
        step(&mut loaded);
        assert_eq!(bare.state_hash(), loaded.state_hash(), "tick {t}");
    }
    assert_eq!(loaded.world.get_block(top + IVec3::new(0, 1, 0)), Some(MINER), "the edits came back");
}

#[test]
fn the_hash_covers_inventories_players_and_time() {
    let hash = |change: &dyn Fn(&mut Sim)| {
        let mut sim = Sim::new(SEED, 2);
        change(&mut sim);
        sim.state_hash()
    };
    let fresh = hash(&|_| {});
    assert_eq!(fresh, Sim::new(SEED, 2).state_hash());
    assert_ne!(hash(&|s| s.apply(A, Action::Give { item: STONE, count: 1 })), fresh);
    assert_ne!(hash(&|s| s.apply(A, Action::SelectSlot { slot: 3 })), fresh);
    assert_ne!(hash(&|s| s.apply(B, Action::Join)), fresh);
    assert_ne!(hash(&|s| s.step()), fresh);
    assert_ne!(Sim::new(SEED + 1, 2).state_hash(), fresh, "the rng follows the seed");
    // A player who left leaves no trace.
    assert_eq!(
        hash(&|s| {
            s.apply(B, Action::Join);
            s.apply(B, Action::Leave);
        }),
        fresh
    );
}

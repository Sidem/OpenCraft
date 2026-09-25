use super::*;
use crate::deposits::DepositKey;
use crate::factory::MinerStatus;

fn run_until_ready(g: &mut Game) {
    for _ in 0..10_000 {
        g.update(1.0 / 60.0);
        g.begin_work();
        while g.work_step() {}
        while g.next_event() != 0 {}
        if g.ready() {
            return;
        }
    }
    panic!("world never became ready");
}

/// An outcrop ore block near spawn whose deposit has at least `min_blocks` blocks.
fn find_outcrop_block(g: &mut Game, min_blocks: u32) -> (IVec3, DepositKey) {
    let base = g.player.pos.floor();
    for y in (8..base.y + 4).rev() {
        for z in -40..40 {
            for x in -40..40 {
                let p = IVec3::new(base.x + x, y, base.z + z);
                let Some(b) = g.world.get_block(p) else { continue };
                if !block::is_ore(b) {
                    continue;
                }
                if let Some(key) = g.factory.deposits.lookup(&mut g.world, p) {
                    let st = g.factory.deposits.get(&key).unwrap();
                    if key.tier == Tier::Outcrop && st.initial_blocks >= min_blocks {
                        return (p, key);
                    }
                }
            }
        }
    }
    panic!("no outcrop near spawn");
}

#[test]
fn mine_collect_place_loop() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    for _ in 0..120 {
        g.update(1.0 / 60.0);
    }
    assert!(g.on_ground(), "player should be standing after spawning");
    g.clear_sounds();

    // Look straight down and mine the block underfoot.
    g.set_look(0.0, -1.5);
    g.update(1.0 / 60.0);
    assert!(g.has_target());
    let (tx, ty, tz) = (g.target_x(), g.target_y(), g.target_z());
    let mined = g.target_block();
    g.set_mining(true);
    for _ in 0..200 {
        g.update(1.0 / 60.0);
        if g.world.get_block(IVec3::new(tx, ty, tz)) == Some(AIR) {
            break;
        }
    }
    g.set_mining(false);
    assert_eq!(g.world.get_block(IVec3::new(tx, ty, tz)), Some(AIR), "block should be mined");

    // The drop falls into the hole and gets magnetised into the inventory.
    for _ in 0..300 {
        g.update(1.0 / 60.0);
    }
    let expected = block::def(mined).drop;
    assert_eq!(g.slot_item(0), expected);
    assert_eq!(g.slot_count(0), 1);
    assert!(g.next_pickup());
    assert_eq!(g.pickup_item(), expected);
    let heard = g.sounds.kinds();
    for kind in [sound::DIG, sound::BREAK, sound::LAND, sound::PICKUP] {
        assert!(heard.contains(&kind), "missing sound {kind} in {heard:?}");
    }
    g.clear_sounds();

    // Hover above the hole and put the block back.
    g.toggle_fly();
    g.teleport(tx as f64 + 0.5, ty as f64 + 3.0, tz as f64 + 0.5);
    g.set_look(0.0, -1.55);
    g.update(1.0 / 60.0);
    assert!(g.has_target());
    g.set_using(true);
    g.update(1.0 / 60.0);
    g.set_using(false);
    assert_eq!(g.slot_count(0), 0, "placing consumes the item");
    assert_eq!(g.world.get_block(IVec3::new(tx, ty, tz)), Some(expected));
    assert_eq!(g.sounds.kinds(), vec![sound::PLACE]);
}

#[test]
fn footsteps_and_landing_make_sounds() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }

    // Drop from 4 blocks up: one landing thud, positioned below the camera.
    g.clear_sounds();
    let (x, y, z) = (g.player_x(), g.player_y(), g.player_z());
    g.teleport(x, y + 4.0, z);
    for _ in 0..90 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sounds.kinds(), vec![sound::LAND]);
    g.clear_sounds();

    // Walking produces footsteps; flying produces none. Carve a flat corridor towards -Z
    // (the forward direction at yaw 0) so terrain can't block the walk.
    let feet = Vec3::new(g.player_x(), g.player_y(), g.player_z()).floor();
    for dz in 0..12 {
        let p = feet - IVec3::new(0, 0, dz);
        g.world.set_block(p - IVec3::new(0, 1, 0), block::STONE);
        g.world.set_block(p, AIR);
        g.world.set_block(p + IVec3::new(0, 1, 0), AIR);
    }
    g.teleport(feet.x as f64 + 0.5, feet.y as f64, feet.z as f64 + 0.5);
    g.set_look(0.0, 0.0);
    g.update(1.0 / 60.0);
    g.clear_sounds();
    g.set_move(1.0, 0.0, false, false, false);
    for _ in 0..90 {
        g.update(1.0 / 60.0);
    }
    let steps = g.sounds.kinds().iter().filter(|&&k| k == sound::STEP).count();
    assert!(steps >= 2, "expected footsteps, got {:?}", g.sounds.kinds());
    g.clear_sounds();
    g.toggle_fly();
    for _ in 0..90 {
        g.update(1.0 / 60.0);
    }
    assert!(g.sounds.kinds().is_empty());
}

#[test]
fn hand_mining_ore_keeps_a_handful_and_costs_a_block() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 2);
    let before = g.factory.deposits.get(&key).unwrap().remaining_blocks;
    let ore = g.world.get_block(p).unwrap();
    assert!(g.break_block(p, ore));
    assert_eq!(g.factory.deposits.get(&key).unwrap().remaining_blocks, before - 1);
    let drop = g.items.list.last().unwrap();
    assert_eq!((drop.item, drop.count), (ore, HAND_YIELD));
    assert!(!block::is_placeable(ore), "ore can't be put back");
}

/// Miner on top of an outcrop block, a belt leading east, and a box at the end.
fn build_mine(g: &mut Game, p: IVec3) -> (IVec3, IVec3) {
    let m = p + IVec3::new(0, 1, 0);
    let belt = m + IVec3::new(1, 0, 0);
    let chest = m + IVec3::new(2, 0, 0);
    g.world.set_block(m, MINER);
    g.world.set_block(belt, BELT);
    g.world.set_block(chest, STORAGE);
    let key = g.factory.deposits.lookup(&mut g.world, p);
    assert!(key.is_some());
    g.factory.add_miner(m, block::FACE_BOTTOM as u8, key);
    g.factory.add_belt(belt, 1);
    g.factory.add_storage(chest);
    (m, chest)
}

#[test]
fn miner_drills_the_pool_and_turns_blocks_into_spent_rock() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 6);
    let ore = key.ore;
    let (m, chest) = build_mine(&mut g, p);
    let st = g.factory.deposits.get(&key).unwrap();
    let (blocks, grade) = (st.remaining_blocks, st.grade() as f64);

    // An outcrop allows 1 unit/s; after 250 s, two blocks' worth (100 units each) are gone.
    g.skip_time(250.0);
    let st = g.factory.deposits.get(&key).unwrap();
    assert_eq!(st.remaining_blocks, blocks - 2);
    assert_eq!(g.world.get_block(p), Some(SPENT_ROCK), "the block under the drill goes first");
    let recovered = (250.0 * MINER_RECOVERY) as u32;
    let stored = g.factory.storage_count_at(chest, ore);
    assert!(stored + 4 >= recovered && stored <= recovered, "stored {stored} of ~{recovered}");
    assert_eq!(g.factory.miner_at(m).status, MinerStatus::Running);
    assert!((st.remaining_units() - (blocks as f64 * grade - 250.0)).abs() < 1.0);

    // The readout names the deposit.
    let detail = g.factory.describe(m).unwrap();
    assert!(detail.contains("outcrop"), "{detail}");
}

#[test]
fn miner_without_output_fills_up_and_stops() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 2);
    let m = p + IVec3::new(0, 1, 0);
    g.world.set_block(m, MINER);
    let k = g.factory.deposits.lookup(&mut g.world, p);
    g.factory.add_miner(m, block::FACE_BOTTOM as u8, k);
    let units = g.factory.deposits.get(&key).unwrap().remaining_units();
    g.skip_time(300.0);
    let miner = g.factory.miner_at(m);
    assert_eq!(miner.status, MinerStatus::OutputFull);
    assert_eq!(miner.held, factory::MINER_BUFFER);
    let used = units - g.factory.deposits.get(&key).unwrap().remaining_units();
    assert!(used < 110.0, "a full miner stops drawing, used {used}");

    // Right-click empties it into the inventory.
    assert!(g.take_from_machine(m));
    assert_eq!(g.item_total(key.ore), factory::MINER_BUFFER);
}

#[test]
fn crafting_consumes_inputs() {
    let mut g = Game::new(7, 2);
    g.give(block::IRON_ORE, 3);
    g.give(block::STONE, 5);
    let belt = RECIPES.iter().position(|r| r.output == BELT).unwrap() as u32;
    assert_eq!(g.craft(belt, 5), 2);
    assert_eq!(g.item_total(BELT), 8);
    assert_eq!(g.item_total(block::IRON_ORE), 1);
    assert_eq!(g.item_total(block::STONE), 1);
    assert!(!g.can_craft(belt));
}

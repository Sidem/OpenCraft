//! Game-level scenario tests: mining, placing, sounds, miners and crafting through the `Game` API.

use super::*;
use crate::block::{AIR, BELT, MINER, SPENT_ROCK, STONE, STORAGE};
use crate::deposits::{DepositKey, Tier, HAND_YIELD};
use crate::factory::{MinerStatus, MINER_TIERS};
const MINER_RECOVERY: f64 = MINER_TIERS[0].recovery;
use crate::inventory::INVENTORY_SLOTS;
use crate::math::Rng;
use crate::recipes::RECIPES;
use crate::sim::SimEvent;

pub(crate) fn run_until_ready(g: &mut Game) {
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

/// Loaded ore blocks near spawn, top layer first.
pub(crate) fn nearby_ore(g: &Game) -> Vec<IVec3> {
    let base = g.body().pos.floor();
    let mut out = Vec::new();
    for y in (8..base.y + 4).rev() {
        for z in -40..40 {
            for x in -40..40 {
                let p = IVec3::new(base.x + x, y, base.z + z);
                if g.sim.world.get_block(p).is_some_and(block::is_ore) {
                    out.push(p);
                }
            }
        }
    }
    out
}

/// An outcrop ore block near spawn whose deposit has at least `min_blocks` blocks (now tracked).
pub(crate) fn find_outcrop_block(g: &mut Game, min_blocks: u32) -> (IVec3, DepositKey) {
    for p in nearby_ore(g) {
        if let Some(key) = g.sim.factory.deposits.lookup(&mut g.sim.world, p) {
            let st = g.sim.factory.deposits.get(&key).unwrap();
            if key.tier == Tier::Outcrop && st.initial_blocks >= min_blocks {
                return (p, key);
            }
        }
    }
    panic!("no outcrop near spawn");
}

/// Points the local player's hands at `block` (targeting normally comes from the camera).
fn aim(g: &mut Game, block: IVec3) {
    let id = g.sim.world.get_block(block).unwrap();
    g.target = Some(RayHit { block, normal: IVec3::new(0, 1, 0), id });
}

#[test]
fn a_ladder_the_player_stands_in_does_not_block_the_hands() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    let at = |x, h, z| IVec3::new(x, h, z);
    for x in 0..=6 {
        for h in y - 2..=y + 5 {
            g.sim.world.set_block_anywhere(at(x, h, 0), if h <= y { STONE } else { AIR });
        }
    }
    let (ladder, wall, beside) = (at(3, y + 1, 0), at(5, y + 2, 0), at(4, y + 2, 0));
    for h in [y + 1, y + 2] {
        g.sim.world.set_block_anywhere(at(3, h, 0), block::LADDER);
    }
    g.sim.world.set_block_anywhere(wall, STONE);
    let body = g.body_mut();
    body.pos = Vec3::new(3.5, (y + 1) as f64, 0.5);
    body.yaw = std::f64::consts::FRAC_PI_2;
    body.pitch = 0.0;
    g.update_target();
    let hit = g.target.expect("something is aimed at");
    assert_eq!(hit.block, wall, "the ladder around the player is looked through");
    // A ladder in front of the player (not the one stood in) is still a target.
    g.sim.world.set_block_anywhere(beside, block::LADDER);
    g.update_target();
    assert_eq!(g.target.map(|h| h.block), Some(beside));
    assert_ne!(ladder, beside);
}

#[test]
fn target_detail_leaves_the_core_unchanged() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let before = g.sim.state_hash();
    let p = nearby_ore(&g)[0];
    aim(&mut g, p);

    let detail = g.target_detail();
    assert!(detail.contains("blocks left"), "{detail}");
    assert_eq!(g.sim.factory.deposits.tracked(), 0, "looking must not track the deposit");
    assert_eq!(g.target_detail(), detail, "the survey is cached and stable");
    assert_eq!(g.sim.state_hash(), before);

    // Once something tracks the deposit, the readout shows the same figures from the core.
    g.sim.factory.deposits.lookup(&mut g.sim.world, p).unwrap();
    assert_eq!(g.target_detail(), detail);

    // Machine readouts are pure too.
    let (m, chest) = build_mine(&mut g, p);
    g.run_ticks(60);
    let before = g.sim.state_hash();
    for block in [m, m + IVec3::new(1, 0, 0), chest, p] {
        aim(&mut g, block);
        assert!(!g.target_detail().is_empty());
    }
    assert_eq!(g.sim.state_hash(), before);
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
        if g.sim.world.get_block(IVec3::new(tx, ty, tz)) == Some(AIR) {
            break;
        }
    }
    g.set_mining(false);
    assert_eq!(g.sim.world.get_block(IVec3::new(tx, ty, tz)), Some(AIR), "block should be mined");

    // The drop falls into the hole and gets magnetised into the inventory.
    for _ in 0..300 {
        g.update(1.0 / 60.0);
    }
    let expected = block::def(mined).drop as u16;
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
    assert_eq!(g.sim.world.get_block(IVec3::new(tx, ty, tz)), Some(expected as u8));
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
        g.sim.world.set_block(p - IVec3::new(0, 1, 0), block::STONE);
        g.sim.world.set_block(p, AIR);
        g.sim.world.set_block(p + IVec3::new(0, 1, 0), AIR);
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
fn standing_on_a_belt_carries_the_player_without_footsteps() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    // A flat stone floor with an east-bound belt line laid on it (direction 1), a few cells wide.
    let feet = Vec3::new(g.player_x(), g.player_y(), g.player_z()).floor();
    for (dx, dz) in (-2..=14).flat_map(|dx| (-2..=2).map(move |dz| (dx, dz))) {
        let p = feet + IVec3::new(dx, 0, dz);
        g.sim.world.set_block(p - IVec3::new(0, 1, 0), STONE);
        for up in 0..3 {
            g.sim.world.set_block(p + IVec3::new(0, up, 0), AIR);
        }
    }
    for dx in 0..=12 {
        let belt = feet + IVec3::new(dx, 0, 0);
        g.sim.world.set_block(belt, BELT);
        g.sim.factory.add_belt(belt, 1);
    }
    g.teleport(feet.x as f64 + 0.5, feet.y as f64, feet.z as f64 + 0.5);
    g.run_ticks(30);
    let (start, sounds) = (g.player_x(), g.sounds.kinds().len());
    g.run_ticks(60);
    let carried = g.player_x() - start;
    assert!((carried - 1.0).abs() < 0.1, "a Mk1 belt carries a body one block a second, moved {carried}");
    assert!((g.player_z() - (feet.z as f64 + 0.5)).abs() < 1e-6, "and only along its heading");
    assert_eq!(g.sounds.kinds().len(), sounds, "being carried makes no footsteps");

    // Flying is never carried.
    g.toggle_fly();
    let up = g.player_x();
    g.run_ticks(30);
    assert!((g.player_x() - up).abs() < 1e-6, "a flying player stays where it is");
}

#[test]
fn a_player_splashes_into_a_pool_where_items_float_and_steps_are_silent() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    for _ in 0..60 {
        g.update(1.0 / 60.0);
    }
    // A 5 × 5 pool, 5 deep, dug where the player stands (only the render cache: bodies aren't core).
    let feet = Vec3::new(g.player_x(), g.player_y(), g.player_z()).floor();
    for dx in -2..=2 {
        for dz in -2..=2 {
            for dy in -6..4 {
                let b = match dy {
                    -6 => block::STONE,
                    -5..0 => block::WATER,
                    _ => AIR,
                };
                g.sim.world.set_block(feet + IVec3::new(dx, dy, dz), b);
            }
        }
    }
    let (x, z) = (feet.x as f64 + 0.5, feet.z as f64 + 0.5);
    g.spawn_item(Vec3::new(x, feet.y as f64 - 4.5, z), Vec3::ZERO, STONE.into(), 1, 0.0);
    g.toggle_fly();
    g.teleport(x, feet.y as f64 + 20.0, z);
    for _ in 0..300 {
        g.update(1.0 / 60.0);
    }
    let item = g.items.list[0].pos;
    assert!((item.y - feet.y as f64).abs() < 0.2, "the item floats at the surface: {item:?}");

    g.toggle_fly();
    g.teleport(x, feet.y as f64 + 3.0, z);
    g.clear_sounds();
    for _ in 0..240 {
        g.update(1.0 / 60.0);
    }
    assert!(g.body().in_water && g.body().on_ground, "sank to the bottom");
    let mut heard = g.sounds.kinds();
    heard.sort_unstable();
    assert_eq!(heard, vec![sound::PICKUP, sound::SPLASH], "picked the item up on the way down");
    g.clear_sounds();
    g.set_move(1.0, 0.0, false, false, false);
    for _ in 0..90 {
        g.update(1.0 / 60.0);
    }
    assert!(g.sounds.kinds().is_empty(), "no footsteps in water: {:?}", g.sounds.kinds());
}

#[test]
fn hand_mining_ore_keeps_a_handful_and_costs_a_block() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 2);
    let before = g.sim.factory.deposits.get(&key).unwrap().remaining_blocks;
    let ore = g.sim.world.get_block(p).unwrap();
    g.act(Action::BreakBlock { pos: p });
    g.run_ticks(1);
    assert_eq!(g.sim.world.get_block(p), Some(AIR));
    assert_eq!(g.sim.factory.deposits.get(&key).unwrap().remaining_blocks, before - 1);
    let drop = g.items.list.last().unwrap();
    assert_eq!((drop.item, drop.count), (ore.into(), HAND_YIELD));
    assert_eq!(drop.item.places(), None, "ore can't be put back");
}

/// Miner on top of an outcrop block, a belt leading east, and a box at the end.
pub(crate) fn build_mine(g: &mut Game, p: IVec3) -> (IVec3, IVec3) {
    let m = p + IVec3::new(0, 1, 0);
    let belt = m + IVec3::new(1, 0, 0);
    let chest = m + IVec3::new(2, 0, 0);
    g.sim.world.set_block(m, MINER);
    g.sim.world.set_block(belt, BELT);
    g.sim.world.set_block(chest, STORAGE);
    let key = g.sim.factory.deposits.lookup(&mut g.sim.world, p);
    assert!(key.is_some());
    g.sim.factory.add_miner(m, block::FACE_BOTTOM as u8, key, 0);
    g.sim.factory.add_belt(belt, 1);
    g.sim.factory.add_storage(chest);
    power_up(g, m);
    (m, chest)
}

/// A pole two above `m` and a generator above that with a stack of coal (factory only, no blocks).
pub(crate) fn power_up(g: &mut Game, m: IVec3) {
    let (pole, gen) = (m + IVec3::new(0, 2, 0), m + IVec3::new(0, 3, 0));
    g.sim.factory.place(&mut g.sim.world, block::POLE, pole, 0, pole, 0);
    g.sim.factory.place(&mut g.sim.world, block::GENERATOR, gen, 0, gen, 0);
    assert_eq!(g.sim.factory.insert(gen, block::COAL_ORE.into(), 64), 64);
}

#[test]
fn miner_drills_the_pool_and_turns_blocks_into_spent_rock() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 6);
    let ore = key.ore;
    let (m, chest) = build_mine(&mut g, p);
    let st = g.sim.factory.deposits.get(&key).unwrap();
    let (blocks, grade) = (st.remaining_blocks, st.grade() as f64);

    // An outcrop allows 1 unit/s; after 250 s, two blocks' worth (100 units each) are gone.
    g.skip_time(250.0);
    let st = g.sim.factory.deposits.get(&key).unwrap();
    assert_eq!(st.remaining_blocks, blocks - 2);
    assert_eq!(g.sim.world.get_block(p), Some(SPENT_ROCK), "the block under the drill goes first");
    let recovered = (250.0 * MINER_RECOVERY) as u32;
    let stored = g.sim.factory.storage_count_at(chest, ore.into());
    assert!(stored + 4 >= recovered && stored <= recovered, "stored {stored} of ~{recovered}");
    assert_eq!(g.sim.factory.miner_at(m).status, MinerStatus::Running);
    assert!((st.remaining_units() - (blocks as f64 * grade - 250.0)).abs() < 1.0);

    // The readout names the deposit.
    let detail = g.sim.factory.describe(m).unwrap();
    assert!(detail.contains("outcrop"), "{detail}");
}

#[test]
fn working_miner_reports_itself_and_is_heard_nearby() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 6);
    build_mine(&mut g, p);
    for _ in 0..120 {
        g.sim.step();
    }
    let mut events = std::mem::take(&mut g.sim.events);
    events.retain(|e| !matches!(e, SimEvent::Produced { .. }));
    assert_eq!(events, vec![SimEvent::MinerWorking { pos: p }; 3], "ticks 0, 54 and 108");

    // The view plays them only near the camera.
    g.clear_sounds();
    g.teleport(p.x as f64 + 0.5, p.y as f64 + 3.0, p.z as f64 + 0.5);
    g.sim.events = events.clone();
    g.handle_sim_events();
    assert_eq!(g.sounds.kinds(), vec![sound::DIG; 3]);
    g.clear_sounds();
    g.teleport(p.x as f64 + 40.0, p.y as f64, p.z as f64);
    g.sim.events = events;
    g.handle_sim_events();
    assert!(g.sounds.kinds().is_empty());
}

#[test]
fn miner_without_output_fills_up_and_stops() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 2);
    let m = p + IVec3::new(0, 1, 0);
    g.sim.world.set_block(m, MINER);
    let k = g.sim.factory.deposits.lookup(&mut g.sim.world, p);
    g.sim.factory.add_miner(m, block::FACE_BOTTOM as u8, k, 0);
    power_up(&mut g, m);
    let units = g.sim.factory.deposits.get(&key).unwrap().remaining_units();
    g.skip_time(300.0);
    let miner = g.sim.factory.miner_at(m);
    assert_eq!(miner.status, MinerStatus::OutputFull);
    assert_eq!(miner.out.total(), crate::item::MAX_STACK);
    let used = units - g.sim.factory.deposits.get(&key).unwrap().remaining_units();
    assert!(used < 110.0, "a full miner stops drawing, used {used}");

    // Right-click empties it into the inventory.
    g.act(Action::TakeContents { pos: m });
    g.run_ticks(1);
    assert_eq!(g.item_total(key.ore.into()), crate::item::MAX_STACK);
    assert!(g.sounds.kinds().contains(&sound::PICKUP));
}

/// Feeds frames of `next_dt()` seconds until exactly `seconds` of frame time have passed.
fn feed(g: &mut Game, seconds: f64, next_dt: &mut impl FnMut() -> f64) {
    let mut fed = 0.0;
    while seconds - fed > 1e-12 {
        let dt = next_dt().min(seconds - fed);
        g.update(dt);
        g.clear_sounds();
        fed += dt;
    }
}

/// Walks and jumps next to a running mine, then digs straight down. Returns the game and a text
/// snapshot of the core state, which must not depend on how the time was split into frames.
fn play_scenario(mut next_dt: impl FnMut() -> f64) -> (Game, String) {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = find_outcrop_block(&mut g, 6);
    let (miner, chest) = build_mine(&mut g, p);
    let first_tick = g.sim.tick;
    g.set_look(0.7, 0.0);
    g.set_move(1.0, 0.3, true, true, false);
    feed(&mut g, 1.5, &mut next_dt);
    g.set_move(0.0, 0.0, false, false, false);
    g.set_look(0.7, -1.5);
    g.set_mining(true);
    feed(&mut g, 2.5, &mut next_dt);
    assert_eq!(g.sim.tick - first_tick, 4 * TICK_RATE as u64, "4 s of frames run 240 ticks");

    let pl = g.body();
    let mut s = format!("pos {:?} vel {:?} ground {}\n", pl.pos, pl.vel, pl.on_ground);
    s += &format!("mining {:?} {} cooldown {}\n", g.mine_block, g.mine_progress, g.mine_cooldown);
    let slots: Vec<_> =
        (0..INVENTORY_SLOTS as u32).map(|i| (g.slot_item(i), g.slot_count(i))).filter(|s| s.1 > 0).collect();
    s += &format!("slots {slots:?}\n");
    for e in &g.items.list {
        s += &format!("item {} x{} at {:?} vel {:?} age {}\n", e.item.0, e.count, e.pos, e.vel, e.age);
    }
    let m = g.sim.factory.miner_at(miner);
    let units = g.sim.factory.deposits.get(&key).unwrap().remaining_units();
    s += &format!(
        "miner {:?} {} box {} deposit {units}\n",
        m.status,
        m.out.total(),
        g.sim.factory.storage_count_at(chest, key.ore.into())
    );
    s += &format!("core {:x}\n", g.sim.state_hash());
    (g, s)
}

#[test]
fn results_do_not_depend_on_frame_rate() {
    let fixed = |dt: f64| move || dt;
    let (g, reference) = play_scenario(fixed(1.0 / 60.0));
    let moved = g.body().pos - g.spawn;
    assert!(moved.x.abs() + moved.z.abs() > 2.0, "the player should have walked:\n{reference}");
    assert!(reference.contains("slots [("), "mined blocks should reach the inventory:\n{reference}");
    assert!(!reference.contains("box 0 "), "the miner should have filled the box:\n{reference}");

    let mut rng = Rng::new(99);
    let irregular = move || rng.range(0.001, 0.1);
    for (name, snapshot) in [
        ("30 fps", play_scenario(fixed(1.0 / 30.0)).1),
        ("144 fps", play_scenario(fixed(1.0 / 144.0)).1),
        ("irregular frames", play_scenario(irregular).1),
    ] {
        assert_eq!(snapshot, reference, "{name} differs from 60 fps");
    }
}

#[test]
fn a_second_player_has_its_own_body_pickups_and_throws() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let b = PlayerId(g.add_player().unwrap() as u8);
    assert_eq!(b, PlayerId(1));

    // A stone ledge high above the ground east of spawn; B stands on it next to a stone block.
    let f = g.spawn.floor() + IVec3::new(8, 12, 0);
    for dx in -1..=2 {
        for dz in -1..=1 {
            for dy in 0..4 {
                g.sim.world.set_block(f + IVec3::new(dx, dy, dz), if dy == 0 { STONE } else { AIR });
            }
        }
    }
    let wall = f + IVec3::new(1, 1, 0);
    g.sim.world.set_block(wall, STONE);
    g.bodies[1].as_mut().unwrap().pos = f.as_vec3() + Vec3::new(0.5, 1.0, 0.5);
    g.run_ticks(30);
    assert!(g.bodies[1].as_ref().unwrap().on_ground, "B stands on the ledge");

    // B breaks the stone: B picks it up; the local player, far below, gets nothing and no toast.
    g.act_as(b, Action::BreakBlock { pos: wall });
    g.run_ticks(60);
    assert_eq!(g.sim.player(b).unwrap().inventory.count(STONE.into()), 1);
    assert_eq!(g.item_total(STONE.into()), 0);
    assert!(!g.next_pickup());

    // B's throw leaves from B's body.
    g.act_as(b, Action::DropSelected { count: 1 });
    g.run_ticks(1);
    let thrown = g.items.list.last().unwrap();
    assert!((thrown.pos - g.bodies[1].as_ref().unwrap().eye()).length() < 1.0);

    // Removing B takes its body now and its inventory at the next tick. The local player stays.
    g.remove_player(1);
    g.remove_player(0);
    assert!(g.bodies[1].is_none() && g.bodies[0].is_some());
    g.run_ticks(1);
    assert!(g.sim.player(b).is_none() && g.sim.player(g.local).is_some());
    assert_eq!(g.add_player(), Some(1), "the id is free again");
}

#[test]
fn crafting_is_queued_takes_time_and_consumes_inputs() {
    use crate::item::{IRON_PLATE, IRON_ROD};
    let mut g = Game::new(7, 2);
    g.give(IRON_PLATE.0, 3);
    g.give(IRON_ROD.0, 5);
    g.run_ticks(1);
    let belt = RECIPES.iter().position(|r| r.output == BELT.into()).unwrap() as u32;
    assert_eq!(g.craft(belt, 5), 3, "what the inventory can pay for");
    assert_eq!(g.item_total(BELT.into()), 0, "applied at the next tick");
    g.run_ticks(1);
    assert_eq!((g.item_total(IRON_PLATE.0), g.item_total(IRON_ROD.0)), (0, 2), "paid for when queued");
    assert_eq!(g.craft_queue()[..2], [BELT as u32, 12]);
    // Each craft takes 2.5 s by hand; the belts arrive one craft at a time.
    g.run_ticks(150);
    assert_eq!(g.item_total(BELT.into()), 4);
    g.run_ticks(300);
    assert_eq!(g.item_total(BELT.into()), 12);
    assert!(g.craft_queue().is_empty());
    let mut notices = Vec::new();
    while g.next_pickup() {
        if g.pickup_item() == BELT as u16 {
            notices.push(g.pickup_count());
        }
    }
    assert_eq!(notices, [12], "the toasts merge into one");
    assert!(!g.can_craft(belt));
}

#[test]
fn a_machine_crafted_from_ingots_queues_its_parts_and_can_be_cancelled() {
    use crate::item::{COPPER_INGOT, IRON_INGOT};
    let mut g = Game::new(7, 2);
    g.give(IRON_INGOT.0, 14);
    g.give(COPPER_INGOT.0, 3);
    g.run_ticks(1);
    let miner = RECIPES.iter().position(|r| r.output == MINER.into()).unwrap() as u32;
    assert_eq!((g.craftable_times(miner), g.recipe_part_crafts(miner)), (1, 12));
    assert_eq!(g.craft(miner, 1), 1);
    g.run_ticks(400);
    assert_eq!(g.craft_queue()[..2], [MINER as u32, 1]);
    g.cancel_craft(0);
    g.run_ticks(1);
    assert!(g.craft_queue().is_empty());
    assert_eq!(g.item_total(MINER.into()), 0);
    // Everything is back, as parts or ingots: the same again can be made.
    assert_eq!(g.craftable_times(miner), 1);
}

#[test]
fn every_hand_recipe_gives_its_output() {
    for (i, r) in RECIPES.iter().enumerate() {
        let mut g = Game::new(7, 2);
        for &(item, n) in r.inputs {
            g.give(item.0, n);
        }
        g.run_ticks(1);
        if g.recipe_locked_by(i as u32) >= 0 {
            assert!(!g.can_craft(i as u32) && g.craft(i as u32, 1) == 0, "recipe {i} waits for research");
            for (t, tech) in crate::research::TECHS.iter().enumerate() {
                (0..tech.units).for_each(|_| g.sim.factory.research.add_unit(t as u8));
            }
        }
        assert_eq!(g.craft(i as u32, 1), 1, "recipe {i}");
        g.run_ticks(r.hand_ticks() + 2);
        assert_eq!(g.item_total(r.output.0), r.count, "recipe {i} made its output");
        assert!(r.inputs.iter().all(|&(item, _)| g.item_total(item.0) == 0), "recipe {i} used its inputs");
    }
}
#[test]
fn a_miner_line_through_a_smelter_fills_a_box_with_ingots() {
    use crate::block::{COAL_ORE, SMELTER};
    use crate::factory::{process_spec, ProcessStatus};
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    // An iron or copper outcrop (coal would be burned, not smelted).
    let (p, key) = nearby_ore(&g)
        .into_iter()
        .find_map(|p| {
            let key = g.sim.factory.deposits.lookup(&mut g.sim.world, p)?;
            (key.tier == Tier::Outcrop && key.ore != COAL_ORE).then_some((p, key))
        })
        .expect("a metal outcrop near spawn");
    // Miner ? belt ? smelter ? belt ? box, and a box of coal ? belt ? smelter from the north.
    let m = p + IVec3::new(0, 1, 0);
    let at = |dx, dz| m + IVec3::new(dx, 0, dz);
    let (smelter, chest, coal) = (at(2, 0), at(4, 0), at(2, -2));
    let blocks = [(m, MINER), (at(1, 0), BELT), (smelter, SMELTER), (at(3, 0), BELT), (chest, STORAGE)];
    for (pos, b) in blocks.into_iter().chain([(coal, STORAGE), (at(2, -1), BELT)]) {
        g.sim.world.set_block(pos, b);
    }
    let f = &mut g.sim.factory;
    f.add_miner(m, block::FACE_BOTTOM as u8, Some(key), 0);
    f.add_belt(at(1, 0), 1);
    f.place(&mut g.sim.world, SMELTER, smelter, 0, m, 0);
    f.add_belt(at(3, 0), 1);
    f.add_storage(chest);
    f.add_storage(coal);
    f.stock(coal, COAL_ORE.into(), 40);
    f.add_belt(at(2, -1), 2);
    power_up(&mut g, m);

    // The miner (0.6 ore/s) is slower than the smelter (1 ingot per 1.5 s), so it sets the rate.
    g.skip_time(120.0);
    let all = [true; crate::recipes::MACHINE_RECIPES.len()];
    let ingot = process_spec(SMELTER)
        .and_then(|s| s.recipe_using(key.ore.into(), &all))
        .map(|i| crate::recipes::MACHINE_RECIPES[i as usize].main().0);
    let ingots = g.sim.factory.storage_count_at(chest, ingot.unwrap());
    let mined = (120.0 * MINER_RECOVERY) as u32;
    assert!(ingots + 6 >= mined && ingots <= mined, "{ingots} ingots of ~{mined} ore");
    // Coal burns only while smelting: 1.5 s of fire per ingot, 8 s per coal.
    let burned =
        40 - g.sim.factory.storage_count_at(coal, COAL_ORE.into()) - g.sim.factory.smelter_at(smelter).fuel.total();
    let needed = (ingots as f64 * 1.5 / 8.0).ceil() as u32;
    assert!((needed..=needed + 1).contains(&burned), "{burned} coal for {ingots} ingots (and one in the works)");
    assert_ne!(g.sim.factory.smelter_at(smelter).status, ProcessStatus::NoFuel);
}

#[test]
fn right_click_opens_a_panel_whose_buttons_drive_the_machine() {
    use crate::block::CONSTRUCTOR;
    use crate::item::{IRON_INGOT, IRON_PLATE};
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let feet = g.body().pos.floor();
    let c = feet + IVec3::new(0, 0, -2);
    g.sim.world.set_block(c, CONSTRUCTOR);
    g.sim.factory.place(&mut g.sim.world, CONSTRUCTOR, c, 0, c, 0);
    g.act(Action::Give { item: IRON_INGOT, count: 7 });
    g.run_ticks(1);

    // Right-click asks the host to open the panel instead of acting.
    g.target = Some(RayHit { block: c, normal: IVec3::new(0, 0, 1), id: CONSTRUCTOR });
    g.using = true;
    g.update_placing(1.0 / 60.0);
    assert_eq!(g.take_panel_request(), vec![c.x, c.y, c.z]);
    assert!(g.take_panel_request().is_empty() && !g.using);
    let panel = g.machine_panel(c.x, c.y, c.z);
    assert_eq!((panel[1], panel[4], panel[5]), (u32::MAX, 1, 2), "no recipe yet, input and output slots");

    // Choose plates, put the ingots in, wait, take the plates.
    let plates =
        g.machine_recipes(CONSTRUCTOR).into_iter().find(|&i| g.machine_recipe_output(i)[0] == IRON_PLATE.0 as u32);
    g.set_machine_recipe(c.x, c.y, c.z, plates.unwrap() as i32);
    g.run_ticks(1);
    assert!(g.machine_wants(c.x, c.y, c.z, IRON_INGOT.0));
    g.insert_into_machine(c.x, c.y, c.z, IRON_INGOT.0);
    g.skip_time(1.0);
    assert!(g.machine_status(c.x, c.y, c.z).starts_with("No power"));

    // Power it: a pole beside it and a generator fuelled through its panel.
    let (pole, gen) = (c + IVec3::new(2, 0, 0), c + IVec3::new(3, 0, 0));
    for (pos, block) in [(pole, block::POLE), (gen, block::GENERATOR)] {
        g.sim.world.set_block(pos, block);
        g.sim.factory.place(&mut g.sim.world, block, pos, 0, pos, 0);
    }
    g.act(Action::Give { item: block::COAL_ORE.into(), count: 2 });
    g.run_ticks(1);
    assert!(g.machine_wants(gen.x, gen.y, gen.z, block::COAL_ORE as u16));
    g.insert_into_machine(gen.x, gen.y, gen.z, block::COAL_ORE as u16);
    g.skip_time(7.0);
    assert_eq!(g.machine_status(gen.x, gen.y, gen.z), "Idle: nothing on its grid needs power");
    assert_eq!(g.machine_panel(gen.x, gen.y, gen.z)[5], 1, "one fuel slot");
    assert!(g.machine_status(c.x, c.y, c.z).starts_with("Waiting for 2 Iron Ingot"));
    g.take_machine_output(c.x, c.y, c.z);
    g.run_ticks(1);
    assert_eq!((g.item_total(IRON_PLATE.0), g.item_total(IRON_INGOT.0)), (3, 0));
    // Clearing the recipe hands back the odd ingot.
    g.set_machine_recipe(c.x, c.y, c.z, -1);
    g.run_ticks(1);
    assert_eq!(g.item_total(IRON_INGOT.0), 1);
}

#[test]
fn a_mk2_gets_more_ore_from_the_same_deposit() {
    // The same outcrop in two identical, powered worlds: a Mk1 in one, a Mk2 in the other.
    let mine = |mk2: bool| {
        let mut g = Game::new(2024, 3);
        run_until_ready(&mut g);
        let (p, key) = find_outcrop_block(&mut g, 6);
        let (m, chest) = build_mine(&mut g, p);
        if mk2 {
            assert!(g.sim.factory.upgrade(m), "upgraded in place, on the same deposit");
        }
        let before = g.sim.factory.deposits.get(&key).unwrap().remaining_units();
        g.skip_time(200.0);
        let drawn = before - g.sim.factory.deposits.get(&key).unwrap().remaining_units();
        (drawn.round() as u32, g.sim.factory.storage_count_at(chest, key.ore.into()))
    };
    let ((drawn1, ore1), (drawn2, ore2)) = (mine(false), mine(true));
    // An outcrop's draw cap (1 unit/s) holds both to the same draw, so the Mk2's extra is all recovery.
    assert_eq!(drawn1, drawn2);
    assert!(ore1.abs_diff((drawn1 as f64 * MINER_RECOVERY) as u32) <= 3, "Mk1 kept {ore1} of {drawn1}");
    assert!(ore2.abs_diff((drawn2 as f64 * MINER_TIERS[1].recovery) as u32) <= 3, "Mk2 kept {ore2}");
    assert!(ore2 > ore1 * 5 / 4 - 3, "{ore2} vs {ore1}");
}

#[test]
fn a_coal_miner_fuels_the_generator_that_powers_it() {
    use crate::block::{COAL_ORE, GENERATOR, LOG, POLE};
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, key) = nearby_ore(&g)
        .into_iter()
        .find_map(|p| {
            let key = g.sim.factory.deposits.lookup(&mut g.sim.world, p)?;
            (key.tier == Tier::Outcrop && key.ore == COAL_ORE).then_some((p, key))
        })
        .expect("the starter coal outcrop");
    // A miner on the coal pushes straight into the generator beside it, which powers it.
    let m = p + IVec3::new(0, 1, 0);
    let (gen, pole) = (m + IVec3::new(1, 0, 0), m + IVec3::new(0, 2, 0));
    g.sim.factory.add_miner(m, block::FACE_BOTTOM as u8, Some(key), 0);
    for (pos, b) in [(gen, GENERATOR), (pole, POLE)] {
        g.sim.factory.place(&mut g.sim.world, b, pos, 0, pos, 0);
    }
    g.skip_time(1.0);
    assert_eq!(g.sim.factory.miner_at(m).status, MinerStatus::NoPower, "a cold generator");
    // One log (135 kJ: 27 s of a 5 kW miner) starts it; then the miner's own coal keeps it going.
    assert_eq!(g.sim.factory.insert(gen, LOG.into(), 1), 1);
    g.skip_time(90.0);
    let (miner, panel) = (g.sim.factory.miner_at(m), g.sim.factory.panel(gen).unwrap());
    assert_eq!((miner.status, panel.status.as_str()), (MinerStatus::Running, "Supplying 5 of 60 kW"));
    // 90 s at 0.6 coal/s is 54 coal; 63 s on coal at 5 kW is 315 kJ, so two coal were lit.
    let held = panel.slots[0].1.count + miner.out.total();
    assert!((51..=54).contains(&held), "{held} coal kept");
}

/// A stone slab at height 200 above spawn, 16 long in x, with a one-block step up from x = 6.
pub(crate) fn belt_test_slab(g: &mut Game) {
    for x in -2..16 {
        for z in -2..3 {
            g.sim.world.set_block(IVec3::new(x, 200, z), STONE);
            if x >= 6 {
                g.sim.world.set_block(IVec3::new(x, 201, z), STONE);
            }
        }
    }
    g.toggle_fly();
    g.teleport(0.5, 202.0, 0.5);
}

#[test]
fn dragging_with_belts_lays_a_line_that_climbs_a_step() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    g.give(BELT.into(), 20);
    g.run_ticks(2);
    let slot = (0..INVENTORY_SLOTS).find(|&s| g.slot_item(s as u32) == BELT as u16).unwrap();
    g.select_slot(slot as u32);
    g.run_ticks(2);

    // Press on the slab just ahead (east), drag the pointer far along it, release.
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
    assert!(planned >= 8, "the line reaches past the step: {:?}", g.line.cells);
    assert!(g.line_label().starts_with(&format!("Belt line\n{planned} belts, 1 on slopes")), "{}", g.line_label());
    assert_eq!(g.line_cells().len(), planned * 4);
    g.set_using(false);
    for _ in 0..30 {
        g.update(1.0 / 60.0);
    }
    assert!(g.line.cells.is_empty() && g.line_label().is_empty());
    assert_eq!(g.slot_count(slot as u32), 20 - planned as u32, "one belt per cell");
    assert_eq!(g.sim.factory.count(factory::Kind::Belt), planned);
    // The belt before the step became an up ramp; the rest run east on top.
    assert_eq!(g.sim.factory.belt_at(IVec3::new(5, 201, 0)).shape, factory::Shape::Up);
    assert_eq!(g.sim.factory.belt_at(IVec3::new(6, 202, 0)).dir, 1);

    // R turns the belt you point at.
    g.teleport(3.5, 203.5, 0.5);
    g.set_look(east, -1.5);
    g.update(1.0 / 60.0);
    assert!(g.rotate_target());
    g.run_ticks(1);
    assert_eq!(g.sim.factory.belt_at(IVec3::new(3, 201, 0)).dir, 2);
}

#[test]
fn a_left_click_cancels_a_belt_line() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    belt_test_slab(&mut g);
    g.give(BELT.into(), 20);
    g.run_ticks(2);
    let slot = (0..INVENTORY_SLOTS).find(|&s| g.slot_item(s as u32) == BELT as u16).unwrap();
    g.select_slot(slot as u32);
    g.run_ticks(2);
    g.set_look(std::f64::consts::FRAC_PI_2, -1.3);
    g.update(1.0 / 60.0);
    g.set_using(true);
    g.update(1.0 / 60.0);
    assert!(!g.line.cells.is_empty());
    g.set_mining(true);
    g.update(1.0 / 60.0);
    g.set_mining(false);
    g.set_using(false);
    for _ in 0..10 {
        g.update(1.0 / 60.0);
    }
    assert_eq!(g.sim.factory.count(factory::Kind::Belt), 0, "nothing was built");
    assert_eq!(g.slot_count(slot as u32), 20);
}

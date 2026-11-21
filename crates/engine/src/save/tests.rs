use super::*;
use crate::action::Action;
use crate::block::{BELT, COAL_ORE, COPPER_ORE, IRON_ORE, LOG, STONE, STORAGE};
use crate::item::IRON_INGOT;
use crate::math::{IVec3, Vec3};
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};

#[test]
fn a_saved_world_loads_back_to_the_same_bytes() {
    // A running mine, a second player, a loose item and a flying local player (flight is creative-only).
    let mut g = Game::new_creative(2024, 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 6);
    build_mine(&mut g, p);
    let b = g.join(0).unwrap();
    g.act_as(b, Action::Give { item: STONE.into(), count: 7 });
    // A keyed player who left: their things wait in the save.
    let c = g.join(5).unwrap();
    g.act_as(c, Action::Give { item: LOG.into(), count: 4 });
    g.run_ticks(1);
    g.leave(c);
    g.give(BELT, 3);
    g.toggle_fly();
    let far = g.body().pos + Vec3::new(20.0, 5.0, 0.0);
    g.items.spawn(far, Vec3::ZERO, STONE.into(), 2, 0.0);
    g.run_ticks(300);

    let saved = g.save();
    let loaded = Game::from_save(&saved, 3).expect("loads");
    assert!(loaded.save() == saved, "the same bytes after a round trip");
    assert_eq!(loaded.worldgen_version(), WORLDGEN_VERSION, "new worlds get the newest generator");
    assert_eq!(loaded.sim.state_hash(), g.sim.state_hash());
    assert_eq!(loaded.body().pos, g.body().pos);
    assert!(loaded.body().flying);
    assert_eq!(loaded.items.list.len(), g.items.list.len());
    assert_eq!(loaded.sim.player(b).unwrap().inventory.count(STONE.into()), 7);
    assert_eq!(loaded.item_total(BELT), 3);
    let away = &loaded.sim.away[0];
    assert_eq!((loaded.sim.away.len(), away.key, away.inventory.count(LOG.into())), (1, 5, 4));

    // Opening the world for play sends the other player home: nobody is connected to drive them.
    let mut opened = Game::load(&saved, 3).expect("loads");
    assert_eq!(opened.bodies.iter().flatten().count(), 1);
    opened.run_ticks(1);
    assert!(opened.sim.player(b).is_none());
}

/// A small world with a bit of everything and no chunks loaded, so the file is a few hundred bytes.
fn small_save() -> Vec<u8> {
    let mut g = Game::new(7, 2);
    let (chest, belt) = (IVec3::new(5, 200, 5), IVec3::new(6, 200, 5));
    g.give(STORAGE, 1);
    g.give(BELT, 2);
    g.act(Action::PlaceBlock { pos: chest, slot: 0, facing: 0, against: chest });
    g.act(Action::PlaceBlock { pos: belt, slot: 1, facing: 1, against: belt });
    g.join(0).unwrap();
    g.items.spawn(Vec3::new(40.5, 150.0, 40.5), Vec3::ZERO, STONE.into(), 3, 0.0);
    g.run_ticks(2);
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Storage), 1);
    g.save()
}

#[test]
fn foreign_old_and_damaged_files_are_refused() {
    let saved = small_save();
    assert!(Game::load(&saved, 2).is_ok());
    let refused = |bytes: &[u8]| Game::load(bytes, 2).err().expect("refused");
    assert!(refused(b"PK\x03\x04 a zip file").contains("not an OpenCraft world"));
    let with = |at: usize, v: u32| {
        let mut b = saved.clone();
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
        b
    };
    assert!(refused(&with(4, SAVE_VERSION + 1)).contains("newer version"));
    assert!(refused(&with(4, OLDEST_VERSION - 1)).contains("older version"));
    assert!(refused(&with(8, WORLDGEN_VERSION + 1)).contains("newer version"));
    assert_eq!(refused(&with(8, 0)), DAMAGED);

    // Every 11th cut and flipped byte (plus the last cut): each load builds a whole `Game`.
    for len in (MAGIC.len()..saved.len()).step_by(11).chain([saved.len() - 1]) {
        assert_eq!(refused(&saved[..len]), DAMAGED, "cut at {len}");
    }
    let mut longer = saved.clone();
    longer.push(0);
    assert_eq!(refused(&longer), DAMAGED);

    // No damaged byte may crash the loader (some still load, as a slightly different world).
    for i in (0..saved.len()).step_by(11) {
        let mut b = saved.clone();
        b[i] ^= 0x5a;
        let _ = Game::load(&b, 2);
    }
}

/// Saved before items got their own ids (version 1: `u8` block ids): a box at (5, 200, 5) that held
/// 20 iron ore and 5 coal, feeding a belt east of it; 30 stone, 5 logs and a belt in the inventory;
/// a second player; 3 copper ore lying at (40.5, 150, 40.5).
const V1_SAVE: &[u8] = include_bytes!("v1.ocworld");

#[test]
fn a_version_1_save_still_loads() {
    assert_eq!(u32::from_le_bytes(V1_SAVE[4..8].try_into().unwrap()), 1);
    let mut g = Game::load(V1_SAVE, 2).expect("loads");
    assert_eq!(g.item_total(STONE), 30);
    assert_eq!(g.item_total(LOG), 5);
    assert_eq!(g.item_total(BELT), 1);
    assert!(g.sim.player(PlayerId(1)).is_some());
    let loose = &g.items.list[0];
    assert_eq!((loose.item, loose.count), (COPPER_ORE.into(), 3));

    // Every ore item is still in the box or on the belt (the box feeds its last stack, coal, first).
    let (chest, belt) = (IVec3::new(5, 200, 5), IVec3::new(6, 200, 5));
    let f = &mut g.sim.factory;
    let in_box = [f.storage_count_at(chest, IRON_ORE.into()), f.storage_count_at(chest, COAL_ORE.into())];
    let on_belt = f.remove(belt);
    assert!(!on_belt.is_empty() && on_belt.iter().all(|s| s.item == COAL_ORE.into()), "{on_belt:?}");
    assert_eq!(in_box[0], 20);
    assert_eq!(in_box[1] + on_belt.iter().map(|s| s.count).sum::<u32>(), 5);

    // It saves in the new format, which holds items that aren't blocks.
    g.give(IRON_INGOT.0, 4);
    g.run_ticks(1);
    let saved = g.save();
    assert_eq!(u32::from_le_bytes(saved[4..8].try_into().unwrap()), SAVE_VERSION);
    assert_eq!(u32::from_le_bytes(saved[8..12].try_into().unwrap()), 1, "an old world keeps its terrain");
    let back = Game::load(&saved, 2).expect("loads");
    assert_eq!(back.worldgen_version(), 1);
    assert_eq!(back.item_total(IRON_INGOT.0), 4);
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
}

/// Saved at version 9, before player keys (`small_save` of step 3.1): a box at (5, 200, 5), a belt east
/// of it, a second player and 3 stone lying at (40.5, 150, 40.5).
const V9_SAVE: &[u8] = include_bytes!("v9.ocworld");

#[test]
fn a_version_9_save_still_loads() {
    assert_eq!(u32::from_le_bytes(V9_SAVE[4..8].try_into().unwrap()), 9);
    let mut g = Game::load(V9_SAVE, 2).expect("loads");
    assert_eq!(g.worldgen_version(), 1);
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Storage), 1);
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Belt), 1);
    assert_eq!(g.sim.player(PlayerId(1)).map(|p| p.key), Some(0), "no keys before version 10");
    assert!(g.sim.away.is_empty());
    assert!(!g.is_creative(), "no mode before version 38");
    let chest = IVec3::new(5, 200, 5);
    assert_eq!(g.sim.factory.box_label(chest), Some(crate::factory::box_label::AUTO), "no labels before 69");
    assert_eq!((g.items.list[0].item, g.items.list[0].count), (STONE.into(), 3));

    g.give(IRON_INGOT.0, 1);
    g.run_ticks(1);
    let back = Game::load(&g.save(), 2).expect("loads");
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
}

/// Saved at version 76, when block ids were bytes: a box at (5, 200, 5), a belt east of it, a second player,
/// 3 stone lying at (40.5, 150, 40.5) and an iron ingot in the inventory. Version 77 writes ids below 255 as the
/// same single byte, so the file is also what version 77 would write for this world; version 78 adds each player's
/// seventh worn slot (two bytes) and the X-ray Goggles tech's progress (four bytes).
const V76_SAVE: &[u8] = include_bytes!("v76.ocworld");

#[test]
fn a_version_76_save_still_loads_and_saves_with_only_the_head_slot_and_the_new_tech_added() {
    assert_eq!(u32::from_le_bytes(V76_SAVE[4..8].try_into().unwrap()), 76);
    let g = Game::load(V76_SAVE, 2).expect("loads");
    assert_eq!(g.item_total(IRON_INGOT.0), 1);
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Storage), 1);
    assert_eq!((g.items.list[0].item, g.items.list[0].count), (STONE.into(), 3));
    let players = g.sim.players.iter().flatten().count();
    assert!(g.sim.players.iter().flatten().all(|p| !p.inventory.has_xray_goggles()), "the Head slot is empty");
    let saved = g.save();
    assert_eq!(u32::from_le_bytes(saved[4..8].try_into().unwrap()), SAVE_VERSION);
    // A Head slot each, eight more techs (X-ray Goggles, LED Lamps, Pigments, Gas Wells, Gas Processing, Flamethrower,
    // Seismic Prospecting, Ore Scanning), the switched-off count (4 bytes), the LED colours' count, active fires (12
    // bytes, save 82), and the three extra generator layer versions in the save 81 header (12 bytes).
    let leds = crate::bytes::ByteWriter::default().bytes.len() + {
        let mut w = crate::bytes::ByteWriter::default();
        g.sim.leds.write_state(&mut w);
        w.bytes.len()
    };
    assert_eq!(saved.len(), V76_SAVE.len() + 2 * players + 32 + 4 + leds + 12 + 12);
    let back = Game::load(&saved, 2).expect("the new format loads");
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
}

#[test]
fn header_layers_round_trip_and_validate() {
    let layers = crate::worldgen::Layers { ground: 7, water: 7, cover: 7, deposits: WORLDGEN_VERSION };
    let mut g = Game::with_generator(WorldGen::with_layers(1337, layers), 2);
    g.run_ticks(2);
    let saved = g.save();
    let back = Game::load(&saved, 2).expect("mixed layers load");
    assert_eq!(back.sim.world.generator().layers(), layers);
    assert_eq!(back.worldgen_layers(), vec![7, 7, 7, WORLDGEN_VERSION]);
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());

    // Offsets 8 (ground), 16 (water), 20 (cover), 24 (deposits) must each be in 1..=WORLDGEN_VERSION.
    let with = |at: usize, v: u32| {
        let mut b = saved.clone();
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
        b
    };
    for at in [8, 16, 20, 24] {
        assert!(
            Game::load(&with(at, WORLDGEN_VERSION + 1), 2).err().expect("refused").contains("newer version"),
            "at {at}"
        );
        assert_eq!(Game::load(&with(at, 0), 2).err().expect("refused"), DAMAGED, "at {at}");
    }
}

#[test]
fn upgrading_deposits_keeps_factory_state_and_refreshes_unedited_chunks() {
    let mut g = Game::with_generator(WorldGen::with_version(1337, 7), 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 6);
    build_mine(&mut g, p);
    g.run_ticks(120);
    assert!(g.can_upgrade_deposits(), "v7 world can upgrade deposits");

    let factory_hash = |g: &Game| {
        let mut w = crate::bytes::ByteWriter::default();
        g.sim.factory.write_state(&mut w);
        crate::bytes::fnv1a(&w.bytes)
    };
    let before_hash = g.sim.state_hash();
    assert_eq!(g.worldgen_layers(), vec![7, 7, 7, 7]);

    // Bare Sim (no loaded chunks) applying the same action stays deterministic with the loaded Game,
    // and a control world that does not upgrade keeps the exact same factory state tick-for-tick.
    let saved_before = g.save();
    let mut bare = Game::load(&saved_before, 2).expect("loads without streaming");
    let mut control = Game::load(&saved_before, 2).expect("loads control");

    g.upgrade_deposits();
    g.run_ticks(1);
    bare.upgrade_deposits();
    bare.run_ticks(1);
    control.run_ticks(1);

    assert_eq!(g.worldgen_layers(), vec![7, 7, 7, WORLDGEN_VERSION]);
    assert!(!g.can_upgrade_deposits(), "already upgraded");
    assert_eq!(factory_hash(&g), factory_hash(&control), "factory state is untouched by the deposits upgrade");
    assert_ne!(g.sim.state_hash(), before_hash, "state hash reflects the upgraded deposits layer");
    assert_eq!(g.sim.state_hash(), bare.sim.state_hash(), "upgrade is deterministic regardless of loaded chunks");

    // Mine keeps working identically to the control world and the upgraded save round-trips cleanly.
    g.run_ticks(180);
    control.run_ticks(180);
    assert_eq!(factory_hash(&g), factory_hash(&control), "factory runs identically after the upgrade");
    let saved_after = g.save();
    let reloaded = Game::load(&saved_after, 3).expect("reloads upgraded save");
    assert_eq!(reloaded.worldgen_layers(), vec![7, 7, 7, WORLDGEN_VERSION]);
    assert_eq!(reloaded.sim.state_hash(), g.sim.state_hash());
}

#[test]
fn pre_v6_upgrade_safety_gates_incompatible_worlds() {
    // v1 has no biomes ("classic terrain"), so it never upgrades deposits.
    let v1 = Game::with_generator(WorldGen::with_version(1337, 1), 2);
    assert!(!v1.can_upgrade_deposits());

    // v8..=12 already have the newest deposits rules, so no upgrade is offered.
    for v in crate::worldgen::DEPOSITS_LATEST..=WORLDGEN_VERSION {
        let g = Game::with_generator(WorldGen::with_version(1337, v), 2);
        assert!(!g.can_upgrade_deposits(), "v{v} already up to date");
    }

    // A v5 world with a mine near spawn (< 200 blocks, where oil/uranium/bauxite have 0 weight) upgrades safely.
    let mut v5 = Game::with_generator(WorldGen::with_version(1337, 5), 3);
    run_until_ready(&mut v5);
    let (p, _) = find_outcrop_block(&mut v5, 4);
    build_mine(&mut v5, p);
    v5.run_ticks(10);
    assert!(v5.can_upgrade_deposits(), "v5 near-spawn deposit keeps the same key in v12");
    v5.upgrade_deposits();
    v5.run_ticks(1);
    assert_eq!(v5.deposits_version(), WORLDGEN_VERSION);
    assert!(Game::load(&v5.save(), 2).is_ok(), "upgraded v5 save reloads cleanly");

    // A v3 world with a tracked v3 deposit refuses to upgrade so no tracked deposit key is stranded.
    let mut v3 = Game::with_generator(WorldGen::with_version(1337, 3), 3);
    run_until_ready(&mut v3);
    let (p3, _) = find_outcrop_block(&mut v3, 4);
    build_mine(&mut v3, p3);
    v3.run_ticks(10);
    assert!(!v3.can_upgrade_deposits(), "v3 tracked deposit key differs from v4+ so upgrade is refused");
}

#[test]
#[ignore]
fn bench_save_upgrades_deposits_without_moving_factory_state() {
    let path = std::env::var("OC_SAVE").unwrap_or_else(|_| "../../artifacts/bench/factory.ocworld".into());
    let raw = std::fs::read(&path).expect("unpacked save file (set OC_SAVE)");
    assert!(!raw.starts_with(&[0x1f, 0x8b]), "unpack gzipped save first: {path}");
    let mut g = Game::load(&raw, 2).expect("bench save loads");
    let mut control = Game::load(&raw, 2).expect("control loads");
    assert_eq!(g.worldgen_version(), 7);
    assert!(g.can_upgrade_deposits());
    let factory_hash = |g: &Game| {
        let mut w = crate::bytes::ByteWriter::default();
        g.sim.factory.write_state(&mut w);
        crate::bytes::fnv1a(&w.bytes)
    };
    g.upgrade_deposits();
    g.run_ticks(60);
    control.run_ticks(60);
    assert_eq!(g.worldgen_layers(), vec![7, 7, 7, WORLDGEN_VERSION]);
    assert_eq!(factory_hash(&g), factory_hash(&control), "bench factory state hash is unchanged after 60 ticks");
    let count_gas = |g: &Game| {
        let mut seeded = Vec::new();
        for cz in -20..=20 {
            for cx in -20..=20 {
                g.sim.world.generator().seed_deposits(cx, cz, &mut seeded);
            }
        }
        seeded.iter().filter(|d| d.ore() == crate::block::GAS_SHALE).count()
    };
    assert_eq!(count_gas(&control), 0, "v7 bench save has no gas fields before upgrade");
    assert!(count_gas(&g) > 0, "upgraded v13 bench save gains natural gas fields");
    let saved = g.save();
    let mut back = Game::load(&saved, 2).expect("upgraded bench save reloads");
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
    back.run_ticks(60);
}

use super::*;
use crate::action::Action;
use crate::block::{BELT, COAL_ORE, COPPER_ORE, IRON_ORE, LOG, STONE, STORAGE};
use crate::item::IRON_INGOT;
use crate::math::{IVec3, Vec3};
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};

#[test]
fn a_saved_world_loads_back_to_the_same_bytes() {
    // A running mine, a second player, a loose item and a flying local player.
    let mut g = Game::new(2024, 3);
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
    g.give(BELT.into(), 3);
    g.toggle_fly();
    let far = g.body().pos + Vec3::new(20.0, 5.0, 0.0);
    g.items.spawn(far, Vec3::ZERO, STONE.into(), 2, 0.0);
    g.run_ticks(300);

    let saved = g.save();
    let loaded = Game::load(&saved, 3).expect("loads");
    assert!(loaded.save() == saved, "the same bytes after a round trip");
    assert_eq!(loaded.sim.state_hash(), g.sim.state_hash());
    assert_eq!(loaded.body().pos, g.body().pos);
    assert!(loaded.body().flying);
    assert_eq!(loaded.items.list.len(), g.items.list.len());
    assert_eq!(loaded.sim.player(b).unwrap().inventory.count(STONE.into()), 7);
    assert_eq!(loaded.item_total(BELT.into()), 3);
    let away = &loaded.sim.away[0];
    assert_eq!((loaded.sim.away.len(), away.key, away.inventory.count(LOG.into())), (1, 5, 4));
}

/// A small world with a bit of everything and no chunks loaded, so the file is a few hundred bytes.
fn small_save() -> Vec<u8> {
    let mut g = Game::new(7, 2);
    let (chest, belt) = (IVec3::new(5, 200, 5), IVec3::new(6, 200, 5));
    g.give(STORAGE.into(), 1);
    g.give(BELT.into(), 2);
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
    assert!(refused(&with(8, WORLDGEN_VERSION + 1)).contains("World generation has changed"));

    for len in MAGIC.len()..saved.len() {
        assert_eq!(refused(&saved[..len]), DAMAGED, "cut at {len}");
    }
    let mut longer = saved.clone();
    longer.push(0);
    assert_eq!(refused(&longer), DAMAGED);

    // No damaged byte may crash the loader (some still load, as a slightly different world).
    for i in 0..saved.len() {
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
    assert_eq!(g.item_total(STONE.into()), 30);
    assert_eq!(g.item_total(LOG.into()), 5);
    assert_eq!(g.item_total(BELT.into()), 1);
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
    let back = Game::load(&saved, 2).expect("loads");
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
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Storage), 1);
    assert_eq!(g.sim.factory.count(crate::factory::Kind::Belt), 1);
    assert_eq!(g.sim.player(PlayerId(1)).map(|p| p.key), Some(0), "no keys before version 10");
    assert!(g.sim.away.is_empty());
    assert_eq!((g.items.list[0].item, g.items.list[0].count), (STONE.into(), 3));

    g.give(IRON_INGOT.0, 1);
    g.run_ticks(1);
    let back = Game::load(&g.save(), 2).expect("loads");
    assert_eq!(back.sim.state_hash(), g.sim.state_hash());
}

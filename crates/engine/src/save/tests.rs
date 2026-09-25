use super::*;
use crate::action::Action;
use crate::block::{BELT, STONE, STORAGE};
use crate::math::{IVec3, Vec3};
use crate::tests::{build_mine, find_outcrop_block, run_until_ready};

#[test]
fn a_saved_world_loads_back_to_the_same_bytes() {
    // A running mine, a second player, a loose item and a flying local player.
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let (p, _) = find_outcrop_block(&mut g, 6);
    build_mine(&mut g, p);
    let b = g.join().unwrap();
    g.act_as(b, Action::Give { item: STONE, count: 7 });
    g.give(BELT, 3);
    g.toggle_fly();
    let far = g.body().pos + Vec3::new(20.0, 5.0, 0.0);
    g.items.spawn(far, Vec3::ZERO, STONE, 2, 0.0);
    g.run_ticks(300);

    let saved = g.save();
    let loaded = Game::load(&saved, 3).expect("loads");
    assert!(loaded.save() == saved, "the same bytes after a round trip");
    assert_eq!(loaded.sim.state_hash(), g.sim.state_hash());
    assert_eq!(loaded.body().pos, g.body().pos);
    assert!(loaded.body().flying);
    assert_eq!(loaded.items.list.len(), g.items.list.len());
    assert_eq!(loaded.sim.player(b).unwrap().inventory.count(STONE), 7);
    assert_eq!(loaded.item_total(BELT), 3);
}

/// A small world with a bit of everything and no chunks loaded, so the file is a few hundred bytes.
fn small_save() -> Vec<u8> {
    let mut g = Game::new(7, 2);
    let (chest, belt) = (IVec3::new(5, 200, 5), IVec3::new(6, 200, 5));
    g.give(STORAGE, 1);
    g.give(BELT, 2);
    g.act(Action::PlaceBlock { pos: chest, slot: 0, facing: 0, against: chest });
    g.act(Action::PlaceBlock { pos: belt, slot: 1, facing: 1, against: belt });
    g.join().unwrap();
    g.items.spawn(Vec3::new(40.5, 150.0, 40.5), Vec3::ZERO, STONE, 3, 0.0);
    g.run_ticks(2);
    assert_eq!(g.sim.factory.storage_count(), 1);
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
    assert!(refused(&with(4, SAVE_VERSION - 1)).contains("older version"));
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

//! FREEZE through `Game`: only the top layer of still water freezes into real mineable `block::ICE` blocks below
//! freezing; bodies stand on the ice and swim in the liquid layer beneath it; ice can be mined and placed, and melts
//! with an exponential half-life both in the world and in inventory when above 0 °C.

use super::*;
use crate::action::Action;
use crate::block::{AIR, ICE, STONE, WATER};
use crate::daytime::{DAY_TICKS, START, YEAR_DAYS};
use crate::item::ICE_ITEM;
use crate::weather::ice::{can_freeze, can_melt, INVENTORY_MELT_HALF_LIFE, WORLD_MELT_HALF_LIFE};
use crate::weather::Reading;
use crate::worldgen::SEA_LEVEL;

/// A 3 × 3 pool of still water TWO blocks deep, walled and floored with stone at height `top_y`: returns the centre
/// cell of its top water layer.
fn build_deep_pool(g: &mut Game, top_y: i32) -> IVec3 {
    let p = g.body().pos.floor();
    let c = IVec3::new(p.x + 4, top_y, p.z + 4);
    for dx in -2..=2 {
        for dz in -2..=2 {
            let at = |dy| IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            g.sim.world.set_block(at(-2), STONE);
            let inner = dx.abs() <= 1 && dz.abs() <= 1;
            g.sim.world.set_block(at(-1), if inner { WATER } else { STONE });
            g.sim.world.set_block(at(0), if inner { WATER } else { STONE });
            g.sim.world.set_block(at(1), AIR);
        }
    }
    c
}

/// Day `day` of the second year at noon.
fn noon(day: u64) -> u64 {
    (YEAR_DAYS + day) * DAY_TICKS + DAY_TICKS / 2 - START
}

#[test]
fn only_the_top_layer_freezes_into_mineable_ice_and_bears_a_body() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    // High pool in mid-winter: well below freezing.
    let c = build_deep_pool(&mut g, SEA_LEVEL + 80);
    g.sim.tick = noon(33);
    g.sim.atmosphere = Default::default();
    g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);
    let temp = Reading::at(&g.sim.atmosphere, g.sim.world.generator(), c.x, c.y, c.z).temp;
    assert!(can_freeze(temp, c.y), "high winter pool should freeze (temp {temp})");

    // Freezing top cell turns it into ICE; attempting to freeze the layer beneath does nothing.
    g.act(Action::FreezeWater { pos: c });
    g.act(Action::FreezeWater { pos: c - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    assert_eq!(g.sim.world.get_block(c), Some(ICE), "top layer freezes into ICE");
    assert_eq!(g.sim.world.get_block(c - IVec3::new(0, 1, 0)), Some(WATER), "water underneath stays liquid");

    // A body stands natively on the solid ICE block.
    let local = g.local.0 as usize;
    let body = g.bodies[local].as_mut().unwrap();
    body.pos = Vec3::new(f64::from(c.x) + 0.5, f64::from(c.y) + 1.2, f64::from(c.z) + 0.5);
    body.vel = Vec3::ZERO;
    body.flying = false;
    for _ in 0..60 {
        g.step_bodies();
    }
    let feet = g.body().pos.y;
    assert!((feet - f64::from(c.y + 1)).abs() < 0.02, "stands on the ice block: feet at {feet}");
    assert!(!g.body().in_water);

    // Mining the ice block breaks it and drops ICE_ITEM, leaving liquid water directly underneath.
    let (_, drops) = g.sim.dismantle(g.local, c, true).expect("ice is mineable");
    assert_eq!(drops.len(), 1);
    assert_eq!((drops[0].item, drops[0].count), (ICE_ITEM, 1));
    assert_eq!(g.sim.world.get_block(c), Some(AIR));
    assert_eq!(g.sim.world.get_block(c - IVec3::new(0, 1, 0)), Some(WATER));
}

#[test]
fn placed_ice_melts_with_a_half_life_above_freezing_and_stays_solid_below_freezing() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let p = g.body().pos.floor();
    let warm_spot = IVec3::new(p.x + 2, SEA_LEVEL + 5, p.z + 2);
    g.sim.tick = noon(13); // mid-summer
    g.sim.atmosphere = Default::default();
    g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);
    assert!(can_melt(
        Reading::at(&g.sim.atmosphere, g.sim.world.generator(), warm_spot.x, warm_spot.y, warm_spot.z).temp
    ));

    // Place a row of 20 ice blocks in warm summer air.
    g.act(Action::Give { item: ICE_ITEM, count: 25 });
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == ICE_ITEM).unwrap() as u8;
    for i in 0..20 {
        let pos = warm_spot + IVec3::new(i, 0, 0);
        g.sim.world.set_block_anywhere(pos, AIR);
        g.act(Action::PlaceBlock { pos, slot, facing: 0, against: pos - IVec3::new(0, 1, 0) });
    }
    g.run_ticks(1);

    // Step through 3 half-lives: almost all warm ice blocks melt into water.
    let ticks = (WORLD_MELT_HALF_LIFE * 3.0 * f64::from(TICK_RATE)) as u32;
    for _ in 0..ticks {
        g.sim.step();
    }
    let melted =
        (0..20).filter(|&i| g.sim.world.block_anywhere_or_generate(warm_spot + IVec3::new(i, 0, 0)) != ICE).count();
    assert!(melted >= 10, "expected most warm ice to melt after 3 half-lives, only {melted}/20 melted");

    // In freezing winter air, placed ice stays solid ICE indefinitely.
    let cold_spot = IVec3::new(p.x + 2, SEA_LEVEL + 80, p.z + 2);
    g.sim.tick = noon(33); // mid-winter
    g.sim.atmosphere = Default::default();
    g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);
    assert!(!can_melt(
        Reading::at(&g.sim.atmosphere, g.sim.world.generator(), cold_spot.x, cold_spot.y, cold_spot.z).temp
    ));
    g.sim.world.set_block_anywhere(cold_spot, AIR);
    g.act(Action::PlaceBlock { pos: cold_spot, slot, facing: 0, against: cold_spot - IVec3::new(0, 1, 0) });
    g.run_ticks(1);
    assert_eq!(g.sim.world.block_anywhere_or_generate(cold_spot), ICE);
    for _ in 0..ticks {
        g.sim.step();
    }
    assert_eq!(g.sim.world.block_anywhere_or_generate(cold_spot), ICE, "cold ice stays frozen");
}

#[test]
fn inventory_ice_decays_above_freezing_and_is_preserved_below_freezing() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let p = g.body().pos.floor();
    let warm = IVec3::new(p.x, SEA_LEVEL + 5, p.z);
    let cold = IVec3::new(p.x, SEA_LEVEL + 260, p.z);
    g.sim.tick = noon(13); // mid-summer
    g.sim.atmosphere = Default::default();
    g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);
    assert!(can_melt(Reading::at(&g.sim.atmosphere, g.sim.world.generator(), warm.x, warm.y, warm.z).temp));
    assert!(!can_melt(Reading::at(&g.sim.atmosphere, g.sim.world.generator(), cold.x, cold.y, cold.z).temp));

    g.act(Action::Give { item: ICE_ITEM, count: 64 });
    g.run_ticks(1);
    assert_eq!(g.inventory().count(ICE_ITEM), 64);

    // Standing below freezing: zero decay over a full half-life.
    for _ in 0..INVENTORY_MELT_HALF_LIFE as u32 {
        g.sim.apply(g.local, Action::MeltInventoryIce { pos: cold });
    }
    assert_eq!(g.inventory().count(ICE_ITEM), 64, "ice in inventory does not melt below 0 °C");

    // Standing above freezing: roughly half melts after one half-life.
    for _ in 0..INVENTORY_MELT_HALF_LIFE as u32 {
        g.sim.apply(g.local, Action::MeltInventoryIce { pos: warm });
    }
    let left = g.inventory().count(ICE_ITEM);
    assert!((16..=48).contains(&left), "expected ~32 of 64 ice left after one half-life, got {left}");
}

/// Writes a ready-to-import savegame standing directly in front of a naturally generated high-elevation
/// `block::SPRING` at a river headwater (zero terrain edits):
///   cargo test -q -p opencraft-engine natural_river_spring_save -- --ignored --nocapture
#[test]
#[ignore]
fn natural_river_spring_save() {
    use crate::block::{SOLID, SPRING};
    use crate::math::Vec3;

    let seed = 2024;
    let mut g = Game::new_creative(seed, 6);
    g.sim.tick = noon(13); // bright mid-summer noon
    g.sim.atmosphere = Default::default();
    g.sim.atmosphere.advance(g.sim.world.generator(), g.sim.tick);

    // Search for a scenic, elevated, naturally generated river headwater SPRING with open water and clear air in front.
    let mut best: Option<(i32, IVec3, IVec3)> = None;
    let mut gen = crate::worldgen::WorldGen::new(seed);
    for cz in -18..=18 {
        for cx in -18..=18 {
            for cy in 2..7 {
                let c = gen.generate(IVec3::new(cx, cy, cz));
                if c.as_uniform().is_some() {
                    continue;
                }
                for i in 0..crate::chunk::CHUNK_VOLUME {
                    let (lx, ly, lz) = (i & 31, i >> 10, (i >> 5) & 31);
                    if c.get(lx, ly, lz) != SPRING {
                        continue;
                    }
                    let sp = IVec3::new(cx * 32 + lx as i32, cy * 32 + ly as i32, cz * 32 + lz as i32);
                    if sp.y < SEA_LEVEL + 10 {
                        continue;
                    }
                    // Find a horizontal direction where open water sits in front of the spring and air lies clear above.
                    for dir in [IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)] {
                        let front = sp + dir;
                        if g.sim.world.block_anywhere_or_generate(front) != WATER {
                            continue;
                        }
                        let clear = (1..=4).all(|k| {
                            let p = sp + dir * k;
                            (1..=4).all(|dy| {
                                !SOLID[g.sim.world.block_anywhere_or_generate(p + IVec3::new(0, dy, 0)) as usize]
                            })
                        });
                        if clear && best.is_none_or(|(hy, _, _)| sp.y > hy) {
                            best = Some((sp.y, sp, dir));
                        }
                    }
                }
            }
        }
    }

    let (_, sp, dir) = best.expect("found a naturally generated exposed river spring");
    let stand = Vec3::new((sp.x + dir.x * 3) as f64 + 0.5, (sp.y + 1) as f64 + 0.2, (sp.z + dir.z * 3) as f64 + 0.5);
    let eye = stand + Vec3::new(0.0, 1.62, 0.0);
    let target =
        Vec3::new(sp.x as f64 + 0.5 + dir.x as f64 * 0.5, sp.y as f64 + 0.5, sp.z as f64 + 0.5 + dir.z as f64 * 0.5);
    let diff = target - eye;
    let len = (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt();
    let yaw = diff.x.atan2(-diff.z);
    let pitch = (diff.y / len).asin();

    g.teleport(stand.x, stand.y, stand.z);
    g.set_look(yaw, pitch);
    g.toggle_fly(); // hovering cleanly right over the headwater pool facing the spring
    run_until_ready(&mut g);
    g.run_ticks(2);

    assert_eq!(g.sim.world.block_anywhere_or_generate(sp), SPRING, "natural SPRING block verified");
    assert_eq!(g.sim.world.block_anywhere_or_generate(sp + dir), WATER, "open water emerging from SPRING");

    let bytes = g.save();
    let dir_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts");
    std::fs::create_dir_all(&dir_path).unwrap();
    let raw_path = dir_path.join("natural-river-spring.raw.ocworld");
    std::fs::write(&raw_path, &bytes).unwrap();
    println!(
        "SPRING at ({}, {}, {}), player at ({:.1}, {:.1}, {:.1}), facing dir {:?}, {} bytes",
        sp.x,
        sp.y,
        sp.z,
        stand.x,
        stand.y,
        stand.z,
        dir,
        bytes.len()
    );

    let mut back = Game::load(&bytes, 6).expect("saved world loads back");
    assert_eq!(back.sim.world.block_anywhere_or_generate(sp), SPRING);
}

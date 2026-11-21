//! Core fire and flamethrower tests (FIRE1 + FIRE2): tree burning to charcoal, heat-bounded spread, peaceful
//! protection of placed blocks/machines/houses, water firebreaks, grass front reach, caps, charcoal-as-coal
//! equivalence, flamethrower rays & fuel, save round-trip, frame-rate invariance and co-op agreement.

use super::*;
use crate::action::Action;
use crate::block::{
    self, AIR, COAL_ORE, DIRT, GRASS, IRON_ORE, LEAVES, LOG, OAK_TREE_LOG, PLANKS, SAPLING, SMELTER, STONE, STORAGE,
    WATER,
};
use crate::item::{CHARCOAL, DIESEL_BARREL, EMPTY_BARREL, FLAMETHROWER, NAPHTHA_BARREL, QUICKLIME, STEEL_INGOT, STICK};
use crate::math::{IVec3, Vec3};
use crate::recipes::{burn_time, fuel_energy, MACHINE_RECIPES, RECIPES};
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::Game;

const P: PlayerId = PlayerId(0);

/// Builds a flat grass clearing centred at `c` (radius `r`, grass at `c.y`, air above).
fn flat_grass(sim: &mut Sim, c: IVec3, r: i32) {
    for z in c.z - r..=c.z + r {
        for x in c.x - r..=c.x + r {
            sim.world.set_block_anywhere_later(IVec3::new(x, c.y - 1, z), DIRT);
            sim.world.set_block_anywhere_later(IVec3::new(x, c.y, z), GRASS);
            for y in c.y + 1..=c.y + 12 {
                sim.world.set_block_anywhere_later(IVec3::new(x, y, z), AIR);
            }
        }
    }
}

/// Plants a synthetic grown tree (`OAK_TREE_LOG` trunk `h` tall + 3×3×2 `LEAVES` crown) with foot at `foot`.
fn plant_grown_tree(sim: &mut Sim, foot: IVec3, h: i32) {
    for y in 0..h {
        sim.world.set_block_anywhere_later(foot + IVec3::new(0, y, 0), OAK_TREE_LOG);
    }
    for dy in h - 1..=h + 1 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let p = foot + IVec3::new(dx, dy, dz);
                if sim.block(p) == AIR {
                    sim.world.set_block_anywhere_later(p, LEAVES);
                }
            }
        }
    }
}

#[test]
fn grown_tree_burns_in_two_stages_to_charcoal() {
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(400, 120, 400);
    flat_grass(&mut sim, base, 8);
    let foot = base + IVec3::new(0, 1, 0);
    plant_grown_tree(&mut sim, foot, 6);

    sim.apply(P, Action::Ignite { pos: foot, heat: 0 });
    assert_eq!(sim.fire.blazes().len(), 1);
    assert_eq!(sim.fire.blazes()[0].stage, Stage::Crown);
    assert!(sim.fire.blazes()[0].radius >= 1, "tree crown radius measured");
    assert!(sim.fire.blazes()[0].crown_y >= 1, "tree crown base measured");

    // Step through Crown -> Trunk transition: leaves vanish while trunk stays standing.
    let mut saw_trunk_stage = false;
    let mut charcoal = 0u32;
    for _ in 0..(CROWN_TICKS + TRUNK_TICKS + 120) {
        sim.step();
        if sim.fire.blazes().first().is_some_and(|b| b.stage == Stage::Trunk) && !saw_trunk_stage {
            saw_trunk_stage = true;
            assert_eq!(sim.block(foot), OAK_TREE_LOG, "trunk remains standing during smouldering stage");
            assert_eq!(sim.block(foot + IVec3::new(1, 6, 0)), AIR, "crown leaves burned off first");
        }
        for e in sim.events.drain(..) {
            if let SimEvent::Dropped { item, count, .. } = e {
                if item == CHARCOAL {
                    charcoal += count;
                }
            }
        }
        if sim.fire.is_empty() {
            break;
        }
    }
    assert!(saw_trunk_stage, "passed through Stage::Trunk");
    assert!(sim.fire.is_empty(), "blaze retired once burned out");
    assert_eq!(sim.block(foot), AIR, "trunk cleared at the end");
    assert_eq!(charcoal, charcoal_for(6), "charcoal dropped at foot");
}

#[test]
fn spread_obeys_heat_budget_and_water_stops_it() {
    // 1. Heat 0 never spreads to a close neighbour.
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(420, 120, 420);
    flat_grass(&mut sim, base, 12);
    let a = base + IVec3::new(0, 1, 0);
    let b = base + IVec3::new(4, 1, 0);
    plant_grown_tree(&mut sim, a, 5);
    plant_grown_tree(&mut sim, b, 5);
    sim.apply(P, Action::Ignite { pos: a, heat: 0 });
    for _ in 0..(CROWN_TICKS + TRUNK_TICKS + 120) {
        sim.step();
        sim.events.clear();
    }
    assert_eq!(sim.block(a), AIR, "tree A burned");
    assert_eq!(sim.block(b), OAK_TREE_LOG, "heat 0 did not spread to tree B");

    // 2. Heat 2 spreads to nearby trees (in a dense cluster), bounded by heat -> 0.
    let mut sim2 = Sim::new(13, 2);
    flat_grass(&mut sim2, base, 14);
    plant_grown_tree(&mut sim2, a, 6);
    let ring = [(3, 0), (-3, 0), (0, 3), (0, -3), (3, 3), (-3, -3), (3, -3), (-3, 3)];
    for (dx, dz) in ring {
        plant_grown_tree(&mut sim2, base + IVec3::new(dx, 1, dz), 6);
    }
    sim2.apply(P, Action::Ignite { pos: a, heat: 2 });
    for _ in 0..(CROWN_TICKS * 2 + TRUNK_TICKS * 2 + 200) {
        sim2.step();
        sim2.events.clear();
    }
    let burned = ring.iter().filter(|&&(dx, dz)| sim2.block(base + IVec3::new(dx, 1, dz)) == AIR).count();
    assert!(burned > 0, "heat 2 spread to at least one neighbouring tree");

    // 3. Water next to a tree foot prevents ignition and extinguishes an existing blaze.
    let mut sim3 = Sim::new(7, 2);
    flat_grass(&mut sim3, base, 10);
    plant_grown_tree(&mut sim3, a, 5);
    sim3.world.set_block_anywhere_later(a + IVec3::new(1, 0, 0), WATER);
    sim3.apply(P, Action::Ignite { pos: a, heat: MAX_HEAT });
    assert!(sim3.fire.is_empty(), "water beside trunk foot blocks ignition");
}

#[test]
fn placed_logs_houses_and_machines_never_burn() {
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(450, 120, 450);
    flat_grass(&mut sim, base, 8);
    let log_pos = base + IVec3::new(0, 1, 0);
    let plank_pos = base + IVec3::new(1, 1, 0);
    let stone_pos = base + IVec3::new(2, 1, 0);
    let box_pos = base + IVec3::new(3, 1, 0);
    let sapling_pos = base + IVec3::new(-2, 1, 0);
    sim.world.set_block_anywhere_later(log_pos, LOG);
    sim.world.set_block_anywhere_later(plank_pos, PLANKS);
    sim.world.set_block_anywhere_later(stone_pos, STONE);
    sim.world.set_block_anywhere_later(sapling_pos, SAPLING);
    sim.apply(P, Action::Give { item: STORAGE.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos: box_pos, slot: 0, facing: 0, against: base + IVec3::new(3, 0, 0) });

    for p in [log_pos, plank_pos, stone_pos, box_pos] {
        sim.apply(P, Action::Ignite { pos: p, heat: MAX_HEAT });
    }
    assert!(sim.fire.is_empty(), "none of placed log, planks, stone or storage ignited");

    // A sapling DOES burn.
    sim.apply(P, Action::Ignite { pos: sapling_pos, heat: 1 });
    assert_eq!(sim.block(sapling_pos), AIR, "sapling burned away");

    // Placed blocks and machines remain untouched even beside a torched ground patch.
    sim.apply(P, Action::Ignite { pos: base + IVec3::new(-1, 0, 0), heat: MAX_HEAT });
    for _ in 0..120 {
        sim.step();
        sim.events.clear();
    }
    assert_eq!(sim.block(log_pos), LOG);
    assert_eq!(sim.block(plank_pos), PLANKS);
    assert_eq!(sim.block(stone_pos), STONE);
    assert_eq!(sim.block(box_pos), STORAGE);
}

#[test]
fn weed_torch_clears_ground_without_igniting_neighbours() {
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(500, 120, 500);
    flat_grass(&mut sim, base, 8);
    sim.apply(P, Action::Ignite { pos: base, heat: MAX_HEAT });
    assert!(sim.world.is_cover_cleared(base.x, base.z), "torched ground column cleared");
    assert_eq!(sim.fire.fronts().len(), 1, "localized ember created");

    for _ in 0..(SCORCH_TICKS + 40) {
        sim.step();
        sim.events.clear();
    }
    assert!(sim.fire.is_empty(), "localized ground ember goes out on its own");
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (2, 0)] {
        assert!(
            !sim.world.is_cover_cleared(base.x + dx, base.z + dz),
            "neighbour ({dx},{dz}) ground plants were never ignited"
        );
    }
}

#[test]
fn flamethrower_uses_liquid_fuel_not_coal_and_acts_as_weed_torch() {
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(600, 120, 600);
    flat_grass(&mut sim, base, 12);
    let tree = base + IVec3::new(5, 1, 0);
    plant_grown_tree(&mut sim, tree, 6);
    let placed = base + IVec3::new(-4, 1, 0);
    sim.world.set_block_anywhere_later(placed, LOG);

    // Give flamethrower in slot 0, NO liquid fuel yet.
    sim.apply(P, Action::Give { item: FLAMETHROWER, count: 1 });
    sim.apply(P, Action::SelectSlot { slot: 0 });
    sim.events.clear();

    // Empty tank does nothing.
    let origin = Vec3::new(base.x as f64 + 0.5, base.y as f64 + 2.0, base.z as f64 + 0.5);
    sim.apply(P, Action::Flame { origin, dir: Vec3::new(1.0, 0.0, 0.0) });
    assert!(sim.fire.is_empty(), "empty tank ignited nothing");
    assert!(sim.events.is_empty(), "empty tank emitted no Flamed event");

    // Coal and charcoal MUST NOT fuel the flamethrower.
    sim.apply(P, Action::Give { item: COAL_ORE.into(), count: 10 });
    sim.apply(P, Action::Give { item: CHARCOAL, count: 10 });
    sim.events.clear();
    sim.apply(P, Action::Flame { origin, dir: Vec3::new(1.0, 0.0, 0.0) });
    assert!(sim.fire.is_empty(), "coal and charcoal must not fuel the flamethrower");
    assert_eq!(sim.player(P).unwrap().inventory.count(COAL_ORE.into()), 10, "coal untouched");
    assert_eq!(sim.player(P).unwrap().inventory.count(CHARCOAL), 10, "charcoal untouched");

    // Naphtha barrel fuels it, returns an empty barrel, and ignites the tree!
    sim.apply(P, Action::Give { item: NAPHTHA_BARREL, count: 1 });
    sim.apply(P, Action::Flame { origin, dir: Vec3::new(1.0, 0.0, 0.0) });
    assert_eq!(sim.fire.blazes().len(), 1, "flamethrower ignited grown tree");
    assert_eq!(sim.player(P).unwrap().inventory.count(NAPHTHA_BARREL), 0, "consumed 1 naphtha barrel");
    assert_eq!(sim.player(P).unwrap().inventory.count(EMPTY_BARREL), 1, "returned 1 empty barrel");
    assert_eq!(sim.fire.tank(P), FLAME_TICKS_PER_BARREL - 1);

    // Diesel barrel also fuels the flamethrower once the canister runs dry.
    sim.fire.set_tank(P, 0);
    sim.apply(P, Action::Give { item: DIESEL_BARREL, count: 1 });
    sim.apply(P, Action::Flame { origin, dir: Vec3::new(0.0, -1.0, 0.0) });
    assert_eq!(sim.player(P).unwrap().inventory.count(DIESEL_BARREL), 0, "consumed 1 diesel barrel");
    assert_eq!(sim.player(P).unwrap().inventory.count(EMPTY_BARREL), 2, "returned second empty barrel");
    assert!(sim.world.is_cover_cleared(base.x, base.z), "ground under weed torch cleared");

    // Step past scorch lifetime: ground fire never spreads beyond what the flame cone actually touched.
    for _ in 0..(SCORCH_TICKS + 40) {
        sim.step();
        sim.events.clear();
    }
    assert!(!sim.world.is_cover_cleared(base.x, base.z + 4), "weed torch did not spread across meadow");

    // Aim at placed oak log: never lights it.
    let blazes_before = sim.fire.blazes().len();
    sim.apply(P, Action::Flame { origin, dir: Vec3::new(-1.0, 0.0, 0.0) });
    assert_eq!(sim.fire.blazes().len(), blazes_before);
    assert_eq!(sim.block(placed), LOG, "placed log untouched");

    // Let tree finish burning and verify charcoal dropped.
    let mut dropped = 0;
    for _ in 0..(CROWN_TICKS + TRUNK_TICKS + 120) {
        sim.step();
        for e in sim.events.drain(..) {
            if let SimEvent::Dropped { item: CHARCOAL, count, .. } = e {
                dropped += count;
            }
        }
    }
    assert_eq!(dropped, charcoal_for(6));
}

#[test]
fn charcoal_serves_as_coal_everywhere() {
    // 1. Fuel tables match coal.
    assert_eq!(burn_time(CHARCOAL), burn_time(COAL_ORE.into()));
    assert_eq!(fuel_energy(CHARCOAL), fuel_energy(COAL_ORE.into()));

    // 2. Hand crafting a torch from charcoal + sticks works.
    let mut sim = Sim::new(7, 2);
    let torch_recipe = RECIPES.iter().position(|r| r.output == block::TORCH.into()).unwrap() as u16;
    sim.apply(P, Action::Give { item: CHARCOAL, count: 4 });
    sim.apply(P, Action::Give { item: STICK, count: 4 });
    sim.apply(P, Action::Craft { recipe: torch_recipe, times: 1 });
    for _ in 0..200 {
        sim.step();
    }
    assert!(sim.player(P).unwrap().inventory.count(block::TORCH.into()) >= 1, "crafted torch using charcoal");

    // 3. Machine recipe (Blast Furnace steel) accepts charcoal in place of coal and produces steel.
    let steel_recipe = MACHINE_RECIPES.iter().position(|r| r.main().0 == STEEL_INGOT).unwrap() as u16;
    let mut world = crate::world::World::new(7, 2);
    let mut f = crate::factory::Factory::default();
    f.research.complete_all();
    let pos = IVec3::new(0, 200, 0);
    f.place(&mut world, block::BLAST_FURNACE, pos, 0, pos, 0);
    f.set_recipe(pos, Some(steel_recipe));
    assert!(f.insert(pos, IRON_ORE.into(), 4) > 0);
    assert!(f.insert(pos, CHARCOAL, 4) > 0, "blast furnace accepts charcoal");
    assert!(f.insert(pos, QUICKLIME, 4) > 0);
    // Smelter also burns charcoal as fuel.
    let sm = IVec3::new(10, 200, 0);
    f.place(&mut world, SMELTER, sm, 0, sm, 0);
    assert!(f.insert(sm, CHARCOAL, 2) > 0, "smelter accepts charcoal as fuel");

    // 4. Jetpack runs on charcoal.
    let mut jsim = Sim::new(7, 2);
    jsim.apply(P, Action::Give { item: crate::item::JETPACK, count: 1 });
    jsim.apply(P, Action::ClickSlot { slot: 0, shift: true }); // wear jetpack
    jsim.apply(P, Action::Give { item: CHARCOAL, count: 1 });
    jsim.apply(P, Action::Jetpack { on: true });
    assert!(jsim.player(P).unwrap().helpers.thrusting, "jetpack runs on charcoal");

    // 5. Item guide knows both where charcoal comes from and what uses it.
    let gen = crate::worldgen::WorldGen::new(7);
    assert!(!crate::item_guide::ways(&gen, CHARCOAL).is_empty(), "item guide has source for charcoal");
    assert!(!crate::item_guide::uses(CHARCOAL).is_empty(), "item guide lists coal recipes as uses of charcoal");
}

#[test]
fn fire_is_deterministic_across_frame_rates_saves_and_coop() {
    let base = IVec3::new(650, 120, 650);
    let foot = base + IVec3::new(0, 1, 0);
    let setup = |g: &mut Game| {
        flat_grass(&mut g.sim, base, 10);
        plant_grown_tree(&mut g.sim, foot, 6);
        plant_grown_tree(&mut g.sim, foot + IVec3::new(4, 0, 0), 6);
        g.act(Action::Ignite { pos: foot, heat: 2 });
        g.act(Action::Ignite { pos: base + IVec3::new(-2, 0, 0), heat: 1 });
    };

    // Frame-rate invariance: 60 Hz vs 30 Hz stepping produce identical hashes.
    let (mut a, mut b) = (Game::new(7, 2), Game::new(7, 2));
    setup(&mut a);
    setup(&mut b);
    for _ in 0..240 {
        a.update(1.0 / 60.0);
    }
    for _ in 0..120 {
        b.update(1.0 / 30.0);
    }
    assert_eq!(a.sim.state_hash(), b.sim.state_hash(), "same hash at 60 Hz and 30 Hz");

    // Save/load round-trip mid-burn produces identical bytes and state hash.
    assert!(!a.sim.fire.is_empty(), "fire still active mid-burn");
    let saved = a.save();
    let loaded = Game::load(&saved, 2).expect("loads with active fire");
    assert_eq!(loaded.sim.state_hash(), a.sim.state_hash());
    assert_eq!(loaded.save(), saved, "exact byte round-trip");

    // Co-op host + client agree across ignition and burning.
    let mut host = Game::new(7, 2);
    host.start_host();
    assert_eq!(host.host_join(0), Ok(1));
    flat_grass(&mut host.sim, base, 10);
    plant_grown_tree(&mut host.sim, foot, 6);
    let snap = host.snapshot();
    let mut client = Game::from_snapshot(&snap, 1, 2).expect("client joins");
    host.act(Action::Ignite { pos: foot, heat: 2 });
    for _ in 0..300 {
        host.run_ticks(1);
        let frames = host.take_frames();
        if !frames.is_empty() {
            assert!(client.push_frames(&frames));
        }
        client.run_ticks(1);
    }
    assert_eq!(host.sim.state_hash(), client.sim.state_hash(), "co-op host and client agree on fire");
}

#[test]
fn flamethrower_cone_grows_with_distance_to_four_block_diameter() {
    let mut sim = Sim::new(7, 2);
    let base = IVec3::new(700, 120, 700);
    flat_grass(&mut sim, base, 14);
    sim.apply(P, Action::Give { item: FLAMETHROWER, count: 1 });
    sim.apply(P, Action::Give { item: NAPHTHA_BARREL, count: 2 });
    sim.apply(P, Action::SelectSlot { slot: 0 });

    // Close shot straight down (~1.5 blocks away): tight footprint near nozzle, leaves 2 blocks away untouched.
    let close_origin = Vec3::new(base.x as f64 + 0.5, base.y as f64 + 2.0, base.z as f64 + 0.5);
    sim.apply(P, Action::Flame { origin: close_origin, dir: Vec3::new(0.0, -1.0, 0.0) });
    assert!(sim.world.is_cover_cleared(base.x, base.z));
    assert!(!sim.world.is_cover_cleared(base.x, base.z + 2), "close nozzle shot stays tight");

    // Far shot (~7 blocks out along +x onto the grass top): expands to a ~4-block diameter circle (radius ~2.15).
    let far_origin = Vec3::new(base.x as f64 - 4.5, base.y as f64 + 2.6, base.z as f64 + 8.5);
    let far_target = Vec3::new(base.x as f64 + 2.5, base.y as f64 + 1.0, base.z as f64 + 8.5);
    let dir = far_target - far_origin;
    sim.apply(P, Action::Flame { origin: far_origin, dir });
    for dz in -1..=1 {
        for dx in -1..=1 {
            assert!(
                sim.world.is_cover_cleared(base.x + 2 + dx, base.z + 8 + dz),
                "weed at offset ({dx},{dz}) inside the far cone must be cleared"
            );
        }
    }
    assert!(
        sim.world.is_cover_cleared(base.x + 2, base.z + 10),
        "far cone reaches ~2 blocks radius (4-block diameter)"
    );
    assert!(sim.world.is_cover_cleared(base.x + 2, base.z + 6), "far cone reaches ~2 blocks radius on both sides");
    assert!(!sim.world.is_cover_cleared(base.x + 2, base.z + 12), "weeds well outside the cone stay untouched");
}

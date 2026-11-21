//! Natural gas end-to-end tests: gas well over gas shale, wellhead separator over oil sand (associated gas),
//! gas plant sweetening & fractionation, gas turbine clean 600 kW power (methane & LPG, ahead of diesel),
//! steam reformer hydrogen production, ethane/LPG cracking to ethylene and polyethylene in the chemical plant,
//! save/reload round-trips and face-fighting checks.

use super::*;
use crate::block::{
    CHEMICAL_PLANT, COAL_ORE, CRACKER, DIESEL_GENERATOR, GAS_SHALE, GENERATOR, OIL_SAND, PIPE, POLE, PUMP, WATER,
};
use crate::deposits::Tier;
use crate::factory::power_mix::Source;
use crate::factory::process::pump::associated_gas_ratio;
use crate::factory::Factory;
use crate::item::{
    CRUDE_BARREL, DIESEL_BARREL, ETHANE_BARREL, ETHYLENE_BARREL, HYDROGEN_BARREL, PLASTIC, RAW_GAS_BARREL, SULFUR,
};
use crate::math::IVec3;
use crate::recipes::POLYETHYLENE_RECIPE;
use crate::world::World;
use crate::Game;

fn run(f: &mut Factory, world: &mut World, from: u64, seconds: u32) -> u64 {
    let mut events = Vec::new();
    let to = from + u64::from(seconds) * u64::from(TICK_RATE);
    for tick in from..to {
        f.update(world, tick, &mut events);
    }
    to
}

fn above_deposit(world: &World, ore: crate::block::BlockId) -> IVec3 {
    let g = world.generator();
    let mut seeded = Vec::new();
    for cz in -32..32 {
        for cx in -32..32 {
            seeded.clear();
            g.seed_deposits(cx, cz, &mut seeded);
            if let Some(d) = seeded.iter().find(|d| d.ore() == ore && d.tier() == Tier::Vein) {
                return IVec3::new(d.center.x, g.height_at(d.center.x, d.center.z) + 1, d.center.z);
            }
        }
    }
    panic!("no deposit of block {ore} within reach");
}

fn powered_factory(world: &mut World, at: IVec3, gens: i32) -> Factory {
    let mut f = Factory::default();
    f.research.finish("Gas Wells");
    f.research.finish("Gas Processing");
    let pole = at + IVec3::new(4, 0, 0);
    f.place(world, POLE, pole, 0, pole, 0);
    for k in 1..=gens {
        let g = pole + IVec3::new(k, 0, 0);
        f.place(world, GENERATOR, g, 0, g, 0);
        f.insert(g, COAL_ORE.into(), 64);
    }
    f
}

/// Pipes a 2×2 water pool through a powered pump into the water inlet of the processor at `anchor`.
fn pipe_water(f: &mut Factory, world: &mut World, anchor: IVec3) {
    let p = f.processors.iter().find(|p| p.pos == anchor).unwrap();
    let (cell, face) = p.footprint().faces(anchor, p.dir, Role::Water)[0];
    let dir = crate::factory::DIRS[face as usize];
    let pipe = cell + dir;
    let pump = pipe + dir;
    let water = pump + dir;
    for dx in 0..=1 {
        for dz in 0..=1 {
            world.set_block_anywhere(water + dir * dx + IVec3::new(0, 0, dz), WATER);
        }
    }
    f.place(world, PIPE, pipe, 0, pipe, 0);
    f.place(world, PUMP, pump, 0, pump, 0);
    let pole = pump + IVec3::new(0, 0, 3);
    f.place(world, POLE, pole, 0, pole, 0);
    let gen = pole + IVec3::new(0, 0, 1);
    f.place(world, GENERATOR, gen, 0, gen, 0);
    f.insert(gen, COAL_ORE.into(), 64);
}

#[test]
fn gas_well_drills_gas_shale_and_fills_raw_gas_barrels() {
    let mut world = World::new(1337, 2);
    let spot = above_deposit(&world, GAS_SHALE);
    let mut f = powered_factory(&mut world, spot, 2);
    f.place(&mut world, GAS_WELL, spot, 0, spot, 0);
    assert_eq!(f.insert(spot, EMPTY_BARREL, 16), 16);
    run(&mut f, &mut world, 0, 30);
    let p = f.processors.iter().find(|p| p.pos == spot).unwrap();
    let full = p.out.count(RAW_GAS_BARREL);
    assert!((8..=12).contains(&full), "expected ~10 raw gas barrels in 30 s, got {full}");
    assert_eq!(p.input.count(EMPTY_BARREL), 16 - full);
    assert_eq!(p.status, Status::Working);
    assert!(p.status_text().contains("Drawing raw gas"));
}

#[test]
fn wellhead_separator_extracts_crude_and_associated_raw_gas_from_oil_sand() {
    let mut world = World::new(1337, 2);
    let spot = above_deposit(&world, OIL_SAND);
    let mut f = powered_factory(&mut world, spot, 2);
    f.place(&mut world, WELLHEAD_SEPARATOR, spot, 0, spot, 0);
    let well = f.processors.iter().find(|p| p.pos == spot).unwrap().pump_state().well.expect("oil below");
    let ratio = associated_gas_ratio(well.key);
    assert!((2..=4).contains(&ratio), "ratio out of range: {ratio}");
    assert_eq!(f.insert(spot, EMPTY_BARREL, 16), 16);
    run(&mut f, &mut world, 0, 30);
    let p = f.processors.iter().find(|p| p.pos == spot).unwrap();
    let crude = p.out.count(CRUDE_BARREL);
    let gas = p.side.count(RAW_GAS_BARREL);
    assert!(gas >= 2, "expected associated raw gas barrels, got {gas}");
    assert_eq!(crude, gas * ratio, "crude/gas must match deposit ratio 1:{ratio}");
    assert_eq!(p.input.count(EMPTY_BARREL), 16 - (crude + gas));
    assert!(p.reservoir_line(&f).contains(&format!("1:{ratio}")));
}

#[test]
fn gas_plant_sweetens_raw_gas_into_methane_ethane_lpg_and_sulfur() {
    let mut world = World::new(1, 2);
    let at = IVec3::new(0, 70, 0);
    let mut f = powered_factory(&mut world, at, 3);
    f.place(&mut world, GAS_PLANT, at, 0, at, 0);
    pipe_water(&mut f, &mut world, at);
    assert_eq!(f.insert(at, RAW_GAS_BARREL, 4), 4);
    run(&mut f, &mut world, 0, 8);
    let p = f.processors.iter().find(|p| p.pos == at).unwrap();
    assert_eq!(p.out.count(METHANE_BARREL), 2);
    assert_eq!(p.side.count(ETHANE_BARREL), 1);
    assert_eq!(p.side.count(LPG_BARREL), 1);
    assert_eq!(p.side.count(SULFUR), 1);
}

#[test]
fn gas_turbine_powers_grid_with_methane_and_lpg_ahead_of_diesel() {
    let mut world = World::new(1, 2);
    let at = IVec3::new(0, 70, 0);
    let mut f = Factory::default();
    f.research.finish("Ore Crushing");
    f.research.finish("Gas Wells");
    f.research.finish("Gas Processing");
    let pole = at + IVec3::new(3, 0, 0);
    f.place(&mut world, POLE, pole, 0, pole, 0);
    f.place(&mut world, GAS_TURBINE, at, 0, at, 0);
    let diesel_at = at + IVec3::new(0, 0, 4);
    f.place(&mut world, DIESEL_GENERATOR, diesel_at, 0, diesel_at, 0);
    // Load consumer on the same pole: a Crusher asking for power (no extra coal generator on the grid).
    let load_at = pole + IVec3::new(2, 0, 0);
    f.place(&mut world, crate::block::CRUSHER, load_at, 0, load_at, 0);
    assert_eq!(f.insert(load_at, crate::block::IRON_ORE.into(), 16), 16);

    assert_eq!(f.insert(at, METHANE_BARREL, 1), 1);
    assert_eq!(f.insert(at, LPG_BARREL, 1), 1);
    assert_eq!(f.insert(diesel_at, DIESEL_BARREL, 2), 2);
    // Rejects non-gas fuel.
    assert_eq!(f.insert(at, DIESEL_BARREL, 1), 0);

    run(&mut f, &mut world, 0, 2);
    let gt = f.processors.iter().find(|p| p.pos == at).unwrap();
    let dg = f.processors.iter().find(|p| p.pos == diesel_at).unwrap();
    assert_eq!(gt.status, Status::Working);
    assert_eq!(gt.input.count(METHANE_BARREL), 0, "methane lit first");
    assert_eq!(gt.input.count(LPG_BARREL), 1, "LPG kept in reserve");
    assert_eq!(gt.out.count(EMPTY_BARREL), 1, "empty barrel returned");
    assert_eq!(dg.input.count(DIESEL_BARREL), 2, "gas turbine runs ahead of diesel");
    let mix = f.power_mix();
    assert!(mix[Source::Gas as usize].given > 0, "Source::Gas reports power in analytics mix");
    assert_eq!(mix[Source::Gas as usize].could, GAS_TURBINE_KW);
}

#[test]
fn steam_reformer_and_ethylene_to_polyethylene_chain_works() {
    let mut world = World::new(1, 2);
    let at = IVec3::new(0, 70, 0);
    let mut f = powered_factory(&mut world, at, 6);

    // 1. Steam Reformer: 1 Methane + 2 Empty + 2 water -> 3 Hydrogen
    f.place(&mut world, STEAM_REFORMER, at, 0, at, 0);
    pipe_water(&mut f, &mut world, at);
    assert_eq!(f.insert(at, METHANE_BARREL, 1), 1);
    assert_eq!(f.insert(at, EMPTY_BARREL, 2), 2);
    run(&mut f, &mut world, 0, 6);
    let reformer = f.processors.iter().find(|p| p.pos == at).unwrap();
    assert_eq!(reformer.out.count(HYDROGEN_BARREL), 3);

    // 2. Cracker: 2 Ethane -> 1 Ethylene + 1 Hydrogen, then (once side cleared) 2 LPG -> 1 Ethylene + 1 Methane
    let crk = at + IVec3::new(0, 0, 5);
    let pole2 = crk + IVec3::new(3, 0, 0);
    f.place(&mut world, POLE, pole2, 0, pole2, 0);
    f.place(&mut world, CRACKER, crk, 0, crk, 0);
    pipe_water(&mut f, &mut world, crk);
    assert_eq!(f.insert(crk, ETHANE_BARREL, 2), 2);
    run(&mut f, &mut world, 1000, 6);
    let c = f.processors.iter_mut().find(|p| p.pos == crk).unwrap();
    assert_eq!((c.out.count(ETHYLENE_BARREL), c.side.count(HYDROGEN_BARREL)), (1, 1));
    c.side.remove(HYDROGEN_BARREL, 1);
    assert_eq!(f.insert(crk, LPG_BARREL, 2), 2);
    run(&mut f, &mut world, 2000, 6);
    let c = f.processors.iter().find(|p| p.pos == crk).unwrap();
    assert_eq!((c.out.count(ETHYLENE_BARREL), c.side.count(METHANE_BARREL)), (2, 1));

    // 3. Chemical Plant: 2 Ethylene -> 6 Plastic + 2 Empty Barrels
    let plant = at + IVec3::new(0, 0, 10);
    let pole3 = plant + IVec3::new(3, 0, 0);
    f.place(&mut world, POLE, pole3, 0, pole3, 0);
    f.place(&mut world, CHEMICAL_PLANT, plant, 0, plant, 0);
    assert!(f.set_recipe(plant, Some(POLYETHYLENE_RECIPE)).is_some());
    assert_eq!(f.insert(plant, ETHYLENE_BARREL, 2), 2);
    run(&mut f, &mut world, 3000, 6);
    let cp = f.processors.iter().find(|p| p.pos == plant).unwrap();
    assert_eq!((cp.out.count(PLASTIC), cp.side.count(EMPTY_BARREL)), (6, 2));
}

#[test]
fn gas_machines_survive_save_and_reload() {
    let mut g = Game::new_normal(1337, 2);
    g.sim.factory.research.finish("Gas Wells");
    g.sim.factory.research.finish("Gas Processing");
    let spot = IVec3::new(0, 72, 0);
    for (i, block) in [GAS_WELL, WELLHEAD_SEPARATOR, GAS_PLANT, GAS_TURBINE, STEAM_REFORMER].into_iter().enumerate() {
        let p = spot + IVec3::new(i as i32 * 6, 0, 0);
        g.sim.factory.place(&mut g.sim.world, block, p, (i % 4) as u8, p, 0);
    }
    let gt_pos = spot + IVec3::new(18, 0, 0);
    g.sim.factory.insert(gt_pos, LPG_BARREL, 2);
    let saved = g.save();
    let loaded = Game::load(&saved, 2).expect("loads gas save");
    assert_eq!(loaded.sim.state_hash(), g.sim.state_hash());
    for i in 0..5 {
        let p = spot + IVec3::new(i * 6, 0, 0);
        assert!(loaded.sim.factory.anchor_of(p) == Some(p), "machine {i} survived reload");
    }
}

//! Steam power in a whole core (`Sim`): a plant on hand-built stone far from spawn, placed by actions like a
//! player places it, feeding nine Mk3 constructors (405 kW) that each press 64 ingots into rods.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, AIR, BOILER, COAL_ORE, CONSTRUCTOR, PIPE, POLE, PUMP, SILO, STONE, TURBINE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::pipes::Fluid;
use crate::factory::process::parts::BOILER_PARTS;
use crate::factory::tests::recipe_for;
use crate::factory::Factory;
use crate::item::{COPPER_INGOT, IRON_INGOT, IRON_ROD};
use crate::sim::{PlayerId, Sim};
use crate::world::World;
use crate::worldgen::SEA_LEVEL;

const P: PlayerId = PlayerId(0);
const O: IVec3 = IVec3::new(4000, 0, 4000);
const L: i32 = SEA_LEVEL;
const COAL: u32 = 40;

fn at(x: i32, y: i32, z: i32) -> IVec3 {
    O + IVec3::new(x, y, z)
}

fn fill(sim: &mut Sim, lo: IVec3, hi: IVec3, b: BlockId) {
    for y in lo.y..=hi.y {
        for z in lo.z..=hi.z {
            for x in lo.x..=hi.x {
                sim.world.set_block_anywhere(IVec3::new(x, y, z), b);
            }
        }
    }
}

fn put(sim: &mut Sim, block: BlockId, pos: IVec3) {
    put_facing(sim, block, pos, 0);
}

fn put_facing(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere_or_generate(pos), block, "placed at {pos:?}");
}

const LOADS: [(i32, i32); 9] = [(12, 8), (14, 8), (16, 8), (12, 10), (16, 10), (12, 12), (14, 12), (16, 12), (14, 14)];

/// A pump in the sea (or a four-block pond) at (3, 60, 5), piped up and across to the boiler at (6, 63, 5)'s west
/// water inlet; steam piped from its two front outlets to the inlets of a turbine west of the pipe (3, 63, 7) and
/// another east of it (11, 63, 7); coal from a box by a belt into its back; three poles; and `LOADS`: Mk3
/// constructors, each with 64 ingots to press. The pump starts with its two blocks of water in hand: a real plant
/// is started by a generator on the same grid.
fn plant(pond: bool) -> Sim {
    let mut sim = Sim::new(1337, 2);
    fill(&mut sim, at(0, 50, 0), at(40, L, 30), STONE);
    fill(&mut sim, at(0, L + 1, 0), at(40, L + 12, 30), AIR);
    if pond {
        fill(&mut sim, at(3, 60, 5), at(3, L, 5), AIR);
        fill(&mut sim, at(2, 60, 5), at(2, 61, 5), WATER);
        fill(&mut sim, at(4, 60, 5), at(4, 61, 5), WATER);
    } else {
        fill(&mut sim, at(1, 55, 1), at(5, L, 9), WATER);
    }
    put(&mut sim, PUMP, at(3, 60, 5));
    for (x, y, z) in [(3, 61, 5), (3, 62, 5), (3, 63, 5), (3, 63, 4), (4, 63, 4), (5, 63, 4)] {
        put(&mut sim, PIPE, at(x, y, z));
    }
    put(&mut sim, BOILER, at(6, 63, 5));
    for (x, z) in [(6, 6), (6, 7), (7, 7), (8, 7)] {
        put(&mut sim, PIPE, at(x, 63, z));
    }
    put(&mut sim, TURBINE, at(3, 63, 7));
    put_facing(&mut sim, TURBINE, at(11, 63, 7), 2);
    for (x, z) in [(2, 6), (8, 9), (14, 10)] {
        put(&mut sim, POLE, at(x, 63, z));
    }
    let f = &mut sim.factory;
    f.add_storage(at(6, 63, 1));
    f.stock(at(6, 63, 1), COAL_ORE.into(), COAL);
    f.add_belt(at(6, 63, 2), 2);
    f.add_belt(at(6, 63, 3), 2);
    for (x, z) in LOADS {
        let pos = at(x, 63, z);
        f.place(&mut sim.world, CONSTRUCTOR, pos, 0, pos, 2);
        assert!(f.set_recipe(pos, Some(recipe_for(IRON_ROD))).is_some());
        assert_eq!(f.insert(pos, IRON_INGOT, 64), 64);
    }
    f.pipework[0].held = 2;
    sim
}

fn rods(sim: &Sim) -> Vec<u32> {
    LOADS.iter().map(|&(x, z)| sim.factory.constructor_at(at(x, 63, z)).out.count(IRON_ROD)).collect()
}

/// Coal not yet burnt: in the box, on the belt and in the boiler.
fn coal_left(sim: &Sim) -> u32 {
    let f = &sim.factory;
    let boxed = f.storage_count_at(at(6, 63, 1), COAL_ORE.into());
    let belts: usize = f.belts.iter().map(|b| b.items.len()).sum();
    boxed + belts as u32 + f.processor_at(at(6, 63, 5), BOILER).fuel.total()
}

#[test]
fn a_sea_pump_feeds_a_boiler_and_two_turbines_at_400_kw() {
    let mut sim = plant(false);
    let (mut peak, mut best) = (0, [0; 2]);
    for tick in 0..55 * crate::TICK_RATE {
        sim.step();
        let p = &sim.factory.power;
        peak = peak.max(p.demand[0]);
        if tick > 300 && p.demand[0] > 0 {
            assert_eq!(p.supply[0], p.demand[0], "no brownout at tick {tick}");
        }
        for (i, pos) in [at(3, 63, 7), at(11, 63, 7)].into_iter().enumerate() {
            best[i] = best[i].max(sim.factory.processor_at(pos, TURBINE).steam.output);
        }
    }
    assert!((405..=410).contains(&peak), "nine Mk3 constructors and a pump: {peak} kW");
    assert_eq!(best, [240, 170], "the first turbine gives all it can, the second the rest of 405 kW and the pump's 5");
    assert_eq!(rods(&sim), [64; 9], "every constructor finished its stack");
    // 405 kW for 43 s is 17,400 kJ: about 32 coal at 540 kJ, and two coal's steam still in the boiler.
    let burnt = COAL - coal_left(&sim);
    assert!((30..=38).contains(&burnt), "{burnt} coal burnt");
    let boiler = sim.factory.processor_at(at(6, 63, 5), BOILER);
    assert!(boiler.steam.water > 0, "water was drawn from the pump");
    assert_eq!(boiler.status, Status::Working);
    let text = sim.factory.describe(at(3, 63, 7)).unwrap();
    assert!(text.starts_with("Standing by") || text.starts_with("Giving"), "{text}");
}

/// `cargo test --release bench_plant -- --ignored --nocapture`: the cost of one tick of the plant above.
#[test]
#[ignore]
fn bench_plant() {
    let mut sim = plant(false);
    let (mut worst, mut at, start) = (std::time::Duration::ZERO, 0, std::time::Instant::now());
    for i in 0..3000 {
        let t = std::time::Instant::now();
        sim.step();
        if t.elapsed() > worst {
            (worst, at) = (t.elapsed(), i);
        }
    }
    println!("plant: {:?} a tick on average, {worst:?} worst (tick {at})", start.elapsed() / 3000);
}

#[test]
fn a_pond_fed_plant_runs_dry_and_says_so() {
    let mut sim = plant(true);
    (0..60 * crate::TICK_RATE).for_each(|_| sim.step());
    let boiler = sim.factory.processor_at(at(6, 63, 5), BOILER);
    assert_eq!(boiler.status, Status::NoWater);
    assert_eq!(boiler.status_text(), "Out of water: pipe a water inlet to a pump with water in reach");
    assert!(rods(&sim).iter().all(|&n| n < 64), "the pond's 6 blocks (12,000 kJ) can't press 43 s of 405 kW");
    let turbine = sim.factory.describe(at(3, 63, 7)).unwrap();
    assert!(turbine.starts_with("No steam"), "{turbine}");
}

/// A boiler at the origin facing north (cells x 0..1, z -1..0; its water inlets at (1, 0, -2), (-1, 0, -1) and
/// (2, 0, 0), its steam outlets at (0, 0, 1) and (1, 0, 1)) with a steam pipe from the outlets to the inlets of
/// three turbines: west of it at (-3, 0, 2) and (-3, 0, 4) facing north, east of it at (5, 0, 0) facing south.
fn boiler_with_turbines() -> Factory {
    let mut f = Factory::default();
    let world = &mut World::new(1, 2);
    for (block, x, z, facing) in [(BOILER, 0, 0, 0), (TURBINE, -3, 2, 0), (TURBINE, 5, 0, 2), (TURBINE, -3, 4, 0)] {
        f.place(world, block, IVec3::new(x, 0, z), facing, IVec3::ZERO, 0);
    }
    for (x, z) in [(0, 1), (1, 1), (2, 1), (0, 2), (0, 3)] {
        f.place(world, PIPE, IVec3::new(x, 0, z), 0, IVec3::ZERO, 0);
    }
    f.relink();
    f
}

#[test]
fn a_boiler_drives_two_turbines_and_no_more() {
    let f = boiler_with_turbines();
    let boilers: Vec<Vec<u32>> = f.processors.iter().map(|p| p.steam.boilers.clone()).collect();
    assert_eq!(boilers, [vec![], vec![0], vec![0], vec![]], "the third turbine finds every seat taken");
    let text = f.describe(IVec3::new(-3, 0, 4)).unwrap();
    assert!(text.starts_with("No boiler: pipe its steam inlet to a boiler's steam outlet\nNo power"), "{text}");
    assert!(f.pipework.iter().all(|p| p.fluid == Fluid::Steam), "every pipe is on the steam network");
}

#[test]
fn a_turbine_touching_a_boiler_gets_no_steam_without_a_pipe() {
    let mut f = Factory::default();
    let world = &mut World::new(1, 2);
    f.place(world, BOILER, IVec3::ZERO, 0, IVec3::ZERO, 0);
    f.place(world, TURBINE, IVec3::new(2, 0, 0), 0, IVec3::ZERO, 0);
    f.relink();
    assert!(f.processors[1].steam.boilers.is_empty());
}

#[test]
fn water_and_steam_on_one_network_work_for_neither() {
    let mut f = boiler_with_turbines();
    let world = &mut World::new(1, 2);
    // A pipe from the boiler's east water inlet down to the steam pipe joins the two networks.
    f.place(world, PIPE, IVec3::new(2, 0, 0), 0, IVec3::ZERO, 0);
    f.relink();
    assert!(f.pipework.iter().all(|p| p.fluid == Fluid::Mixed));
    assert!(f.processors.iter().all(|p| p.steam.crossed));
    assert!(f.processors[1].steam.boilers.is_empty(), "no steam reaches the turbine");
    assert!(f.processors[0].steam.nets.is_empty(), "no water reaches the boiler");
    let text = f.describe(IVec3::new(-3, 0, 2)).unwrap();
    assert!(text.starts_with("Water and steam share a pipe network"), "{text}");
}

#[test]
fn a_boiler_burns_coal_for_540_kj_and_a_water_block_lasts_2000() {
    let mut f = boiler_with_turbines();
    let boiler = &mut f.processors[0];
    boiler.steam.water = 32_400;
    assert_eq!(boiler.insert(COAL_ORE.into(), 2, &[]), 2);
    boiler.boil();
    assert_eq!((boiler.steam.steam, boiler.steam.water, boiler.status), (32_400, 0, Status::Working));
    boiler.boil();
    assert_eq!(boiler.steam.steam, 32_400, "the second coal waits for water");
    assert_eq!(UNIT_ENERGY, 4 * 32_400 - 9_600, "a water block is 2,000 kJ, a coal's steam 540");
}

#[test]
fn boilers_keep_their_steam_and_water_through_a_save() {
    let mut f = boiler_with_turbines();
    f.processors[0].steam.steam = 12_345;
    f.processors[0].steam.water = 6_789;
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    let b = &back.processors[0].steam;
    assert_eq!((b.steam, b.water), (12_345, 6_789));
}

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A box of `n` `item` at `from` and a belt from it into the cell ahead, facing `dir`.
fn feed(f: &mut Factory, from: IVec3, dir: u8, item: crate::item::ItemId, n: u32) {
    f.add_storage(from);
    f.stock(from, item, n);
    f.add_belt(from + crate::factory::DIRS[dir as usize], dir);
}

/// A silo at the origin facing north (cells x 0..1, z -1..0, three high), fed from behind (iron), the left
/// (copper) and the right (coal); its front faces south.
fn silo_fed_from_three_sides() -> Factory {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), SILO, IVec3::ZERO, 0, IVec3::ZERO, 0);
    feed(&mut f, v(0, 0, -3), 2, IRON_INGOT, 10);
    feed(&mut f, v(-2, 0, 0), 1, COPPER_INGOT, 10);
    feed(&mut f, v(3, 0, 0), 3, COAL_ORE.into(), 10);
    f
}

#[test]
fn a_silo_takes_belts_on_three_sides_and_holds_144_stacks() {
    let mut f = silo_fed_from_three_sides();
    crate::factory::tests::run(&mut f, 20.0, |_| {});
    let silo = f.processor_at(IVec3::ZERO, SILO);
    assert_eq!(silo.out.slots.len(), 144);
    assert_eq!([IRON_INGOT, COPPER_INGOT, COAL_ORE.into()].map(|i| silo.out.count(i)), [10, 10, 10]);
    assert_eq!(
        f.describe(v(1, 2, -1)).unwrap(),
        "Holding 3 of 144 stacks\nOut: 10 Iron Ingot, 10 Copper Ingot, 10 Coal Ore\nRight-click to open"
    );
}

#[test]
fn a_silo_gives_to_a_belt_leading_away_from_its_front() {
    let mut f = silo_fed_from_three_sides();
    f.add_belt(v(1, 0, 1), 2);
    f.add_storage(v(1, 0, 2));
    crate::factory::tests::run(&mut f, 40.0, |_| {});
    assert_eq!(f.storage_count_at(v(1, 0, 2), IRON_INGOT) + f.storage_count_at(v(1, 0, 2), COPPER_INGOT), 20);
    assert_eq!(f.storage_count_at(v(1, 0, 2), COAL_ORE.into()), 10);
    assert_eq!(f.processor_at(IVec3::ZERO, SILO).out.total(), 0, "all of it passed through");
}

#[test]
fn a_silo_saves_what_it_holds() {
    let mut f = silo_fed_from_three_sides();
    crate::factory::tests::run(&mut f, 20.0, |_| {});
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.processor_at(IVec3::ZERO, SILO).out.total(), 30);
}

/// A boiler at the origin (cells x 0..1, z -1..0) with a lone pipe on its west water inlet and a lone pump on its
/// east one: two networks that touch it and nothing else. A pipe at (-1, 0, 0) sits on its west coal inlet, where
/// nothing joins.
fn boiler_with_two_networks() -> Factory {
    let mut f = Factory::default();
    let world = &mut World::new(1, 2);
    f.place(world, BOILER, IVec3::ZERO, 0, IVec3::ZERO, 0);
    f.place(world, PIPE, IVec3::new(-1, 0, -1), 0, IVec3::ZERO, 0);
    f.place(world, PUMP, IVec3::new(2, 0, 0), 0, IVec3::ZERO, 0);
    f.relink();
    f
}

#[test]
fn a_boiler_takes_water_from_every_network_that_touches_it() {
    let mut f = boiler_with_two_networks();
    let b = &f.processors[0].steam;
    assert_eq!(b.nets.len(), 2, "the pipe and the pump are on different networks");
    assert_eq!(b.taps, [(IVec3::new(0, 0, -1), IVec3::new(-1, 0, 0)), (IVec3::new(1, 0, 0), IVec3::new(1, 0, 0))]);
    let pump = f.pipework.iter().position(|p| p.part == Part::Pump).unwrap();
    f.pipework[pump].held = 1;
    draw_water(&mut f.processors, &mut f.pipework);
    assert_eq!(
        (f.processors[0].steam.water, f.pipework[pump].held),
        (UNIT_ENERGY, 0),
        "the pump on the second network fed it"
    );
}

#[test]
fn pipes_and_pumps_on_a_water_inlet_reach_into_the_boiler() {
    let mut f = boiler_with_two_networks();
    let east = crate::factory::FACES.iter().position(|&d| d == IVec3::new(1, 0, 0)).unwrap();
    let west = crate::factory::FACES.iter().position(|&d| d == IVec3::new(-1, 0, 0)).unwrap();
    let arms: Vec<(Part, u8)> = f.pipework.iter().map(|p| (p.part, p.arms)).collect();
    assert_eq!(arms, [(Part::Pipe, 1 << east), (Part::Pump, 1 << west)]);
    // A pipe on a coal inlet or beside the tank joins nothing.
    f.place(&mut World::new(1, 2), PIPE, IVec3::new(0, 0, -2), 0, IVec3::ZERO, 0);
    f.place(&mut World::new(1, 2), PIPE, IVec3::new(0, 2, 0), 0, IVec3::ZERO, 0);
    f.relink();
    assert_eq!(f.processors[0].steam.taps.len(), 2);
    assert!(f.pipework.iter().filter(|p| p.pos.z == -2 || p.pos.y == 2).all(|p| p.arms == 0));
}

#[test]
fn a_boiler_draws_three_chutes_three_water_inlets_two_steam_outlets_and_a_gauge() {
    use crate::factory::process::parts::TURBINE_PARTS;
    use crate::factory::{Machine, INSTANCE_FLOATS};
    let boxes = |p: &Processor| {
        let mut out = Vec::new();
        p.model(&mut out, crate::math::Vec3::ZERO, 0.0);
        out.len() / INSTANCE_FLOATS
    };
    let mut f = boiler_with_turbines();
    // The body parts but the smoke (none while cold); a chute of 2 boxes at each of 3 coal inlets, a nipple and a
    // flange at each of 3 water inlets and each of 2 steam outlets; the gauge frame. A turbine: its parts and 2 steam
    // inlets of 2 boxes.
    let body = BOILER_PARTS.len() - 1;
    assert_eq!(boxes(&f.processors[0]), body + 6 + 6 + 4 + 1);
    f.processors[0].steam.water = UNIT_ENERGY;
    assert_eq!(boxes(&f.processors[0]), body + 6 + 6 + 4 + 2, "water shows in the gauge");
    assert_eq!(boxes(&f.processors[1]), TURBINE_PARTS.len() + 4);
}

#[test]
fn a_pipe_is_blue_for_water_and_pale_for_steam() {
    use crate::factory::Machine;
    let colours = |p: &crate::factory::pipes::Pipework| {
        let mut out = Vec::new();
        p.model(&mut out, crate::math::Vec3::ZERO, 0.0);
        out
    };
    let steam = boiler_with_turbines();
    let water = boiler_with_two_networks();
    assert_ne!(colours(&steam.pipework[0]), colours(&water.pipework[0]));
    assert_eq!(steam.pipework[0].fluid, Fluid::Steam);
    assert_eq!(water.pipework[0].fluid, Fluid::Water);
}

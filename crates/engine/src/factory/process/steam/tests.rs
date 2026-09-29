//! Steam power in a whole core (`Sim`): a plant on hand-built stone far from spawn, placed by actions like a
//! player places it, feeding nine Mk3 constructors (405 kW) that each press 64 ingots into rods.

use super::*;
use crate::action::Action;
use crate::block::{BlockId, AIR, BOILER, COAL_ORE, CONSTRUCTOR, PIPE, POLE, PUMP, SILO, STONE, TURBINE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
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
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing: 0, against: pos - IVec3::new(0, 1, 0) });
    assert_eq!(sim.world.block_anywhere_or_generate(pos), block, "placed at {pos:?}");
}

const LOADS: [(i32, i32); 9] = [(12, 8), (14, 8), (16, 8), (12, 10), (16, 10), (12, 12), (14, 12), (16, 12), (14, 14)];

/// A pump in the sea (or a four-block pond) at (3, 60, 5), piped up and across to a boiler at (6, 63, 5)
/// with a turbine on its east side and another on its north, coal from a box by a belt into its back,
/// three poles, and `LOADS`: Mk3 constructors, each with 64 ingots to press. The pump starts with its two
/// blocks of water in hand: a real plant is started by a generator on the same grid.
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
    for (x, y) in [(3, 61), (3, 62), (3, 63), (4, 63), (5, 63)] {
        put(&mut sim, PIPE, at(x, y, 5));
    }
    put(&mut sim, BOILER, at(6, 63, 5));
    put(&mut sim, TURBINE, at(8, 63, 5));
    put(&mut sim, TURBINE, at(7, 63, 3));
    for (x, z) in [(4, 6), (9, 6), (14, 10)] {
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
        for (i, pos) in [at(8, 63, 5), at(7, 63, 3)].into_iter().enumerate() {
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
    let text = sim.factory.describe(at(8, 63, 5)).unwrap();
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
    assert_eq!(boiler.status_text(), "Out of water: pipe it to a pump with water in reach");
    assert!(rods(&sim).iter().all(|&n| n < 64), "the pond's 6 blocks (12,000 kJ) can't press 43 s of 405 kW");
    let turbine = sim.factory.describe(at(8, 63, 5)).unwrap();
    assert!(turbine.starts_with("No steam"), "{turbine}");
}

/// A boiler at the origin facing north (cells x 0..1, z -1..0) with turbines against it: east, north, south.
fn boiler_with_turbines() -> Factory {
    let mut f = Factory::default();
    let world = &mut World::new(1, 2);
    for (block, x, z) in [(BOILER, 0, 0), (TURBINE, 2, 0), (TURBINE, 0, -2), (TURBINE, 0, 2)] {
        f.place(world, block, IVec3::new(x, 0, z), 0, IVec3::ZERO, 0);
    }
    f.relink();
    f
}

#[test]
fn a_boiler_drives_two_turbines_and_no_more() {
    let f = boiler_with_turbines();
    let boilers: Vec<Vec<u32>> = f.processors.iter().map(|p| p.steam.boilers.clone()).collect();
    assert_eq!(boilers, [vec![], vec![0], vec![0], vec![]], "the third turbine finds every seat taken");
    let text = f.describe(IVec3::new(0, 0, 2)).unwrap();
    assert!(text.starts_with("No boiler: set it against a boiler's side\nNot connected"), "{text}");
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

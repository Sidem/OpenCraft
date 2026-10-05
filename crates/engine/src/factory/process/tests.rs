use super::*;
use crate::block::{
    ARC_FURNACE, ASSEMBLER, BAUXITE_ORE, BLAST_FURNACE, COAL_ORE, CONSTRUCTOR, CRUSHER, ELECTROLYTIC_CELL, GENERATOR,
    IRON_ORE, LIMESTONE, POLE, QUARTZ_ORE, SAND, SLAG, SMELTER, STONE, STONE_BRICKS,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::{powered, recipe_for, run};
use crate::factory::tiers::FAMILIES;
use crate::factory::upgrades::Step;
use crate::item::{
    ALUMINIUM_INGOT, ALUMINIUM_PLATE, BATTERY, CIRCUIT, COPPER_WIRE, CRUSHED_BAUXITE, GEAR, GREEN_KIT, IRON_INGOT,
    IRON_PLATE, IRON_ROD, MOTOR, QUICKLIME, SILICON, STEEL_INGOT,
};
use crate::research::{Research, TECHS};
use crate::world::World;

/// Test-only: taking items as a fresh world's research allows.
impl Processor {
    pub(crate) fn accept_fresh(&mut self, item: ItemId) -> bool {
        self.accept(item, &Research::default().machine_recipes_unlocked())
    }

    pub(crate) fn can_accept_fresh(&self, item: ItemId) -> bool {
        self.room_for(item, &Research::default().machine_recipes_unlocked()) > 0
    }
}

fn place(f: &mut Factory, block: BlockId, pos: IVec3, tier: u8) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
}

#[test]
fn every_spec_has_a_row_per_tier_and_fitting_buffers() {
    for s in SPECS {
        let family = FAMILIES.iter().find(|f| f.block == s.block).map_or(1, |f| f.items.len());
        assert!(s.tiers.len() >= family, "block {} has fewer tier rows than items", s.block);
        let [input, fuel, out] = s.buffers;
        let steam = s
            .tiers
            .iter()
            .any(|t| matches!(t.energy, Energy::Boiler | Energy::Turbine | Energy::Solar | Energy::Accumulator));
        assert!(steam || s.pick == Pick::Hangar || (input > 0 || s.pick.stores()) && out > 0, "block {}", s.block);
        assert_eq!(
            fuel > 0,
            s.tiers.iter().any(|t| matches!(t.energy, Energy::Burner | Energy::Boiler)),
            "only burners hold fuel (block {})",
            s.block
        );
        assert_eq!(crate::factory::machine(s.block).map(|m| m.kind), Some(crate::factory::Kind::Process));
    }
}

#[test]
fn a_mk2_smelter_works_twice_as_fast_on_a_quarter_less_fuel() {
    let ingots = |tier: u8, seconds: f64| {
        let mut f = Factory::default();
        place(&mut f, SMELTER, IVec3::ZERO, tier);
        assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 20), 20);
        assert_eq!(f.insert(IVec3::ZERO, COAL_ORE.into(), 1), 1);
        run(&mut f, seconds, |_| {});
        let s = f.smelter_at(IVec3::ZERO);
        (s.out.count(IRON_INGOT), s.status)
    };
    // A coal fuels 8 s of Mk1 work: 5⅓ ingots at Mk1, 7⅑ at Mk2 (in 5⅓ s).
    assert_eq!(ingots(0, 6.0), (4, Status::Working));
    assert_eq!(ingots(0, 12.0), (5, Status::NoFuel));
    assert_eq!(ingots(1, 6.0), (7, Status::NoFuel));
}

#[test]
fn a_mk2_constructor_presses_twice_as_fast_and_draws_twice_the_power() {
    let plates = |tier: u8| {
        let mut f = Factory::default();
        powered(&mut f);
        place(&mut f, CONSTRUCTOR, IVec3::ZERO, tier);
        assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(IRON_PLATE))).is_some());
        f.insert(IVec3::ZERO, IRON_INGOT, 20);
        run(&mut f, 4.05, |_| {});
        (f.constructor_at(IVec3::ZERO).out.count(IRON_PLATE), f.power.demand[0])
    };
    assert_eq!(plates(0), (2, 15));
    assert_eq!(plates(1), (4, 30));
}

#[test]
fn masonry_lets_smelters_fire_stone_and_limestone() {
    let mut f = Factory::default();
    place(&mut f, SMELTER, IVec3::ZERO, 0);
    assert_eq!(f.insert(IVec3::ZERO, STONE.into(), 4), 0, "bricks are locked");
    assert!(!f.smelter_at(IVec3::ZERO).can_accept_fresh(LIMESTONE.into()));
    let masonry = TECHS.iter().position(|t| t.name == "Masonry").unwrap() as u8;
    (0..TECHS[masonry as usize].units).for_each(|_| f.research.add_unit(masonry));
    assert_eq!(f.insert(IVec3::ZERO, STONE.into(), 4), 4);
    f.insert(IVec3::ZERO, COAL_ORE.into(), 1);
    run(&mut f, 6.05, |_| {});
    assert_eq!(f.smelter_at(IVec3::ZERO).out.count(STONE_BRICKS.into()), 2, "2 stone a brick, 3 s each");
    f.take_contents(IVec3::ZERO, |_, n| n);
    assert_eq!(f.insert(IVec3::ZERO, LIMESTONE.into(), 1), 1);
    run(&mut f, 2.05, |_| {});
    assert_eq!(f.smelter_at(IVec3::ZERO).out.count(QUICKLIME), 1);
}

#[test]
fn a_smelter_upgrades_in_place_with_four_kits() {
    let mut f = Factory::default();
    place(&mut f, SMELTER, IVec3::ZERO, 0);
    f.insert(IVec3::ZERO, IRON_ORE.into(), 5);
    assert_eq!(f.next_upgrade(IVec3::ZERO), Some(Step { block: SMELTER, tier: 1, kit: GREEN_KIT, kits: 4 }));
    assert!(f.upgrade(IVec3::ZERO));
    let s = f.smelter_at(IVec3::ZERO);
    assert_eq!((s.tier, s.stats().speed, s.input.total()), (1, 2000, 5));
    let text = f.describe(IVec3::ZERO).unwrap();
    assert!(text.starts_with("Mk2 · next: Blue Kit ×4\nWaiting for"), "{text}");
}

#[test]
fn version_18_processors_had_no_facing_and_load_facing_north() {
    let mut p = Processor::new(IVec3::new(1, 2, 3), spec(CONSTRUCTOR).unwrap(), 1);
    p.dir = 2;
    p.recipe = Some(recipe_for(IRON_PLATE));
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    // Version 18 wrote the same record without the facing byte after position (12), block and tier.
    let mut old = w.bytes.clone();
    assert_eq!(old.remove(14), 2);
    let mut r = ByteReader::new(&old);
    r.version = 18;
    let back = Processor::read_state(&mut r).unwrap();
    assert_eq!((back.pos, back.tier, back.dir, back.recipe), (p.pos, 1, 0, p.recipe));
    assert_eq!(r.u8(), None, "every byte read");
}

/// A factory saved by version 17: a smelter at the origin a third into a batch (2 ore, 2 coal, 2 ingots,
/// 270 ticks of fire) and a constructor at (2, 0, 0) pressing plates, 1.5 s into a batch (5 ingots, 1
/// plate), with the pole and generator that power it.
const V17_FACTORY: &str =
    "000000000000000000000000010000000000000000000000000000000800020000000700020000000001020000000100\
    001e0000000e01000000000000000100000002000000000000000000000001020000010500000002010100000001905f0100000000000100\
    0000000100000003000000030000000000000007003f000000fa320000010000000200000003000000000000000000000000000000000000\
    00000000000000000000000000ff0700000000000000000000000000000000000000000000000000000000000000";

#[test]
fn version_17_smelters_and_constructors_load_as_processors_mid_batch() {
    let hex = V17_FACTORY;
    let bytes: Vec<u8> = (0..hex.len() / 2).map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap()).collect();
    let mut r = ByteReader::new(&bytes);
    r.version = 17;
    let mut f = Factory::read_state(&mut World::new(1, 2), &mut r).unwrap();
    let s = f.smelter_at(IVec3::ZERO);
    assert_eq!((s.batch, s.progress, s.burn, s.status), (Some(0), 30_000, 270_000, Status::Working));
    assert_eq!((s.input.total(), s.fuel.total(), s.out.total(), s.tier), (2, 2, 2, 0));
    let c = f.constructor_at(IVec3::new(2, 0, 0));
    let plates = Some(recipe_for(IRON_PLATE));
    assert_eq!((c.recipe, c.batch, c.progress, c.input.total(), c.out.total()), (plates, plates, 90_000, 5, 1));
    // Today's format round-trips, and both carry on where they were.
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let mut again = ByteWriter::default();
    Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap().write_state(&mut again);
    assert!(again.bytes == w.bytes);
    run(&mut f, 1.0, |_| {});
    assert_eq!(f.smelter_at(IVec3::ZERO).out.count(IRON_INGOT), 3, "the batch finished 1 s later");
    assert_eq!(f.constructor_at(IVec3::new(2, 0, 0)).out.count(IRON_PLATE), 2, "and the plate 0.5 s later");
}

const NORTH: u8 = 0;
const EAST: u8 = 1;
const SOUTH: u8 = 2;

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

/// A box of `n` `item` at `from` and a belt from it into the cell ahead, facing `dir`.
fn feed(f: &mut Factory, from: IVec3, dir: u8, item: ItemId, n: u32) {
    f.add_storage(from);
    f.stock(from, item, n);
    f.add_belt(from + crate::factory::DIRS[dir as usize], dir);
}

/// A blast furnace at the origin facing north, ore from behind, coal and quicklime from its left, and a
/// belt and box at its front (steel).
fn blast_furnace(ore: u32, coal: u32, lime: u32) -> Factory {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), BLAST_FURNACE, IVec3::ZERO, NORTH, IVec3::ZERO, 0);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(STEEL_INGOT))).is_some());
    feed(&mut f, v(0, 0, -3), SOUTH, IRON_ORE.into(), ore);
    feed(&mut f, v(-2, 0, 0), EAST, COAL_ORE.into(), coal);
    feed(&mut f, v(-2, 0, -1), EAST, QUICKLIME, lime);
    f.add_belt(v(0, 0, 1), SOUTH);
    f.add_storage(v(0, 0, 2));
    f
}

#[test]
fn a_blast_furnace_makes_steel_at_its_front_and_slag_at_its_side() {
    let mut f = blast_furnace(4, 2, 2);
    f.add_belt(v(2, 0, 0), EAST);
    f.add_storage(v(3, 0, 0));
    run(&mut f, 11.0, |_| {});
    assert_eq!(f.storage_count_at(v(0, 0, 2), STEEL_INGOT), 2, "4 s a batch, no power needed");
    assert_eq!(f.storage_count_at(v(3, 0, 0), SLAG.into()), 2);
    assert_eq!(f.processors[0].status, Status::NoInput);
}

#[test]
fn a_blast_furnace_with_nowhere_for_slag_stops_and_says_so() {
    let mut f = blast_furnace(40, 20, 20);
    run(&mut f, 90.0, |_| {});
    assert_eq!(f.storage_count_at(v(0, 0, 2), STEEL_INGOT), 16, "the side buffer holds 16 slag");
    let p = &f.processors[0];
    assert_eq!((p.status, p.side.count(SLAG.into())), (Status::OutputFull, 16));
    let text = f.describe(IVec3::ZERO).unwrap();
    assert!(text.contains("Slag has nowhere to go"), "{text}");
    // Taking the slag by hand starts it again.
    let mut slag = 0;
    f.take_contents(IVec3::ZERO, |item, n| {
        slag += n * u32::from(item == SLAG.into());
        n
    });
    assert_eq!(slag, 16);
    run(&mut f, 5.0, |_| {});
    assert_eq!(f.storage_count_at(v(0, 0, 2), STEEL_INGOT), 17);
}

#[test]
fn a_blast_furnace_saves_its_byproduct_buffer() {
    let mut f = Factory::default();
    f.place(&mut World::new(1, 2), BLAST_FURNACE, IVec3::ZERO, EAST, IVec3::ZERO, 0);
    f.processors[0].side.add(SLAG.into(), 5);
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let back = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.processors[0].side.count(SLAG.into()), 5);
    assert!(back.processors[0].contents().contains(&Stack { item: SLAG.into(), count: 5 }));
    let mut again = ByteWriter::default();
    back.write_state(&mut again);
    assert!(again.bytes == w.bytes);
}

#[test]
fn a_mk3_smelter_is_electric_and_burns_nothing() {
    let ingots = |tier: u8| {
        let mut f = Factory::default();
        powered(&mut f);
        place(&mut f, SMELTER, IVec3::ZERO, tier);
        assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 20), 20);
        let fuel = f.insert(IVec3::ZERO, COAL_ORE.into(), 1);
        run(&mut f, 3.05, |_| {});
        (f.smelter_at(IVec3::ZERO).out.count(IRON_INGOT), fuel, f.power.demand[0])
    };
    // A Mk3 makes 3 ingots a 1.5 s: 6 in 3 s at 40 kW, and takes no coal (the Mk1 fires up on the one).
    assert_eq!(ingots(2), (6, 0, 40));
    assert_eq!(ingots(0).1, 1);
}

#[test]
fn upgrading_a_burning_smelter_to_mk3_hands_it_to_the_grid() {
    let mut f = Factory::default();
    powered(&mut f);
    place(&mut f, SMELTER, IVec3::ZERO, 1);
    f.insert(IVec3::ZERO, IRON_ORE.into(), 20);
    assert!(f.upgrade(IVec3::ZERO) && f.next_upgrade(IVec3::ZERO).is_some_and(|s| s.tier == 3));
    run(&mut f, 3.05, |_| {});
    let s = f.smelter_at(IVec3::ZERO);
    assert_eq!((s.tier, s.energy(), s.out.count(IRON_INGOT), s.status), (2, Energy::Electric, 6, Status::Working));
}

#[test]
fn a_mk3_constructor_presses_three_times_as_fast_and_draws_45_kw() {
    let mut f = Factory::default();
    powered(&mut f);
    place(&mut f, CONSTRUCTOR, IVec3::ZERO, 2);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(IRON_PLATE))).is_some());
    f.insert(IVec3::ZERO, IRON_INGOT, 20);
    run(&mut f, 4.05, |_| {});
    assert_eq!((f.constructor_at(IVec3::ZERO).out.count(IRON_PLATE), f.power.demand[0]), (6, 45));
}

#[test]
fn assemblers_and_blast_furnaces_work_faster_at_each_tier() {
    let motors = |tier: u8| {
        let mut f = Factory::default();
        powered(&mut f);
        f.place(&mut World::new(1, 2), ASSEMBLER, IVec3::ZERO, NORTH, IVec3::ZERO, tier);
        f.set_recipe(IVec3::ZERO, Some(recipe_for(MOTOR)));
        for (item, n) in [(IRON_ROD, 6), (GEAR, 12), (COPPER_WIRE, 24)] {
            f.insert(IVec3::ZERO, item, n);
        }
        run(&mut f, 5.05, |_| {});
        (f.processors[0].out.count(MOTOR), f.power.demand[0])
    };
    // A motor takes 5 s at Mk1, 2.5 s at Mk2 and 5/3 s at Mk3; the assembler draws 20, 40 and 60 kW.
    assert_eq!([motors(0), motors(1), motors(2)], [(1, 20), (2, 40), (3, 60)]);
    let steel = |tier: u8| {
        let mut f = Factory::default();
        f.place(&mut World::new(1, 2), BLAST_FURNACE, IVec3::ZERO, NORTH, IVec3::ZERO, tier);
        f.set_recipe(IVec3::ZERO, Some(recipe_for(STEEL_INGOT)));
        for (item, n) in [(IRON_ORE.into(), 8), (COAL_ORE.into(), 4), (QUICKLIME, 4)] {
            f.insert(IVec3::ZERO, item, n);
        }
        run(&mut f, 4.05, |_| {});
        f.processors[0].out.count(STEEL_INGOT)
    };
    assert_eq!([steel(0), steel(1), steel(2)], [1, 2, 3]);
}

/// Research done, as if a lab had finished the tech named `name`.
fn research_done(f: &mut Factory, name: &str) {
    let tech = TECHS.iter().position(|t| t.name == name).unwrap() as u8;
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
}

#[test]
fn a_crusher_line_gives_one_and_a_half_ingots_an_ore() {
    // Box of ore, crusher, smelter (with coal), box: 40 ore make 20 batches of 3 crushed, 60 ingots.
    let mut f = Factory::default();
    powered(&mut f);
    research_done(&mut f, "Ore Crushing");
    feed(&mut f, v(0, 0, 0), EAST, IRON_ORE.into(), 40);
    place(&mut f, CRUSHER, v(2, 0, 0), 0);
    f.add_belt(v(3, 0, 0), EAST);
    place(&mut f, SMELTER, v(4, 0, 0), 0);
    feed(&mut f, v(4, 0, -2), SOUTH, COAL_ORE.into(), 20);
    f.add_belt(v(5, 0, 0), EAST);
    f.add_storage(v(6, 0, 0));
    run(&mut f, 200.0, |_| {});
    assert_eq!(f.storage_count_at(v(6, 0, 0), IRON_INGOT), 60, "1.5 ingots an ore");
    assert_eq!(f.processors[0].power(), 30);
}

#[test]
fn a_crusher_needs_its_research_and_grinds_slag_to_sand() {
    let mut f = Factory::default();
    powered(&mut f);
    place(&mut f, CRUSHER, IVec3::ZERO, 0);
    assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 4), 0, "locked until Ore Crushing");
    research_done(&mut f, "Ore Crushing");
    assert_eq!(f.insert(IVec3::ZERO, SLAG.into(), 2), 2);
    run(&mut f, 1.0, |_| {});
    assert_eq!(f.power.demand[0], 30);
    run(&mut f, 1.05, |_| {});
    assert_eq!(f.processors[0].out.count(SAND.into()), 2, "a second a slag");
    let text = f.describe(IVec3::ZERO).unwrap();
    assert!(!text.starts_with("Mk"), "one-tier machines say no Mk: {text}");
}

#[test]
fn quartz_and_coal_become_circuits_through_an_arc_furnace_and_an_assembler() {
    let mut f = Factory::default();
    powered(&mut f);
    // The arc furnace (120 kW) and assembler (20 kW) need more than one small generator gives.
    for z in [-1, 1] {
        let gen = v(3, 3, z);
        f.place(&mut World::new(1, 2), GENERATOR, gen, 0, gen, 1);
        f.insert(gen, COAL_ORE.into(), 64);
    }
    research_done(&mut f, "Electronics");
    // Quartz from behind and coal from the left into the arc furnace; its silicon runs straight into an
    // assembler behind it, which also takes wire from its left and plates from its right.
    f.place(&mut World::new(1, 2), ARC_FURNACE, IVec3::ZERO, NORTH, IVec3::ZERO, 0);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(SILICON))).is_some());
    feed(&mut f, v(0, 0, -3), SOUTH, QUARTZ_ORE.into(), 8);
    feed(&mut f, v(-2, 0, 0), EAST, COAL_ORE.into(), 8);
    f.add_belt(v(0, 0, 1), SOUTH);
    f.place(&mut World::new(1, 2), ASSEMBLER, v(0, 0, 3), NORTH, v(0, 0, 3), 0);
    assert!(f.set_recipe(v(0, 0, 3), Some(recipe_for(CIRCUIT))).is_some());
    feed(&mut f, v(-2, 0, 3), EAST, COPPER_WIRE, 24);
    feed(&mut f, v(3, 0, 3), 3, IRON_PLATE, 8);
    f.add_belt(v(0, 0, 4), SOUTH);
    f.add_storage(v(0, 0, 5));
    run(&mut f, 60.0, |_| {});
    // 8 quartz and 8 coal make 8 silicon (4 s each); every 3 wire, plate and silicon make 2 circuits.
    assert_eq!(f.storage_count_at(v(0, 0, 5), CIRCUIT), 16);
    assert_eq!(f.processors[0].power(), 120);
}

/// Moves everything `from` holds into `to` (what a belt between them would do), returning what moved.
fn carry(f: &mut Factory, from: IVec3, to: IVec3) -> u32 {
    let mut taken = Vec::new();
    f.take_contents(from, |item, n| {
        taken.push((item, n));
        n
    });
    taken.into_iter().map(|(item, n)| f.insert(to, item, n)).sum()
}

#[test]
fn bauxite_becomes_batteries_through_a_crusher_a_cell_a_constructor_and_an_assembler() {
    let mut f = Factory::default();
    powered(&mut f);
    // The cell draws 300 kW: four Mk2 generators and a second pole cover it.
    f.place(&mut World::new(1, 2), POLE, v(4, 3, 2), 0, v(4, 3, 2), 0);
    for (x, z) in [(3, -1), (3, 1), (4, -1), (4, 1)] {
        f.place(&mut World::new(1, 2), GENERATOR, v(x, 3, z), 0, v(x, 3, z), 1);
        f.insert(v(x, 3, z), COAL_ORE.into(), 64);
    }
    let (crusher, cell, constructor, assembler) = (v(0, 0, 0), v(0, 0, 4), v(6, 0, 0), v(6, 0, 4));
    assert_eq!(f.insert(crusher, BAUXITE_ORE.into(), 4), 0, "locked until Bauxite Processing");
    research_done(&mut f, "Bauxite Processing");
    place(&mut f, CRUSHER, crusher, 0);
    f.place(&mut World::new(1, 2), ELECTROLYTIC_CELL, cell, NORTH, cell, 0);
    place(&mut f, CONSTRUCTOR, constructor, 0);
    f.place(&mut World::new(1, 2), ASSEMBLER, assembler, NORTH, assembler, 0);
    assert!(f.set_recipe(cell, Some(recipe_for(ALUMINIUM_INGOT))).is_some());
    assert!(f.set_recipe(constructor, Some(recipe_for(ALUMINIUM_PLATE))).is_some());
    assert!(f.set_recipe(assembler, Some(recipe_for(BATTERY))).is_some());

    // 4 ore make 6 crushed bauxite (2 s a batch of 2 ore).
    assert_eq!(f.insert(crusher, BAUXITE_ORE.into(), 4), 4);
    run(&mut f, 4.1, |_| {});
    assert_eq!(f.processors[0].out.count(CRUSHED_BAUXITE), 6);
    // 6 crushed and 3 quicklime make 3 ingots (6 s each) and 3 slag, which waits at the byproduct hatch.
    assert_eq!(carry(&mut f, crusher, cell), 6);
    assert_eq!(f.insert(cell, QUICKLIME, 3), 3);
    run(&mut f, 18.5, |_| {});
    let cell_at = f.processors.iter().position(|p| p.pos == cell).unwrap();
    assert_eq!(
        (f.processors[cell_at].out.count(ALUMINIUM_INGOT), f.processors[cell_at].side.count(SLAG.into())),
        (3, 3)
    );
    // The ingots press to plates (3 s each), which an assembler turns into batteries with circuits and wire.
    assert_eq!(carry(&mut f, cell, constructor), 3);
    run(&mut f, 9.5, |_| {});
    assert_eq!(carry(&mut f, constructor, assembler), 3);
    f.insert(assembler, CIRCUIT, 3);
    f.insert(assembler, COPPER_WIRE, 12);
    run(&mut f, 18.5, |_| {});
    let assembler_at = f.processors.iter().position(|p| p.pos == assembler).unwrap();
    assert_eq!(f.processors[assembler_at].out.count(BATTERY), 3);
}

#[test]
fn an_arc_furnace_takes_only_its_recipes_inputs_a_stack_of_each() {
    let mut f = Factory::default();
    powered(&mut f);
    f.place(&mut World::new(1, 2), ARC_FURNACE, IVec3::ZERO, NORTH, IVec3::ZERO, 0);
    assert!(f.set_recipe(IVec3::ZERO, Some(recipe_for(SILICON))).is_some());
    assert_eq!(f.insert(IVec3::ZERO, QUARTZ_ORE.into(), 200), 64, "one stack, so coal still fits");
    assert_eq!(f.insert(IVec3::ZERO, COAL_ORE.into(), 5), 5);
    assert_eq!(f.insert(IVec3::ZERO, IRON_ORE.into(), 5), 0);
}

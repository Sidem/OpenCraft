use super::*;
use crate::block::{COAL_ORE, CONSTRUCTOR, IRON_ORE, LIMESTONE, SMELTER, STONE, STONE_BRICKS};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tests::{powered, recipe_for, run};
use crate::factory::tiers::FAMILIES;
use crate::factory::upgrades::Step;
use crate::item::{GREEN_KIT, IRON_INGOT, IRON_PLATE, QUICKLIME};
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
        assert!(input > 0 && out > 0, "block {}", s.block);
        assert_eq!(fuel > 0, s.energy == Energy::Burner, "only burners hold fuel (block {})", s.block);
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
    assert!(text.starts_with("Mk2\nWaiting for"), "{text}");
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

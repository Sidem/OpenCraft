use super::*;
use crate::block::IRON_ORE;
use crate::factory::belt::BELT_TIERS;
use crate::factory::tests::run;
use crate::factory::Factory;
use crate::factory::MINER_TIERS;
use crate::math::IVec3;
use crate::world::World;

#[test]
fn every_tier_has_numbers_and_round_trips() {
    let numbers = |block: BlockId| match block {
        BELT => BELT_TIERS.len(),
        MINER => MINER_TIERS.len(),
        POLE => crate::factory::pole::POLE_TIERS.len(),
        STORAGE => crate::factory::storage::BOX_SLOTS.len(),
        PUMP => crate::factory::pumping::PUMP_TIERS.len(),
        QUARRY => crate::factory::quarry::QUARRY_TIERS.len(),
        LAB => crate::factory::lab::LAB_TIERS.len(),
        GENERATOR => crate::factory::generator::GENERATOR_TIERS.len(),
        _ => crate::factory::process::spec(block).map_or(0, |s| s.tiers.len()),
    };
    for f in FAMILIES {
        assert_eq!(f.items.len(), numbers(f.block), "tiers of block {}", f.block);
        for (t, &item) in f.items.iter().enumerate() {
            assert_eq!(placed_by(item), Some((f.block, t as u8)));
            assert_eq!(item_of(f.block, t as u8), Some(item));
        }
        assert_eq!(item_of(f.block, f.items.len() as u8), None);
    }
}

#[test]
fn a_mk2_item_places_the_family_block() {
    assert_eq!(ItemId::block(FAST_BELT).places(), Some(BELT));
    assert_eq!(ItemId::block(MINER_MK2).places(), Some(MINER));
    assert_eq!(ItemId::block(BELT).places(), Some(BELT));
    assert_eq!(placed_by(ItemId::block(crate::block::STONE)), None);
}

/// A factory saved by version 17 (before tiers): a Mk1 belt, a fast belt and a Mk2 miner. Tier bytes
/// replaced the `fast` and `mk2` bools byte for byte (processors: `process/tests.rs`).
const V17_FACTORY: &str =
    "020000000000000000000000000000000100000000000001000000000000000000000001000100000000010000000000\
    000001000000000000000300000000000000000000000000000000000200000000000000000000000001000000000000000000000000\
    000000000000000000000000000000000000000000000000000000000000000000000000ff07000000000000000000000000000000000000\
    00000000000000000000000000";

#[test]
fn a_version_17_factory_loads_with_the_same_speeds_and_rates() {
    use crate::bytes::{ByteReader, ByteWriter};
    use crate::factory::Factory;
    use crate::math::IVec3;
    let bytes: Vec<u8> =
        (0..V17_FACTORY.len() / 2).map(|i| u8::from_str_radix(&V17_FACTORY[2 * i..2 * i + 2], 16).unwrap()).collect();
    let mut r = ByteReader::new(&bytes);
    r.version = 17;
    let f = Factory::read_state(&mut crate::world::World::new(1, 2), &mut r).unwrap();
    assert_eq!(f.belt_at(IVec3::new(0, 0, 0)).speed(), 1.0);
    assert_eq!(f.belt_at(IVec3::new(1, 0, 0)).speed(), 2.0);
    let miner = f.miner_at(IVec3::new(0, 1, 0));
    assert_eq!((miner.rate(), miner.recovery(), miner.stats().power), (2.0, 0.75, 20));
    // Today's format (a processor list since version 18) keeps the tiers.
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let g = Factory::read_state(&mut crate::world::World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((g.belt_at(IVec3::new(1, 0, 0)).speed(), g.miner_at(IVec3::new(0, 1, 0)).rate()), (2.0, 2.0));
}

#[test]
fn a_mk3_belt_carries_about_ten_items_a_second_three_and_a_half_times_a_mk1() {
    let arrived = |tier: u8| {
        let mut f = Factory::default();
        f.add_storage(IVec3::ZERO);
        f.stock(IVec3::ZERO, IRON_ORE.into(), 1000);
        for x in 1..=4 {
            let p = IVec3::new(x, 0, 0);
            f.place(&mut World::new(1, 2), BELT, p, 1, p, tier);
        }
        f.add_storage(IVec3::new(5, 0, 0));
        run(&mut f, 5.0, |_| {});
        let before = f.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into());
        run(&mut f, 5.0, |_| {});
        f.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into()) - before
    };
    // Items are 0.35 blocks apart, and a belt takes a new one on the first tick past that gap: 1 / 2 / 4
    // blocks a second is 2.9 / 5.5 / 10 items a second at 60 ticks a second (2.9 / 5.7 / 11.4 on paper).
    let counts = [arrived(0), arrived(1), arrived(2)];
    assert!(counts[0] >= 13 && counts[0] <= 15, "Mk1 {counts:?}");
    assert!(counts[1] >= 27 && counts[1] <= 29, "Mk2 {counts:?}");
    assert!(counts[2] >= 49 && counts[2] <= 51, "Mk3 {counts:?}");
}

/// A pole, a coal-fed generator at tier `gen_tier` and nothing else: what the machines below hang on.
fn powered(f: &mut Factory, gen_tier: u8) {
    let (pole, gen) = (IVec3::new(2, 3, 0), IVec3::new(3, 3, 0));
    f.place(&mut World::new(1, 2), POLE, pole, 0, pole, 0);
    f.place(&mut World::new(1, 2), GENERATOR, gen, 0, gen, gen_tier);
    assert_eq!(f.insert(gen, crate::block::COAL_ORE.into(), 64), 64);
}

fn place_at(f: &mut Factory, block: BlockId, pos: IVec3, tier: u8) {
    f.place(&mut World::new(1, 2), block, pos, 0, pos, tier);
}

#[test]
fn every_tier_item_places_its_tier_and_a_save_keeps_it() {
    use crate::bytes::{ByteReader, ByteWriter};
    let mut f = Factory::default();
    let mut spots = Vec::new();
    for (row, fam) in FAMILIES.iter().enumerate() {
        for (tier, &item) in fam.items.iter().enumerate() {
            let pos = IVec3::new(8 * tier as i32, 0, 8 * row as i32);
            let (block, t) = placed_by(item).unwrap();
            place_at(&mut f, block, pos, t);
            assert_eq!(f.tiered_at(pos), Some((fam.block, tier as u8)), "item {}", item.0);
            spots.push((pos, fam.block, tier as u8));
        }
    }
    let mut w = ByteWriter::default();
    f.write_state(&mut w);
    let g = Factory::read_state(&mut World::new(1, 2), &mut ByteReader::new(&w.bytes)).unwrap();
    for (pos, block, tier) in spots {
        assert_eq!(g.tiered_at(pos), Some((block, tier)), "block {block} after a save");
    }
}

#[test]
fn older_saves_load_boxes_poles_and_generators_at_tier_one() {
    use crate::bytes::{ByteReader, ByteWriter};
    use crate::factory::generator::Generator;
    use crate::factory::pole::Pole;
    use crate::factory::storage::Storage;
    use crate::factory::Machine;
    let bytes = |m: &dyn Fn(&mut ByteWriter)| {
        let mut w = ByteWriter::default();
        m(&mut w);
        w.bytes
    };
    let mut old = bytes(&|w| Storage::new(IVec3::ZERO).write_state(w));
    old.remove(12); // Version 21 wrote the tier right after the position.
    let mut r = ByteReader::new(&old);
    r.version = 20;
    let s = Storage::read_state(&mut r).unwrap();
    assert_eq!((s.tier, s.buf.slots.len(), r.u8()), (0, 24, None));
    // The poles and generators wrote it last.
    let mut old = bytes(&|w| Pole { pos: IVec3::ZERO, tier: 0 }.write_state(w));
    old.pop();
    let mut r = ByteReader::new(&old);
    r.version = 20;
    assert!(Pole::read_state(&mut r).is_some_and(|p| p.tier == 0) && r.u8().is_none());
    let mut old = bytes(&|w| Generator::new(IVec3::ZERO).write_state(w));
    old.pop();
    let mut r = ByteReader::new(&old);
    r.version = 20;
    assert!(Generator::read_state(&mut r).is_some_and(|g| g.tier == 0) && r.u8().is_none());
}

#[test]
fn boxes_hold_24_36_and_48_stacks_and_an_upgrade_keeps_what_is_in_them() {
    let mut f = Factory::default();
    f.add_storage(IVec3::ZERO);
    f.stock(IVec3::ZERO, IRON_ORE.into(), 100);
    for (tier, slots) in [24, 36, 48].into_iter().enumerate() {
        assert_eq!(f.storages[0].buf.slots.len(), slots);
        assert_eq!(f.storage_count_at(IVec3::ZERO, IRON_ORE.into()), 100, "Mk{} keeps its stock", tier + 1);
        if tier < 2 {
            assert!(f.upgrade(IVec3::ZERO));
        }
    }
    // Full: 48 stacks of 64.
    assert_eq!(f.storages[0].buf.add(IRON_ORE.into(), 10_000), 10_000 - (48 * 64 - 100));
}

#[test]
fn taller_poles_link_further_and_reach_further() {
    use crate::block::CONSTRUCTOR;
    let mut f = Factory::default();
    let (near, far) = (IVec3::ZERO, IVec3::new(12, 0, 0));
    place_at(&mut f, POLE, near, 0);
    place_at(&mut f, POLE, far, 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.power.pole_grid, [0, 1], "12 blocks is past a Mk1's 10");
    assert!(f.upgrade(far));
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.power.pole_grid, [0, 0], "a Mk2 links 16, and the longer of the two counts");
    // A Mk3 links poles 32 blocks away, but not 33.
    let mut f = Factory::default();
    place_at(&mut f, POLE, IVec3::ZERO, 2);
    for (x, grids) in [(32, [0, 0]), (33, [0, 1])] {
        place_at(&mut f, POLE, IVec3::new(x, 0, 0), 0);
        run(&mut f, 0.1, |_| {});
        assert_eq!(f.power.pole_grid, grids, "a Mk1 pole {x} blocks from a Mk3");
        f.remove(IVec3::new(x, 0, 0));
    }
    // A machine 8 blocks from a lone pole: Mk1 reaches 5, Mk2 7, Mk3 9.
    let mut f = Factory::default();
    place_at(&mut f, POLE, IVec3::ZERO, 0);
    place_at(&mut f, CONSTRUCTOR, IVec3::new(8, 0, 0), 0);
    let hung = |f: &mut Factory| {
        run(f, 0.1, |_| {});
        f.power.process_pole[0].is_some()
    };
    assert!(!hung(&mut f));
    assert!(f.upgrade(IVec3::ZERO) && !hung(&mut f));
    assert!(f.upgrade(IVec3::ZERO) && hung(&mut f));
}

#[test]
fn a_mk2_generator_gives_100_kw_and_a_quarter_more_energy_from_the_same_coal() {
    let coal = crate::recipes::fuel_energy(crate::block::COAL_ORE.into()).unwrap();
    // Five Mk3 pumps want 100 kW between them (20 kW each, with room for water).
    let first_tick = |tier: u8| {
        let mut f = Factory::default();
        powered(&mut f, tier);
        for x in 0..5 {
            place_at(&mut f, PUMP, IVec3::new(x, 0, 0), 2);
        }
        run(&mut f, 1.5 / crate::TICK_RATE as f64, |_| {});
        let g = &f.generators[0];
        (f.power.demand[0], g.output, f.power.capacity[0], g.energy + g.output)
    };
    assert_eq!(first_tick(0), (100, 60, 60, coal * crate::TICK_RATE));
    assert_eq!(first_tick(1), (100, 100, 100, coal * 125 / 100 * crate::TICK_RATE));
}

#[test]
fn labs_work_at_one_two_and_three_times_the_speed_and_a_mk3_skips_every_fifth_pack() {
    use crate::item::RED_PACK;
    // Belt Routing: 5 s a unit at full power. 10 s and a bit is 603 ticks.
    let units = |tier: u8, packs: u32, ticks: f64| {
        let mut f = Factory::default();
        powered(&mut f, 0);
        place_at(&mut f, LAB, IVec3::ZERO, tier);
        assert_eq!(f.insert(IVec3::ZERO, RED_PACK, packs), packs);
        f.research.set_current(Some(0));
        run(&mut f, ticks / crate::TICK_RATE as f64, |_| {});
        (f.research.progress(0), f.lab_at(IVec3::ZERO).packs.total(), f.power.demand[0])
    };
    assert_eq!(units(0, 20, 603.5), (2, 17, 10), "Mk1: two units, a third started");
    assert_eq!(units(1, 20, 603.5), (4, 15, 20), "Mk2: 2.5 s a unit");
    assert_eq!(units(2, 20, 603.5), (6, 14, 30), "Mk3: six units for five packs and a free fifth");
    // Four packs are enough for five units at Mk3, and only four units at Mk1.
    assert_eq!(units(2, 4, 500.5).0, 5);
    assert_eq!(units(0, 4, 2000.0).0, 4);
    // The lab says so.
    let mut f = Factory::default();
    place_at(&mut f, LAB, IVec3::ZERO, 2);
    assert!(f.describe(IVec3::ZERO).unwrap().contains("×3 speed · needs 30 kW · every 5th unit is free"));
}

#[test]
fn mk4_pins_its_numbers_for_every_family() {
    use crate::factory::lab::LAB_TIERS;
    use crate::factory::pole::POLE_TIERS;
    assert_eq!(BELT_TIERS[3].speed, 8.0);
    let m = &MINER_TIERS[3];
    assert_eq!((m.rate, m.recovery, m.power), (6.0, 0.92, 90));
    assert_eq!((POLE_TIERS[3].link, POLE_TIERS[3].reach), (32, 16));
    let l = &LAB_TIERS[3];
    assert_eq!((l.speed, l.power, l.free_every), (4, 40, 3));
    // Processors: five times a Mk1, and their power grows alike.
    let tier4 = |block| {
        let t = &crate::factory::process::spec(block).unwrap().tiers[3];
        (t.speed, t.power)
    };
    assert_eq!(tier4(SMELTER), (5000, 80));
    assert_eq!(tier4(CONSTRUCTOR), (5000, 75));
    assert_eq!(tier4(ASSEMBLER), (5000, 100));
    assert_eq!(tier4(BLAST_FURNACE), (5000, 0));
}

#[test]
fn a_mk4_belt_carries_about_twenty_items_a_second() {
    let mut f = Factory::default();
    f.add_storage(IVec3::ZERO);
    f.stock(IVec3::ZERO, IRON_ORE.into(), 1000);
    for x in 1..=4 {
        let p = IVec3::new(x, 0, 0);
        f.place(&mut World::new(1, 2), BELT, p, 1, p, 3);
    }
    f.add_storage(IVec3::new(5, 0, 0));
    run(&mut f, 5.0, |_| {});
    let before = f.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into());
    run(&mut f, 5.0, |_| {});
    let n = f.storage_count_at(IVec3::new(5, 0, 0), IRON_ORE.into()) - before;
    // A gap of 0.35 blocks at 8 blocks a second is 2.6 ticks, so 20 a second at 60 ticks a second.
    assert!((97..=103).contains(&n), "Mk4 {n}");
}

#[test]
fn a_mk4_lab_works_four_times_as_fast_and_every_third_unit_is_free() {
    use crate::item::RED_PACK;
    let mut f = Factory::default();
    powered(&mut f, 0);
    place_at(&mut f, LAB, IVec3::ZERO, 3);
    assert_eq!(f.insert(IVec3::ZERO, RED_PACK, 20), 20);
    f.research.set_current(Some(0));
    // 5 s a unit at full power is 1.25 s at Mk4: eight units in 10 s, two of them free (6 packs), and the ninth,\n    // a free one too, is under way.
    run(&mut f, 603.5 / crate::TICK_RATE as f64, |_| {});
    assert_eq!((f.research.progress(0), f.lab_at(IVec3::ZERO).packs.total(), f.power.demand[0]), (8, 14, 40));
    assert!(f.describe(IVec3::ZERO).unwrap().contains("×4 speed · needs 40 kW · every 3rd unit is free"));
}

#[test]
fn a_substation_reaches_sixteen_blocks_and_links_like_a_pylon() {
    let mut f = Factory::default();
    place_at(&mut f, POLE, IVec3::ZERO, 3);
    place_at(&mut f, POLE, IVec3::new(32, 0, 0), 2);
    place_at(&mut f, MINER, IVec3::new(-16, 0, 0), 0);
    run(&mut f, 0.1, |_| {});
    assert_eq!(f.power.pole_grid, [0, 0], "32 blocks link");
    assert_eq!(f.power.miner_pole, [Some(0)], "a machine 16 blocks away hangs on it");
}

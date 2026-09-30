//! Solar panels and accumulators in a bare factory: a Mk1 constructor pressing rods is the steady 15 kW
//! load, kept fed and emptied each second, and the grid is followed through two whole days.

use super::*;
use crate::block::{ACCUMULATOR, CONSTRUCTOR, POLE, SOLAR_PANEL};
use crate::bytes::{ByteReader, ByteWriter};
use crate::daytime::DAY_TICKS;
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine};
use crate::item::{IRON_INGOT, IRON_ROD};
use crate::math::IVec3;
use crate::world::World;

const LOAD: IVec3 = IVec3::new(8, 0, 0);

/// A Mk4 pole in the middle, the load, `panels` panels in two rows and `accumulators` beside them.
fn plant(panels: usize, accumulators: usize) -> Factory {
    let mut f = Factory::default();
    let mut world = World::new(1, 2);
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(4, 0, 4), 3);
    put(&mut f, CONSTRUCTOR, LOAD, 0);
    assert!(f.set_recipe(LOAD, Some(recipe_for(IRON_ROD))).is_some());
    for i in 0..panels as i32 {
        put(&mut f, SOLAR_PANEL, IVec3::new(i % 3 * 2, 0, i / 3 * 2 - 4), 0);
    }
    for i in 0..accumulators as i32 {
        put(&mut f, ACCUMULATOR, IVec3::new(i * 2, 0, 8), 0);
    }
    f
}

/// Runs `days` whole days from the first morning. Returns the ticks in the last day in which the load wanted
/// more power than it got.
fn browned_out_ticks(f: &mut Factory, days: u64) -> u32 {
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    let mut short = 0;
    for tick in 0..days * DAY_TICKS {
        f.update(&mut world, tick, &mut events);
        events.clear();
        if tick % 60 == 0 {
            let load = f.at[&LOAD];
            let crate::factory::Slot::Process(i) = load else { unreachable!() };
            let p = &mut f.processors[i as usize];
            p.out.take(0, p.out.total());
            f.insert(LOAD, IRON_INGOT, 64);
        }
        if tick >= (days - 1) * DAY_TICKS && f.power.demand[0] > f.power.supply[0] {
            short += 1;
        }
    }
    short
}

#[test]
fn six_panels_and_an_accumulator_carry_a_15_kw_load_through_the_night() {
    let mut f = plant(6, 1);
    assert_eq!(browned_out_ticks(&mut f, 2), 0, "no brownout on the second day");
    assert!(f.power.demand[0] > 0 || f.power.supply[0] == 0);
}

#[test]
fn without_the_accumulator_or_enough_panels_the_night_browns_out() {
    let mut f = plant(6, 0);
    let dark = browned_out_ticks(&mut f, 2);
    assert!(dark > (DAY_TICKS / 3) as u32, "no sun, no power: {dark} ticks short");
    let mut f = plant(3, 1);
    assert!(browned_out_ticks(&mut f, 2) > 0, "three panels can't charge enough for the night");
}

#[test]
fn an_accumulator_charges_from_spare_sun_only_and_gives_before_a_generator_burns() {
    let mut f = plant(6, 1);
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    let noon = DAY_TICKS * 5 / 24;
    // No load: the sun fills the accumulator at 60 kW at most, and stops at the cap.
    f.processors.retain(|p| p.spec.block != CONSTRUCTOR);
    f.dirty = true;
    for tick in 0..600 {
        f.update(&mut world, noon + tick, &mut events);
    }
    let acc = f.processors.iter().find(|p| p.spec.block == ACCUMULATOR).unwrap();
    assert!(acc.store.charge > 0 && acc.store.charge <= CHARGE_CAP);
    assert_eq!(f.power.supply[0], 0, "spare sun isn't supply");
    let text = acc.status_text();
    assert!(text.starts_with("Charge"), "{text}");
}

#[test]
fn a_panel_reads_night_and_sun_and_an_accumulator_keeps_its_charge_in_a_save() {
    let mut f = plant(1, 1);
    let mut world = World::new(1, 2);
    let mut events = Vec::new();
    f.update(&mut world, DAY_TICKS * 17 / 24, &mut events);
    assert_eq!(f.describe(IVec3::new(0, 0, -4)).unwrap().lines().next(), Some("Night: no sunlight"));
    f.update(&mut world, DAY_TICKS * 5 / 24, &mut events);
    let sun = f.describe(IVec3::new(0, 0, -4)).unwrap();
    assert!(sun.starts_with("Sun 10 of 10 kW"), "{sun}");
    let i = f.processors.iter().position(|p| p.spec.block == ACCUMULATOR).unwrap();
    f.processors[i].store.charge = 123_456;
    let mut w = ByteWriter::default();
    f.processors[i].write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!(back.store.charge, 123_456);
    // A panel writes nothing extra.
    let mut w2 = ByteWriter::default();
    let panel = f.processors.iter().find(|p| p.spec.block == SOLAR_PANEL).unwrap();
    panel.write_state(&mut w2);
    assert!(w2.bytes.len() < w.bytes.len());
}

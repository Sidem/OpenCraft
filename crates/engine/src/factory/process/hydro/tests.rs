//! The water wheel in a bare factory high above the ground (so only the water the tests place is around it):
//! falling water on one side drives it and spins it in the matching direction, still/horizontal water or falling
//! water on both sides stalls it, it gives only what the grid lacks, renders water-in-buckets & splashes when wet,
//! and saves nothing of its own.

use super::*;
use crate::block::{flow, AIR, CONSTRUCTOR, FLOW_7, POLE, WATER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::process::draw_model;
use crate::factory::render::INSTANCE_FLOATS;
use crate::factory::tests::recipe_for;
use crate::factory::{Factory, Machine};
use crate::item::{IRON_INGOT, IRON_ROD};
use crate::math::Vec3;

const WHEEL: IVec3 = IVec3::new(0, 100, 0);
const LOAD: IVec3 = IVec3::new(8, 100, 0);

/// A wheel (facing 0: right = +X, front = +Z, upright 3×1×3), a pole and a 15 kW load.
fn plant() -> (Factory, World) {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    let mut put = |f: &mut Factory, block, pos: IVec3, tier| {
        f.place(&mut world, block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
    };
    put(&mut f, POLE, IVec3::new(4, 100, 4), 3);
    put(&mut f, WATER_WHEEL, WHEEL, 0);
    put(&mut f, CONSTRUCTOR, LOAD, 0);
    assert!(f.set_recipe(LOAD, Some(recipe_for(IRON_ROD))).is_some());
    f.insert(LOAD, IRON_INGOT, 64);
    (f, world)
}

/// Places a vertical waterfall column of `height` (1..=3) falling cells at column `(x, z)` next to the wheel:
/// `FLOW_7` for `y = 0..height` topped by a `WATER` source at `y = height`.
fn pour_column(world: &mut World, x: i32, z: i32, height: i32) {
    for y in 0..height {
        world.set_block_anywhere(WHEEL + IVec3::new(x, y, z), FLOW_7);
    }
    world.set_block_anywhere(WHEEL + IVec3::new(x, height, z), WATER);
}

fn clear_column(world: &mut World, x: i32, z: i32) {
    for y in 0..=3 {
        world.set_block_anywhere(WHEEL + IVec3::new(x, y, z), AIR);
    }
}

fn tick(f: &mut Factory, world: &mut World, n: u64) {
    let mut events = Vec::new();
    for t in 0..n {
        f.update(world, t, &mut events);
    }
}

fn wheel(f: &Factory) -> &Processor {
    f.processors.iter().find(|p| p.energy() == Energy::Hydro).unwrap()
}

#[test]
fn a_dry_wheel_gives_nothing_and_does_not_spin() {
    let (mut f, mut world) = plant();
    tick(&mut f, &mut world, 5);
    assert_eq!((wheel(&f).store.flow, f.power.supply[0]), (0, 0));
    assert_eq!(wheel(&f).store.wheel_spin, 0.0);
    assert!(wheel(&f).status_text().starts_with("Dry"), "{}", wheel(&f).status_text());
}

#[test]
fn still_and_horizontal_water_do_not_run_the_wheel() {
    let (mut f, mut world) = plant();
    // Still lake water along the left side (3 deep).
    for y in 0..=3 {
        world.set_block_anywhere(WHEEL + IVec3::new(-1, y, 0), WATER);
    }
    tick(&mut f, &mut world, 5);
    assert_eq!((wheel(&f).store.flow, wheel(&f).store.wheel_spin), (0, 0.0), "still water never drives it");

    // Flat horizontal flowing water with air above it.
    clear_column(&mut world, -1, 0);
    world.set_block_anywhere(WHEEL + IVec3::new(-1, 0, 0), flow(5));
    tick(&mut f, &mut world, 5);
    assert_eq!((wheel(&f).store.flow, wheel(&f).store.wheel_spin), (0, 0.0), "flat flow never drives it");

    // Falling water on the front axle face (+Z) does not push the paddles.
    pour_column(&mut world, 1, 1, 3);
    tick(&mut f, &mut world, 5);
    assert_eq!((wheel(&f).store.flow, wheel(&f).store.wheel_spin), (0, 0.0), "front face water never drives it");
}

#[test]
fn falling_water_on_one_side_drives_power_and_spin_direction() {
    let (mut f, mut world) = plant();

    // Left side (-1): 1, 2, 3 falling cells -> 16, 32, 48 kW and positive (left-down) spin.
    pour_column(&mut world, -1, 0, 1);
    tick(&mut f, &mut world, 10);
    assert_eq!(wheel(&f).store.flow, FALL_KW);
    assert!(wheel(&f).store.wheel_spin > 0.0, "left waterfall spins left-down (+)");

    pour_column(&mut world, -1, 0, 2);
    tick(&mut f, &mut world, 10);
    assert_eq!(wheel(&f).store.flow, 2 * FALL_KW);

    pour_column(&mut world, -1, 0, 3);
    tick(&mut f, &mut world, 20);
    assert_eq!(wheel(&f).store.flow, MAX_KW);
    assert!(wheel(&f).store.wheel_angle > 0.0);

    // Switch to right side (+3): spins in the opposite (right-down, -) direction.
    clear_column(&mut world, -1, 0);
    pour_column(&mut world, 3, 0, 3);
    tick(&mut f, &mut world, 80);
    assert_eq!(wheel(&f).store.flow, MAX_KW);
    assert!(wheel(&f).store.wheel_spin < 0.0, "right waterfall spins right-down (-)");
}

#[test]
fn falling_water_on_both_sides_cancels_out_and_stalls() {
    let (mut f, mut world) = plant();
    pour_column(&mut world, -1, 0, 3);
    pour_column(&mut world, 3, 0, 3);
    tick(&mut f, &mut world, 10);
    assert_eq!((wheel(&f).store.flow, f.power.supply[0]), (0, 0));
    assert_eq!(wheel(&f).store.wheel_spin, 0.0);
    assert!(wheel(&f).status_text().starts_with("Stalled"), "{}", wheel(&f).status_text());
}

#[test]
fn it_gives_only_what_the_grid_lacks_and_follows_the_water_each_tick() {
    let (mut f, mut world) = plant();
    pour_column(&mut world, -1, 0, 3);
    tick(&mut f, &mut world, 5);
    assert_eq!(f.power.demand[0], 15);
    assert_eq!((f.power.supply[0], wheel(&f).store.given, f.power.capacity[0]), (15, 15, MAX_KW));
    assert!(wheel(&f).status_text().contains("giving 15 kW"), "{}", wheel(&f).status_text());

    clear_column(&mut world, -1, 0);
    tick(&mut f, &mut world, 2);
    assert_eq!((f.power.supply[0], wheel(&f).store.flow), (0, 0), "the water was drained");
}

#[test]
fn visual_model_spins_and_adds_water_buckets_and_splashes_when_wet() {
    let (mut f, mut world) = plant();
    let mut dry_out = Vec::new();
    draw_model(wheel(&f), &mut dry_out, Vec3::ZERO, 0.0);
    let dry_count = dry_out.len() / INSTANCE_FLOATS;
    assert!(dry_count > 50, "wheel model should have rich circular geometry, got {dry_count}");

    pour_column(&mut world, -1, 0, 3);
    tick(&mut f, &mut world, 30);
    let mut wet_out = Vec::new();
    draw_model(wheel(&f), &mut wet_out, Vec3::ZERO, 0.25);
    let wet_count = wet_out.len() / INSTANCE_FLOATS;
    assert!(wet_count > dry_count, "wet wheel should emit bucket water fills & splashes ({wet_count} vs {dry_count})");
}

#[test]
fn a_wheel_writes_no_state_of_its_own() {
    let (f, _) = plant();
    let mut w = ByteWriter::default();
    wheel(&f).write_state(&mut w);
    let back = Processor::read_state(&mut ByteReader::new(&w.bytes)).unwrap();
    assert_eq!((back.spec.block, back.store.flow, back.store.charge), (WATER_WHEEL, 0, 0));
}

#[test]
fn two_wheels_sharing_one_waterfall_column_split_its_power() {
    let (mut f, mut world) = plant();
    // Place a second wheel on the far side of the left waterfall column at x = -1 (anchor at x = -4, so its right
    // paddle side is also x = -4 + 3 = -1).
    let second = WHEEL + IVec3::new(-4, 0, 0);
    f.place(&mut world, WATER_WHEEL, second, 0, second - IVec3::new(0, 1, 0), 0);
    pour_column(&mut world, -1, 0, 3);
    tick(&mut f, &mut world, 5);
    let flows: Vec<u32> = f.processors.iter().filter(|p| p.energy() == Energy::Hydro).map(|p| p.store.flow).collect();
    assert_eq!(flows, vec![15, 15], "two wheels sharing one 30 kW column split it 15/15");
}

#[test]
fn pumped_wheel_loses_net_energy_and_collapses_without_external_power() {
    use crate::action::Action;
    use crate::block::{BlockId, COAL_ORE, GENERATOR, OUTLET, PIPE, PUMP, STONE};
    use crate::sim::{PlayerId, Sim};

    const P: PlayerId = PlayerId(0);
    const BASE: IVec3 = IVec3::new(4200, 100, 4200);
    let mut sim = Sim::new(1337, 2);
    for x in -4..=8 {
        for z in -4..=4 {
            sim.world.set_block_anywhere(BASE + IVec3::new(x, -1, z), STONE);
            for y in 0..=8 {
                sim.world.set_block_anywhere(BASE + IVec3::new(x, y, z), AIR);
            }
        }
    }
    for x in -3..=-1 {
        for z in -2..=2 {
            sim.world.set_block_anywhere(BASE + IVec3::new(x, 0, z), WATER);
        }
    }
    let place = |sim: &mut Sim, b: BlockId, pos: IVec3, facing: u8| {
        sim.apply(P, Action::Give { item: b.into(), count: 1 });
        sim.apply(P, Action::PlaceBlock { pos, slot: 0, facing, against: pos - IVec3::new(0, 1, 0) });
    };
    // Water wheel at x = 2..4, y = 0..2, z = 0 (left side is x = 1).
    let wheel_pos = BASE + IVec3::new(2, 0, 0);
    place(&mut sim, WATER_WHEEL, wheel_pos, 0);
    // Mk1 pump at (-1, 0, 0) piped up +3 blocks to y = 3 -> lift power = 5 + 3*14 = 47 kW (> 30 kW wheel max!).
    place(&mut sim, PUMP, BASE + IVec3::new(-1, 0, 0), 0);
    for y in 1..=3 {
        place(&mut sim, PIPE, BASE + IVec3::new(-1, y, 0), 0);
    }
    place(&mut sim, OUTLET, BASE + IVec3::new(0, 3, 0), 1); // EAST -> pours into (1, 3, 0) -> falls down x = 1!
    place(&mut sim, POLE, BASE + IVec3::new(2, 0, 2), 0);
    let gen_pos = BASE + IVec3::new(3, 0, 2);
    place(&mut sim, GENERATOR, gen_pos, 0);
    assert_eq!(sim.factory.insert(gen_pos, COAL_ORE.into(), 8), 8);

    // Run 3 seconds with generator: wheel recovers real partial power, strictly less than the pump's 47 kW!
    for _ in 0..3 * crate::TICK_RATE {
        sim.step();
    }
    let pump_kw = sim.factory.pipework[0].power_kw();
    let w = sim.factory.processors.iter().find(|p| p.energy() == Energy::Hydro).unwrap();
    assert!(w.store.flow > 0, "falling water generates real partial recovery power");
    assert!(w.store.flow < pump_kw, "wheel ({}) strictly less than pump ({pump_kw}): net energy loss", w.store.flow);

    // Remove the generator: brownout spiral naturally collapses the loop to 0 kW — no perpetuum mobile!
    sim.apply(P, Action::BreakBlock { pos: gen_pos });
    for _ in 0..10 * crate::TICK_RATE {
        sim.step();
    }
    let w = sim.factory.processors.iter().find(|p| p.energy() == Energy::Hydro).unwrap();
    assert_eq!(w.store.flow, 0, "brownout spiral collapsed the loop to 0 kW");
    assert_eq!(sim.world.block_anywhere_or_generate(BASE + IVec3::new(1, 2, 0)), AIR, "falling stream dried up");
}

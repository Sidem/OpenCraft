//! The launch site: its pad and power, a rocket built and launched (the count, the empties coming back), what a pad missing a
//! part says, the rocket on screen only while it builds, saving, and the Rocketry tech that gates the parts.

use crate::block::LAUNCH_SITE;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::power::FULL_SPEED;
use crate::factory::{Factory, Machine};
use crate::item::{
    name, ItemId, AVIONICS, EMPTY_CANISTER, FUEL_TANK, HYDROGEN_CANISTER, LIGHT_FRAME, OXYGEN_CANISTER, ROCKET_ENGINE,
    ROCKET_FUEL_CANISTER,
};
use crate::math::{IVec3, Vec3};
use crate::recipes::{MACHINE_RECIPES, RECIPES, ROCKET_FUEL_RECIPE, ROCKET_PART_RECIPES, ROCKET_RECIPE};
use crate::research::TECHS;
use crate::world::World;

use super::super::refinery::PLANT_SPEC;
use super::super::{model, spec};
use super::*;

const ALL: [bool; MACHINE_RECIPES.len()] = [true; MACHINE_RECIPES.len()];
/// One rocket: 4 light frames, 2 engines, 2 fuel tanks, an avionics and 8 fuel canisters.
const ROCKET: [(ItemId, u32); 5] =
    [(LIGHT_FRAME, 4), (ROCKET_ENGINE, 2), (FUEL_TANK, 2), (AVIONICS, 1), (ROCKET_FUEL_CANISTER, 8)];

fn pad() -> Processor {
    Processor::new(IVec3::ZERO, &LAUNCH_SPEC, 0)
}

fn feed(p: &mut Processor, items: &[(ItemId, u32)]) {
    for &(item, n) in items {
        assert_eq!(p.insert(item, n, &ALL), n, "{} fits", name(item));
    }
}

fn run(p: &mut Processor, seconds: u32) {
    for _ in 0..seconds * crate::TICK_RATE {
        p.step(&mut [], FULL_SPEED, &ALL);
    }
}

fn boxes(p: &Processor) -> usize {
    let mut out = Vec::new();
    model::draw(p, &mut out, Vec3::ZERO, 0.0);
    out.len()
}

#[test]
fn a_pad_seven_blocks_across_with_hatches_at_the_back_and_front_and_two_megawatts() {
    assert_eq!(LAUNCH_SPEC.footprint.size, [7, 7, 6]);
    let faces = |role| LAUNCH_SPEC.footprint.faces(IVec3::ZERO, 0, role).len();
    assert_eq!((faces(Role::In), faces(Role::Out), faces(Role::Side)), (7, 7, 0));
    assert_eq!(pad().power(), 2000);
}

#[test]
fn a_full_load_builds_a_rocket_in_a_minute_and_launches_it() {
    let mut p = pad();
    feed(&mut p, &ROCKET);
    run(&mut p, 59);
    assert_eq!((p.launches, p.status), (0, Status::Working), "{}", p.status_text());
    assert!(p.status_text().starts_with("Building a rocket: 98%"), "{}", p.status_text());
    run(&mut p, 2);
    assert_eq!((p.launches, p.out.count(EMPTY_CANISTER), p.input.total()), (1, 8, 0));
    assert_eq!(p.readout().lines().filter(|l| *l == "1 in orbit").count(), 1);
}

#[test]
fn the_empties_must_leave_before_a_third_rocket_and_each_launch_counts() {
    let mut p = pad();
    for _ in 0..2 {
        feed(&mut p, &ROCKET);
        run(&mut p, 61);
    }
    assert_eq!((p.launches, p.out.count(EMPTY_CANISTER)), (2, 16));
    feed(&mut p, &ROCKET);
    run(&mut p, 1);
    assert_eq!(p.status, Status::OutputFull, "{}", p.status_text());
    p.out.take(0, 16);
    run(&mut p, 61);
    assert_eq!(p.launches, 3);
}

#[test]
fn a_pad_missing_a_part_says_which_and_does_not_start() {
    let mut p = pad();
    assert_eq!(p.status_text(), LAUNCH_SPEC.waiting, "an empty pad says what it takes");
    feed(&mut p, &ROCKET[..4]);
    feed(&mut p, &[(ROCKET_FUEL_CANISTER, 5)]);
    run(&mut p, 100);
    assert_eq!((p.launches, p.batch), (0, None));
    assert_eq!(p.status_text(), "Waiting for 3 Rocket Fuel Canister");
    feed(&mut p, &[(ROCKET_FUEL_CANISTER, 3)]);
    run(&mut p, 61);
    assert_eq!(p.launches, 1);
}

#[test]
fn the_rocket_stands_on_the_pad_only_while_it_builds() {
    let mut p = pad();
    let idle = boxes(&p);
    feed(&mut p, &ROCKET);
    run(&mut p, 10);
    assert!(boxes(&p) > idle, "a rocket is drawn while a batch runs");
    run(&mut p, 60);
    assert_eq!(boxes(&p), idle, "and gone once it has launched");
}

#[test]
fn a_launch_count_and_a_batch_in_progress_survive_a_save() {
    let mut p = pad();
    feed(&mut p, &ROCKET);
    run(&mut p, 61);
    feed(&mut p, &ROCKET);
    p.out.take(0, 8);
    run(&mut p, 20);
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let mut back = Processor::read_state(&mut r).unwrap();
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!((back.launches, back.batch, back.progress), (1, p.batch, p.progress));
    run(&mut back, 41);
    assert_eq!(back.launches, 2);
}

#[test]
fn the_shells_come_back_and_the_parts_come_from_the_right_machines() {
    let r = &MACHINE_RECIPES[ROCKET_RECIPE as usize];
    assert_eq!(r.inputs, &ROCKET[..]);
    assert_eq!(r.outputs, &[(EMPTY_CANISTER, 8)], "eight fuel canisters in, eight empties out");
    let fuel = &MACHINE_RECIPES[ROCKET_FUEL_RECIPE as usize];
    let shells = |l: &[(ItemId, u32)]| l.iter().map(|x| x.1).sum::<u32>();
    assert_eq!((shells(fuel.inputs), shells(fuel.outputs)), (3, 3));
    assert_eq!(fuel.inputs, &[(HYDROGEN_CANISTER, 2), (OXYGEN_CANISTER, 1)]);
    assert!(PLANT_SPEC.recipe(ROCKET_FUEL_RECIPE).is_some(), "the chemical plant mixes the fuel");
    assert!(LAUNCH_SPEC.recipe(ROCKET_RECIPE).is_some());
    let assembler = spec(crate::block::ASSEMBLER).expect("the assembler is a processor");
    assert!(ROCKET_PART_RECIPES.iter().all(|&i| assembler.recipe(i).is_some()), "the assembler builds the parts");
}

#[test]
fn the_launch_site_waits_for_rocketry() {
    let tech = TECHS.iter().position(|t| t.name == "Rocketry").unwrap() as u8;
    assert_eq!(tech, 78, "saved by index");
    for need in ["Electrolysis", "Accelerators"] {
        let n = TECHS.iter().position(|t| t.name == need).unwrap() as u8;
        assert!(TECHS[tech as usize].needs.contains(&n), "needs {need}");
    }
    assert!(RECIPES.iter().any(|r| r.output == ItemId::block(LAUNCH_SITE)));

    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, LAUNCH_SITE, IVec3::ZERO, 0, IVec3::ZERO, 0);
    assert_eq!(f.insert(IVec3::ZERO, LIGHT_FRAME, 4), 0, "no rocket recipe before Rocketry");
    (0..TECHS[tech as usize].units).for_each(|_| f.research.add_unit(tech));
    assert_eq!(f.insert(IVec3::ZERO, LIGHT_FRAME, 4), 4);
}

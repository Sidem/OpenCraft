//! The launch site: its pad and power, a rocket built and launched (the count, the empties coming back), what a pad missing a
//! part says, the rocket on screen only while it builds, saving, and the Rocketry tech that gates the parts.

use crate::block::LAUNCH_SITE;
use crate::block::{COAL_ORE, GENERATOR, POLE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::power::FULL_SPEED;
use crate::factory::{Factory, Machine};
use crate::inventory::Stack;
use crate::item::{
    name, ItemId, AI_ACCELERATOR, AVIONICS, EMPTY_CANISTER, FUEL_TANK, HYDROGEN_CANISTER, LIGHT_FRAME, OXYGEN_CANISTER,
    ROCKET_ENGINE, ROCKET_FUEL_CANISTER, SURVEY_SATELLITE,
};
use crate::math::{IVec3, Vec3};
use crate::recipes::{
    MACHINE_RECIPES, RECIPES, ROCKET_FUEL_RECIPE, ROCKET_PART_RECIPES, ROCKET_RECIPE, SURVEY_SATELLITE_RECIPE,
};
use crate::research::{Unlock, TECHS};
use crate::sim::SimEvent;
use crate::world::World;

use super::super::refinery::PLANT_SPEC;
use super::super::{model, spec};
use super::*;

const ALL: [bool; MACHINE_RECIPES.len()] = [true; MACHINE_RECIPES.len()];
/// One rocket: 4 light frames, 2 engines, 2 fuel tanks, an avionics, 8 fuel canisters and a survey satellite.
const ROCKET: [(ItemId, u32); 6] = [
    (LIGHT_FRAME, 4),
    (ROCKET_ENGINE, 2),
    (FUEL_TANK, 2),
    (AVIONICS, 1),
    (ROCKET_FUEL_CANISTER, 8),
    (SURVEY_SATELLITE, 1),
];

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
    assert_eq!(p.status_text(), "Waiting for 3 Rocket Fuel Canister, 1 Survey Satellite");
    feed(&mut p, &[(ROCKET_FUEL_CANISTER, 3), (SURVEY_SATELLITE, 1)]);
    run(&mut p, 61);
    assert_eq!(p.launches, 1);
}

#[test]
fn a_launch_hands_its_satellite_up_once() {
    let mut p = pad();
    feed(&mut p, &ROCKET);
    run(&mut p, 59);
    assert_eq!(p.lifted, None, "still on the pad");
    run(&mut p, 2);
    assert_eq!(p.lifted, Some(SURVEY_SATELLITE));
    assert_eq!(p.input.count(SURVEY_SATELLITE), 0, "the satellite went with the rocket");
}

/// A pad at the origin with a pole and a coal generator: too little power to build a rocket in a lifetime, but any share
/// finishes a batch that is one tick from done.
fn rig() -> Factory {
    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, LAUNCH_SITE, IVec3::ZERO, 0, IVec3::ZERO, 0);
    let (pole, gen) = (IVec3::new(8, 0, 0), IVec3::new(9, 0, 0));
    f.place(&mut world, POLE, pole, 0, pole, 0);
    f.place(&mut world, GENERATOR, gen, 0, gen, 0);
    assert_eq!(f.insert(gen, COAL_ORE.into(), 64), 64);
    f
}

#[test]
fn a_launched_satellite_is_in_the_factorys_orbit_and_the_pad_says_so() {
    let mut f = rig();
    let batch = Some(ROCKET_RECIPE);
    f.processors[0].batch = batch;
    f.processors[0].progress = 60 * crate::TICK_RATE * FULL_SPEED - 1;
    let (mut world, mut events) = (World::new(1, 2), Vec::new());
    assert!(!f.describe(IVec3::ZERO).unwrap().contains("satellites"), "nothing is up yet");
    for tick in 600..900 {
        f.update(&mut world, tick, &mut events);
    }
    assert_eq!(f.processors[0].launches, 1);
    let launched = events.iter().filter(|e| matches!(e, SimEvent::Launched { .. })).collect::<Vec<_>>();
    assert_eq!(launched, [&SimEvent::Launched { pos: IVec3::ZERO, item: SURVEY_SATELLITE }], "reported once");
    let sat = f.orbit.list();
    assert_eq!((sat.len(), sat[0].kind, sat[0].from), (1, 0, [0, 0]));
    assert!(
        f.describe(IVec3::ZERO).unwrap().contains("Survey satellites: 1 up"),
        "{}",
        f.describe(IVec3::ZERO).unwrap()
    );
    let before = f.orbit.list().to_vec();
    for tick in 900..1200 {
        f.update(&mut world, tick, &mut events);
    }
    assert_eq!(f.orbit.list(), &before[..], "one rocket, one satellite");
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
fn a_pad_saved_before_the_satellite_slot_still_loads() {
    let mut p = pad();
    feed(&mut p, &ROCKET[..5]);
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    // Save 44 wrote five input slots: cut the sixth (the empty satellite slot) out of the bytes. The header is a position
    // (12), block, tier, facing (3) and the recipe and batch (3 each).
    let mut slot = ByteWriter::default();
    Stack::default().write_state(&mut slot);
    let at = 12 + 3 + 6 + 5 * slot.bytes.len();
    let mut old = w.bytes.clone();
    old.drain(at..at + slot.bytes.len());
    let mut r = ByteReader::new(&old);
    r.version = 44;
    let back = Processor::read_state(&mut r).expect("a save 44 pad loads");
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!(back.input.slots.len(), 6);
    assert_eq!(back.input.contents(), p.input.contents());
    let mut new = ByteReader::new(&old);
    assert!(Processor::read_state(&mut new).is_none(), "the current layout does not read the old bytes");
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
fn an_assembler_builds_the_satellite_and_the_rocket_carries_it_last() {
    let sat = &MACHINE_RECIPES[SURVEY_SATELLITE_RECIPE as usize];
    assert_eq!(sat.outputs, &[(SURVEY_SATELLITE, 1)]);
    assert!(sat.inputs.iter().any(|&(i, _)| i == AI_ACCELERATOR), "an accelerator is its brain");
    assert!(spec(crate::block::ASSEMBLER).unwrap().recipe(SURVEY_SATELLITE_RECIPE).is_some());
    let payload = MACHINE_RECIPES[ROCKET_RECIPE as usize].inputs.last().unwrap().0;
    assert_eq!(crate::satellites::kind_of(payload), Some(0), "the last input of a launch is its satellite");
    let rocketry = TECHS.iter().find(|t| t.name == "Rocketry").unwrap();
    assert!(rocketry.unlocks.iter().any(|u| matches!(u, Unlock::MachineRecipe(i) if *i == SURVEY_SATELLITE_RECIPE)));
}

#[test]
fn the_comms_satellite_has_its_own_tech_assembler_row_and_launch() {
    let [build, launch] = crate::recipes::COMMS_SATELLITE_RECIPES;
    assert_eq!(MACHINE_RECIPES[build as usize].outputs, &[(crate::item::COMMS_SATELLITE, 1)]);
    assert!(spec(crate::block::ASSEMBLER).unwrap().recipe(build).is_some());
    assert!(LAUNCH_SPEC.recipe(launch).is_some());
    let payload = MACHINE_RECIPES[launch as usize].inputs.last().unwrap().0;
    assert_eq!(crate::satellites::kind_of(payload), Some(crate::satellites::COMMS));
    let tech = TECHS.iter().position(|t| t.name == "Orbital Comms").unwrap();
    assert_eq!(tech, 79, "saved by index");
    assert!(TECHS[tech].unlocks.iter().any(|u| matches!(u, Unlock::MachineRecipe(i) if *i == launch)));
    let rocketry = TECHS.iter().position(|t| t.name == "Rocketry").unwrap() as u8;
    assert!(TECHS[tech].needs.contains(&rocketry));
}

#[test]
fn the_power_satellite_has_its_own_tech_assembler_row_launch_and_rectenna() {
    let [build, launch] = crate::recipes::POWER_SATELLITE_RECIPES;
    assert_eq!(MACHINE_RECIPES[build as usize].outputs, &[(crate::item::POWER_SATELLITE, 1)]);
    assert!(spec(crate::block::ASSEMBLER).unwrap().recipe(build).is_some());
    assert!(LAUNCH_SPEC.recipe(launch).is_some());
    let payload = MACHINE_RECIPES[launch as usize].inputs.last().unwrap().0;
    assert_eq!(crate::satellites::kind_of(payload), Some(crate::satellites::POWER));
    let tech = TECHS.iter().position(|t| t.name == "Orbital Power").unwrap();
    assert_eq!(tech, 80, "saved by index");
    assert!(TECHS[tech].unlocks.iter().any(|u| matches!(u, Unlock::MachineRecipe(i) if *i == launch)));
    assert!(TECHS[tech].unlocks.iter().any(|u| matches!(u, Unlock::Recipe(_))), "the rectenna's hand recipe");
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

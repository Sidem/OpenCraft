//! The chip fab: its footprint and power, a wafer batch and an accelerator batch (shells conserved), one fab holding both
//! recipes' inputs, what a half-fed fab says it lacks, pure water in the chemical plant, and the techs that gate it all.

use crate::block::CHIP_FAB;
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::power::FULL_SPEED;
use crate::factory::{Factory, Machine};
use crate::item::{
    name, ItemId, ACID_CANISTER, AI_ACCELERATOR, EMPTY_CANISTER, PLASTIC, PROCESSOR, PURE_WATER_CANISTER, SILICON,
    WAFER,
};
use crate::math::IVec3;
use crate::recipes::{water_use, ACCELERATOR_RECIPE, MACHINE_RECIPES, PURE_WATER_RECIPE, RECIPES, WAFER_RECIPE};
use crate::research::{Unlock, TECHS};
use crate::world::World;

use super::super::refinery::PLANT_SPEC;
use super::*;

const ALL: [bool; MACHINE_RECIPES.len()] = [true; MACHINE_RECIPES.len()];

fn fab() -> Processor {
    Processor::new(IVec3::ZERO, &FAB_SPEC, 0)
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

#[test]
fn a_clean_room_with_hatches_on_three_sides_and_a_megawatt() {
    assert_eq!(FAB_SPEC.footprint.size, [4, 4, 3]);
    let faces = |role| FAB_SPEC.footprint.faces(IVec3::ZERO, 0, role).len();
    assert_eq!((faces(Role::In), faces(Role::Out), faces(Role::Side)), (4, 4, 4));
    assert_eq!(fab().power(), 1000);
}

#[test]
fn a_wafer_takes_silicon_acid_and_pure_water_and_gives_both_shells_back() {
    let mut p = fab();
    feed(&mut p, &[(SILICON, 2), (ACID_CANISTER, 1), (PURE_WATER_CANISTER, 1)]);
    run(&mut p, 7);
    assert_eq!((p.out.total(), p.status), (0, Status::Working), "8 seconds a wafer");
    run(&mut p, 2);
    assert_eq!((p.out.count(WAFER), p.side.count(EMPTY_CANISTER), p.input.total()), (1, 2, 0));
}

#[test]
fn an_accelerator_takes_two_wafers_two_processors_and_a_plastic() {
    let mut p = fab();
    feed(&mut p, &[(WAFER, 2), (PROCESSOR, 2), (PLASTIC, 1)]);
    run(&mut p, 19);
    assert_eq!(p.out.total(), 0, "20 seconds an accelerator");
    run(&mut p, 2);
    assert_eq!((p.out.count(AI_ACCELERATOR), p.input.total()), (1, 0));
}

#[test]
fn one_fab_holds_the_inputs_of_both_recipes_and_a_full_output_stops_the_second_batch() {
    let mut p = fab();
    feed(&mut p, &[(SILICON, 2), (ACID_CANISTER, 1), (PURE_WATER_CANISTER, 1)]);
    feed(&mut p, &[(WAFER, 2), (PROCESSOR, 2), (PLASTIC, 1)]);
    run(&mut p, 9);
    assert_eq!(p.out.count(WAFER), 1, "a wafer came first: its inputs were held first");
    assert_eq!(p.status, Status::OutputFull, "the accelerator waits for the wafer to leave: {}", p.status_text());
    p.out.take(0, 1);
    run(&mut p, 21);
    assert_eq!(p.out.count(AI_ACCELERATOR), 1);
}

#[test]
fn a_half_fed_fab_says_what_it_lacks() {
    let mut p = fab();
    assert_eq!(p.status_text(), FAB_SPEC.waiting, "an empty fab says what it takes");
    feed(&mut p, &[(SILICON, 2)]);
    run(&mut p, 1);
    assert_eq!(p.status_text(), "Waiting for 1 Acid Canister, 1 Pure Water Canister");
    feed(&mut p, &[(WAFER, 1), (PROCESSOR, 2)]);
    run(&mut p, 1);
    assert_eq!(p.status_text(), "Waiting for 1 Acid Canister, 1 Pure Water Canister", "a tie keeps the first recipe");
    feed(&mut p, &[(WAFER, 1)]);
    run(&mut p, 1);
    assert_eq!(p.status_text(), "Waiting for 1 Plastic");
}

#[test]
fn pure_water_is_a_chemical_plant_recipe_that_spends_two_units_of_water() {
    assert!(PLANT_SPEC.recipe(PURE_WATER_RECIPE).is_some());
    assert!(FAB_SPEC.recipe(PURE_WATER_RECIPE).is_none(), "the fab takes it canned");
    assert_eq!(water_use(PURE_WATER_RECIPE), 2);
    let r = &MACHINE_RECIPES[PURE_WATER_RECIPE as usize];
    assert_eq!((r.inputs[0], r.outputs), ((EMPTY_CANISTER, 1), &[(PURE_WATER_CANISTER, 1)][..]));
}

#[test]
fn a_wafer_gives_back_every_shell_it_took() {
    let r = &MACHINE_RECIPES[WAFER_RECIPE as usize];
    let shells = |list: &[(ItemId, u32)]| {
        let is_shell = |i: ItemId| [ACID_CANISTER, PURE_WATER_CANISTER, EMPTY_CANISTER].contains(&i);
        list.iter().filter(|x| is_shell(x.0)).map(|x| x.1).sum::<u32>()
    };
    assert_eq!((shells(r.inputs), shells(r.outputs)), (2, 2));
    assert_eq!(MACHINE_RECIPES[ACCELERATOR_RECIPE as usize].outputs, &[(AI_ACCELERATOR, 1)]);
}

#[test]
fn the_fab_waits_for_its_techs() {
    let tech = |name: &str| TECHS.iter().position(|t| t.name == name).unwrap() as u8;
    let (wafers, accelerators) = (tech("Wafers"), tech("Accelerators"));
    assert_eq!(wafers, 56, "saved by index");
    assert!(TECHS[accelerators as usize].needs.contains(&wafers));
    assert!(TECHS[wafers as usize].needs.contains(&tech("Gold Science")));
    assert!(RECIPES.iter().any(|r| r.output == ItemId::block(CHIP_FAB)));

    let mut world = World::new(1, 2);
    let mut f = Factory::default();
    f.place(&mut world, CHIP_FAB, IVec3::ZERO, 0, IVec3::ZERO, 0);
    assert_eq!(f.insert(IVec3::ZERO, SILICON, 2), 0, "no wafer recipe before Wafers");
    (0..TECHS[wafers as usize].units).for_each(|_| f.research.add_unit(wafers));
    assert_eq!(f.insert(IVec3::ZERO, SILICON, 2), 2);
    assert_eq!(f.insert(IVec3::ZERO, PROCESSOR, 2), 0, "no accelerator recipe before Accelerators");
    (0..TECHS[accelerators as usize].units).for_each(|_| f.research.add_unit(accelerators));
    assert_eq!(f.insert(IVec3::ZERO, PROCESSOR, 2), 2);
    assert!(f.research.locked_by(Unlock::MachineRecipe(ACCELERATOR_RECIPE)).is_none());
}

#[test]
fn a_batch_in_progress_survives_a_save() {
    let mut p = fab();
    feed(&mut p, &[(SILICON, 2), (ACID_CANISTER, 1), (PURE_WATER_CANISTER, 1), (PLASTIC, 1)]);
    run(&mut p, 3);
    let mut w = ByteWriter::default();
    p.write_state(&mut w);
    let mut r = ByteReader::new(&w.bytes);
    let mut back = Processor::read_state(&mut r).unwrap();
    assert_eq!(r.u8(), None, "every byte read");
    assert_eq!((back.batch, back.progress, back.input.count(PLASTIC)), (p.batch, p.progress, 1));
    run(&mut back, 6);
    assert_eq!(back.out.count(WAFER), 1);
}

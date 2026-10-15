use crate::block::{POLE, STORAGE, UPGRADER};
use crate::factory::tests::{powered, recipe_for, run};
use crate::factory::Factory;
use crate::item::{BLUE_KIT, BOX_MK3, GREEN_KIT, POLE_MK3, POLE_MK4, VIOLET_KIT};
use crate::math::IVec3;
use crate::research::Unlock;
use crate::world::World;

fn upgrader(f: &mut Factory, makes: crate::item::ItemId) -> IVec3 {
    powered(f);
    let pos = IVec3::new(0, 0, 0);
    f.place(&mut World::new(1, 2), UPGRADER, pos, 0, pos, 0);
    assert!(f.set_recipe(pos, Some(recipe_for(makes))).is_some());
    pos
}

#[test]
fn a_mk1_pole_and_three_kinds_of_kit_make_a_mk4_pole_in_one_machine() {
    let mut f = Factory::default();
    f.research.complete_all();
    let pos = upgrader(&mut f, POLE_MK4);
    for (item, n) in [(POLE.into(), 1), (GREEN_KIT, 1), (BLUE_KIT, 1), (VIOLET_KIT, 1)] {
        assert_eq!(f.insert(pos, item, n), n, "takes every input of the chosen tier");
    }
    run(&mut f, 13.0, |_| {});
    let p = &f.processors[0];
    assert_eq!(p.out.count(POLE_MK4), 1, "12 s for three steps");
    assert_eq!(p.input.total(), 0, "the kits are used up");
}

#[test]
fn it_takes_only_what_the_chosen_tier_uses() {
    let mut f = Factory::default();
    f.research.complete_all();
    let pos = upgrader(&mut f, BOX_MK3);
    assert_eq!(f.insert(pos, VIOLET_KIT, 4), 0, "a Mk3 box takes no violet kit");
    assert_eq!(f.insert(pos, STORAGE.into(), 1), 1);
    assert_eq!(f.insert(pos, GREEN_KIT, 8), 8, "four kits a step, green and blue");
    assert_eq!(f.insert(pos, BLUE_KIT, 8), 8);
    run(&mut f, 9.0, |_| {});
    assert_eq!(f.processors[0].out.count(BOX_MK3), 1);
    assert_eq!(f.processors[0].input.count(GREEN_KIT), 4, "the second box's kits wait");
}

#[test]
fn a_tier_is_locked_until_its_upgrade_is_researched() {
    // `Action::SetRecipe` refuses a recipe research still locks (`action/apply.rs`).
    let mut f = Factory::default();
    let row = Unlock::MachineRecipe(recipe_for(POLE_MK3));
    assert!(!f.research.has(row), "Mk3 poles wait for their tech");
    f.research.complete_all();
    assert!(f.research.has(row));
}

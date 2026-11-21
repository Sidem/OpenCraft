use std::collections::HashSet;

use super::{DYE, HUES};
use crate::block;
use crate::factory::process_spec;
use crate::item::{self, ItemId};
use crate::recipes::{b, water_use, Category, DYE_FIRST, DYE_PLANT_ROWS, DYE_ROWS, MACHINE_RECIPES, RECIPES};
use crate::research::{Research, Unlock, TECHS};

/// The pigment of hue `i`: the ids run on from blue (`item/later.rs`).
fn pigment(i: usize) -> ItemId {
    ItemId(item::PIGMENT_BLUE.0 + i as u16)
}

/// What the ground gives and the pumpjack fills: the only things a player does not make from something else.
fn raw() -> HashSet<ItemId> {
    let ground = [
        block::IRON_ORE,
        block::COPPER_ORE,
        block::COAL_ORE,
        block::LIMESTONE,
        block::SAND,
        block::STONE,
        block::QUARTZ_ORE,
    ];
    ground.iter().map(|&x| b(x)).chain([item::CRUDE_BARREL]).collect()
}

/// Everything reachable from `raw` by any hand recipe or machine row (research, machines and water ignored).
fn reachable() -> HashSet<ItemId> {
    let mut have = raw();
    loop {
        let before = have.len();
        for r in RECIPES {
            if r.inputs.iter().all(|i| have.contains(&i.0)) {
                have.insert(r.output);
            }
        }
        for r in MACHINE_RECIPES {
            if r.inputs.iter().all(|i| have.contains(&i.0)) {
                have.extend(r.outputs.iter().map(|o| o.0));
            }
        }
        if have.len() == before {
            return have;
        }
    }
}

#[test]
fn the_rows_are_where_saved_machines_expect_them() {
    assert_eq!(DYE.len(), DYE_ROWS as usize);
    let glass = crate::recipes::glass::STAINED_GLASS.len();
    let gas = crate::recipes::gas::GAS_MACHINES.len();
    assert_eq!(
        MACHINE_RECIPES.len(),
        DYE_FIRST as usize + DYE.len() + glass + gas,
        "only stained glass and gas rows follow the dye rows"
    );
    assert_eq!(MACHINE_RECIPES[DYE_FIRST as usize - 1].category, Category::Refit, "REFIT_LATER ends right before them");
    assert_eq!(DYE_ROWS, 2 * DYE_PLANT_ROWS);
    assert_eq!(HUES.len(), usize::from(DYE_PLANT_ROWS));
    for i in 0..HUES.len() {
        let (pigment, concrete) = (pigment(i), block::BLUE_CONCRETE + i as block::BlockId);
        let (plant, tint) = (&DYE[i], &DYE[usize::from(DYE_PLANT_ROWS) + i]);
        assert_eq!((plant.category, plant.outputs[0]), (Category::Chemistry, (pigment, 2)), "plant row {i}");
        assert_eq!(tint.category, Category::Assembly);
        assert_eq!(
            (tint.inputs, tint.outputs),
            (&[(b(block::CONCRETE), 4), (pigment, 1)][..], &[(b(concrete), 4)][..])
        );
        // The pigment, the concrete and the row agree on the hue's name ("Blue Pigment", "Blue Concrete").
        let word = |name: &str| name.split(' ').next().unwrap().to_string();
        assert_eq!(word(item::name(pigment)), word(item::name(b(concrete))), "hue {i}");
    }
}

#[test]
fn every_hue_is_made_from_raw_and_every_pigment_has_a_sink() {
    let have = reachable();
    let all_pigments =
        [item::PIGMENT_RED, item::PIGMENT_GREEN, item::PIGMENT_BLACK, item::PIGMENT_WHITE, item::PIGMENT_YELLOW];
    for p in all_pigments.into_iter().chain((0..HUES.len()).map(pigment)) {
        assert!(have.contains(&p), "{} cannot be made from raw", item::name(p));
        let used = MACHINE_RECIPES.iter().any(|r| r.inputs.iter().any(|i| i.0 == p))
            || RECIPES.iter().any(|r| r.inputs.iter().any(|i| i.0 == p));
        assert!(used, "{} is made and used nowhere", item::name(p));
    }
    for i in 0..HUES.len() {
        let concrete = b(block::BLUE_CONCRETE + i as block::BlockId);
        assert!(have.contains(&concrete), "{} cannot be made from raw", item::name(concrete));
    }
}

#[test]
fn acid_gains_its_second_sink_and_a_mix_keeps_the_ingredients_of_its_hue() {
    let acid_users = MACHINE_RECIPES.iter().filter(|r| r.inputs.iter().any(|i| i.0 == item::ACID_BARREL)).count();
    assert!(acid_users >= 2, "the chip fab's wafer and the blue pigment");
    assert_eq!(DYE[0].inputs, &[(item::COPPER_INGOT, 1), (item::ACID_BARREL, 1)][..]);
    assert_eq!((water_use(DYE_FIRST), water_use(DYE_FIRST + 1)), (1, 0), "only blue dissolves in water");
    let empties: u32 = DYE[0].outputs.iter().filter(|o| o.0 == item::EMPTY_BARREL).map(|o| o.1).sum();
    assert_eq!(empties, 1, "the acid barrel's shell comes back");
}

#[test]
fn rows_fit_a_stack_and_the_machine_that_takes_them() {
    for (i, r) in DYE.iter().enumerate() {
        assert!(r.inputs.iter().chain(r.outputs).all(|&(it, n)| n <= item::stack_size(it)), "row {i}");
        let machine = if r.category == Category::Chemistry { block::CHEMICAL_PLANT } else { block::ASSEMBLER };
        assert!(process_spec(machine).unwrap().categories.contains(&r.category));
        assert!(r.inputs.len() <= 3, "a machine holds three kinds");
    }
}

#[test]
fn the_pigments_tech_unlocks_every_row_and_nothing_earlier_does() {
    let tech = TECHS.iter().position(|t| t.name == "Pigments").unwrap() as u8;
    assert_eq!(tech, 89, "saved index");
    let mut research = Research::default();
    for row in DYE_FIRST..DYE_FIRST + DYE_ROWS {
        assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), Some(tech));
    }
    for t in TECHS[usize::from(tech)].needs.iter().copied().chain([tech]) {
        (0..TECHS[usize::from(t)].units).for_each(|_| research.add_unit(t));
    }
    for row in DYE_FIRST..DYE_FIRST + DYE_ROWS {
        assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), None);
    }
}

#[test]
fn every_dye_row_has_a_card_note_where_the_plant_makes_it() {
    for row in DYE_FIRST..DYE_FIRST + DYE_PLANT_ROWS {
        assert!(!crate::recipes::process_notes::note(usize::from(row)).is_empty(), "row {row}");
    }
}

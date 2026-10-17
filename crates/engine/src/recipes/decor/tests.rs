use super::{b, DECOR};
use crate::block::{self, BLOCK_COUNT};
use crate::factory::process_spec;
use crate::item;
use crate::recipes::{Category, DECOR_FIRST, DECOR_ROWS, MACHINE_RECIPES};
use crate::research::{Research, Unlock, TECHS};

const FIRST_DECOR_BLOCK: block::BlockId = block::RED_CONCRETE;

fn decor_blocks() -> impl Iterator<Item = block::BlockId> {
    FIRST_DECOR_BLOCK..BLOCK_COUNT as block::BlockId
}

#[test]
fn the_decor_rows_are_the_last_of_the_table_where_research_says() {
    assert_eq!(DECOR.len(), DECOR_ROWS as usize);
    assert_eq!(MACHINE_RECIPES.len(), DECOR_FIRST as usize + DECOR.len());
    assert_eq!(MACHINE_RECIPES[DECOR_FIRST as usize - 1].inputs, &[(item::SCRAP_METAL, 3)][..], "the salvage row");
    assert_eq!(block::BLOCK_COUNT, FIRST_DECOR_BLOCK as usize + block::DECOR_COUNT);
}

#[test]
fn every_decor_block_has_one_machine_row_and_no_hand_recipe() {
    for id in decor_blocks() {
        let def = block::def(id);
        assert!(def.placeable && def.drop == id && def.break_time > 0.0, "{} is a plain block", def.name);
        let rows = DECOR.iter().filter(|r| r.outputs[0].0 == b(id)).count();
        assert_eq!(rows, 1, "{} is made by exactly one machine row", def.name);
        assert!(!crate::recipes::RECIPES.iter().any(|r| r.output == b(id)), "{} is machine-made only", def.name);
    }
}

#[test]
fn rows_fit_a_stack_and_the_machine_that_takes_them() {
    for r in DECOR {
        assert!(r.inputs.iter().chain(r.outputs).all(|&(it, n)| n <= item::stack_size(it)), "{:?}", r.outputs);
        let machine = if r.category == Category::Smelting { block::SMELTER } else { block::ASSEMBLER };
        let spec = process_spec(machine).unwrap();
        assert!(spec.categories.contains(&r.category));
        assert!(r.inputs.len() <= 3, "an assembler holds three kinds");
    }
}

#[test]
fn colours_come_from_concrete_and_a_mineral() {
    let concrete = b(block::CONCRETE);
    let tinted: Vec<_> = DECOR.iter().filter(|r| r.inputs[0].0 == concrete).collect();
    assert_eq!(tinted.len(), 5, "red, green, black, white, yellow");
    for r in tinted {
        assert_eq!((r.inputs[0], r.inputs.len(), r.outputs[0].1), ((concrete, 4), 2, 4), "one mineral, four blocks");
    }
}

#[test]
fn the_shapes_behave() {
    let solid = |id| block::def(id).solid;
    assert!(solid(block::FRAMED_GLASS) && solid(block::CATWALK) && solid(block::METAL_PANEL));
    assert!(!solid(block::RAILING), "a railing is walked through");
    assert_eq!(block::def(block::PANEL_LIGHT).light, crate::light::TORCH_LIGHT);
    assert_eq!(block::def(block::FRAMED_GLASS).render, block::Render::Cutout);
    assert_eq!(block::def(block::RAILING).render, block::Render::Cutout);
    assert_eq!(block::def(block::RED_CONCRETE).render, block::Render::Opaque);
}

#[test]
fn architecture_unlocks_every_row_and_nothing_earlier_does() {
    let mut research = Research::default();
    let tech = TECHS.iter().position(|t| t.name == "Architecture").unwrap() as u8;
    assert_eq!(tech, 85, "saved index");
    for row in DECOR_FIRST..DECOR_FIRST + DECOR_ROWS {
        assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), Some(tech));
    }
    for prerequisite in TECHS[tech as usize].needs.iter().copied().chain([tech]) {
        (0..TECHS[prerequisite as usize].units).for_each(|_| research.add_unit(prerequisite));
    }
    for row in DECOR_FIRST..DECOR_FIRST + DECOR_ROWS {
        assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), None);
    }
}

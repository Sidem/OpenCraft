//! Content lint for the recipe tables: every item has a source and a use (or is an end product), every
//! category has a machine, outputs fit their machines, every tier has a recipe. Each lint takes the
//! tables as arguments, so a planted mistake shows it catches its kind.

use super::*;
use crate::block::{self, BLOCK_COUNT};
use crate::factory::tiers::{Family, FAMILIES};
use crate::factory::{Kind, ProcessSpec, SPECS};
use crate::item;
use crate::research::pack_slot;
use crate::tools;
use Category::{Arc, Assembly, Blasting, Crushing, Electrolysis, Pressing, Smelting};

/// Every category (the match below stops compiling until a new one is listed).
const CATEGORIES: [Category; 7] = [Smelting, Pressing, Assembly, Blasting, Crushing, Arc, Electrolysis];
const _: fn(Category) = |c| match c {
    Smelting | Pressing | Assembly | Blasting | Crushing | Arc | Electrolysis => {}
};

/// Blocks the world has (generated, or left by worked-out deposits): breaking them is how their drops
/// are first had.
const WORLD_BLOCKS: &[BlockId] = &[
    SPENT_ROCK,
    STONE,
    DIRT,
    GRASS,
    SAND,
    LOG,
    LEAVES,
    COAL_ORE,
    IRON_ORE,
    COPPER_ORE,
    GRANITE,
    SANDSTONE,
    BASALT,
    LIMESTONE,
    QUARTZ_ORE,
    BAUXITE_ORE,
    RUSTY_SOIL,
    DARK_SOIL,
    GREEN_SOIL,
    PALE_SOIL,
    RUSTY_SAND,
    DARK_SAND,
    GREEN_SAND,
    PALE_SAND,
];
/// Items the world gives other than block drops (leaves drop saplings: `action.rs`).
const GATHERED: &[ItemId] = &[ItemId::block(SAPLING)];
/// Known exceptions: items whose use waits for a later step (circuits: the violet pack, logic and drones;
/// batteries: the hover pack and cargo drones, Milestone 9 steps 9.5).
const NO_USE_YET: &[ItemId] = &[item::CIRCUIT, item::DRONE];

/// Every item a player can hold: what breakable blocks drop, and the non-block items.
fn items() -> Vec<ItemId> {
    let dropped =
        |b: BlockId| (0..BLOCK_COUNT as BlockId).any(|o| block::def(o).drop == b && block::def(o).break_time >= 0.0);
    let blocks = (1..BLOCK_COUNT as BlockId).filter(|&b| dropped(b)).map(ItemId::block);
    let extra = (256..).map(ItemId).take_while(|&i| item::def(i).is_some());
    blocks.chain(extra).collect()
}

fn lint_items(hand: &[Recipe], machine: &[MachineRecipe]) -> Vec<String> {
    let mut errors = Vec::new();
    for it in items() {
        let made = hand.iter().any(|r| r.output == it) || machine.iter().any(|r| r.outputs.iter().any(|o| o.0 == it));
        let gathered = GATHERED.contains(&it) || WORLD_BLOCKS.iter().any(|&w| ItemId::block(block::def(w).drop) == it);
        if !made && !gathered {
            errors.push(format!("{} has no source", item::name(it)));
        }
        let input = |ins: &[(ItemId, u32)]| ins.iter().any(|i| i.0 == it);
        let used = hand.iter().any(|r| input(r.inputs))
            || machine.iter().any(|r| input(r.inputs))
            || burn_time(it).is_some()
            || pack_slot(it).is_some();
        let end = it.places().is_some()
            || it == item::CARGO_DRONE
            || it == item::LOCOMOTIVE
            || it == item::WAGON
            || it == item::RAIL_SIGNAL
            || tools::tool(it).is_some()
            || tools::device(it).is_some()
            || crate::equipment::gear(it).is_some();
        if !used && !end && !NO_USE_YET.contains(&it) {
            errors.push(format!("{} has no use", item::name(it)));
        }
    }
    errors
}

fn lint_categories(specs: &[ProcessSpec]) -> Vec<String> {
    let mut errors: Vec<String> = CATEGORIES
        .iter()
        .filter(|&&c| !specs.iter().any(|s| s.categories.contains(&c)))
        .map(|c| format!("{c:?} has no machine"))
        .collect();
    let processor = |b: BlockId| crate::factory::machine(b).is_some_and(|m| m.kind == Kind::Process);
    for s in specs.iter().filter(|s| !processor(s.block)) {
        errors.push(format!("{} takes recipes but is no processor", block::def(s.block).name));
    }
    errors
}

/// A recipe's inputs and outputs fit the buffers of every machine that makes it.
fn lint_outputs(machine: &[MachineRecipe], specs: &[ProcessSpec]) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, r) in machine.iter().enumerate() {
        for s in specs.iter().filter(|s| s.categories.contains(&r.category)) {
            let [ins, _, outs] = s.buffers;
            let outs = if s.side > 0 { outs + s.side } else { outs };
            if r.outputs.is_empty() || r.outputs.len() > outs || r.inputs.len() > ins {
                let name = block::def(s.block).name;
                errors.push(format!("machine recipe {i} has {} outputs; {name} holds {outs}", r.outputs.len()));
            }
        }
    }
    errors
}

/// Every tier of a family can be made: Mk1 has a recipe, and each higher tier has a kit and the recipe
/// "the tier below plus the family's kits" (the same step an upgrade in place takes).
fn lint_tiers(families: &[Family], hand: &[Recipe]) -> Vec<String> {
    let mut errors = Vec::new();
    for f in families {
        for (t, &it) in f.items.iter().enumerate() {
            let want = match (t, crate::factory::upgrades::kit(t as u8)) {
                (0, _) => None,
                (_, Some(kit)) => Some([(f.items[t - 1], 1), (kit, f.kits)]),
                (_, None) => {
                    errors.push(format!("tier {t} of block {} has no kit", f.block));
                    continue;
                }
            };
            let fits = |r: &Recipe| r.output == it && want.is_none_or(|w| r.inputs == w && r.count == 1);
            if !hand.iter().any(fits) {
                errors.push(format!("tier item {} has no recipe", it.0));
            }
        }
    }
    errors
}

#[test]
fn todays_content_passes_the_lint() {
    let mut errors = lint_items(RECIPES, MACHINE_RECIPES);
    errors.extend(lint_categories(SPECS));
    errors.extend(lint_outputs(MACHINE_RECIPES, SPECS));
    errors.extend(lint_tiers(FAMILIES, RECIPES));
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn the_lint_catches_a_planted_mistake_of_each_kind() {
    let has = |errors: Vec<String>, what: &str| errors.iter().any(|e| e == what);
    // An item nothing makes, and one nothing uses.
    let no_sticks: Vec<Recipe> = RECIPES.iter().filter(|r| r.output != item::STICK).map(copy).collect();
    assert!(has(lint_items(&no_sticks, MACHINE_RECIPES), "Stick has no source"));
    let no_sticks_used: Vec<Recipe> =
        RECIPES.iter().filter(|r| !r.inputs.iter().any(|i| i.0 == item::STICK)).map(copy).collect();
    assert!(has(lint_items(&no_sticks_used, MACHINE_RECIPES), "Stick has no use"));
    // A category no machine takes, and a spec for a block that isn't a processor.
    assert!(has(lint_categories(&SPECS[..1]), "Pressing has no machine"));
    const STONY: [ProcessSpec; 1] = [ProcessSpec { block: STONE, ..SPECS[0] }];
    assert!(has(lint_categories(&STONY), "Stone takes recipes but is no processor"));
    // More outputs than the smelter holds.
    const TWO: [MachineRecipe; 1] = [MachineRecipe {
        category: Smelting,
        inputs: &[],
        outputs: &[(item::IRON_INGOT, 1), (b(GLASS), 1)],
        seconds: 1.0,
    }];
    assert!(has(lint_outputs(&TWO, SPECS), "machine recipe 0 has 2 outputs; Smelter holds 1"));
    // A tier nobody can make.
    const FAMILY: [Family; 1] = [Family { block: BELT, items: &[b(BELT), b(BEDROCK)], kits: 1 }];
    assert!(has(lint_tiers(&FAMILY, RECIPES), "tier item 10 has no recipe"));
    // A tier recipe that doesn't match the kit step (the belt family takes 1 kit, not 4).
    const COSTLY: [Family; 1] = [Family { block: BELT, items: &[b(BELT), b(FAST_BELT)], kits: 4 }];
    assert!(has(lint_tiers(&COSTLY, RECIPES), "tier item 28 has no recipe"));
    // A tier with no kit yet.
    const HIGH: [Family; 1] =
        [Family { block: BELT, items: &[b(BELT), b(FAST_BELT), item::BELT_MK3, item::BELT_MK4, b(STONE)], kits: 1 }];
    assert!(has(lint_tiers(&HIGH, RECIPES), "tier 4 of block 12 has no kit"));
}

fn copy(r: &Recipe) -> Recipe {
    Recipe { output: r.output, group: r.group, count: r.count, inputs: r.inputs, blurb: r.blurb }
}

/// Metal ores and quartz become ingots in a furnace first: nothing built by hand is made of raw ore.
#[test]
fn no_hand_recipe_is_made_of_raw_ore() {
    let raw = [b(IRON_ORE), b(COPPER_ORE), b(QUARTZ_ORE)];
    for r in RECIPES {
        assert!(r.inputs.iter().all(|i| !raw.contains(&i.0)), "{} is made of raw ore", item::name(r.output));
    }
}

/// The way in: stone makes a furnace, the furnace makes ingots, ingots make plates, rods and wire, and those a miner.
#[test]
fn the_first_machines_come_from_stone_by_way_of_ingots() {
    let made = |o: ItemId| RECIPES.iter().find(|r| r.output == o).unwrap();
    assert!(made(b(SMELTER)).inputs.iter().all(|i| i.0 == b(STONE)));
    for part in [IRON_PLATE, IRON_ROD, COPPER_WIRE] {
        assert!(made(part).inputs.iter().all(|i| i.0 == IRON_INGOT || i.0 == COPPER_INGOT), "{}", item::name(part));
    }
    assert!(made(b(MINER)).inputs.iter().all(|i| [IRON_PLATE, IRON_ROD, COPPER_WIRE].contains(&i.0)));
    assert!(RECIPES.iter().all(|r| r.hand_ticks() >= BASE_TICKS && r.hand_ticks() <= MAX_TICKS));
}

/// Green packs need belts, so an assembler must be able to make them from parts a constructor makes.
#[test]
fn belts_are_machine_made_for_green_packs() {
    let belts = MACHINE_RECIPES.iter().position(|m| m.main() == (b(BELT), 4)).unwrap();
    assert_eq!(MACHINE_RECIPES[belts].category, Assembly);
    assert!(ASSEMBLY_RECIPES.contains(&(belts as u16)));
    let hand = RECIPES.iter().find(|r| r.output == b(BELT)).unwrap();
    assert_eq!(MACHINE_RECIPES[belts].inputs, hand.inputs);
}

/// Time by hand follows the formula, and a row in the override table (`timing.rs`) replaces it for one recipe.
#[test]
fn a_recipes_hand_time_is_the_formula_unless_overridden() {
    let plate = RECIPES.iter().find(|r| r.output == IRON_PLATE).unwrap();
    assert_eq!(plate.hand_ticks(), BASE_TICKS + 2 * timing::TICKS_PER_ITEM);
    assert_eq!(plate.ticks_with(&[(IRON_PLATE, 15)]), 90, "1.5 s");
    assert_eq!(plate.ticks_with(&[(IRON_ROD, 15)]), plate.hand_ticks(), "another recipe's row");
}

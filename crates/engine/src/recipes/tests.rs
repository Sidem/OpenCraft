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
use Category::{Pressing, Smelting};

/// Every category (the match below stops compiling until a new one is listed).
const CATEGORIES: [Category; 2] = [Smelting, Pressing];
const _: fn(Category) = |c| match c {
    Smelting | Pressing => {}
};

/// Blocks the world has (generated, or left by worked-out deposits): breaking them is how their drops
/// are first had.
const WORLD_BLOCKS: &[BlockId] = &[
    SPENT_ROCK, STONE, DIRT, GRASS, SAND, LOG, LEAVES, COAL_ORE, IRON_ORE, COPPER_ORE, GRANITE, SANDSTONE, BASALT,
    LIMESTONE, QUARTZ_ORE, RUSTY_SOIL, DARK_SOIL, GREEN_SOIL, PALE_SOIL, RUSTY_SAND, DARK_SAND, GREEN_SAND, PALE_SAND,
];
/// Items the world gives other than block drops (leaves drop saplings: `action.rs`).
const GATHERED: &[ItemId] = &[ItemId::block(SAPLING)];
/// Known exception: quicklime waits for concrete and steel (steps 6.5 and 6.6).
const NO_USE_YET: &[ItemId] = &[item::QUICKLIME];

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
        let end = it.places().is_some() || tools::tool(it).is_some() || tools::device(it).is_some();
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
    let no_screws: Vec<Recipe> =
        RECIPES.iter().filter(|r| !r.inputs.iter().any(|i| i.0 == item::SCREW)).map(copy).collect();
    assert!(has(lint_items(&no_screws, MACHINE_RECIPES), "Screws has no use"));
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
    const HIGH: [Family; 1] = [Family { block: BELT, items: &[b(BELT), b(FAST_BELT), b(STONE)], kits: 1 }];
    assert!(has(lint_tiers(&HIGH, RECIPES), "tier 2 of block 12 has no kit"));
}

fn copy(r: &Recipe) -> Recipe {
    Recipe { output: r.output, group: r.group, count: r.count, inputs: r.inputs, blurb: r.blurb }
}

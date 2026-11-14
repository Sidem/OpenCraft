//! How to get any item, and what it is for (presentation, queries only): the build menu's guide (`ui/item-guide.ts`).
//! The ways come from the tables, so a new recipe shows up by itself: hand recipes (`recipes::RECIPES`), machine
//! recipes (`MACHINE_RECIPES`, made in the first machine whose spec takes the category: `factory::SPECS` order), ore
//! as this world's generator lays it (`ore_guide::ore_way`), and [`FOUND`] for what the world or a machine of its own
//! gives without a recipe (ground, trees, ruins, the pumpjack, the recycler).
//!
//! Invariant: every item a recipe, a fuel slot or a town asks for, and every block the world drops, has at least one
//! way, so a player who needs something can always find out where it comes from (`tests.rs` names any that has none).
//!
//! A machine way also says how many machines feed how many at full speed (`ratios`), with the panel's own sums.
//!
//! To add a source no recipe describes: a row in `FOUND`.

use crate::block::{self, BlockId};
use crate::factory::{first_maker, tenths, tenths_a_minute, tiers};
use crate::item::{ItemId, COIN, CRUDE_BARREL, SCRAP_METAL};
use crate::recipes::{Category, MACHINE_RECIPES, RECIPES};
use crate::research::{Research, Unlock, TECHS};
use crate::wood::Form;
use crate::worldgen::WorldGen;

/// One way to come by an item.
#[derive(Clone, PartialEq, Debug)]
pub enum Way {
    /// Found or dug in the world, or given by a machine without a recipe: how, in a sentence or two.
    Found(String),
    /// Hand recipe `RECIPES[i]`.
    Hand(usize),
    /// Machine recipe `MACHINE_RECIPES[recipe]`, made in `machine` (the item that places the first machine for it).
    Machine { recipe: usize, machine: ItemId },
}

/// Every way to get `item` in the world `gen` makes: found first (the simple way for raw materials), then hand
/// recipes, then machine recipes in table order.
pub fn ways(gen: &WorldGen, item: ItemId) -> Vec<Way> {
    let mut out = Vec::new();
    if let Some(text) = found(gen, item) {
        out.push(Way::Found(text));
    }
    out.extend(RECIPES.iter().enumerate().filter(|(_, r)| r.output == item).map(|(i, _)| Way::Hand(i)));
    for (i, r) in MACHINE_RECIPES.iter().enumerate() {
        // A launch's outputs are the empty barrels coming back: not a way to make barrels.
        if r.category == Category::Launch || !r.outputs.iter().any(|o| o.0 == item) {
            continue;
        }
        if let Some(machine) = machine_for(r.category) {
            out.push(Way::Machine { recipe: i, machine });
        }
    }
    out
}

/// Where `item` comes from, in a sentence or three, for a machine waiting for it ("Dig it on beaches… A Crusher makes it
/// from Slag (research Ore Crushing) or Tailings (research Ore Washing)."): how it is found (its first sentence, and any
/// that names the scanner or a quarry), then the first two machines that make it, each with what from. An item only a
/// machine makes from one found input says where that is found too. `None` for an item no machine makes and nothing
/// finds (hand-crafted parts: the recipe list says it all).
pub fn source_line(gen: &WorldGen, research: &Research, item: ItemId) -> Option<String> {
    let ways = ways(gen, item);
    let mut parts: Vec<String> =
        ways.iter().filter_map(|w| if let Way::Found(t) = w { Some(brief(t)) } else { None }).collect();
    // Each machine once, with every input it makes this from: "a Crusher makes it from Slag or Tailings".
    let mut machines: Vec<(ItemId, Vec<String>)> = Vec::new();
    for way in &ways {
        let Way::Machine { recipe, machine } = way else { continue };
        let r = &MACHINE_RECIPES[*recipe];
        let from: Vec<&str> = r.inputs.iter().map(|i| crate::item::name(i.0)).collect();
        let from = format!("{}{}", from.join(" and "), research_note(research, *recipe));
        if let Some(m) = machines.iter_mut().find(|m| m.0 == *machine) {
            m.1.push(from);
        } else if machines.len() < 2 {
            machines.push((*machine, vec![from]));
        }
    }
    parts.extend(
        machines.iter().map(|(m, from)| format!("A {} makes it from {}.", crate::item::name(*m), from.join(" or "))),
    );
    if parts.is_empty() {
        return None;
    }
    // Made from one raw material: say where that is found (quicklime from limestone, dug below the ground).
    if let Some(Way::Machine { recipe, .. }) = ways.first() {
        let inputs = MACHINE_RECIPES[*recipe].inputs;
        if let (1, Some(t)) = (inputs.len(), found(gen, inputs[0].0)) {
            parts.push(format!("{}: {}", crate::item::name(inputs[0].0), lower_first(&brief(&t))));
        }
    }
    Some(parts.join(" "))
}

/// What a stuck output is good for, for a machine that has nowhere to put it ("A Crusher makes Sand from it (research
/// Ore Crushing)."): the first two machine recipes that take it. `None` when none does.
pub fn use_line(research: &Research, item: ItemId) -> Option<String> {
    let each: Vec<String> = uses(item)
        .into_iter()
        .filter(|&(machine, _)| machine)
        .filter_map(|(_, i)| {
            let r = &MACHINE_RECIPES[i];
            let by = machine_for(r.category)?;
            let made = crate::item::name(r.main().0);
            Some(format!("a {} makes {made} from it{}", crate::item::name(by), research_note(research, i)))
        })
        .take(2)
        .collect();
    (!each.is_empty()).then(|| format!("{}.", upper_first(&each.join("; "))))
}

/// " (research Ore Crushing)" while machine recipe `recipe` is locked.
fn research_note(research: &Research, recipe: usize) -> String {
    research
        .locked_by(Unlock::MachineRecipe(recipe as u16))
        .map_or(String::new(), |t| format!(" (research {})", TECHS[t as usize].name))
}

/// A found text cut to its first sentence and any sentence naming the scanner or a quarry.
fn brief(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for (k, s) in text.split_inclusive(". ").enumerate() {
        let lower = s.to_ascii_lowercase();
        if k == 0 || lower.contains("scanner") || lower.contains("quarry") {
            out.push(s.trim_end());
        }
    }
    out.join(" ")
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_lowercase().chain(c).collect())
}

fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

/// What uses `item`: the hand recipes (`false`, index) and machine recipes (`true`, index) that take it, in table order.
pub fn uses(item: ItemId) -> Vec<(bool, usize)> {
    let hand = RECIPES.iter().enumerate().filter(|(_, r)| r.inputs.iter().any(|x| x.0 == item));
    let machine = MACHINE_RECIPES.iter().enumerate().filter(|(_, r)| r.inputs.iter().any(|x| x.0 == item));
    hand.map(|(i, _)| (false, i)).chain(machine.map(|(i, _)| (true, i))).collect()
}

/// How many machines feed how many on machine recipe `recipe`, Mk1 at full speed, one entry per input another machine
/// makes: "2 Smelter to 1 Constructor (Iron Ingot)", joined by " · ". Empty when no input comes from a machine.
pub fn ratios(gen: &WorldGen, recipe: usize) -> String {
    let r = &MACHINE_RECIPES[recipe];
    let Some((block, work)) = maker(r.category) else { return String::new() };
    let mut out = Vec::new();
    for &(input, n) in r.inputs {
        // The first recipe whose main product it is (a byproduct is no way to plan a line).
        let way = ways(gen, input).into_iter().find_map(|w| match w {
            Way::Machine { recipe: s, .. } if s != recipe && MACHINE_RECIPES[s].main().0 == input => Some(s),
            _ => None,
        });
        let Some(s) = way else { continue };
        let supplier = &MACHINE_RECIPES[s];
        let Some((from, from_work)) = maker(supplier.category) else { continue };
        let need = tenths_a_minute(r, n, work);
        let give = tenths_a_minute(supplier, supplier.main().1, from_work);
        let (a, b) = small_ratio(need, give);
        let name = |b: BlockId| crate::item::name(tiers::item_of(b, 0).unwrap_or(ItemId::block(b)));
        out.push(format!("{a} {} to {b} {} ({})", name(from), name(block), crate::item::name(input)));
    }
    out.join(" · ")
}

/// The block of the first machine making recipes of `category`, and its Mk1 work a tick at full power.
fn maker(category: Category) -> Option<(BlockId, u32)> {
    let block = first_maker(category)?;
    Some((block, crate::factory::process_spec(block)?.tiers[0].speed))
}

/// `need / give` as "a to b": b at most [`MAX_SHARE`] when that is within 3%, else 1 to many when one machine
/// feeds more than that (within 4%), else in tenths to 1.
fn small_ratio(need: u32, give: u32) -> (String, u32) {
    let (need, give) = (u64::from(need.max(1)), u64::from(give.max(1)));
    for b in 1..=MAX_SHARE {
        let a = (need * b + give / 2) / give;
        if a > 0 && (a * give).abs_diff(need * b) * 100 <= 3 * need * b {
            return (a.to_string(), b as u32);
        }
    }
    if need < give {
        return ("1".to_string(), ((give + need / 2) / need) as u32);
    }
    (tenths(((need * 10 + give / 2) / give) as u32), 1)
}

/// The most machines on the fed side of a ratio before it is given in tenths.
const MAX_SHARE: u64 = 12;

/// The item that places the first machine making recipes of `category`.
pub fn machine_for(category: Category) -> Option<ItemId> {
    let block = first_maker(category)?;
    Some(tiers::item_of(block, 0).unwrap_or(ItemId::block(block)))
}

/// Sources that are not recipes, by the item they give.
const FOUND: &[(ItemId, &str)] = &[
    (
        b(block::STONE),
        "Dig it almost anywhere below the soil, best with a pickaxe. A quarry digs it for you by the beltful.",
    ),
    (b(block::DIRT), "Dig the ground: grass and soil give dirt."),
    (b(block::SAND), "Dig it on beaches, along rivers and in deserts. A quarry set on a beach digs it for you."),
    (b(block::LOG), "Chop the trunk of any tree, best with an axe."),
    (b(block::LEAVES), "Break the leaves of a tree."),
    (b(block::SAPLING), "Leaves now and then leave one behind when broken or when they wither."),
    (b(block::BIRCH_SAPLING), "Birch leaves now and then leave one behind when broken or when they wither."),
    (b(block::PINE_SAPLING), "Pine needles now and then leave one behind when broken or when they wither."),
    (b(block::PALM_SAPLING), "Palm fronds now and then leave one behind: palms grow on warm shores."),
    (b(block::CHERRY_SAPLING), "Cherry blossom now and then leaves one behind: cherries grow in groves on the plains."),
    (b(block::BAOBAB_SAPLING), "Baobab leaves now and then leave one behind: baobabs grow on hot, dry plains."),
    (b(block::ACACIA_SAPLING), "Acacia leaves now and then leave one behind: acacias grow on hot, dry plains."),
    (b(block::REDWOOD_SAPLING), "Redwood needles now and then leave one behind: redwoods grow on wet highlands."),
    (b(block::EUCALYPTUS_SAPLING), "Eucalyptus leaves now and then leave one behind: they grow on warm, wet ground."),
    (b(block::BAMBOO_SHOOT), "Bamboo leaves now and then leave one behind: bamboo grows in warm, wet lowlands."),
    (b(block::CACTUS), "Grows in deserts: break it."),
    (b(block::GRANITE), "The rock under the highlands: dig into granite hills and mountains."),
    (b(block::SANDSTONE), "The rock under desert sand: dig below it."),
    (b(block::BASALT), "The black rock of basalt fields."),
    (b(block::SPENT_ROCK), "What ore turns into once a miner has drawn its share out: dig it where a deposit ran dry."),
    (SCRAP_METAL, "Break the rusted plating, girders and machines of ruins. Taking iron things apart gives some too."),
    (CRUDE_BARREL, "A pumpjack set on oil sand fills empty barrels with crude oil."),
    (COIN, "A recycler pays coins for the items it destroys."),
];

/// How `item` is found or dug, if it is.
fn found(gen: &WorldGen, item: ItemId) -> Option<String> {
    let as_block = (item.0 < 256).then_some(item.0 as BlockId);
    if let Some(b) = as_block.filter(|&b| block::is_ore(b)) {
        let lacks = "This world was made before it was in the game: only a new world has it.";
        return Some(crate::ore_guide::ore_way(gen, b).unwrap_or_else(|| lacks.to_string()));
    }
    if let Some(text) = other_wood(item) {
        return Some(text);
    }
    FOUND.iter().find(|f| f.0 == item).map(|f| f.1.to_string())
}

/// What an item of one of the woods after the oak is (`wood.rs`): recipes take any wood's as the oak's, so these have
/// no recipe of their own to describe them.
fn other_wood(item: ItemId) -> Option<String> {
    let (w, form) = crate::wood::of(item).filter(|&(w, _)| w > 0)?;
    let name = crate::item::name(item);
    let wood = name.split(' ').next().unwrap_or(name);
    let lower = wood.to_lowercase();
    let same = |what: &str| format!("{name}: the same as oak {what} in every recipe, in {lower}'s own colour.");
    Some(match form {
        Form::Log if w == crate::wood::WOODS - 1 => {
            "Cut from bamboo. Works in any recipe that takes logs; saws into bamboo planks.".to_string()
        }
        Form::Log => {
            format!("Felled from {lower} trees. Saws into {lower} planks; works in any recipe that takes logs.")
        }
        Form::Planks => same("planks"),
        Form::Floor => same("floor"),
        Form::Door => same("door"),
    })
}

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

#[cfg(test)]
mod tests;

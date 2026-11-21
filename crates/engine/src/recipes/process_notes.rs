//! One line of real chemistry or physics for each chemical machine recipe, shown under the recipe's card in the machine
//! panel and in the item guide (`api/guide.rs` `machine_recipe_note`): what the game's batch stands for, so a card says why
//! as well as what. Presentation only; nothing here touches a save.
//!
//! Invariant: every row of the refinery, cracker, chemical plant, electrolyser and centrifuge has a note
//! (`recipes/tests.rs`). To add one: a row in [`NOTES`] by the recipe's index (`rows.rs`), one plain sentence, no formulas a
//! player must know to follow it.

use super::rows::*;

const NOTES: &[(u16, &str)] = &[
    (
        DISTIL_RECIPE,
        "Crude oil is heated and its vapours rise a tall column: light naphtha condenses near the top, diesel in the \
         middle, heavy oil at the bottom.",
    ),
    (
        CRACK_RECIPE,
        "Cracking breaks big, heavy molecules into small, light ones with heat and a catalyst, so heavy oil becomes \
         naphtha and diesel.",
    ),
    (
        PLASTIC_RECIPE,
        "Naphtha is cracked into small molecules that are then linked into very long chains: that is what a plastic is.",
    ),
    (
        ACID_RECIPE,
        "Sulfur is burned to a gas, oxidised further and dissolved in water: that is sulfuric acid (the contact process).",
    ),
    (
        LUBRICANT_RECIPE,
        "Lubricant is the thick heavy-oil fraction, refined and blended with additives so moving metal slides instead of \
         grinding.",
    ),
    (
        ELECTROLYSE_RECIPE,
        "A current through water pulls it apart: 2 H₂O → 2 H₂ + O₂, hydrogen at one electrode and oxygen at the other.",
    ),
    (
        ENRICH_RECIPE,
        "Uranium is spun in centrifuges: the rarer, lighter U-235 drifts to the middle until there is enough of it to \
         fuel a reactor.",
    ),
    (
        PURE_WATER_RECIPE,
        "Water is filtered and stripped of every dissolved mineral, because a chip fab cannot have a stray ion near its \
         wafers.",
    ),
    (
        ROCKET_FUEL_RECIPE,
        "Hydrogen burning in oxygen leaves only steam and a great deal of thrust: it is what the Space Shuttle's main \
         engines ran on.",
    ),
    (
        GAS_PROCESS_RECIPE,
        "Raw gas is scrubbed of toxic hydrogen sulfide (recovered as sulfur), then chilled to separate methane, ethane \
         and LPG.",
    ),
    (
        REFORM_RECIPE,
        "Steam methane reforming reacts hot methane with steam over a nickel catalyst (CH₄ + 2 H₂O → CO₂ + 4 H₂), the \
         cheapest industrial hydrogen.",
    ),
    (
        ETHANE_CRACK_RECIPE,
        "Steam cracking heats ethane past 800 °C for a split second so it drops two hydrogens and becomes ethylene \
         plus hydrogen gas.",
    ),
    (
        LPG_CRACK_RECIPE,
        "Thermal cracking breaks propane and butane in LPG into ethylene for plastics, leaving methane behind as \
         fuel gas.",
    ),
    (
        POLYETHYLENE_RECIPE,
        "Under pressure with a catalyst, ethylene molecules snap end to end into long chains of polyethylene — the \
         world's most common plastic.",
    ),
];

const BLUE_NOTE: &str =
    "Copper dissolved in sulfuric acid gives copper sulfate, the bright blue crystals that coloured \
                         pigments and old glass.";
const MIX_NOTE: &str = "Pigments mix like paint: stir two hues together and the result sits between them, as red and \
                       yellow make orange.";

/// The note of machine recipe `recipe`, or "" when it has none. The ten mixed hues share one.
pub fn note(recipe: usize) -> &'static str {
    let dye = recipe.wrapping_sub(usize::from(DYE_FIRST));
    if dye == 0 {
        return BLUE_NOTE;
    }
    if dye < usize::from(DYE_PLANT_ROWS) {
        return MIX_NOTE;
    }
    NOTES.iter().find(|n| usize::from(n.0) == recipe).map_or("", |n| n.1)
}

#[cfg(test)]
mod tests;

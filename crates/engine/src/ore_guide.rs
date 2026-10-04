//! Helping the player find ore (presentation, queries only): the ore guide beside the map and the
//! readout for stained soil. Every number comes from the generator the world was made with (biome
//! weights `worldgen/geology.rs`, depth bands and the starter set `worldgen/strata.rs`, lode heights
//! `worldgen/ore.rs`), so the guide stays true for old worlds and follows any retuning.
//!
//! To add an ore: a row in `GUIDE`.

use crate::block::{self, BlockId, COAL_ORE, COPPER_ORE, IRON_ORE, LIMESTONE, QUARTZ_ORE};
use crate::deposits::Tier;
use crate::math::IVec3;
use crate::minimap::ore_color;
use crate::prospect::SCANNERS;
use crate::worldgen::{ore_shares, WorldGen, LODE_HEIGHTS};

/// Per ore: its block, the stain it leaves on grass (on sand it is the same colour), how it looks.
const GUIDE: [(BlockId, &str, &str); 5] = [
    (COAL_ORE, "dark soil", "black lumps in the rock"),
    (IRON_ORE, "rusty soil", "rust-brown nodules in the rock"),
    (COPPER_ORE, "green soil", "orange and green crusts"),
    (LIMESTONE, "pale soil", "pale rock with fossils"),
    (QUARTZ_ORE, "pale soil", "pink-white crystals"),
];

/// One line per ore, fields split by tabs: block id, colour (0xRRGGBB), name, the band it lies in
/// (lowest and highest depth below the ground), where it is common, and how to spot it.
pub fn guide_rows(gen: &WorldGen) -> String {
    let mut out = String::new();
    for (ore, stain, looks) in GUIDE {
        let (lo, hi) = gen.ore_band(ore);
        let shares = ore_shares(ore);
        let common = if gen.version() < 2 || shares.is_empty() {
            "Everywhere".to_string()
        } else {
            let parts: Vec<String> = shares.iter().map(|(b, pct)| format!("{} {pct}%", b.name())).collect();
            format!("Share of deposits: {}", parts.join(", "))
        };
        let mut signs = format!("Exposed: {looks}.");
        if gen.version() >= 2 {
            signs += &format!(" Buried: {stain} or sand above a vein or lode.");
        }
        let label = block::ore_label(ore);
        let name = if ore == LIMESTONE { label.to_string() } else { format!("{label} ore") };
        out += &format!("{ore}\t{}\t{name}\t{lo}\t{hi}\t{common}\t{signs}\n", ore_color(ore));
    }
    out
}

/// General advice, one line each, true for this world's generator version.
pub fn guide_notes(gen: &WorldGen) -> String {
    let mut lines = Vec::new();
    let (near, far) = gen.starter_reach();
    if gen.version() >= 3 {
        lines.push(format!(
            "A starter patch of coal, iron and copper lies {near} to {far} blocks from where the world starts."
        ));
        lines.push("Exposed ore shows only on bare rock: cliffs, mountain tops, deserts and basalt fields.".into());
        lines
            .push("Depth counts down from the ground above, not from sea level: under a hill, ore lies higher.".into());
    }
    if gen.version() >= 2 {
        lines.push("Stained soil lies over veins and lodes. Point at it to see how far down the ore is.".into());
    }
    let (lo, hi) = LODE_HEIGHTS;
    lines.push(format!("Lodes, the biggest deposits, lie near bedrock (height {lo} to {hi}) under any ground."));
    let [(_, near), (_, far)] = SCANNERS;
    lines.push(format!(
        "A scanner lists every deposit within {near} blocks ({far} for the Mk2); a core drill measures the ground under it."
    ));
    lines.join("\n")
}

/// What a stained soil at `pos` tells: the ore below and how far down its top lies; `None` for any
/// other block.
pub fn stain_reading(gen: &WorldGen, pos: IVec3, stain: BlockId) -> Option<String> {
    if !(block::RUSTY_SOIL..=block::PALE_SAND).contains(&stain) {
        return None;
    }
    let Some(d) = gen.hint_source(pos.x, pos.z, stain) else {
        return Some("Stains like this lie over ore veins and lodes.".into());
    };
    let depth = (pos.y - d.bounds().1.y).max(1);
    let (article, name) = (if d.ore() == IRON_ORE { "An" } else { "A" }, d.name().to_ascii_lowercase());
    let bedrock = if d.tier() == Tier::Lode { ", near bedrock" } else { "" };
    Some(format!(
        "{article} {name} lies below: its top is about {depth} blocks down{bedrock}.\nDig straight down, or scan here."
    ))
}

#[cfg(test)]
mod tests;

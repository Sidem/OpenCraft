//! Onboarding hints: short plain-language tips in the order a new player needs them, each with the
//! state of the game that shows it has been done. Read-only queries on the core for one player; which
//! hints a player dismissed is UI state the host keeps (`web/src/ui/hints.ts`).
//!
//! Invariant: `progress` is the index after the last hint that is done, so doing a later step implies
//! the earlier ones (a player who already has a smelter skips the mining tips).
//! To add a hint: a row in `HINTS`, where it belongs in the order.

use crate::block::{COAL_ORE, COPPER_ORE, IRON_ORE, MINER};
use crate::factory::{Factory, Kind};
use crate::inventory::Inventory;
use crate::research::TECHS;

pub struct Hint {
    pub text: &'static str,
    done: fn(&Inventory, &Factory) -> bool,
}

pub const HINTS: &[Hint] = &[
    Hint {
        text: "Find ore: rock speckled with colour (dark coal, rusty iron, orange and green copper). Point at \
               it to see the deposit, then hold the left mouse button to mine it. The nearest one may be \
               under a block or two of ground.",
        done: |inv, _| [COAL_ORE, IRON_ORE, COPPER_ORE].iter().any(|&o| inv.count(o.into()) > 0),
    },
    Hint {
        text: "Build a miner: press E for the build menu. A Miner Mk1 needs iron ore, copper ore and stone. \
               Mining by hand wastes most of the ore, so build one early.",
        done: |inv, _| inv.count(MINER.into()) > 0,
    },
    Hint {
        text: "Place the miner: select it on the hotbar and right-click an ore block. It drills the whole \
               deposit for you.",
        done: |_, f| f.count(Kind::Miner) > 0,
    },
    Hint {
        text: "Carry the ore away: place belts leading away from the miner (they run the way you face) and a \
               storage box at the end.",
        done: |_, f| f.count(Kind::Belt) > 0 && f.count(Kind::Storage) > 0,
    },
    Hint {
        text: "Make ingots: build a smelter next to the ore line and give it fuel too (coal ore or logs), by \
               belt or from its panel (right-click it).",
        done: |_, f| f.count(Kind::Smelter) > 0,
    },
    Hint {
        text: "Make parts: a constructor shapes ingots into plates, rods, screws and wire. It needs power: a \
               coal generator and a power pole within 5 blocks of both.",
        done: |_, f| f.count(Kind::Constructor) > 0 && f.count(Kind::Generator) > 0,
    },
    Hint {
        text: "Research: build a research lab near a pole, give it red science packs, and press R to choose \
               what to research. New machines unlock as you go.",
        done: |_, f| (0..TECHS.len()).any(|t| f.research.progress(t as u8) > 0),
    },
];

/// How far the player got: the index after the last hint that is done (0 when none is).
pub fn progress(inv: &Inventory, f: &Factory) -> usize {
    HINTS.iter().rposition(|h| (h.done)(inv, f)).map_or(0, |i| i + 1)
}

#[cfg(test)]
mod tests;

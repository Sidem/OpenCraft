//! Onboarding hints: short plain-language tips in the order a new player needs them, each with the
//! state of the game that shows it has been done. Read-only queries on the core for one player; which
//! hints a player dismissed is UI state the host keeps (`web/src/ui/hints.ts`).
//!
//! Invariant: `progress` is the index after the last hint that is done, so doing a later step implies
//! the earlier ones (a player who already has a smelter skips the mining tips).
//! To add a hint: a row in `HINTS`, where it belongs in the order.

use crate::block::{ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, COAL_ORE, COPPER_ORE, IRON_ORE, MINER, TURBINE};
use crate::factory::{Factory, Kind};
use crate::inventory::Inventory;
use crate::item::IRON_INGOT;
use crate::research::{TechState, TECHS};

pub struct Hint {
    pub text: &'static str,
    done: fn(&Inventory, &Factory) -> bool,
}

pub const HINTS: &[Hint] = &[
    Hint {
        text: "Find ore: rock speckled with colour (dark coal, rusty iron, orange and green copper). Patches of \
               coal, iron and copper show a short walk from where you start; press M for the map, which marks \
               ore you have seen with a diamond and has a guide to every ore. Hold the left mouse button to \
               mine it.",
        done: |inv, _| [COAL_ORE, IRON_ORE, COPPER_ORE].iter().any(|&o| inv.count(o.into()) > 0),
    },
    Hint {
        text: "Make ingots: press E and craft a Stone Furnace from 16 stone, place it, and right-click it to \
               give it coal ore or logs as fuel and iron or copper ore to melt. Ingots come out after a few \
               seconds; take them from its panel.",
        done: |inv, f| f.processors_of(crate::block::SMELTER) > 0 || inv.count(IRON_INGOT) > 0,
    },
    Hint {
        text: "Build a miner: in the build menu craft iron plates, iron rods and copper wire from ingots (a \
               craft takes a few seconds and queues up), then a Miner Mk1. Asking for the miner queues the \
               parts itself when you hold the ingots. Mining by hand wastes most of the ore, so build one early.",
        done: |inv, _| inv.count(MINER.into()) > 0,
    },
    Hint {
        text: "Place the miner: select it on the hotbar and right-click an ore block. It drills the whole \
               deposit for you.",
        done: |_, f| f.count(Kind::Miner) > 0,
    },
    Hint {
        text: "Power it: a miner runs on electricity. Build a coal generator and a power pole within 5 blocks \
               of both, and start the generator with coal ore or logs (right-click it). A miner on coal next \
               to the generator keeps it fuelled: one coal runs a miner long enough to dig about 32.",
        done: |_, f| f.count(Kind::Generator) > 0 && f.count(Kind::Pole) > 0,
    },
    Hint {
        text: "Carry the ore away: hold right-click on the ground next to the miner and drag to lay a \
               line of belts, and put a storage box at its end.",
        done: |_, f| f.count(Kind::Belt) > 0 && f.count(Kind::Storage) > 0,
    },
    Hint {
        text: "Ingots by belt: put a smelter at the end of the ore line and fuel it (coal ore or logs) by belt \
               too; a belt leading away takes the ingots.",
        done: |_, f| f.processors_of(crate::block::SMELTER) > 0 && f.count(Kind::Belt) > 0,
    },
    Hint {
        text: "Make parts: a constructor shapes ingots into plates, rods, screws and wire. It needs power \
               like the miner: a pole within 5 blocks.",
        done: |_, f| f.processors_of(crate::block::CONSTRUCTOR) > 0,
    },
    Hint {
        text: "Research: build a research lab near a pole, give it red science packs, and press T to choose \
               what to research. New machines unlock as you go.",
        done: |_, f| (0..TECHS.len()).any(|t| f.research.progress(t as u8) > 0),
    },
    Hint {
        text: "Move water: research Fluid Handling, put a pump in a pond or a flooded pit and power it, and \
               pipe it to an outlet facing where the water should go. Nothing lowers the sea.",
        done: |_, f| f.count(Kind::Pipe) > 0,
    },
    Hint {
        text: "Dig for real: a quarry digs the ground in front of it into a pit and fills a belt or box with the \
               stone and dirt. Hold one to see its box (R turns it); right-click it to choose size and depth.",
        done: |_, f| f.count(Kind::Quarry) > 0,
    },
    Hint {
        text: "Upgrade in place: Mechanics unlocks green kits (by hand, or by the beltful in an assembler). Hold \
               kits and right-click a machine to make it Mk2, twice as fast; drag along a belt line to upgrade \
               the belts. Blue kits make Mk3.",
        done: |_, f| tech_done(f, "Mechanics"),
    },
    Hint {
        text: "Assemble: research Assembly and build an assembler, a 2×2×2 machine. Belts feed its back and \
               sides, you choose its recipe in its panel, and parts come out of its front. Motors, concrete and \
               science packs are made here.",
        done: |_, f| f.processors_of(ASSEMBLER) > 0,
    },
    Hint {
        text: "Make steel: research Steelmaking and build a blast furnace (2×2×3). Iron ore, coal and quicklime \
               in, steel out of the front, slag out of the side: put a belt at each, or it stops. Blue science \
               needs steel.",
        done: |_, f| f.processors_of(BLAST_FURNACE) > 0,
    },
    Hint {
        text: "Steam: research Steam Power. Pipe water from a pump to a boiler, belt the boiler coal, and set \
               steam turbines (two to a boiler, 240 kW each) against it, hung on a pole. A generator on the same \
               grid starts the pump. A pond runs dry; the sea doesn't.",
        done: |_, f| f.processors_of(TURBINE) > 0,
    },
    Hint {
        text: "Electronics: research it, then quartz (25 to 50 blocks down; scan for pale soil) and coal make \
               silicon in an arc furnace (120 kW), and an assembler turns silicon, copper wire and iron plates \
               into circuits.",
        done: |_, f| f.processors_of(ARC_FURNACE) > 0,
    },
    Hint {
        text: "Violet science: research it, then assemble violet packs from circuits, a steel beam and a motor. \
               Labs take them in a fourth slot; violet kits make Mk4.",
        done: |_, f| tech_done(f, "Violet Science"),
    },
];

fn tech_done(f: &Factory, name: &str) -> bool {
    TECHS.iter().position(|t| t.name == name).is_some_and(|i| f.research.state(i as u8) == TechState::Done)
}

/// How far the player got: the index after the last hint that is done (0 when none is).
pub fn progress(inv: &Inventory, f: &Factory) -> usize {
    HINTS.iter().rposition(|h| (h.done)(inv, f)).map_or(0, |i| i + 1)
}

#[cfg(test)]
mod tests;

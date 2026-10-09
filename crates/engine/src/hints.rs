//! Onboarding hints: short plain-language tips in the order a new player needs them, each with the
//! state of the game that shows it has been done. Read-only queries on the core for one player; which
//! hints a player dismissed is UI state the host keeps (`web/src/ui/hints.ts`).
//!
//! Invariant: `progress` is the index after the last hint that is done, so doing a later step implies
//! the earlier ones (a player who already has a smelter skips the mining tips).
//! To add a hint: a row in `HINTS`, where it belongs in the order.

use crate::block::{
    ARC_FURNACE, ASSEMBLER, BLAST_FURNACE, COAL_ORE, COPPER_ORE, DATACENTER, DRONE_PORT, IRON_ORE, LASER_EMITTER,
    MINER, PUMPJACK, REACTOR, SOLAR_PANEL, TURBINE,
};
use crate::factory::{Factory, Kind};
use crate::inventory::Inventory;
use crate::item::{CARGO_DRONE, HOVER_PACK, IRON_INGOT, JETPACK, PERSONAL_DRONE, PLANNER};
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
               of both. A new pole is selected: click the generator and the miner to wire them to it (a Mk1 \
               pole holds 4 wires). Start the generator with coal ore or logs (right-click it). A miner on coal \
               next to the generator keeps it fuelled: one coal runs a miner long enough to dig about 32.",
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
               like the miner: right-click a pole to select it, then click the constructor to wire it.",
        done: |_, f| f.processors_of(crate::block::CONSTRUCTOR) > 0,
    },
    Hint {
        text: "Research: build a research lab, wire it to a pole, give it red science packs, and press T to choose \
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
               the belts, or Shift-click a belt for its whole line. Blue kits make Mk3.",
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
        text: "Steam: research Steam Power. Pipe water from a pump to a boiler's blue water inlet, belt coal into a \
               dark coal inlet, and pipe its two front steam outlets to the steam inlets of turbines (two to a \
               boiler, 240 kW each), wired to a pole. A generator on the same grid starts the pump. A pond runs \
               dry; the sea doesn't.",
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
    Hint {
        text: "Solar power: research Solar Power. A panel gives up to 10 kW by day and nothing at night; wire it to a \
               pole. An accumulator stores 10 MJ of spare sun and gives it back after dark, before a generator \
               burns fuel. About six panels and one accumulator carry 15 kW round the clock.",
        done: |_, f| f.processors_of(SOLAR_PANEL) > 0,
    },
    Hint {
        text: "Ghosts: press B for ghost mode, hold a block or machine and right-click to plant a see-through plan \
               instead of building it; left-click marks a block to tear down. Z marks two corners and Enter copies \
               a build as a blueprint (L lists them) you can stamp elsewhere.",
        done: |_, f| tech_done(f, "Processors"),
    },
    Hint {
        text: "Drones: a long road. Research Processors, Robotics, Drone Power and Navigation, then Construction \
               Drones: an assembler builds the drone from actuators, a drone cell and a guidance module. Build a \
               drone port (a 3×3 pad, 40 kW while drones fly), put drones in it, and set storage boxes touching \
               the pad with what your ghosts need. They build ghosts and clear your marks within 32 blocks.",
        done: |_, f| f.processors_of(DRONE_PORT) > 0,
    },
    Hint {
        text: "Fly and fetch: research Jetpack, wear it in the Jetpack slot (Shift-click it in the inventory) \
               and hold jump in the air: it burns a coal per 10 seconds from your pack. A Personal Drone in your pack fetches the item in your \
               hand from the nearest box within 32 blocks when you press Y.",
        done: |inv, _| inv.count(JETPACK) > 0 || inv.has_jetpack() || inv.count(PERSONAL_DRONE) > 0,
    },
    Hint {
        text: "Move the ground: research Earthworks and craft the Planner. Right-click one corner block and then the \
               opposite one to mark an area to dig, fill or flatten (or a tunnel) and pick the level in its panel. \
               Drone ports within reach do the work, digging into the storage boxes touching their pad and filling \
               from them. Sites show on the map as hollow squares.",
        done: |inv, f| inv.count(PLANNER) > 0 || !f.sites.list.is_empty(),
    },
    Hint {
        text: "Far ground: bauxite, the ore of aluminium, lies only in deserts and basalt fields 600 or more blocks \
               from where you started. Research Advanced Scanning for the Scanner Mk2, press R to filter for \
               bauxite, and with none in range it gives a compass bearing and a distance band to head for. \
               Research Bauxite Processing for crushers and the electrolytic cell.",
        done: |_, f| tech_done(f, "Bauxite Processing"),
    },
    Hint {
        text: "Trains: research Rails, Trains and Freight. Place rail nodes like power poles (a curve of track joins \
               each pair, up to 32 blocks apart), put a locomotive on a node, couple wagons behind it, and build a \
               loading dock at one end and an unloading dock at the other. Hold the locomotive and click docks to \
               set its schedule; rail signals keep trains apart.",
        done: |_, f| f.count(Kind::Rail) > 0,
    },
    Hint {
        text: "Hover and cargo: the Hover Pack (research it; charge it within 6 blocks of a power pole) holds you in \
               the air while you hold jump. Cargo Drones (research them, assemble them from a drone, aluminium \
               plates and batteries) live in a drone port: hold one, click a port and then another to set a \
               route, and keep batteries in the first port's boxes, one for every 150 blocks flown there and back.",
        done: |inv, _| inv.count(HOVER_PACK) > 0 || inv.count(CARGO_DRONE) > 0,
    },
    Hint {
        text:
            "Oil: research Oil Processing, find oil sand with the Scanner Mk2 (200 or more blocks out, deep down) and \
               stand a pumpjack right above it. It fills empty canisters, pressed from steel plates, with crude oil; \
               the refinery splits that into naphtha, diesel, heavy oil and sulfur, and every stream must be used or \
               stored. Diesel generators burn the diesel; the chemical plant makes plastic.",
        done: |_, f| f.processors_of(PUMPJACK) > 0,
    },
    Hint {
        text: "Nuclear: research Nuclear Power, scan for uranium (400 or more blocks out, 45 to 85 down), and make \
               fuel cells in a centrifuge. A reactor gives up to 2 MW from them, but it must be cooled: pipe water \
               from a pump to its blue inlet, or it overheats and shuts down until it has cooled.",
        done: |_, f| f.processors_of(REACTOR) > 0,
    },
    Hint {
        text: "Gold science: research Gold Science and a research center, which alone has slots for gold packs. \
               Assemble packs from a plastic, a battery and a processor; Mk5 Machines, researched with them, makes \
               gold kits for Mk5.",
        done: |_, f| tech_done(f, "Gold Science"),
    },
    Hint {
        text: "Compute: research Data Network and Datacenters. Fibre nodes within 12 blocks join into a data grid, and \
               an AI datacenter (4×4×3, 3 MW) puts 100 TF on it while you pipe it cooling water (a cooling tower \
               saves water). AI labs research with that compute, and the optimizer node makes machines within 16 \
               blocks work 25% faster.",
        done: |_, f| f.processors_of(DATACENTER) > 0,
    },
    Hint {
        text: "Lasers: research Photonics. A laser emitter aimed along a clear line (glass lets the beam through) \
               joins its power grid to a receiver up to 128 blocks away; mirrors, turned with R, bend the beam round \
               corners, and a data receiver joins two data grids instead.",
        done: |_, f| f.processors_of(LASER_EMITTER) > 0,
    },
    Hint {
        text: "Smarter tools: AI Research opens more. A swarm hub gives every drone port within 12 blocks 50% more \
               drones; Auto-Routing makes a dragged belt line find its own way to a machine (as ghosts in ghost \
               mode, B); AI Survey rings likely ore on the maps from the stained soil you have seen.",
        done: |_, f| ["Drone Swarms", "Auto-Routing", "AI Survey"].iter().any(|t| tech_done(f, t)),
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

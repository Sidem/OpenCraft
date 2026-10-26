//! Processing machines as data: one [`ProcessSpec`] row per block that turns inputs into outputs
//! (docs/TECH_TREE.md section 8): the recipe categories it takes, how it is driven, how it picks a
//! recipe, its buffers, its numbers per tier, its footprint and ports (`footprint/`) and its model parts.
//!
//! Invariants: a spec has a tier row for every tier of its family (`tiers.rs`; tested); a spec with no
//! burner tier has no fuel buffer; its block is a `Kind::Process` row in `MACHINES`.
//!
//! To add a processor: its block (`block/`), a `MACHINES` row of `Kind::Process`, its row with its parts (in
//! `basic.rs`, or its own file if it has logic of its own) named in `SPECS`, and its recipes' category
//! (`recipes/machine.rs`).

use crate::block::BlockId;
use crate::item::ItemId;
use crate::recipes::{Category, MachineRecipe, MACHINE_RECIPES};

use super::super::footprint::Footprint;
use super::model::Part;

/// How a processor is driven.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Energy {
    /// Burns fuel (`recipes::FUELS`) from its fuel buffer while a batch runs.
    Burner,
    /// Draws `ProcessTier::power` from a power grid (`power.rs`) while it works.
    Electric,
    /// Needs nothing beyond its recipe (a blast furnace's coal is an input): always at full speed.
    Recipe,
    /// A boiler: burns fuel from its fuel buffer into steam for the turbines piped to its outlets (steam.rs).
    Boiler,
    /// A steam turbine: a power source fed by the boilers piped to its steam inlet (steam.rs).
    Turbine,
    /// A solar panel: a power source that follows the sun (solar.rs).
    Solar,
    /// An accumulator: stores spare solar power and gives it back before any fuel burns (solar.rs).
    Accumulator,
    /// A diesel generator: a power source burning diesel canisters (diesel.rs).
    Diesel,
    /// A water wheel: a power source driven by the water that touches it (hydro.rs).
    Hydro,
    /// A hoist winch: a power sink that makes the shaft beside it fast (hoist.rs).
    Hoist,
    /// A nuclear reactor: a water-cooled power source burning fuel cells (nuclear.rs).
    Reactor,
    /// A datacenter: draws power at all times and gives the data grid compute, water-cooled (datacenter.rs).
    Compute,
    /// A cooling tower: a power sink that closes a datacenter's water loop (tower.rs).
    Cooling,
    /// An optimizer or a swarm hub: a power sink and data consumer that boosts what is around it (optimizer.rs, hub.rs).
    Optimizer,
    /// A laser block (emitter, mirror, receiver): draws nothing, joins power or data grids by a beam (laser.rs).
    Beam,
    /// A rectenna: a power source fed by the power satellites over it (rectenna.rs).
    Rectenna,
}

impl Energy {
    /// Whether it is a power source or store with a step of its own in `Power::balance` (nothing to do in `step`).
    pub fn is_source(self) -> bool {
        matches!(
            self,
            Energy::Turbine
                | Energy::Solar
                | Energy::Accumulator
                | Energy::Diesel
                | Energy::Hydro
                | Energy::Reactor
                | Energy::Rectenna
        )
    }
}

/// How a processor picks its recipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    /// The player chooses in its panel (`SetRecipe`); it takes only that recipe's inputs.
    Chosen,
    /// What it holds decides: it takes anything an unlocked recipe of its categories uses.
    ByInput,
    /// Takes anything into its output buffer, which belts empty (a silo: steam.rs).
    Store,
    /// Takes only drones, up to its fleet, and keeps them for `drones/` to fly (hangar.rs).
    Hangar,
    /// Takes only empty canisters and fills them from the reservoir below (pump.rs).
    Pump,
    /// A loading dock: stores like a silo, and a train stopped beside it takes what it holds (docks.rs).
    Load,
    /// An unloading dock: stores like a silo, a train stopped beside it gives it its cargo (docks.rs).
    Unload,
    /// A research center: takes science packs and researches the chosen tech (center.rs).
    Research,
    /// A recycler: destroys any item it is given and pays coins (recycler.rs).
    Recycle,
    /// A pod launcher: stores cargo like a silo and keeps drop pods in its input slot; sends them to a landing pad (pods.rs).
    Pods,
    /// A landing pad: stores what drop pods bring, like a silo that belts empty (pods.rs).
    Land,
    /// A town depot: stores only what its town wants (`Processor::wants`), which the town uses up (depot.rs).
    Depot,
}

impl Pick {
    /// Whether it holds what it is given in its output buffer: silos, docks, pod launchers, landing pads and depots.
    pub fn stores(self) -> bool {
        matches!(self, Pick::Store | Pick::Load | Pick::Unload | Pick::Pods | Pick::Land | Pick::Depot)
    }

    /// Whether trains stop at it.
    pub fn docks(self) -> bool {
        matches!(self, Pick::Load | Pick::Unload)
    }
}

/// One tier's numbers.
pub struct ProcessTier {
    /// How this tier is driven: a smelter is a burner at Mk1 and Mk2 and electric from Mk3.
    pub energy: Energy,
    /// Work per tick at full power, in thousandths of a Mk1 tick.
    pub speed: u32,
    /// Fuel per unit of work, in thousandths (burners: 750 burns a quarter less a batch).
    pub fuel: u32,
    /// kW drawn while working (electric).
    pub power: u32,
}

pub struct ProcessSpec {
    pub block: BlockId,
    pub categories: &'static [Category],
    pub pick: Pick,
    /// Slots of the input, fuel and output buffers.
    pub buffers: [usize; 3],
    /// Slots of the byproduct buffer (0: byproducts go to the output buffer). It is emptied by belts at the
    /// footprint's `Role::Side` ports.
    pub side: usize,
    /// Mk1 first.
    pub tiers: &'static [ProcessTier],
    /// Its cells and ports (`SINGLE`: one cell, every side).
    pub footprint: Footprint,
    /// Status words: what it does ("Smelting"), what it makes ("ingots"), its line while it holds
    /// nothing to work on (`Pick::ByInput`).
    pub verb: &'static str,
    pub products: &'static str,
    pub waiting: &'static str,
    /// Its mark on the maps, 0xRRGGBB.
    pub map_colour: i32,
    /// Data grid compute at full power, in TF (fibre.rs): positive makes it, negative uses it, 0 takes no part.
    pub compute: i32,
    /// Boxes relative to the footprint's centre, front towards +z (turned with the machine).
    pub parts: &'static [Part],
}

pub const SPECS: &[ProcessSpec] = &[
    super::solar::SOLAR_SPEC,
    super::solar::ACCUMULATOR_SPEC,
    super::hangar::HANGAR_SPEC,
    super::docks::LOADING_DOCK_SPEC,
    super::docks::UNLOADING_DOCK_SPEC,
    super::pump::PUMPJACK_SPEC,
    super::refinery::REFINERY_SPEC,
    super::refinery::CRACKER_SPEC,
    super::refinery::PLANT_SPEC,
    super::refinery::ELECTROLYSER_SPEC,
    super::diesel::DIESEL_SPEC,
    super::washer::WASHER_SPEC,
    super::center::CENTER_SPEC,
    super::hydro::WHEEL_SPEC,
    super::hoist::WINCH_SPEC,
    super::nuclear::CENTRIFUGE_SPEC,
    super::nuclear::REACTOR_SPEC,
    super::recycler::RECYCLER_SPEC,
    super::fab::FAB_SPEC,
    super::datacenter::DATACENTER_SPEC,
    super::tower::TOWER_SPEC,
    super::ailab::AI_LAB_SPEC,
    super::optimizer::OPTIMIZER_SPEC,
    super::laser::EMITTER_SPEC,
    super::laser::RECEIVER_SPEC,
    super::laser::MIRROR_SPEC,
    super::laser::DATA_RECEIVER_SPEC,
    super::hub::HUB_SPEC,
    super::launch::LAUNCH_SPEC,
    super::rectenna::RECTENNA_SPEC,
    super::ground::STATION_SPEC,
    super::pods::LAUNCHER_SPEC,
    super::pods::PAD_SPEC,
    super::upgrader::UPGRADER_SPEC,
    super::depot::DEPOT_SPEC,
    super::basic::SMELTER_SPEC,
    super::basic::CONSTRUCTOR_SPEC,
    super::basic::ASSEMBLER_SPEC,
    super::basic::BLAST_SPEC,
    super::basic::BOILER_SPEC,
    super::basic::TURBINE_SPEC,
    super::basic::CRUSHER_SPEC,
    super::basic::SILO_SPEC,
    super::basic::ARC_SPEC,
    super::basic::CELL_SPEC,
];

/// The spec of the processor `block` is, if it is one.
pub fn spec(block: BlockId) -> Option<&'static ProcessSpec> {
    SPECS.iter().find(|s| s.block == block)
}

/// The block of the first processor in [`SPECS`] that makes recipes of `category` (the item guide names it).
pub fn first_maker(category: Category) -> Option<BlockId> {
    SPECS.iter().find(|s| s.categories.contains(&category)).map(|s| s.block)
}

/// Whether the machine `block` makes recipes of `category`.
pub fn makes(block: BlockId, category: Category) -> bool {
    spec(block).is_some_and(|s| s.categories.contains(&category))
}

impl ProcessSpec {
    /// Recipe `i` if this machine makes it.
    pub fn recipe(&self, i: u16) -> Option<&'static MachineRecipe> {
        MACHINE_RECIPES.get(i as usize).filter(|r| self.categories.contains(&r.category))
    }

    /// The first recipe it makes that uses `item` and `unlocked` allows (by recipe index).
    pub fn recipe_using(&self, item: ItemId, unlocked: &[bool]) -> Option<u16> {
        self.recipe_using_if(item, unlocked, |_| true)
    }

    /// Like `recipe_using`, among only the recipes `ok` accepts.
    pub fn recipe_using_if(&self, item: ItemId, unlocked: &[bool], ok: impl Fn(&MachineRecipe) -> bool) -> Option<u16> {
        let fits = |(i, r): &(usize, &MachineRecipe)| {
            self.categories.contains(&r.category)
                && unlocked.get(*i) == Some(&true)
                && r.inputs.iter().any(|x| x.0 == item)
                && ok(r)
        };
        MACHINE_RECIPES.iter().enumerate().find(fits).map(|(i, _)| i as u16)
    }
}

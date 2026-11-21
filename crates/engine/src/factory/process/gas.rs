//! Natural gas machines (CHEM-GAS): the gas well (`GAS_WELL`), wellhead separator (`WELLHEAD_SEPARATOR`), gas plant
//! (`GAS_PLANT`), gas turbine (`GAS_TURBINE`) and steam reformer (`STEAM_REFORMER`).
//!
//! - `GAS_WELL` and `WELLHEAD_SEPARATOR` are `Pick::Pump` wells (`pump.rs`): the gas well draws raw gas from `GAS_SHALE`
//!   deposits; the wellhead separator sits over `OIL_SAND` and recovers associated raw gas alongside crude oil.
//! - `GAS_PLANT` (`Category::GasProcessing`) sweetens and fractionates raw gas into methane, ethane, LPG and sulfur
//!   (water-cooled, 3 side outputs like the refinery).
//! - `STEAM_REFORMER` (`Category::Reforming`) reforms methane with steam into hydrogen barrels at 100 kW (a fifth of
//!   the electrolyser's draw).
//! - `GAS_TURBINE` (`Energy::Gas`) is a clean 600 kW power source burning methane barrels (`METHANE_KJ`) or LPG barrels
//!   (`LPG_KJ`), returning empty barrels and running before smoky diesel generators in `Power::balance`.

use crate::block::{tex, GAS_PLANT, GAS_TURBINE, GAS_WELL, STEAM_REFORMER, WELLHEAD_SEPARATOR};
use crate::inventory::Stack;
use crate::item::{ItemId, EMPTY_BARREL, LPG_BARREL, METHANE_BARREL};
use crate::recipes::Category;
use crate::TICK_RATE;

use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{Power, NOT_WIRED};
use super::super::power_mix::Source;
use super::model::{part, round, round_x, Look, Part};
use super::parts::DISPLAY;
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

/// Maximum electrical output of a gas turbine, in kW.
pub const GAS_TURBINE_KW: u32 = 600;
/// Energy per methane barrel in kJ: 600 kW for 40 s.
pub const METHANE_KJ: u32 = 24_000;
/// Energy per LPG barrel in kJ: 600 kW for 60 s.
pub const LPG_KJ: u32 = 36_000;
pub const MAX_GAS_CHARGE: u32 = LPG_KJ * TICK_RATE;

const IN_BACK: Port = Port { side: Side::Back, role: Role::In, cell: Which::All };
const OUT_FRONT: Port = Port { side: Side::Front, role: Role::Out, cell: Which::All };
const SIDE_RIGHT: Port = Port { side: Side::Right, role: Role::Side, cell: Which::All };

const STEEL: Look = Look::Tex([tex::STEEL; 3]);
const FRAME: Look = Look::Tex([tex::FRAME; 3]);
const SOOT: Look = Look::Tex([tex::SOOT; 3]);
const GUNMETAL: Look = Look::Tex([tex::MAT_GUNMETAL; 3]);
const BRASS: Look = Look::Tex([tex::MAT_BRASS; 3]);
const EMBER: Look = Look::Tex([tex::MAT_EMBER; 3]);

/// Gas Well (2×2×3, 90 kW): drills straight down to `GAS_SHALE` and fills empty barrels with raw gas.
pub const GAS_WELL_SPEC: ProcessSpec = ProcessSpec {
    block: GAS_WELL,
    categories: &[],
    pick: Pick::Pump,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 }],
    footprint: Footprint { size: [2, 2, 3], ports: &[IN_BACK, OUT_FRONT] },
    verb: "Drawing",
    products: "raw gas barrels",
    waiting: "Idle",
    map_colour: 0x38bdf8,
    compute: 0,
    parts: &GAS_WELL_PARTS,
};

/// Wellhead Separator (2×3×3, 110 kW): drills down to `OIL_SAND`, separating crude oil (front) and associated raw gas
/// (right side) at a deterministic per-deposit gas-oil ratio.
pub const WELLHEAD_SEPARATOR_SPEC: ProcessSpec = ProcessSpec {
    block: WELLHEAD_SEPARATOR,
    categories: &[],
    pick: Pick::Pump,
    buffers: [1, 0, 1],
    side: 1,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 110 }],
    footprint: Footprint { size: [2, 3, 3], ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT] },
    verb: "Separating",
    products: "crude oil barrels",
    waiting: "Idle",
    map_colour: 0x629677,
    compute: 0,
    parts: &SEPARATOR_PARTS,
};

/// Gas Plant (3×3×4, 140 kW): sweetens and fractionates raw gas into methane (front), ethane, LPG and sulfur (side).
pub const GAS_PLANT_SPEC: ProcessSpec = ProcessSpec {
    block: GAS_PLANT,
    categories: &[Category::GasProcessing],
    pick: Pick::ByInput,
    buffers: [1, 0, 1],
    side: 3,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 140 }],
    footprint: Footprint {
        size: [3, 3, 4],
        ports: &[IN_BACK, OUT_FRONT, SIDE_RIGHT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(1) }],
    },
    verb: "Sweetening",
    products: "methane",
    waiting: "Waiting for raw gas barrels",
    map_colour: 0x4ca6c4,
    compute: 0,
    parts: &GAS_PLANT_PARTS,
};

/// Gas Turbine (3×2×2, 600 kW): burns methane or LPG barrels for clean grid power and returns empty barrels.
pub const GAS_TURBINE_SPEC: ProcessSpec = ProcessSpec {
    block: GAS_TURBINE,
    categories: &[],
    pick: Pick::ByInput,
    buffers: [2, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Gas, speed: 1000, fuel: 0, power: 0 }],
    footprint: Footprint { size: [3, 2, 2], ports: &[IN_BACK, OUT_FRONT] },
    verb: "Generating",
    products: "empty barrels",
    waiting: "Waiting for methane or LPG barrels",
    map_colour: 0x38a3d1,
    compute: 0,
    parts: &GAS_TURBINE_PARTS,
};

/// Steam Reformer (2×2×3, 100 kW): reacts methane with steam to produce hydrogen barrels at low electrical draw.
pub const STEAM_REFORMER_SPEC: ProcessSpec = ProcessSpec {
    block: STEAM_REFORMER,
    categories: &[Category::Reforming],
    pick: Pick::ByInput,
    buffers: [2, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 100 }],
    footprint: Footprint {
        size: [2, 2, 3],
        ports: &[IN_BACK, OUT_FRONT, Port { side: Side::Left, role: Role::Water, cell: Which::Nth(0) }],
    },
    verb: "Reforming",
    products: "hydrogen",
    waiting: "Waiting for methane and empty barrels",
    map_colour: 0x8f7466,
    compute: 0,
    parts: &REFORMER_PARTS,
};

const WELL_TEX: [u16; 3] = [tex::GAS_WELL_TOP, tex::GAS_WELL_SIDE, tex::FRAME];
/// Gas wellhead (2×2×3, ±1, ±1.5, ±1): banded plinth, cellar flange, vertical Christmas-tree spool, two bolted
/// collars, wing-valve cross with red handwheels, brass pressure bonnet, and status lamp.
const GAS_WELL_PARTS: [Part; 10] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::CONCRETE)),
    round([0.0, -1.1, 0.0], [1.1, 0.4, 1.1], GUNMETAL),
    round([0.0, -0.05, 0.0], [0.56, 1.7, 0.56], Look::Tex(WELL_TEX)),
    round([0.0, -0.45, 0.0], [0.84, 0.14, 0.84], STEEL),
    round([0.0, 0.35, 0.0], [0.84, 0.14, 0.84], STEEL),
    round_x([0.0, -0.05, 0.0], [1.5, 0.34, 0.34], Look::Tex(WELL_TEX)),
    round_x([-0.8, -0.05, 0.0], [0.1, 0.58, 0.58], EMBER),
    round_x([0.8, -0.05, 0.0], [0.1, 0.58, 0.58], EMBER),
    round([0.0, 1.02, 0.0], [0.46, 0.44, 0.46], BRASS),
    part([0.78, -1.18, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

const SEP_TEX: [u16; 3] = [tex::SEPARATOR_TOP, tex::SEPARATOR_SIDE, tex::FRAME];
/// Wellhead separator (2×3×3, ±1, ±1.5, ±1.5): banded plinth, vertical well riser at the front with brass cap,
/// horizontal three-phase pressure drum on two steel saddles behind it, mist extractor dome on top, and status lamp.
const SEPARATOR_PARTS: [Part; 10] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 2.96], Look::Band(tex::CONCRETE)),
    round([0.0, -0.1, 0.85], [0.52, 2.4, 0.52], Look::Tex(SEP_TEX)),
    round([0.0, 1.22, 0.85], [0.64, 0.24, 0.64], BRASS),
    part([-0.45, -1.0, -0.45], [0.22, 0.6, 0.9], FRAME),
    part([0.45, -1.0, -0.45], [0.22, 0.6, 0.9], FRAME),
    round_x([0.0, -0.35, -0.45], [1.66, 0.9, 0.9], Look::Tex(SEP_TEX)),
    round_x([-0.86, -0.35, -0.45], [0.06, 0.78, 0.78], STEEL),
    round_x([0.86, -0.35, -0.45], [0.06, 0.78, 0.78], STEEL),
    round([0.0, 0.36, -0.45], [0.48, 0.52, 0.48], STEEL),
    part([0.78, -1.18, 1.38], [0.14, 0.1, 0.14], Look::Lamp),
];

const PLANT_BODY: [u16; 3] = [tex::GAS_PLANT_TOP, tex::GAS_PLANT_SIDE, tex::FRAME];
const PLANT_COL: [u16; 3] = [tex::FRAME, tex::GAS_PLANT_SIDE, tex::FRAME];
/// Gas plant (3×3×4, ±1.5, ±2, ±1.5): banded plinth, horizontal scrubber vessel in front, tall amine absorber column
/// with catwalk ring, second fractionator tower linked by an overhead pipe bridge, flare stack, lamp and product display.
const GAS_PLANT_PARTS: [Part; 12] = [
    part([0.0, -1.9, 0.0], [2.96, 0.2, 2.96], Look::Band(tex::FRAME)),
    part([-0.6, -1.5, 0.75], [0.24, 0.6, 0.9], FRAME),
    part([0.6, -1.5, 0.75], [0.24, 0.6, 0.9], FRAME),
    round_x([0.0, -1.05, 0.75], [2.2, 0.8, 0.8], Look::Tex(PLANT_BODY)),
    round([-0.55, 0.05, -0.55], [1.06, 3.7, 1.06], Look::Tex(PLANT_COL)),
    part([-0.55, 0.55, -0.55], [1.32, 0.08, 1.32], FRAME),
    round([0.78, -0.3, -0.65], [0.78, 3.0, 0.78], Look::Tex(PLANT_COL)),
    part([0.12, 0.98, -0.6], [1.3, 0.12, 0.12], STEEL),
    part([1.05, 0.2, -1.05], [0.16, 2.2, 0.16], STEEL),
    part([1.05, 1.4, -1.05], [0.2, 0.2, 0.2], Look::Fire(tex::SOOT)),
    part([1.25, -1.62, 1.42], [0.14, 0.1, 0.14], Look::Lamp),
    part([-0.55, 2.2, -0.55], DISPLAY, Look::Product),
];

const TURB_BODY: [u16; 3] = [tex::GAS_TURBINE_TOP, tex::GAS_TURBINE_SIDE, tex::FRAME];
/// Gas turbine (3×2×2, ±1.5, ±1, ±1): banded plinth, air-intake filter house on left, horizontal cylindrical
/// compressor/turbine barrel with steel flange ring, generator housing with copper bus on right, clean exhaust stack, lamp.
const GAS_TURBINE_PARTS: [Part; 9] = [
    part([0.0, -0.9, 0.0], [2.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([-0.96, -0.15, 0.0], [0.76, 1.3, 1.48], Look::Tex(TURB_BODY)),
    round_x([0.04, -0.18, 0.0], [1.24, 1.1, 1.1], Look::Tex(TURB_BODY)),
    round_x([0.04, -0.18, 0.0], [0.14, 1.22, 1.22], STEEL),
    part([1.02, -0.25, 0.0], [0.74, 1.1, 1.16], GUNMETAL),
    part([1.02, 0.42, 0.0], [0.3, 0.24, 0.3], Look::Tex([tex::COPPER_INGOT; 3])),
    round([-0.96, 0.72, -0.2], [0.5, 0.44, 0.5], STEEL),
    round([-0.96, 0.96, -0.2], [0.58, 0.06, 0.58], GUNMETAL),
    part([1.22, -0.55, 0.92], [0.14, 0.1, 0.14], Look::Lamp),
];

const REF_BODY: [u16; 3] = [tex::REFORMER_TOP, tex::REFORMER_SIDE, tex::FRAME];
/// Steam reformer (2×2×3, ±1, ±1.5, ±1): banded plinth, firebox base, four vertical catalyst tubes rising into an
/// overhead horizontal steam drum, rear flue stack with smoke, status lamp and product display.
const REFORMER_PARTS: [Part; 12] = [
    part([0.0, -1.4, 0.0], [1.96, 0.2, 1.96], Look::Band(tex::FRAME)),
    part([0.0, -0.82, 0.1], [1.78, 0.96, 1.56], Look::Tex(REF_BODY)),
    part([0.0, -0.3, 0.1], [1.86, 0.08, 1.64], FRAME),
    round([-0.46, 0.3, -0.15], [0.28, 1.12, 0.28], STEEL),
    round([0.06, 0.3, -0.15], [0.28, 1.12, 0.28], STEEL),
    round([-0.46, 0.3, 0.35], [0.28, 1.12, 0.28], STEEL),
    round([0.06, 0.3, 0.35], [0.28, 1.12, 0.28], STEEL),
    round_x([-0.2, 1.06, 0.1], [1.1, 0.46, 0.46], Look::Tex([tex::PIPE_STEAM; 3])),
    part([0.66, 0.35, -0.5], [0.28, 1.9, 0.28], SOOT),
    part([0.66, 1.35, -0.5], [0.7, 0.0, 0.0], Look::Smoke),
    part([0.75, -0.62, 0.95], [0.12, 0.08, 0.12], Look::Lamp),
    part([-0.2, 1.59, 0.1], DISPLAY, Look::Product),
];

/// Energy (in kJ) of a fuel accepted by the gas turbine, or `None` for any other item.
pub fn gas_fuel_kj(item: ItemId) -> Option<u32> {
    match item {
        METHANE_BARREL => Some(METHANE_KJ),
        LPG_BARREL => Some(LPG_KJ),
        _ => None,
    }
}

impl Processor {
    /// The status line of a gas turbine (`None`: another processor).
    pub(super) fn gas_text(&self) -> Option<String> {
        if self.energy() != Energy::Gas {
            return None;
        }
        Some(match self.status {
            Status::Working => {
                format!("Giving {} of {GAS_TURBINE_KW} kW · {} kJ lit", self.store.given, self.store.charge / TICK_RATE)
            }
            Status::NoFuel => "Out of fuel: bring methane or LPG barrels".to_string(),
            Status::OutputFull => "Empty barrels have nowhere to go: put a belt leading away, or take them".to_string(),
            Status::NoPower => NOT_WIRED.to_string(),
            _ => "Standing by: the grid wants no more power".to_string(),
        })
    }
}

/// Picks the next fuel barrel to burn in a gas turbine (methane first, then LPG).
fn next_gas_fuel(p: &Processor) -> Option<(ItemId, u32)> {
    if p.input.count(METHANE_BARREL) > 0 {
        Some((METHANE_BARREL, METHANE_KJ * TICK_RATE))
    } else if p.input.count(LPG_BARREL) > 0 {
        Some((LPG_BARREL, LPG_KJ * TICK_RATE))
    } else {
        None
    }
}

/// One tick of every gas turbine on a grid: supplies up to `GAS_TURBINE_KW`, burning a methane or LPG barrel and
/// returning the empty barrel when more charge is needed. Runs before diesel generators in `Power::balance`.
pub(in crate::factory) fn run(
    power: &mut Power,
    processors: &mut [Processor],
    turbines: &[u32],
    manual_off: &[crate::math::IVec3],
) {
    for &i in turbines {
        let (i, p) = (i as usize, &mut processors[i as usize]);
        p.store.given = 0;
        if super::super::switch::is_listed(manual_off, p.pos) {
            p.status = Status::Off;
            continue;
        }
        let Some(pole) = power.process_pole[i] else {
            p.status = Status::NoPower;
            continue;
        };
        let grid = power.pole_grid[pole as usize] as usize;
        let want = (power.demand[grid] - power.supply[grid].min(power.demand[grid])).min(GAS_TURBINE_KW);
        let mut stuck = false;
        if want > p.store.charge {
            if let Some((fuel, charge)) = next_gas_fuel(p) {
                if p.out.space_for(EMPTY_BARREL) > 0 {
                    p.input.remove(fuel, 1);
                    p.out.add(EMPTY_BARREL, 1);
                    p.used.push(Stack { item: fuel, count: 1 });
                    p.made.push(Stack { item: EMPTY_BARREL, count: 1 });
                    p.store.charge += charge;
                } else {
                    stuck = true;
                }
            }
        }
        let given = want.min(p.store.charge);
        p.store.charge -= given;
        p.store.given = given;
        let fuelled = p.store.charge > 0 || next_gas_fuel(p).is_some();
        power.give(Source::Gas, grid, given, if fuelled { GAS_TURBINE_KW } else { 0 });
        p.status = match () {
            _ if given > 0 => Status::Working,
            _ if stuck => Status::OutputFull,
            _ if fuelled => Status::NoInput,
            _ => Status::NoFuel,
        };
    }
}

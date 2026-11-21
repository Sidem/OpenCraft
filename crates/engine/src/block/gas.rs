//! Natural gas blocks (CHEM-GAS, ids 255–260): the deep gas shale deposit block (`GAS_SHALE`, id 255, the last block
//! sharing its own id as an item) and the five gas machines (`GAS_WELL` to `STEAM_REFORMER`, ids 256–260, the first wide
//! blocks paired with `EXTRA` items in `item::WIDE_BLOCK_ITEMS`).
//!
//! To add a gas block: append its id, its `GAS_DEFS` row, its `WIDE_BLOCK_ITEMS` pair (for ids >= 256) and bump `GAS_COUNT`.

use super::make::{all, machine, ore, pillar};
use super::{tex, BlockDef, BlockId};

/// Deep shale holding trapped natural gas (generator version 13, `worldgen/gas.rs`).
pub const GAS_SHALE: BlockId = 255;
/// Draws raw natural gas from a gas shale deposit into empty barrels (`factory/process/pump.rs`).
pub const GAS_WELL: BlockId = 256;
/// Extracts crude oil and associated raw gas together from an oil sand deposit (`factory/process/pump.rs`).
pub const WELLHEAD_SEPARATOR: BlockId = 257;
/// Sweetens and fractionates raw gas into methane, ethane, LPG and sulfur (`factory/process/gas.rs`).
pub const GAS_PLANT: BlockId = 258;
/// Burns methane or LPG barrels for 600 kW of clean power (`factory/process/gas.rs`).
pub const GAS_TURBINE: BlockId = 259;
/// Steam methane reformer: turns methane and water into hydrogen barrels (`factory/process/gas.rs`).
pub const STEAM_REFORMER: BlockId = 260;

pub const GAS_COUNT: usize = 6;

/// The rows of `DEFS` from block 255 on, in id order.
/// Machine blocks are `Render::None` (never meshed as terrain cubes), so their cube-face fallback layers stay under 512;
/// their placed models and wide-block item icons use their dedicated `tex::GAS_*` layers.
pub(super) const GAS_DEFS: [BlockDef; GAS_COUNT] = [
    ore("Gas Shale", all(tex::GAS_SHALE), GAS_SHALE),
    machine("Gas Well", true, 1.6, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), GAS_WELL),
    machine("Wellhead Separator", true, 1.8, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), WELLHEAD_SEPARATOR),
    machine("Gas Plant", true, 2.0, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), GAS_PLANT),
    machine("Gas Turbine", true, 1.8, pillar(tex::TURBINE_SIDE, tex::TURBINE_TOP, tex::FRAME), GAS_TURBINE),
    machine("Steam Reformer", true, 1.8, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), STEAM_REFORMER),
];

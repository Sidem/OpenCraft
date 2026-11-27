//! The Weather Station's block (ATMOS, id 271): a wide block paired with `WEATHER_STATION_ITEM` in
//! `item::WIDE_BLOCK_ITEMS`. What it does: `factory/process/weather_station.rs`.

use super::make::{machine, pillar};
use super::{tex, BlockDef, BlockId};

/// A mast with an instrument screen and an anemometer.
pub const WEATHER_STATION: BlockId = 271;

pub const WEATHER_STATION_COUNT: usize = 1;

pub(super) const WEATHER_STATION_DEFS: [BlockDef; WEATHER_STATION_COUNT] =
    [machine("Weather Station", true, 1.5, pillar(tex::PIGMENT_WHITE, tex::STEEL, tex::FRAME), WEATHER_STATION)];

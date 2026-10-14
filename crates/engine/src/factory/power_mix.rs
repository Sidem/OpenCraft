//! Where a tick's power came from and where it went, for the analytics screen (`analytics/`). Presentation only:
//! `Power::balance` fills it as a side effect of the sums it already does, and nothing in the core reads it.
//!
//! - Sources: every place that adds to a grid's supply calls [`Power::give`] with its [`Source`], so the totals per
//!   source (what was given, what could have been) always add up to the grids' own.
//! - Users: each powered machine asks through [`Power::note_use`] with its block, so `uses` holds the kW each kind of
//!   machine asked for this tick (indexed by block id).
//!
//! To add a power source: a `Source` variant with its row in `ALL`, `name` and `colour`, and `give` where it supplies.

use crate::block::{BlockId, BLOCK_COUNT};

use super::power::Power;

/// How many kinds of power source there are.
pub const SOURCES: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// Fuel-burning generators (coal, wood).
    Generator,
    Turbine,
    Solar,
    Accumulator,
    Hydro,
    Diesel,
    Reactor,
    /// Power satellites' beams, received by rectennas.
    Rectenna,
}

pub const ALL: [Source; SOURCES] = [
    Source::Generator,
    Source::Turbine,
    Source::Solar,
    Source::Accumulator,
    Source::Hydro,
    Source::Diesel,
    Source::Reactor,
    Source::Rectenna,
];

impl Source {
    pub fn name(self) -> &'static str {
        match self {
            Source::Generator => "Fuel generators",
            Source::Turbine => "Steam turbines",
            Source::Solar => "Solar panels",
            Source::Accumulator => "Accumulators",
            Source::Hydro => "Water wheels",
            Source::Diesel => "Diesel generators",
            Source::Reactor => "Nuclear reactors",
            Source::Rectenna => "Orbital power",
        }
    }

    /// The colour the graphs draw it in, as `0xRRGGBB`.
    pub fn colour(self) -> u32 {
        match self {
            Source::Generator => 0xc08a5a,
            Source::Turbine => 0x7dd3fc,
            Source::Solar => 0xfacc15,
            Source::Accumulator => 0xa3e635,
            Source::Hydro => 0x2dd4bf,
            Source::Diesel => 0xfb923c,
            Source::Reactor => 0xc084fc,
            Source::Rectenna => 0xf472b6,
        }
    }
}

/// Last tick's kW from one kind of source, summed over every grid.
#[derive(Clone, Copy, Default, Debug)]
pub struct Mix {
    /// Delivered to a grid that needed it.
    pub given: u32,
    /// What its fuelled (or sunlit, or watered) machines could have delivered.
    pub could: u32,
}

/// Per-block kW asked for this tick: a table `BLOCK_COUNT` long.
pub fn empty_uses() -> Vec<u32> {
    vec![0; BLOCK_COUNT]
}

/// Notes in `uses` (`Power::uses`) that a machine of `block` asks for `kw` this tick.
pub(super) fn note_use(uses: &mut [u32], block: BlockId, kw: u32) {
    if let Some(asked) = uses.get_mut(block as usize) {
        *asked += kw;
    }
}

impl Power {
    /// Adds `given` kW (of a possible `could`) from `source` to `grid`'s supply and capacity.
    pub(crate) fn give(&mut self, source: Source, grid: usize, given: u32, could: u32) {
        self.supply[grid] += given;
        self.capacity[grid] += could;
        let mix = &mut self.mix[source as usize];
        mix.given += given;
        mix.could += could;
    }

    /// Calls `f` with the block and kW of every kind of machine that asked for power last tick.
    pub fn each_use(&self, mut f: impl FnMut(BlockId, u32)) {
        for (block, &kw) in self.uses.iter().enumerate().filter(|(_, &kw)| kw > 0) {
            f(block as BlockId, kw);
        }
    }

    /// Clears the per-tick totals (the start of `balance`).
    pub(super) fn clear_mix(&mut self) {
        self.mix = [Mix::default(); SOURCES];
        self.uses.iter_mut().for_each(|u| *u = 0);
    }
}

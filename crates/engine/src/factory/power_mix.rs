//! Where a tick's power came from and where it went, for the analytics screen (`analytics/`). Presentation only:
//! `Power::balance` fills it as a side effect of the sums it already does, and nothing in the core reads it.
//!
//! - Sources: every place that adds to a grid's supply calls [`Power::give`] with its [`Source`], so the totals per
//!   source (what was given, what could have been) always add up to the grids' own.
//!   `grid_mix` keeps the same per grid.
//! - Users: `Factory::power_uses` walks `Power::each_demand`, the list the balance itself sums, by grid and block.
//! - Grids: `Factory::power_grids` reads each grid with a machine on it, keyed by where its oldest pole stands.
//!
//! To add a power source: a `Source` variant with its row in `ALL`, `name` and `colour`, and `give` where it supplies.

use crate::block::BlockId;
use crate::math::IVec3;

use super::grid_colour::PALETTE;
use super::power::{Power, Users};
use super::Factory;

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

impl Power {
    /// Adds `given` kW (of a possible `could`) from `source` to `grid`'s supply and capacity.
    pub(crate) fn give(&mut self, source: Source, grid: usize, given: u32, could: u32) {
        self.supply[grid] += given;
        self.capacity[grid] += could;
        for mix in [&mut self.mix[source as usize], &mut self.grid_mix[grid][source as usize]] {
            mix.given += given;
            mix.could += could;
        }
    }

    /// Clears the per-tick totals (the start of `balance`).
    pub(super) fn clear_mix(&mut self) {
        self.mix = [Mix::default(); SOURCES];
        self.grid_mix.iter_mut().for_each(|m| *m = [Mix::default(); SOURCES]);
    }
}

/// One grid as the analytics show it (`Factory::power_grids`): last tick's totals and sources.
#[derive(Clone, Copy, Debug)]
pub struct GridReading {
    /// Its number in this relink's `Power` lists (`power_uses` names grids by it).
    pub grid: usize,
    /// Where the grid's oldest pole stands: it names the grid from one reading to the next, through relinks, as long
    /// as that pole stays (when two grids join, the older pole's name goes on).
    pub key: IVec3,
    /// Its accent colour (`grid_colour::PALETTE`) as a name and as `0xRRGGBB`.
    pub colour: &'static str,
    pub rgb: u32,
    pub poles: u32,
    /// Machines and power sources wired to it.
    pub machines: u32,
    pub capacity: u32,
    pub supply: u32,
    pub demand: u32,
    pub mix: [Mix; SOURCES],
}

impl Factory {
    /// Every grid with at least one machine or power source on it, in grid order (the main grid is not always first).
    pub fn power_grids(&self) -> Vec<GridReading> {
        let p = &self.power;
        let mut machines = vec![0u32; p.supply.len()];
        let lists = [&p.gen_pole, &p.miner_pole, &p.process_pole, &p.lab_pole, &p.pipe_pole, &p.quarry_pole];
        for pole in lists.into_iter().flatten().flatten() {
            machines[p.pole_grid[*pole as usize] as usize] += 1;
        }
        let mut out: Vec<Option<GridReading>> = vec![None; p.supply.len()];
        for (i, &g) in p.pole_grid.iter().enumerate() {
            let g = g as usize;
            match &mut out[g] {
                Some(r) => r.poles += 1,
                None if machines[g] > 0 => {
                    let (colour, [r, gr, b]) = PALETTE[usize::from(p.grid_colour[g]) % PALETTE.len()];
                    out[g] = Some(GridReading {
                        grid: g,
                        key: self.poles[i].pos,
                        colour,
                        rgb: u32::from_be_bytes([0, r, gr, b]),
                        poles: 1,
                        machines: machines[g],
                        capacity: p.capacity[g],
                        supply: p.supply[g],
                        demand: p.demand[g],
                        mix: p.grid_mix[g],
                    });
                }
                None => {}
            }
        }
        out.into_iter().flatten().collect()
    }

    /// Calls `f` with the grid, block and kW of every wired machine asking for power now: the list the tick's balance
    /// sums (`Power::each_demand`), for the analytics' "where the power goes", whole or per grid.
    pub fn power_uses(&self, mut f: impl FnMut(usize, BlockId, u32)) {
        let unlocked = self.research.machine_recipes_unlocked();
        let users = Users {
            miners: &self.miners,
            processors: &self.processors,
            labs: &self.labs,
            pipework: &self.pipework,
            quarries: &self.quarries,
            research: &self.research,
            unlocked: &unlocked,
        };
        self.power.each_demand(&users, |pole, block, kw| f(self.power.pole_grid[pole as usize] as usize, block, kw));
    }
}

//! Electric power: poles, the grids they form, and how supply meets demand each tick.
//!
//! - Poles are wired by hand to poles and machines (`wiring.rs`); each connected group of poles is a grid,
//!   and a machine is on the grid of the pole it is wired to. Cables are the exception (`pole.rs`): they
//!   join poles and machines in reach by themselves, and a machine with no wire hangs on the nearest one.
//!   The grids are derived from the stored wires (`rebuild`, run by `relink`), never saved.
//! - Each tick `balance` adds up what each grid's machines need in kW (a miner while it drills, a
//!   electric processor (a constructor) while it works, a lab while it researches,
//!   a pump while it has room, a quarry while it digs), then takes it from the generators in list
//!   order, each up to its tier's power (`GENERATOR_TIERS`). A grid short of power runs its machines at
//!   `speed` = supply / demand.
//! - Energy: a generator holds the energy of the fuel it lit, in kW·ticks (1 kJ = `TICK_RATE`
//!   kW·ticks), and gives only what is drawn, so fuel lasts exactly as long as the load allows. It
//!   lights the next item (`recipes::fuel_energy`) when what it holds can't cover this tick.
//!
//! Sources: generators, then steam turbines (`process/steam.rs`). Consumers: miners, electric processors,
//! labs, pumps and quarries. Burner processors (the smelter) burn their own fuel, and belts, splitters and
//! filters need no power. To power a new machine: its `*_pole` list here (filled in `rebuild`), its
//! demand in `balance`, a speed argument to its `step`, and its arm in `wiring.rs` (`powered_cells`).

use crate::block::tex;
use crate::math::{IVec3, Vec3};
use crate::recipes::fuel_energy;
use crate::research::Research;
use crate::TICK_RATE;

use super::generator::Generator;
use super::lab::Lab;
use super::miner::Miner;
use super::pipes::{Part, Pipework};
use super::pole::{dist2, hang_cable, linked, nearest_of, Pole};
use super::process::{run_renewables, run_turbine, Energy, Processor};
use super::quarry::Quarry;
use super::render::push_box;
use super::wiring::{takes_pole, Hooked};
use super::Factory;

/// What a machine with no pole says (the readouts).
pub const NOT_WIRED: &str = "No power: select a power pole in reach, then click this machine to wire it";
/// Full speed, in thousandths.
pub const FULL_SPEED: u32 = 1000;

/// The grids, derived from positions by `rebuild`, plus last tick's supply and demand per grid.
#[derive(Default)]
pub(crate) struct Power {
    /// The grid of each pole.
    pub pole_grid: Vec<u32>,
    /// Pole pairs that are wired together (lower index first).
    pub wires: Vec<(u32, u32)>,
    /// The pole (or cable) each generator, miner, electric processor and lab is on, if it is wired.
    pub gen_pole: Vec<Option<u32>>,
    pub miner_pole: Vec<Option<u32>>,
    pub process_pole: Vec<Option<u32>>,
    pub lab_pole: Vec<Option<u32>>,
    /// Per piece of pipework: the pole of each pump (other pieces: `None`).
    pub pipe_pole: Vec<Option<u32>>,
    pub quarry_pole: Vec<Option<u32>>,
    /// Per grid, last tick: kW supplied, kW wanted, and kW its fuelled generators could give.
    pub supply: Vec<u32>,
    pub demand: Vec<u32>,
    pub capacity: Vec<u32>,
}

impl Power {
    /// Recomputes grids and hookups from the stored wires (`hooked`) and the cables' positions.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn rebuild(
        poles: &[Pole],
        hooked: &Hooked,
        gens: &[Generator],
        miners: &[Miner],
        processors: &[Processor],
        labs: &[Lab],
        pipework: &[Pipework],
        quarries: &[Quarry],
    ) -> Power {
        let n = poles.len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        let mut wires = hooked.pairs.clone();
        for i in 0..n {
            for j in i + 1..n {
                if (poles[i].is_cable() || poles[j].is_cable()) && linked(&poles[i], &poles[j]) {
                    wires.push((i as u32, j as u32));
                }
            }
        }
        for &(i, j) in &wires {
            let (a, b) = (root(&mut parent, i), root(&mut parent, j));
            parent[a.max(b) as usize] = a.min(b);
        }
        // Number the grids in order of their lowest pole.
        let mut grid_of_root = vec![u32::MAX; n];
        let mut grids = 0;
        let mut pole_grid = Vec::with_capacity(n);
        for i in 0..n as u32 {
            let r = root(&mut parent, i) as usize;
            if grid_of_root[r] == u32::MAX {
                grid_of_root[r] = grids;
                grids += 1;
            }
            pole_grid.push(grid_of_root[r]);
        }
        // A wired machine is on its pole; one with no wire hangs on the nearest cable in reach.
        let wired = |pos: IVec3| hooked.by_target.get(&pos).copied();
        let hang = |pos: IVec3| wired(pos).or_else(|| nearest_of(poles, pos, true));
        Power {
            pole_grid,
            wires,
            gen_pole: gens.iter().map(|g| hang(g.pos)).collect(),
            miner_pole: miners.iter().map(|m| hang(m.pos)).collect(),
            process_pole: processors
                .iter()
                .map(|p| if takes_pole(p) { wired(p.pos).or_else(|| hang_cable(poles, &p.cells())) } else { None })
                .collect(),
            lab_pole: labs.iter().map(|l| hang(l.pos)).collect(),
            pipe_pole: pipework.iter().map(|p| if p.part == Part::Pump { hang(p.pos) } else { None }).collect(),
            quarry_pole: quarries.iter().map(|q| hang(q.pos)).collect(),
            supply: vec![0; grids as usize],
            demand: vec![0; grids as usize],
            capacity: vec![0; grids as usize],
        }
    }

    /// One tick of supply and demand: draws on generators as needed and records each grid's totals.
    #[allow(clippy::too_many_arguments)]
    pub fn balance(
        &mut self,
        gens: &mut [Generator],
        miners: &[Miner],
        processors: &mut [Processor],
        labs: &[Lab],
        pipework: &[Pipework],
        quarries: &[Quarry],
        research: &Research,
        unlocked: &[bool],
        tick: u64,
    ) {
        self.supply.iter_mut().for_each(|s| *s = 0);
        self.demand.iter_mut().for_each(|d| *d = 0);
        self.capacity.iter_mut().for_each(|c| *c = 0);
        for (m, p) in miners.iter().zip(&self.miner_pole) {
            if let Some(&p) = p.as_ref().filter(|_| m.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += m.stats().power;
            }
        }
        for (m, p) in processors.iter().zip(&self.process_pole) {
            if let Some(&p) = p.as_ref().filter(|_| m.wants_power(unlocked)) {
                self.demand[self.pole_grid[p as usize] as usize] += m.power();
            }
        }
        for (l, p) in labs.iter().zip(&self.lab_pole) {
            if let Some(&p) = p.as_ref().filter(|_| l.wants_power(research)) {
                self.demand[self.pole_grid[p as usize] as usize] += l.stats().power;
            }
        }
        for (m, p) in pipework.iter().zip(&self.pipe_pole) {
            if let Some(&p) = p.as_ref().filter(|_| m.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += m.pump_stats().power;
            }
        }
        for (q, p) in quarries.iter().zip(&self.quarry_pole) {
            if let Some(&p) = p.as_ref().filter(|_| q.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += q.stats().power;
            }
        }
        // The sun and the accumulators come before any fuel is burned.
        run_renewables(self, processors, tick);
        for (g, p) in gens.iter_mut().zip(&self.gen_pole) {
            g.output = 0;
            let Some(p) = *p else { continue };
            let grid = self.pole_grid[p as usize] as usize;
            let tier = g.stats();
            let want = (self.demand[grid] - self.supply[grid].min(self.demand[grid])).min(tier.power);
            if want > g.energy {
                let s = g.fuel.slots[0];
                if let Some(kj) = fuel_energy(s.item).filter(|_| !s.is_empty()) {
                    g.fuel.take(0, 1);
                    g.energy += kj * tier.yield_percent / 100 * TICK_RATE;
                }
            }
            if g.energy > 0 || g.fuel.total() > 0 {
                self.capacity[grid] += tier.power;
            }
            g.output = want.min(g.energy);
            g.energy -= g.output;
            self.supply[grid] += g.output;
        }
        // Steam turbines fill what the generators left.
        for t in 0..processors.len() {
            if processors[t].energy() != Energy::Turbine {
                continue;
            }
            let Some(p) = self.process_pole[t] else { continue };
            let grid = self.pole_grid[p as usize] as usize;
            let want = self.demand[grid] - self.supply[grid].min(self.demand[grid]);
            let (given, could) = run_turbine(processors, t, want);
            self.supply[grid] += given;
            self.capacity[grid] += could;
        }
    }

    /// Speed in thousandths for a machine hanging on `pole` (0 with no pole or no power).
    pub fn speed(&self, pole: Option<u32>) -> u32 {
        let Some(grid) = pole.map(|p| self.pole_grid[p as usize] as usize) else { return 0 };
        match (self.supply[grid], self.demand[grid]) {
            (_, 0) => FULL_SPEED,
            (s, d) => (s * FULL_SPEED / d).min(FULL_SPEED),
        }
    }

    /// One line about the grid `pole` is on, for readouts.
    pub fn grid_line(&self, pole: Option<u32>) -> String {
        let Some(grid) = pole.map(|p| self.pole_grid[p as usize] as usize) else {
            return NOT_WIRED.to_string();
        };
        let (s, d, c) = (self.supply[grid], self.demand[grid], self.capacity[grid]);
        if s < d {
            format!("Grid short: {s} of {d} kW needed · machines at {}%", self.speed(pole) / 10)
        } else {
            format!("Grid: {d} kW used of {c} kW")
        }
    }
}

impl Factory {
    /// Wires between linked poles and from each powered machine to its pole, as thin sagging segments.
    pub(super) fn write_wires(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        let top = |pos: IVec3, h: f64| pos.as_vec3() + Vec3::new(0.5, h, 0.5);
        // A pole's wire leaves its crossarm, a cable's its centre.
        let pole = |i: u32| {
            let p = &self.poles[i as usize];
            top(p.pos, if p.is_cable() { 0.5 } else { 0.92 })
        };
        let mut span = |a: Vec3, b: Vec3| {
            let mid = (a + b) * 0.5 - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z <= range * range {
                wire(out, a - eye, b - eye);
            }
        };
        // Cables touching each other need no wire: their models meet.
        let bare = |i: u32| self.poles[i as usize].is_cable();
        for &(i, j) in self.power.wires.iter().filter(|&&(i, j)| !(bare(i) && bare(j))) {
            span(pole(i), pole(j));
        }
        // A multi-block processor's wire goes to its cell nearest the pole.
        let hook = |(p, pole): (&Processor, &Option<u32>)| {
            let d = |c: &IVec3| pole.map_or(0, |i| dist2(self.poles[i as usize].pos, *c));
            p.cells().into_iter().min_by_key(d).unwrap_or(p.pos)
        };
        let hookups = [
            (&self.power.gen_pole, self.generators.iter().map(|g| g.pos).collect::<Vec<_>>()),
            (&self.power.miner_pole, self.miners.iter().map(|m| m.pos).collect()),
            (&self.power.process_pole, self.processors.iter().zip(&self.power.process_pole).map(hook).collect()),
            (&self.power.lab_pole, self.labs.iter().map(|l| l.pos).collect()),
            (&self.power.pipe_pole, self.pipework.iter().map(|p| p.pos).collect()),
            (&self.power.quarry_pole, self.quarries.iter().map(|q| q.pos).collect()),
        ];
        for (poles, positions) in hookups {
            for (p, pos) in poles.iter().zip(positions) {
                if let Some(p) = *p {
                    span(pole(p), top(pos, 0.7));
                }
            }
        }
    }
}

/// Union-find root with path halving.
fn root(parent: &mut [u32], mut i: u32) -> u32 {
    while parent[i as usize] != i {
        parent[i as usize] = parent[parent[i as usize] as usize];
        i = parent[i as usize];
    }
    i
}

/// A wire from `a` to `b` (camera-relative) as short flat segments that sag in the middle.
pub(super) fn wire(out: &mut Vec<f32>, a: Vec3, b: Vec3) {
    let d = b - a;
    let len = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
    let n = ((len * 2.0).ceil() as usize).max(1);
    let yaw = d.x.atan2(-d.z) as f32;
    let sag = 0.04 * len;
    for i in 0..n {
        let t = (i as f64 + 0.5) / n as f64;
        let p = a + d * t - Vec3::new(0.0, sag * 4.0 * t * (1.0 - t), 0.0);
        let c = [tex::BELT_TOP; 3];
        push_box(out, p, yaw, [0.035, 0.035, (len / n as f64) as f32 + 0.02], 0.0, c, false);
    }
}

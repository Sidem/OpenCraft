//! Electric power: poles, the grids they form, and how supply meets demand each tick.
//!
//! - A pole links to every pole within `WIRE_RANGE`; each connected group is a grid. A generator or a
//!   powered machine joins the grid of the nearest pole within `POLE_REACH` of any of its cells (ties:
//!   the lower pole index). All of this is derived from positions (`rebuild`, run by `relink`), never saved.
//! - Each tick `balance` adds up what each grid's machines need in kW (a miner while it drills, a
//!   electric processor (a constructor) while it works, a splitter or filter while it holds an item, a lab while it researches,
//!   a pump while it has room, a quarry while it digs), then takes it from the generators in list
//!   order, each up to `GENERATOR_POWER`. A grid short of power runs its machines at `speed` =
//!   supply / demand.
//! - Energy: a generator holds the energy of the fuel it lit, in kW·ticks (1 kJ = `TICK_RATE`
//!   kW·ticks), and gives only what is drawn, so fuel lasts exactly as long as the load allows. It
//!   lights the next item (`recipes::fuel_energy`) when what it holds can't cover this tick.
//!
//! Consumers: miners, electric processors, splitters, filters, labs, pumps and quarries. Burner
//! processors (the smelter) burn their own fuel. To power a new machine: its `*_pole` list here (filled in `rebuild`), its demand in
//! `balance`, and a speed argument to its `step`.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};
use crate::recipes::fuel_energy;
use crate::research::Research;
use crate::TICK_RATE;

use super::generator::Generator;
use super::lab::Lab;
use super::miner::Miner;
use super::pipes::{Part, Pipework};
use super::process::{Energy, Processor};
use super::quarry::Quarry;
use super::render::push_box;
use super::router::Router;
use super::{Factory, Machine};

/// The most one generator supplies, in kW.
pub const GENERATOR_POWER: u32 = 60;
/// What a splitter or filter draws while it holds an item, in kW.
pub const ROUTER_POWER: u32 = 1;
/// What a researching lab draws, in kW.
pub const LAB_POWER: u32 = 10;
/// What a pump draws while it has room for water, in kW.
pub const PUMP_POWER: u32 = 5;
/// What a digging quarry draws, in kW.
pub const QUARRY_POWER: u32 = 10;
/// Poles closer than this (between cell centres, in blocks) are wired together.
pub const WIRE_RANGE: i32 = 10;
/// Machines this close to a pole (between cell centres) join its grid.
pub const POLE_REACH: i32 = 5;
/// Full speed, in thousandths.
pub const FULL_SPEED: u32 = 1000;

/// A power pole: nothing but a position; its links are derived.
pub struct Pole {
    pub pos: IVec3,
}

/// The grids, derived from positions by `rebuild`, plus last tick's supply and demand per grid.
#[derive(Default)]
pub(crate) struct Power {
    /// The grid of each pole.
    pub pole_grid: Vec<u32>,
    /// Pole pairs that are wired together (lower index first).
    pub wires: Vec<(u32, u32)>,
    /// The pole each generator, miner, electric processor, router and lab hangs on, if any is in reach.
    pub gen_pole: Vec<Option<u32>>,
    pub miner_pole: Vec<Option<u32>>,
    pub process_pole: Vec<Option<u32>>,
    pub router_pole: Vec<Option<u32>>,
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
    /// Recomputes grids and hookups from the machines' positions.
    #[allow(clippy::too_many_arguments)]
    pub fn rebuild(
        poles: &[Pole],
        gens: &[Generator],
        miners: &[Miner],
        processors: &[Processor],
        routers: &[Router],
        labs: &[Lab],
        pipework: &[Pipework],
        quarries: &[Quarry],
    ) -> Power {
        let n = poles.len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        let mut wires = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                if dist2(poles[i].pos, poles[j].pos) <= WIRE_RANGE * WIRE_RANGE {
                    wires.push((i as u32, j as u32));
                    let (a, b) = (root(&mut parent, i as u32), root(&mut parent, j as u32));
                    parent[a.max(b) as usize] = a.min(b);
                }
            }
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
        let hang = |pos: IVec3| nearest_pole(poles, pos);
        Power {
            pole_grid,
            wires,
            gen_pole: gens.iter().map(|g| hang(g.pos)).collect(),
            miner_pole: miners.iter().map(|m| hang(m.pos)).collect(),
            process_pole: processors
                .iter()
                .map(|p| if p.spec.energy == Energy::Electric { hang_any(poles, &p.cells()) } else { None })
                .collect(),
            router_pole: routers.iter().map(|r| hang(r.pos)).collect(),
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
        processors: &[Processor],
        routers: &[Router],
        labs: &[Lab],
        pipework: &[Pipework],
        quarries: &[Quarry],
        research: &Research,
        unlocked: &[bool],
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
        for (r, p) in routers.iter().zip(&self.router_pole) {
            if let Some(&p) = p.as_ref().filter(|_| r.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += ROUTER_POWER;
            }
        }
        for (l, p) in labs.iter().zip(&self.lab_pole) {
            if let Some(&p) = p.as_ref().filter(|_| l.wants_power(research)) {
                self.demand[self.pole_grid[p as usize] as usize] += LAB_POWER;
            }
        }
        for (m, p) in pipework.iter().zip(&self.pipe_pole) {
            if let Some(&p) = p.as_ref().filter(|_| m.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += PUMP_POWER;
            }
        }
        for (q, p) in quarries.iter().zip(&self.quarry_pole) {
            if let Some(&p) = p.as_ref().filter(|_| q.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += QUARRY_POWER;
            }
        }
        for (g, p) in gens.iter_mut().zip(&self.gen_pole) {
            g.output = 0;
            let Some(p) = *p else { continue };
            let grid = self.pole_grid[p as usize] as usize;
            let want = (self.demand[grid] - self.supply[grid].min(self.demand[grid])).min(GENERATOR_POWER);
            if want > g.energy {
                let s = g.fuel.slots[0];
                if let Some(kj) = fuel_energy(s.item).filter(|_| !s.is_empty()) {
                    g.fuel.take(0, 1);
                    g.energy += kj * TICK_RATE;
                }
            }
            if g.energy > 0 || g.fuel.total() > 0 {
                self.capacity[grid] += GENERATOR_POWER;
            }
            g.output = want.min(g.energy);
            g.energy -= g.output;
            self.supply[grid] += g.output;
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
            return format!("Not connected: place a power pole within {POLE_REACH} blocks");
        };
        let (s, d, c) = (self.supply[grid], self.demand[grid], self.capacity[grid]);
        if s < d {
            format!("Grid short: {s} of {d} kW needed · machines at {}%", self.speed(pole) / 10)
        } else {
            format!("Grid: {d} kW used of {c} kW")
        }
    }
}

impl Machine for Pole {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
    }

    fn read_state(r: &mut ByteReader) -> Option<Pole> {
        Some(Pole { pos: r.ivec3()? })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return "Power pole".to_string();
        }
        let me = f.poles.iter().position(|p| p.pos == self.pos).map(|i| i as u32);
        let grid = me.map(|i| f.power.pole_grid[i as usize]);
        let on = |p: &Option<u32>| p.is_some_and(|p| Some(f.power.pole_grid[p as usize]) == grid);
        let poles = f.power.pole_grid.iter().filter(|&&g| Some(g) == grid).count();
        let gens = f.power.gen_pole.iter().filter(|p| on(p)).count();
        let machines = f.power.miner_pole.iter().chain(&f.power.process_pole).chain(&f.power.router_pole);
        let machines = machines.chain(&f.power.lab_pole).chain(&f.power.pipe_pole).chain(&f.power.quarry_pole);
        let machines = machines.filter(|p| on(p)).count();
        format!(
            "{}\n{poles} poles, {gens} generators, {machines} machines on this grid\nLinks to poles within \
             {WIRE_RANGE} blocks and powers machines within {POLE_REACH}",
            f.power.grid_line(me)
        )
    }

    /// A steel post with a crossarm and two copper insulators.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        push_box(out, rel + Vec3::new(0.0, -0.42, 0.0), 0.0, [0.38, 0.15, 0.38], 0.0, [tex::FRAME; 3], false);
        push_box(out, rel + Vec3::new(0.0, -0.04, 0.0), 0.0, [0.13, 0.85, 0.13], 0.0, [tex::FRAME; 3], true);
        push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), 0.0, [0.72, 0.08, 0.11], 0.0, [tex::FRAME; 3], false);
        for x in [-0.28, 0.28] {
            let c = [tex::COPPER_WIRE; 3];
            push_box(out, rel + Vec3::new(x, 0.45, 0.0), 0.0, [0.12, 0.17, 0.12], 0.0, [tex::FLASK_GLASS; 3], false);
            push_box(out, rel + Vec3::new(x, 0.55, 0.0), 0.0, [0.08, 0.05, 0.08], 0.0, c, false);
        }
    }
}

impl Factory {
    /// Wires between linked poles and from each powered machine to its pole, as thin sagging segments.
    pub(super) fn write_wires(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        let top = |pos: IVec3, h: f64| pos.as_vec3() + Vec3::new(0.5, h, 0.5);
        let pole = |i: u32| top(self.poles[i as usize].pos, 0.92);
        let mut span = |a: Vec3, b: Vec3| {
            let mid = (a + b) * 0.5 - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z <= range * range {
                wire(out, a - eye, b - eye);
            }
        };
        for &(i, j) in &self.power.wires {
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
            (&self.power.router_pole, self.routers.iter().map(|r| r.pos).collect()),
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

/// Squared distance between two cell centres.
fn dist2(a: IVec3, b: IVec3) -> i32 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

/// The nearest pole within `POLE_REACH` of `pos` (the lower index on ties).
fn nearest_pole(poles: &[Pole], pos: IVec3) -> Option<u32> {
    let mut best: Option<(i32, u32)> = None;
    for (i, p) in poles.iter().enumerate() {
        let d = dist2(p.pos, pos);
        if d <= POLE_REACH * POLE_REACH && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i as u32));
        }
    }
    best.map(|b| b.1)
}

/// The pole nearest any of `cells` within `POLE_REACH` (a machine several cells big hangs on it).
fn hang_any(poles: &[Pole], cells: &[IVec3]) -> Option<u32> {
    let near = |&c: &IVec3| nearest_pole(poles, c).map(|i| (dist2(poles[i as usize].pos, c), i));
    cells.iter().filter_map(near).min().map(|best| best.1)
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
fn wire(out: &mut Vec<f32>, a: Vec3, b: Vec3) {
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

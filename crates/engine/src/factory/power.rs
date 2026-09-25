//! Electric power: poles, the grids they form, and how supply meets demand each tick.
//!
//! - A pole links to every pole within `WIRE_RANGE`; each connected group is a grid. A generator or a
//!   powered machine joins the grid of the nearest pole within `POLE_REACH` (ties: the lower pole
//!   index). All of this is derived from positions (`rebuild`, run by `relink`), never saved.
//! - Each tick `balance` adds up what each grid's machines need (`CONSTRUCTOR_POWER` while one works,
//!   `ROUTER_POWER` while a splitter or filter holds an item, `LAB_POWER` while a lab researches), then burns generators in list order
//!   until supply covers demand. A grid short of power runs its machines at `speed` = supply / demand.
//!   Generators burn only while their grid needs power.
//!
//! Consumers: the constructor, splitter, filter and lab. The miner and smelter are the unpowered burner
//! tier. To power a new machine: its `*_pole` list here (filled in `rebuild`), its demand in
//! `balance`, and a speed argument to its `step`.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};
use crate::recipes::burn_time;
use crate::research::Research;

use super::constructor::Constructor;
use super::generator::Generator;
use super::lab::Lab;
use super::render::push_box;
use super::router::Router;
use super::{ticks, Factory, Machine};

/// What one burning generator supplies, in kW.
pub const GENERATOR_POWER: u32 = 60;
/// What a working constructor draws, in kW.
pub const CONSTRUCTOR_POWER: u32 = 15;
/// What a splitter or filter draws while it holds an item, in kW.
pub const ROUTER_POWER: u32 = 1;
/// What a researching lab draws, in kW.
pub const LAB_POWER: u32 = 10;
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
    /// The pole each generator, constructor, router and lab hangs on, if any is in reach.
    pub gen_pole: Vec<Option<u32>>,
    pub constructor_pole: Vec<Option<u32>>,
    pub router_pole: Vec<Option<u32>>,
    pub lab_pole: Vec<Option<u32>>,
    /// Per grid, last tick: kW supplied and kW wanted.
    pub supply: Vec<u32>,
    pub demand: Vec<u32>,
}

impl Power {
    /// Recomputes grids and hookups from the machines' positions.
    pub fn rebuild(
        poles: &[Pole],
        gens: &[Generator],
        constructors: &[Constructor],
        routers: &[Router],
        labs: &[Lab],
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
            constructor_pole: constructors.iter().map(|c| hang(c.pos)).collect(),
            router_pole: routers.iter().map(|r| hang(r.pos)).collect(),
            lab_pole: labs.iter().map(|l| hang(l.pos)).collect(),
            supply: vec![0; grids as usize],
            demand: vec![0; grids as usize],
        }
    }

    /// One tick of supply and demand: burns generators as needed and records each grid's totals.
    pub fn balance(
        &mut self,
        gens: &mut [Generator],
        constructors: &[Constructor],
        routers: &[Router],
        labs: &[Lab],
        research: &Research,
    ) {
        self.supply.iter_mut().for_each(|s| *s = 0);
        self.demand.iter_mut().for_each(|d| *d = 0);
        for (c, p) in constructors.iter().zip(&self.constructor_pole) {
            if let Some(&p) = p.as_ref().filter(|_| c.wants_power()) {
                self.demand[self.pole_grid[p as usize] as usize] += CONSTRUCTOR_POWER;
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
        for (g, p) in gens.iter_mut().zip(&self.gen_pole) {
            g.running = false;
            let Some(p) = *p else { continue };
            let grid = self.pole_grid[p as usize] as usize;
            if self.supply[grid] >= self.demand[grid] {
                continue;
            }
            if g.burn == 0 {
                let s = g.fuel.slots[0];
                if let Some(secs) = burn_time(s.item).filter(|_| !s.is_empty()) {
                    g.fuel.take(0, 1);
                    g.burn = ticks(secs);
                }
            }
            if g.burn > 0 {
                g.burn -= 1;
                g.running = true;
                self.supply[grid] += GENERATOR_POWER;
            }
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
        let (s, d) = (self.supply[grid], self.demand[grid]);
        let speed = self.speed(pole) / 10;
        format!("Grid: {s} kW supplied, {d} kW needed · machines at {speed}%")
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
        let machines = f.power.constructor_pole.iter().chain(&f.power.router_pole).chain(&f.power.lab_pole);
        let machines = machines.filter(|p| on(p)).count();
        format!(
            "{}\n{poles} poles, {gens} generators, {machines} machines on this grid\nLinks to poles within \
             {WIRE_RANGE} blocks and powers machines within {POLE_REACH}",
            f.power.grid_line(me)
        )
    }

    /// A steel post with a crossarm and two copper insulators.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        push_box(out, rel + Vec3::new(0.0, -0.05, 0.0), 0.0, [0.12, 0.9, 0.12], 0.0, [tex::FRAME; 3], true);
        push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), 0.0, [0.6, 0.06, 0.08], 0.0, [tex::FRAME; 3], true);
        for x in [-0.25, 0.25] {
            let c = [tex::COPPER_WIRE; 3];
            push_box(out, rel + Vec3::new(x, 0.42, 0.0), 0.0, [0.07, 0.1, 0.07], 0.0, c, false);
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
        let hookups = [
            (&self.power.gen_pole, self.generators.iter().map(|g| g.pos).collect::<Vec<_>>()),
            (&self.power.constructor_pole, self.constructors.iter().map(|c| c.pos).collect()),
            (&self.power.router_pole, self.routers.iter().map(|r| r.pos).collect()),
            (&self.power.lab_pole, self.labs.iter().map(|l| l.pos).collect()),
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

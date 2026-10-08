//! The data grid (Milestone 11): fibre nodes, and compute (TF) carried over them the way power is carried over poles.
//!
//! - A [`Node`] is a block with no state but its place. Nodes within [`LINK`] blocks of each other are one grid by
//!   themselves (like cables: no hand wiring), and a processor whose spec has a non-zero `compute` hangs on the nearest
//!   node within [`REACH`] of any of its cells. A positive `compute` is a producer (TF at full power, scaled by the
//!   power it gets), a negative one a consumer (it wants that many TF while it wants power).
//! - [`Data`] is derived at every relink (`Data::rebuild`, never saved) and balanced every tick (`Data::balance`, after
//!   `Power::balance`): per grid, supply against demand, and `satisfaction` in thousandths. A consumer runs at its power
//!   speed times its grid's satisfaction, and not at all with no node in reach (`update` in `mod.rs`).
//! - `write_fibre` draws the thin light lines between linked nodes and from each machine to its node. Presentation only.
//!
//! To add a producer or consumer: a processor spec with a `compute` number (`process/specs.rs`); nothing here changes.

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::pole::dist2;
use super::power::{wire, Power, FULL_SPEED};
use super::process::{Processor, Status};
use super::render::push_box;
use super::{Factory, Machine};

/// Nodes this close (blocks between cell centres) are one grid.
pub const LINK: i32 = 12;
/// A producer or consumer this close to a node (any of its cells) hangs on it.
pub const REACH: i32 = 5;

/// What a processor with no node in reach says.
pub const NO_NODE: &str = "No data link: put a fibre node within 5 blocks";

pub struct Node {
    pub pos: IVec3,
}

/// The data grids, derived by `rebuild`, plus last tick's supply and demand per grid, in TF.
#[derive(Default)]
pub(crate) struct Data {
    /// The grid of each node.
    pub node_grid: Vec<u32>,
    /// Node pairs that are linked (lower index first).
    pub links: Vec<(u32, u32)>,
    /// The node each processor hangs on, by processor index (`None`: no compute, or no node in reach).
    pub process_node: Vec<Option<u32>>,
    pub supply: Vec<u32>,
    pub demand: Vec<u32>,
}

impl Data {
    /// Numbers the grids (in order of their lowest node) and hangs every producer and consumer on a node.
    pub(super) fn rebuild(nodes: &[Node], processors: &[Processor]) -> Data {
        let n = nodes.len();
        let mut parent: Vec<u32> = (0..n as u32).collect();
        let mut links = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                if dist2(nodes[i].pos, nodes[j].pos) <= LINK * LINK {
                    links.push((i as u32, j as u32));
                    let (a, b) = (root(&mut parent, i as u32), root(&mut parent, j as u32));
                    parent[a.max(b) as usize] = a.min(b);
                }
            }
        }
        let mut grid_of_root = vec![u32::MAX; n];
        let mut grids = 0;
        let mut node_grid = Vec::with_capacity(n);
        for i in 0..n as u32 {
            let r = root(&mut parent, i) as usize;
            if grid_of_root[r] == u32::MAX {
                grid_of_root[r] = grids;
                grids += 1;
            }
            node_grid.push(grid_of_root[r]);
        }
        let hang = |p: &Processor| {
            if p.spec.compute == 0 {
                return None;
            }
            let near = |c: IVec3| {
                let best = nodes.iter().enumerate().map(|(i, nd)| (dist2(nd.pos, c), i as u32));
                best.filter(|&(d, _)| d <= REACH * REACH).min()
            };
            p.cells().into_iter().filter_map(near).min().map(|(_, i)| i)
        };
        Data {
            node_grid,
            links,
            process_node: processors.iter().map(hang).collect(),
            supply: vec![0; grids as usize],
            demand: vec![0; grids as usize],
        }
    }

    /// One tick of supply and demand per grid. A producer gives its `compute` scaled by the power speed it gets; a
    /// consumer wants its `compute` (as a positive number) while it wants power.
    pub(super) fn balance(
        &mut self,
        processors: &[Processor],
        power: &Power,
        unlocked: &[bool],
        research: &crate::research::Research,
    ) {
        self.supply.iter_mut().for_each(|s| *s = 0);
        self.demand.iter_mut().for_each(|d| *d = 0);
        for (i, p) in processors.iter().enumerate() {
            let Some(node) = self.process_node[i] else { continue };
            let grid = self.node_grid[node as usize] as usize;
            if p.spec.compute > 0 {
                if p.status == Status::Overheated {
                    continue; // a tripped datacenter gives nothing
                }
                let speed = power.speed(power.process_pole[i]);
                self.supply[grid] += p.spec.compute as u32 * speed / FULL_SPEED;
            } else if p.wants_power(unlocked, research) {
                self.demand[grid] += p.spec.compute.unsigned_abs();
            }
        }
    }

    /// How much of its demand the grid of `node` meets, in thousandths (0 with no node, full with nothing wanted).
    pub fn satisfaction(&self, node: Option<u32>) -> u32 {
        let Some(grid) = node.map(|n| self.node_grid[n as usize] as usize) else { return 0 };
        match (self.supply[grid], self.demand[grid]) {
            (_, 0) => FULL_SPEED,
            (s, d) => (s as u64 * FULL_SPEED as u64 / d as u64).min(FULL_SPEED as u64) as u32,
        }
    }
}

impl Machine for Node {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
    }

    fn read_state(r: &mut ByteReader) -> Option<Node> {
        Some(Node { pos: r.ivec3()? })
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        let Some(me) = f.nodes.iter().position(|n| n.pos == self.pos).filter(|_| !f.dirty) else {
            return "Fibre node".to_string();
        };
        let grid = f.data.node_grid[me];
        let nodes = f.data.node_grid.iter().filter(|&&g| g == grid).count();
        let on = |p: &Option<u32>| p.is_some_and(|n| f.data.node_grid[n as usize] == grid);
        let here = f.processors.iter().zip(&f.data.process_node).filter(|(_, n)| on(n));
        let makers = here.clone().filter(|(p, _)| p.spec.compute > 0).count();
        let users = here.count() - makers;
        let (s, d) = (f.data.supply[grid as usize], f.data.demand[grid as usize]);
        format!(
            "Data grid: {s} TF supplied, {d} TF wanted\n{nodes} nodes, {makers} producers, {users} consumers on this grid\n\
             Nodes within {LINK} blocks join; machines within {REACH} blocks hang on the nearest node"
        )
    }

    /// A short post on a base, a cap and a lit tip.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        let side = [tex::NODE_SIDE; 3];
        push_box(out, rel + Vec3::new(0.0, -0.4, 0.0), 0.0, [0.52, 0.2, 0.52], 0.0, [tex::FRAME; 3], false);
        push_box(out, rel + Vec3::new(0.0, -0.05, 0.0), 0.0, [0.2, 0.5, 0.2], 0.0, side, false);
        push_box(out, rel + Vec3::new(0.0, 0.26, 0.0), 0.0, [0.36, 0.14, 0.36], 0.0, [tex::NODE_TOP; 3], false);
        push_box(out, rel + Vec3::new(0.0, 0.42, 0.0), 0.0, [0.09, 0.2, 0.09], 0.0, [tex::FIBRE; 3], false);
    }
}

impl Factory {
    /// The data line of the processor `i`, for machines that make or use compute.
    pub(super) fn compute_line(&self, i: usize) -> Option<String> {
        let p = &self.processors[i];
        if p.spec.compute == 0 || self.dirty {
            return None;
        }
        let Some(node) = self.data.process_node[i] else { return Some(NO_NODE.to_string()) };
        let grid = self.data.node_grid[node as usize] as usize;
        let (s, d) = (self.data.supply[grid], self.data.demand[grid]);
        Some(match p.spec.compute > 0 {
            true => format!("Data grid: {s} TF supplied, {d} TF wanted"),
            false => {
                format!("Data grid: {} TF wanted of {s} TF supplied · {}%", d, self.data.satisfaction(Some(node)) / 10)
            }
        })
    }

    /// Thin light lines between linked nodes and from each producer and consumer to its node, within `range`.
    pub(super) fn write_fibre(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        let top = |pos: IVec3, h: f64| pos.as_vec3() + Vec3::new(0.5, h, 0.5);
        let mut span = |a: Vec3, b: Vec3| {
            let mid = (a + b) * 0.5 - eye;
            if mid.x * mid.x + mid.y * mid.y + mid.z * mid.z <= range * range {
                wire(out, a - eye, b - eye, tex::FIBRE);
            }
        };
        for &(i, j) in &self.data.links {
            span(top(self.nodes[i as usize].pos, 0.9), top(self.nodes[j as usize].pos, 0.9));
        }
        for (p, node) in self.processors.iter().zip(&self.data.process_node) {
            let Some(node) = *node else { continue };
            let at = self.nodes[node as usize].pos;
            let cell = p.cells().into_iter().min_by_key(|&c| dist2(at, c)).unwrap_or(p.pos);
            span(top(at, 0.9), top(cell, 0.7));
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

#[cfg(test)]
mod tests;

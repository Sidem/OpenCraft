//! Bounded A* over walkable voxel feet positions. Routes avoid water, require body/head clearance,
//! allow one-block steps only, and never query generated or unloaded terrain. Execution uses player physics.

use crate::block::{self, AIR};
use crate::math::{IVec3, Vec3};
use crate::player::{Player, PlayerInput};
use crate::world::World;
use rustc_hash::FxHashMap;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const MAX_NODES: usize = 8192;
const MAX_ROUTE_DISTANCE: i32 = 96;
const ARRIVAL: f64 = 0.18;
const STUCK_SECONDS: f64 = 1.5;

#[derive(Default)]
pub(crate) struct Navigation {
    path: Vec<IVec3>,
    pub goal: Option<IVec3>,
    /// 0 idle, 1 moving, 2 arrived, 3 unreachable, 4 changed/blocked.
    pub status: u32,
    last: Option<Vec3>,
    stuck: f64,
}

impl Navigation {
    pub fn cancel(&mut self) {
        *self = Self::default();
    }
    pub fn reject(&mut self) {
        self.cancel();
        self.status = 3;
    }
    pub fn order(&mut self, path: Option<Vec<IVec3>>, goal: IVec3) {
        self.cancel();
        if let Some(path) = path {
            self.path = path;
            self.goal = Some(goal);
            self.status = 1;
        } else {
            self.status = 3;
        }
    }

    pub fn step(&mut self, body: &mut Player, world: &World, dt: f64) {
        body.input = PlayerInput::default();
        if self.status != 1 {
            return;
        }
        while let Some(next) = self.path.last() {
            let target = next.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
            let to = target - body.pos;
            if (to.x * to.x + to.z * to.z).sqrt() > ARRIVAL || to.y.abs() > 0.35 {
                break;
            }
            self.path.pop();
        }
        let Some(&next) = self.path.last() else {
            self.status = 2;
            return;
        };
        if !walkable(world, next) || body.flying || body.in_water {
            self.path.clear();
            self.status = 4;
            return;
        }
        if let Some(last) = self.last {
            let moved = body.pos - last;
            self.stuck = if moved.length() < 0.005 { self.stuck + dt } else { 0.0 };
            if self.stuck >= STUCK_SECONDS {
                self.path.clear();
                self.status = 4;
                return;
            }
        }
        self.last = Some(body.pos);
        let target = next.as_vec3() + Vec3::new(0.5, 0.0, 0.5);
        let to = target - body.pos;
        body.yaw = to.x.atan2(-to.z).rem_euclid(std::f64::consts::TAU);
        body.pitch = 0.0;
        body.input.forward = ((to.x * to.x + to.z * to.z).sqrt() * 2.0).min(1.0);
        body.input.jump = to.y > 0.35;
    }
}

struct Node {
    at: IVec3,
    parent: usize,
    cost: i32,
}

pub(super) fn find_path(world: &World, start: IVec3, goal: IVec3) -> Option<Vec<IVec3>> {
    if distance(start, goal) > MAX_ROUTE_DISTANCE || !walkable(world, start) || !walkable(world, goal) {
        return None;
    }
    let mut nodes = vec![Node { at: start, parent: 0, cost: 0 }];
    let mut seen = FxHashMap::default();
    seen.insert(start, 0);
    let mut open = BinaryHeap::from([Reverse((distance(start, goal), 0, 0usize))]);
    while let Some(Reverse((_, cost, index))) = open.pop() {
        if nodes[index].cost != cost {
            continue;
        }
        let at = nodes[index].at;
        if at == goal {
            let mut path = Vec::new();
            let mut i = index;
            while i != 0 {
                path.push(nodes[i].at);
                i = nodes[i].parent;
            }
            return Some(path);
        }
        for (dx, dz) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            for dy in [0, 1, -1] {
                let next = at + IVec3::new(dx, dy, dz);
                if !walkable(world, next) {
                    continue;
                }
                // Rising needs room above the current head for the jump; descending needs an open lip.
                let ceiling = if dy > 0 { at + IVec3::new(0, 2, 0) } else { next + IVec3::new(0, 2, 0) };
                if dy != 0 && !clear(world, ceiling) {
                    continue;
                }
                let next_cost = cost + 10 + dy.abs() * 4;
                let i = if let Some(&i) = seen.get(&next) {
                    if nodes[i].cost <= next_cost {
                        break;
                    }
                    nodes[i].cost = next_cost;
                    nodes[i].parent = index;
                    i
                } else {
                    if nodes.len() >= MAX_NODES {
                        return None;
                    }
                    let i = nodes.len();
                    nodes.push(Node { at: next, parent: index, cost: next_cost });
                    seen.insert(next, i);
                    i
                };
                open.push(Reverse((next_cost + distance(next, goal) * 10, next_cost, i)));
                break;
            }
        }
    }
    None
}

fn distance(a: IVec3, b: IVec3) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

pub(super) fn walkable(world: &World, feet: IVec3) -> bool {
    clear(world, feet)
        && clear(world, feet + IVec3::new(0, 1, 0))
        && world.get_block(feet + IVec3::new(0, -1, 0)).is_some_and(|b| block::SOLID[b as usize])
}

fn clear(world: &World, at: IVec3) -> bool {
    world.visible_column(at.x >> 5, at.z >> 5)
        && world.get_block(at).is_some_and(|b| !block::SOLID[b as usize] && !block::LIQUID[b as usize])
}

/// Sample a small footprint at the current surface level. Ignore plants, trees and factory roofs;
/// large cliffs settle at a capped speed, and pits follow the actual edited terrain.
pub(super) fn camera_ground(world: &World, focus: Vec3) -> Option<f64> {
    let mut samples = Vec::new();
    for (dx, dz) in [(0, 0), (-2, 0), (2, 0), (0, -2), (0, 2)] {
        let (x, z) = (focus.x.floor() as i32 + dx, focus.z.floor() as i32 + dz);
        for y in (0..256).rev() {
            let Some(b) = world.get_block(IVec3::new(x, y, z)) else {
                continue;
            };
            if b != AIR
                && (block::SOLID[b as usize] || block::LIQUID[b as usize])
                && b != block::LOG
                && b != block::LEAVES
                && crate::factory::machine(b).is_none()
                && b != block::MACHINE_PART
            {
                samples.push(f64::from(y + 1));
                break;
            }
        }
    }
    if samples.is_empty() {
        return None;
    }
    crate::math::sort_small_by_key(&mut samples, |h| *h as i32);
    Some(samples[samples.len() / 2])
}

//! The shapes `grow` builds from a `Kind` row, into its `Plot`: the trunk (leaning, narrowing, rooted into a slope
//! when wide), limbs (log blocks laid along a bending path in half-block steps, so each touches the last), leaf
//! clusters (ragged ellipsoids), and one builder per `Crown`. Every builder returns the height above the foot below
//! which the trunk stays clear of leaves. To add a crown: a `Crown` variant and its arm in `build`.

use std::f32::consts::TAU;

use super::kinds::{Crown, Kind, GOLDEN};
use super::{Plot, BELOW, REACH};
use crate::math::{hash3, unit, IVec3, Rng};

type V = [f32; 3];

/// The least bare trunk under a conifer's lowest whorl or bamboo's leaves: above where a player on the ground reaches, so the logs they
/// can chop stand clear of needles.
const CLEAR: i32 = 5;
/// How far a conifer's needles run on past its limb's wood, along the twigs (blocks): within a leaf's support reach.
const TWIG: f32 = 2.5;

/// Builds `k`'s tree of trunk `height` into `plot`; returns the height its bole is clear to.
pub(super) fn build(k: &Kind, plot: &mut Plot, height: i32, seed: u32) -> i32 {
    let mut b = Builder { rng: Rng::new(u64::from(seed) | 1 << 40), seed, foot: plot.foot() };
    match k.crown {
        Crown::Round => b.round(k, plot, height),
        Crown::Cone => b.cone(k, plot, height),
        Crown::Umbrella => b.umbrella(k, plot, height),
        Crown::Palm => b.palm(k, plot, height),
        Crown::Clump => b.clump(k, plot, height),
    }
}

struct Builder {
    rng: Rng,
    seed: u32,
    foot: IVec3,
}

impl Builder {
    fn f(&mut self) -> f32 {
        self.rng.next_f64() as f32
    }

    fn count(&mut self, (least, more): (u32, u32)) -> u32 {
        least + self.rng.below(more + 1)
    }

    /// Lays the trunk up to `top` blocks above the foot; returns its centre at each height (index 0: the foot).
    fn trunk(&mut self, k: &Kind, plot: &mut Plot, top: i32, height: i32) -> Vec<V> {
        let a = self.f() * TAU;
        let lean = k.lean * height as f32 * (0.6 + 0.4 * self.f());
        let (fx, fy, fz) = (self.foot.x as f32, self.foot.y as f32, self.foot.z as f32);
        let mut centres = vec![[fx, fy, fz]];
        for dy in 1..=top {
            let t = dy as f32 / height as f32;
            let off = lean * t * t;
            let (x, z) = (fx + off * a.cos(), fz + off * a.sin());
            let w = if t <= k.narrow || k.width == 1 {
                k.width
            } else {
                (k.width - ((t - k.narrow) / (1.0 - k.narrow) * k.width as f32) as i32).max(1)
            };
            let lo = |c: f32| (c - (w - 1) as f32 / 2.0 + 0.5).floor() as i32;
            let (x0, z0) = (lo(x), lo(z));
            let from = if dy == 1 && w > 1 { 1 - BELOW } else { dy };
            for y in from..=dy {
                for dz in 0..w {
                    for dx in 0..w {
                        plot.wood(IVec3::new(x0 + dx, self.foot.y + y, z0 + dz));
                    }
                }
            }
            let half = (w - 1) as f32 / 2.0;
            let centre = [x0 as f32 + half, fy + dy as f32, z0 as f32 + half];
            plot.trunk_at(centre[0], centre[2], half);
            centres.push(centre);
        }
        centres
    }

    /// Lays a limb of logs from `from` along `dir` (bending down by `droop`) for `len` blocks, stopping short of the
    /// plot's edge (`stop`: blocks of it kept for leaves, and a height); returns the points along it, half a
    /// block apart.
    fn limb(&mut self, plot: &mut Plot, from: V, dir: V, len: f32, droop: f32, stop: (f32, f32)) -> Vec<V> {
        let (room, ceil) = stop;
        let steps = (len * 2.0).round().max(1.0) as i32;
        let (mut p, mut d) = (from, dir);
        let mut path = vec![p];
        for _ in 0..steps {
            d[1] -= droop * 2.0 / steps as f32;
            d = unit3(d);
            let q = [p[0] + d[0] * 0.5, p[1] + d[1] * 0.5, p[2] + d[2] * 0.5];
            let edge = REACH as f32 - room;
            if (q[0] - self.foot.x as f32).abs() > edge || (q[2] - self.foot.z as f32).abs() > edge || q[1] > ceil {
                break;
            }
            p = q;
            plot.wood(cell(p));
            path.push(p);
        }
        path
    }

    /// Runs `path` on along its last heading for `len` more blocks without wood (a twig only leaves show), stopping
    /// `room` short of the plot's edge.
    fn twig(&self, path: &mut Vec<V>, len: f32, room: f32) {
        let n = path.len();
        if n < 2 {
            return;
        }
        let (a, b) = (path[n - 2], path[n - 1]);
        let d = unit3([b[0] - a[0], b[1] - a[1], b[2] - a[2]]);
        let edge = REACH as f32 - room;
        for i in 1..=(len * 2.0).round() as i32 {
            let q = [b[0] + d[0] * 0.5 * i as f32, b[1] + d[1] * 0.5 * i as f32, b[2] + d[2] * 0.5 * i as f32];
            if (q[0] - self.foot.x as f32).abs() > edge || (q[2] - self.foot.z as f32).abs() > edge {
                break;
            }
            path.push(q);
        }
    }

    /// A ragged ellipsoid of leaves round `c`: `rx` across, `ry` up, `fill` of its outer cells.
    fn cluster(&mut self, plot: &mut Plot, c: V, rx: f32, ry: f32, fill: f32) {
        let (ex, ey) = (rx.ceil() as i32, ry.ceil() as i32);
        let o = cell(c);
        for dy in -ey..=ey {
            for dz in -ex..=ex {
                for dx in -ex..=ex {
                    let p = o + IVec3::new(dx, dy, dz);
                    let q = [p.x as f32 - c[0], p.y as f32 - c[1], p.z as f32 - c[2]];
                    let d = (q[0] * q[0] + q[2] * q[2]) / (rx * rx) + q[1] * q[1] / (ry * ry);
                    if d <= 1.0 && (d < 0.45 || (unit(hash3(self.seed ^ 0x1EAF, p.x, p.y, p.z)) as f32) < fill) {
                        plot.leaf(p);
                    }
                }
            }
        }
    }

    /// Broadleaves: limbs out of the upper trunk, longer below, a cluster at each end, fork and the leader's top.
    fn round(&mut self, k: &Kind, plot: &mut Plot, height: i32) -> i32 {
        let trunk = self.trunk(k, plot, height, height);
        let lo = ((height as f32 * k.bole).round() as i32).min(height - 1).max(2);
        let n = self.count(k.limbs);
        let az0 = self.f() * TAU;
        let (rx, ry) = k.cluster;
        for i in 0..n {
            let s = ((i as f32 + 0.3 + 0.4 * self.f()) / n as f32).min(0.98);
            let dy = lo + (s * (height - lo) as f32) as i32;
            let az = az0 + i as f32 * GOLDEN;
            let angle = k.angle * (1.1 - 0.3 * s);
            let len = k.reach * height as f32 * (1.0 - 0.45 * s) * (0.85 + 0.3 * self.f());
            let path = self.limb(plot, trunk[dy as usize], aim(az, angle), len, k.droop, (rx + 0.5, f32::MAX));
            let end = *path.last().unwrap();
            if path.len() > 4 {
                let mid = path[path.len() * 3 / 5];
                self.cluster(plot, mid, rx * 0.6, ry * 0.7, k.fill);
            }
            if k.forks && path.len() > 5 {
                let side = if self.f() < 0.5 { -0.7 } else { 0.7 };
                let fork = self.limb(
                    plot,
                    path[path.len() / 2],
                    aim(az + side, angle * 0.9),
                    len * 0.5,
                    k.droop,
                    (rx, f32::MAX),
                );
                self.cluster(plot, raise(*fork.last().unwrap(), ry * 0.3), rx * 0.8, ry * 0.85, k.fill);
            }
            let r = 0.8 + 0.3 * self.f();
            self.cluster(plot, raise(end, ry * 0.3), rx * r, ry * r, k.fill);
        }
        self.cluster(plot, raise(trunk[height as usize], ry * 0.5), rx * 0.85, ry, k.fill);
        lo - 1
    }

    /// Conifers: whorls of near-level, drooping limbs all up the trunk, longest at the bottom, needles along them,
    /// and a spire at the top.
    fn cone(&mut self, k: &Kind, plot: &mut Plot, height: i32) -> i32 {
        let trunk = self.trunk(k, plot, height, height);
        let lo = ((height as f32 * k.bole).round() as i32).max(CLEAR).min(height - 2).max(1);
        let (rx, ry) = k.cluster;
        let mut dy = lo;
        let mut whorl = 0;
        while dy < height - 1 {
            let s = (dy - lo) as f32 / (height - lo) as f32;
            let m = 3 + self.rng.below(2);
            let az0 = whorl as f32 * GOLDEN + self.f();
            for j in 0..m {
                let az = az0 + j as f32 * TAU / m as f32 + 0.4 * self.f();
                let len = k.reach * height as f32 * (1.0 - s) * (0.8 + 0.4 * self.f()) + 0.8;
                let wood = (len - TWIG).max(len * 0.4);
                let droop = k.droop * wood / len;
                let mut path = self.limb(plot, trunk[dy as usize], aim(az, k.angle), wood, droop, (rx + 0.5, f32::MAX));
                self.twig(&mut path, len - wood, rx + 0.5);
                let r = rx * (0.7 + 0.3 * (1.0 - s));
                for p in path.iter().skip(1) {
                    self.cluster(plot, *p, r, ry, k.fill);
                }
            }
            whorl += 1;
            dy += 1 + self.rng.below(2) as i32;
        }
        let top = trunk[height as usize];
        self.cluster(plot, raise(top, -0.5), rx * 0.9, 1.6, k.fill);
        for up in 1..=2 {
            plot.leaf(cell(top) + IVec3::new(0, up, 0));
        }
        lo - 1
    }

    /// Acacias: a short trunk that splits into limbs rising and spreading to flat pads of leaves, one or two layers
    /// deep, each a block above or below its neighbour's, the limbs ending under them.
    fn umbrella(&mut self, k: &Kind, plot: &mut Plot, height: i32) -> i32 {
        let fork = ((height as f32 * k.bole).round() as i32).min(height - 2).max(2);
        let trunk = self.trunk(k, plot, fork, height);
        let canopy = self.foot.y as f32 + height as f32 + 1.0;
        let (rx, ry) = k.cluster;
        let n = self.count(k.limbs);
        let az0 = self.f() * TAU;
        for i in 0..n {
            let pad = canopy - (i % 2) as f32;
            let az = az0 + i as f32 * TAU / n as f32 + 0.5 * self.f();
            let from = trunk[(fork - self.rng.below(2) as i32) as usize];
            let len = k.reach * height as f32 * (0.8 + 0.3 * self.f());
            let ceil = pad - 0.6;
            let path = self.limb(plot, from, aim(az, k.angle), len, k.droop, (rx * 0.7, ceil));
            let mut ends = vec![*path.last().unwrap()];
            if k.forks && path.len() > 4 {
                let fork =
                    self.limb(plot, path[path.len() / 2], aim(az + 0.8, k.angle), len * 0.5, k.droop, (rx * 0.7, ceil));
                ends.push(*fork.last().unwrap());
            }
            for mut end in ends {
                while end[1] < ceil {
                    end[1] = (end[1] + 1.0).min(ceil);
                    plot.wood(cell(end));
                }
                let r = rx * (0.75 + 0.3 * self.f());
                self.cluster(plot, [end[0], pad + 0.5, end[2]], r, ry, k.fill);
            }
        }
        fork - 1
    }

    /// Palms: a bare, bending stem with a star of drooping fronds round its top.
    fn palm(&mut self, k: &Kind, plot: &mut Plot, height: i32) -> i32 {
        let trunk = self.trunk(k, plot, height, height);
        let top = trunk[height as usize];
        let n = self.count(k.limbs);
        let az0 = self.f() * TAU;
        for i in 0..n {
            let az = az0 + i as f32 * TAU / n as f32 + 0.3 * self.f();
            // Out of the stem's top a little upwards, then arching over and down.
            let mut d = aim(az, 0.6);
            let mut p = top;
            let mut last = cell(p);
            for step in 0..9 {
                d[1] -= 0.2 * (step as f32 / 4.0);
                d = unit3(d);
                p = [p[0] + d[0] * 0.5, p[1] + d[1] * 0.5, p[2] + d[2] * 0.5];
                let c = cell(p);
                // Keep the frond joined face to face, one axis at a time.
                let mut walk = last;
                for axis in 0..3 {
                    while get(walk, axis) != get(c, axis) {
                        walk = with(walk, axis, get(walk, axis) + (get(c, axis) - get(walk, axis)).signum());
                        plot.leaf(walk);
                    }
                }
                last = c;
            }
        }
        plot.leaf(cell(top) + IVec3::new(0, 1, 0));
        height - 1
    }

    /// Bamboo: a clump of thin culms two blocks apart (so none touch), leaves along their upper parts.
    fn clump(&mut self, k: &Kind, plot: &mut Plot, height: i32) -> i32 {
        let mut spots: Vec<(i32, i32)> =
            (0..9).map(|i| (i % 3 * 2 - 2, i / 3 * 2 - 2)).filter(|&s| s != (0, 0)).collect();
        for i in (1..spots.len()).rev() {
            let j = self.rng.below(i as u32 + 1) as usize;
            spots.swap(i, j);
        }
        spots.insert(0, (0, 0));
        let n = (self.count(k.limbs) as usize).min(spots.len());
        let mut base = height;
        for (i, &(sx, sz)) in spots.iter().take(n).enumerate() {
            let h = if i == 0 { height } else { (height as f32 * (0.6 + 0.4 * self.f())) as i32 };
            let lo = ((h as f32 * k.bole) as i32).max(CLEAR).min(h);
            base = base.min(lo);
            let foot = self.foot + IVec3::new(sx, 0, sz);
            for dy in (if i == 0 { 1 } else { 1 - BELOW })..=h {
                plot.wood(foot + IVec3::new(0, dy, 0));
            }
            if i == 0 {
                for _ in 1..=h {
                    plot.trunk_at(foot.x as f32, foot.z as f32, 0.0);
                }
            }
            for dy in lo..=h {
                for (j, f) in [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().enumerate() {
                    let p = foot + IVec3::new(f.0, dy, f.1);
                    if unit(hash3(self.seed ^ (0xBA0 + j as u32), p.x, p.y, p.z)) < 0.5 && !plot.is_wood(p) {
                        plot.leaf(p);
                        if dy % 2 == 0 {
                            plot.leaf(p + IVec3::new(f.0, -1, f.1));
                        }
                    }
                }
            }
            plot.leaf(foot + IVec3::new(0, h + 1, 0));
        }
        base - 1
    }
}

/// A unit direction `angle` from upright, turned `az` round.
fn aim(az: f32, angle: f32) -> V {
    [angle.sin() * az.cos(), angle.cos(), angle.sin() * az.sin()]
}

fn raise(p: V, by: f32) -> V {
    [p[0], p[1] + by, p[2]]
}

fn unit3(d: V) -> V {
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    [d[0] / l, d[1] / l, d[2] / l]
}

fn cell(p: V) -> IVec3 {
    IVec3::new(p[0].round() as i32, p[1].round() as i32, p[2].round() as i32)
}

fn get(p: IVec3, axis: usize) -> i32 {
    [p.x, p.y, p.z][axis]
}

fn with(p: IVec3, axis: usize, v: i32) -> IVec3 {
    match axis {
        0 => IVec3::new(v, p.y, p.z),
        1 => IVec3::new(p.x, v, p.z),
        _ => IVec3::new(p.x, p.y, v),
    }
}

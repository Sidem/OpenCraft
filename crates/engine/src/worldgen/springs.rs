//! Version 15 standalone highland springs: natural aquifer springs emerging from elevated hillsides and mountains,
//! each carving a continuous winding mountain stream with open rapids and waterfalls down the slope to a valley plunge
//! pool, river, or sea.
//!
//! Pure function of `seed` and unshaped terrain (`base_height`), cached per `SPRING_CELL`² cell so chunk generation
//! stays completely order-independent. Never runs when `layers.water < 15`.

use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashMap;

use crate::math::{hash2, smoothstep, unit, IVec3};

use super::{WorldGen, SEA_LEVEL};

/// One highland spring candidate per cell of this size (blocks).
const SPRING_CELL: i32 = 160;
const SPRING_MARGIN: i32 = 24;
/// Chance of a highland spring in an eligible elevated cell.
const SPRING_CHANCE: f64 = 0.55;
/// Minimum unshaped elevation for a standalone highland spring.
const SPRING_MIN_H: i32 = SEA_LEVEL + 18;
/// Keep highland springs and their streams clear of spawn.
const SPRING_SPAWN_DRY: f64 = 100.0;
/// Stream trace step, length bounds and minimum vertical drop.
const STEP: f64 = 3.0;
const MAX_STREAM_LEN: f64 = 96.0;
const MIN_STREAM_LEN: f64 = 16.0;
const MIN_STREAM_DROP: i32 = 5;
/// Shortest pool length between height steps.
const MIN_POOL: f64 = 7.0;
/// Channel geometry: bank width, ramp blend into hillside, and max reach beyond the stream axis.
const BANK: f64 = 2.5;
const MAX_RAMP: f64 = 18.0;
const REACH: f64 = BANK + MAX_RAMP + 6.0;

/// One standalone highland spring and its carved downhill stream.
pub(super) struct HighlandSpring {
    /// World coordinates of the `SPRING` aquifer block embedded into the hillside rim.
    pub head: IVec3,
    /// Winding downhill stream points `(x, z)` and cumulative arc-length `at`.
    pts: Vec<(f64, f64)>,
    at: Vec<f64>,
    /// Pool start distances along `at` and their water levels (monotonically non-increasing).
    ends: Vec<f64>,
    levels: Vec<i32>,
    /// Optional small plunge tarn radius at the downstream end (0 if it joins a river or the sea).
    end_pool_r: f64,
    lo: (f64, f64),
    hi: (f64, f64),
}

#[derive(Clone, Copy)]
struct SpringSample {
    sd: f64,
    width: f64,
    axis: f64,
    level: i32,
    bank: i32,
}

type SpringMap = FxHashMap<(i32, i32), Option<Rc<HighlandSpring>>>;

#[derive(Default)]
pub(super) struct SpringCache {
    cells: RefCell<SpringMap>,
}

impl HighlandSpring {
    fn reaches(&self, min: (f64, f64), max: (f64, f64)) -> bool {
        self.lo.0 - REACH < max.0 && self.hi.0 + REACH > min.0 && self.lo.1 - REACH < max.1 && self.hi.1 + REACH > min.1
    }

    fn nearest(&self, px: f64, pz: f64, limit: f64) -> Option<(f64, f64)> {
        if px < self.lo.0 - limit || px > self.hi.0 + limit || pz < self.lo.1 - limit || pz > self.hi.1 + limit {
            return None;
        }
        let mut best = limit;
        let mut along = None;
        for i in 0..self.pts.len() - 1 {
            let ((ax, az), (bx, bz)) = (self.pts[i], self.pts[i + 1]);
            let (dx, dz) = (bx - ax, bz - az);
            let len2 = dx * dx + dz * dz;
            let t = if len2 > 0.0 { (((px - ax) * dx + (pz - az) * dz) / len2).clamp(0.0, 1.0) } else { 0.0 };
            let d = ((px - ax - dx * t).powi(2) + (pz - az - dz * t).powi(2)).sqrt();
            if d < best {
                best = d;
                along = Some(self.at[i] + (self.at[i + 1] - self.at[i]) * t);
            }
        }
        along.map(|s| (best, s))
    }

    fn sample(&self, px: f64, pz: f64) -> Option<SpringSample> {
        let limit = REACH;
        let (axis, s) = self.nearest(px, pz, limit)?;
        let total = *self.at.last().unwrap_or(&1.0);
        let width = 1.6 + 0.9 * (s / total.max(1.0));
        let pool = |dist: f64| self.ends.partition_point(|&e| e <= dist).max(1) - 1;
        let i = pool(s);
        let level = self.levels[i];
        let bank = self.levels[pool((s - 3.5).max(0.0))];
        let mut out = SpringSample { sd: axis - width, width, axis, level, bank };

        if self.end_pool_r > 0.0 {
            let &(ex, ez) = self.pts.last().unwrap();
            let d = ((px - ex).powi(2) + (pz - ez).powi(2)).sqrt();
            let pool_sd = d - self.end_pool_r;
            if pool_sd < out.sd {
                let end_lvl = *self.levels.last().unwrap();
                out = SpringSample {
                    sd: pool_sd,
                    width: self.end_pool_r,
                    axis: d,
                    level: end_lvl,
                    bank: end_lvl.max(out.bank),
                };
            }
        }
        (out.sd < BANK + MAX_RAMP).then_some(out)
    }
}

impl WorldGen {
    /// Version 15: reshapes ground height `h` for any nearby standalone highland spring stream.
    pub(super) fn shape_springs(&self, x: i32, z: i32, h: i32) -> i32 {
        let Some(s) = self.spring_sample(x, z) else { return h };
        if s.sd < 0.0 {
            let u = (s.axis / s.width.max(0.5)).min(1.0);
            let bed = s.level - 1 - (1.8 * (1.0 - u * u)).round() as i32;
            return if s.level <= SEA_LEVEL { bed.min(h) } else { bed };
        }
        let target = s.bank + 1;
        let excess = (h - target) as f64;
        if excess <= 0.0 && s.level <= SEA_LEVEL {
            return h;
        }
        let ramp = (5.0 + 0.5 * excess.abs()).min(MAX_RAMP);
        if s.sd >= BANK + ramp {
            return h;
        }
        (target as f64 + excess * smoothstep(BANK, BANK + ramp, s.sd)).round() as i32
    }

    /// Version 15: top water block held by a highland spring stream at `(x, z)` with shaped ground `h`.
    pub(super) fn spring_water(&self, x: i32, z: i32, h: i32) -> Option<i32> {
        let s = self.spring_sample(x, z)?;
        (s.sd < 0.0 && h < s.level).then_some(s.level)
    }

    /// Whether any highland spring shapes the column `(x, z)` (ponds keep off).
    pub(super) fn spring_near(&self, x: i32, z: i32) -> bool {
        self.spring_sample(x, z).is_some()
    }

    /// Highland `SPRING` block world positions falling inside chunk column `(cx, cz)`.
    pub(super) fn highland_springs_in(&self, cx: i32, cz: i32) -> Vec<IVec3> {
        let (x0, z0) = (cx * 32, cz * 32);
        let (gx0, gz0) = (x0.div_euclid(SPRING_CELL), z0.div_euclid(SPRING_CELL));
        let mut out = Vec::new();
        for gz in gz0 - 1..=gz0 + 1 {
            for gx in gx0 - 1..=gx0 + 1 {
                if let Some(sp) = self.spring_in(gx, gz) {
                    let h = sp.head;
                    if h.x >= x0 && h.x < x0 + 32 && h.z >= z0 && h.z < z0 + 32 {
                        out.push(h);
                    }
                }
            }
        }
        out
    }

    fn spring_sample(&self, x: i32, z: i32) -> Option<SpringSample> {
        let (gx0, gz0) = (x.div_euclid(SPRING_CELL), z.div_euclid(SPRING_CELL));
        let (px, pz) = (x as f64, z as f64);
        let mut best: Option<SpringSample> = None;
        let mut high = i32::MIN;
        for gz in gz0 - 1..=gz0 + 1 {
            for gx in gx0 - 1..=gx0 + 1 {
                let Some(sp) = self.spring_in(gx, gz) else { continue };
                if !sp.reaches((px, pz), (px, pz)) {
                    continue;
                }
                if let Some(s) = sp.sample(px, pz) {
                    if s.sd < BANK {
                        high = high.max(s.bank);
                    }
                    if best.is_none_or(|b| s.sd < b.sd) {
                        best = Some(s);
                    }
                }
            }
        }
        best.map(|b| SpringSample { bank: b.bank.max(high), ..b })
    }

    fn spring_in(&self, gx: i32, gz: i32) -> Option<Rc<HighlandSpring>> {
        if let Some(cached) = self.springs.cells.borrow().get(&(gx, gz)) {
            return cached.clone();
        }
        let sp = self.build_spring(gx, gz).map(Rc::new);
        self.springs.cells.borrow_mut().insert((gx, gz), sp.clone());
        sp
    }

    fn build_spring(&self, gx: i32, gz: i32) -> Option<HighlandSpring> {
        if unit(hash2(self.seed ^ 0x51A0, gx, gz)) >= SPRING_CHANCE {
            return None;
        }
        let j = hash2(self.seed ^ 0x51A1, gx, gz);
        let span = (SPRING_CELL - 2 * SPRING_MARGIN) as u32;
        let sx = gx * SPRING_CELL + SPRING_MARGIN + (j % span) as i32;
        let sz = gz * SPRING_CELL + SPRING_MARGIN + ((j >> 16) % span) as i32;
        let h0 = self.base_height(sx, sz);
        if h0 < SPRING_MIN_H || self.river_near(sx, sz) {
            return None;
        }
        // Pick initial steepest downhill direction over 8 compass bearings.
        let mut best_dir: Option<(f64, f64)> = None;
        let mut best_h = h0 - 1;
        for &(dx, dz) in &DIRS8 {
            let nx = sx + (dx * 5.0).round() as i32;
            let nz = sz + (dz * 5.0).round() as i32;
            let nh = self.base_height(nx, nz);
            if nh < best_h {
                best_h = nh;
                best_dir = Some((dx, dz));
            }
        }
        let (mut dir_x, mut dir_z) = best_dir?;
        let mut pts = vec![(sx as f64, sz as f64)];
        let mut at = vec![0.0f64];
        let mut cur_x = sx as f64;
        let mut cur_z = sz as f64;
        let mut cur_h = h0;
        let mut joined_water = false;

        while *at.last().unwrap() < MAX_STREAM_LEN {
            let mut next_dir: Option<(f64, f64)> = None;
            let mut best_score = f64::MAX;
            for &(dx, dz) in &DIRS8 {
                let dot = dx * dir_x + dz * dir_z;
                if dot < -0.2 {
                    continue;
                }
                let nx = (cur_x + dx * STEP).round() as i32;
                let nz = (cur_z + dz * STEP).round() as i32;
                let nh = self.base_height(nx, nz);
                // Prefer steeper descent while keeping smooth downhill momentum.
                let score = f64::from(nh) - 0.6 * dot;
                if nh <= cur_h && score < best_score {
                    best_score = score;
                    next_dir = Some((dx, dz));
                }
            }
            let Some((dx, dz)) = next_dir else { break };
            let nx = cur_x + dx * STEP;
            let nz = cur_z + dz * STEP;
            if nx * nx + nz * nz < (SPRING_SPAWN_DRY + 12.0).powi(2) {
                return None;
            }
            let (ix, iz) = (nx.round() as i32, nz.round() as i32);
            let nh = self.base_height(ix, iz);
            let s = *at.last().unwrap() + STEP;
            pts.push((nx, nz));
            at.push(s);
            dir_x = (dir_x * 0.4 + dx * 0.6).clamp(-1.0, 1.0);
            dir_z = (dir_z * 0.4 + dz * 0.6).clamp(-1.0, 1.0);
            let norm = (dir_x * dir_x + dir_z * dir_z).sqrt().max(1e-6);
            (dir_x, dir_z) = (dir_x / norm, dir_z / norm);
            cur_x = nx;
            cur_z = nz;
            cur_h = cur_h.min(nh);
            if nh <= SEA_LEVEL + 1 || self.river_near(ix, iz) {
                joined_water = true;
                break;
            }
        }

        let total = *at.last().unwrap();
        if total < MIN_STREAM_LEN || pts.len() < 4 {
            return None;
        }
        if (sx as f64).powi(2) + (sz as f64).powi(2) < (SPRING_SPAWN_DRY + 12.0).powi(2) {
            return None;
        }

        let start_lvl = (h0 - 2).max(SEA_LEVEL + 2);
        let mut ends = vec![0.0];
        let mut levels = vec![start_lvl];
        let mut last_s = 0.0;
        for (&(px, pz), &s) in pts.iter().zip(&at).skip(1) {
            if s - last_s < MIN_POOL {
                continue;
            }
            let h = self.base_height(px.round() as i32, pz.round() as i32);
            let want = (h - 2).max(SEA_LEVEL);
            let prev = *levels.last().unwrap();
            if want < prev {
                let lvl = if want < SEA_LEVEL + 2 { SEA_LEVEL } else { want };
                ends.push(s);
                levels.push(lvl);
                last_s = s;
            }
        }
        let end_lvl = *levels.last().unwrap();
        if start_lvl - end_lvl < MIN_STREAM_DROP {
            return None;
        }

        let lo = pts.iter().fold((f64::MAX, f64::MAX), |m, p| (m.0.min(p.0), m.1.min(p.1)));
        let hi = pts.iter().fold((f64::MIN, f64::MIN), |m, p| (m.0.max(p.0), m.1.max(p.1)));
        let end_pool_r = if joined_water || end_lvl <= SEA_LEVEL { 0.0 } else { 3.5 };
        let mut spring = HighlandSpring { head: IVec3::ZERO, pts, at, ends, levels, end_pool_r, lo, hi };

        // Embed the SPRING block on the uphill rim touching the first pool at `start_lvl`.
        let (dx0, dz0) = (spring.pts[1].0 - spring.pts[0].0, spring.pts[1].1 - spring.pts[0].1);
        let len0 = (dx0 * dx0 + dz0 * dz0).sqrt().max(1e-6);
        let (ux, uz) = (-dx0 / len0, -dz0 / len0);
        let mut head = None;
        for step in 1..=6 {
            let cx = sx + (ux * step as f64).round() as i32;
            let cz = sz + (uz * step as f64).round() as i32;
            let dry = spring.sample(cx as f64, cz as f64).is_none_or(|s| s.sd >= 0.0);
            if dry {
                let touches_pool = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(ox, oz)| {
                    spring
                        .sample((cx + ox) as f64, (cz + oz) as f64)
                        .is_some_and(|s| s.sd < 0.0 && s.level == start_lvl)
                });
                if touches_pool {
                    head = Some(IVec3::new(cx, start_lvl, cz));
                    break;
                }
            }
        }
        spring.head = head?;
        Some(spring)
    }
}

const DIAG: f64 = std::f64::consts::FRAC_1_SQRT_2;

const DIRS8: [(f64, f64); 8] =
    [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0), (DIAG, DIAG), (-DIAG, DIAG), (DIAG, -DIAG), (-DIAG, -DIAG)];

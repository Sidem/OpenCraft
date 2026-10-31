//! Reading a train: which car a ray enters (`car_hit`), what a car says when aimed at (`car_name`, `car_report`), the
//! train panel's numbers (`train_panel`), a wagon's heap (`heap`), and a wagon's cargo addressed like a storage box
//! (`car_box`), so the box screen's clicks, sorts and shift-moves work on it unchanged (`Factory::box_slots`).
//! Presentation and hands only: the core changes a train's cargo through the box actions and its schedule through
//! `Action::DropStop`.
//! Invariants: car 0 is the locomotive, then 1..=`cars` the wagons; wagon `c` holds cargo slots
//! `(c - 1) * CAR_SLOTS..c * CAR_SLOTS`. To extend: a readout line goes in `car_report`, a panel number in `train_panel`.

use std::ops::Range;

use crate::inventory::Stack;
use crate::item::{self, stack_size, ItemId};
use crate::math::{IVec3, Vec3};

use super::super::process::Pick;
use super::model::RAIL_TOP;
use super::{Factory, Train, CAR_SLOTS};

/// The `y` of a cargo address (`car_box`): no block is ever there.
const CARGO_Y: i32 = i32::MIN;
/// A car's half width and half length, and the top of the locomotive and of a wagon over the rails, in blocks.
const HALF_W: f64 = 0.46;
const HALF_L: f64 = 1.2;
const LOCO_TOP: f64 = 1.4;
const WAGON_TOP: f64 = 1.04;
/// Items named on a wagon's readout at most.
const NAMED: usize = 3;

/// A car aimed at: its train's id and the car (0 the locomotive, then the wagons).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CarAim {
    pub train: u32,
    pub car: u8,
}

/// The cell car `car` of the train with id `train` is addressed by, as a box and as a panel request: off the world.
pub fn car_box(train: u32, car: u8) -> IVec3 {
    IVec3::new(train as i32, CARGO_Y, i32::from(car))
}

/// The car a cargo address names, if `pos` is one.
pub fn car_at(pos: IVec3) -> Option<CarAim> {
    (pos.y == CARGO_Y).then(|| CarAim { train: pos.x as u32, car: pos.z.clamp(0, 255) as u8 })
}

impl Factory {
    pub fn train_by_id(&self, id: u32) -> Option<&Train> {
        self.trains.iter().find(|t| t.id == id)
    }

    /// The nearest car the ray from `origin` along unit `dir` enters within `max` blocks, and how far along it.
    pub fn car_hit(&self, origin: Vec3, dir: Vec3, max: f64) -> Option<(f64, CarAim)> {
        let mut best: Option<(f64, CarAim)> = None;
        for t in &self.trains {
            for car in 0..=t.cars {
                let Some(d) = self.car_entry(t, car, origin, dir).filter(|&d| d <= max) else { continue };
                if best.is_none_or(|(b, _)| d < b) {
                    best = Some((d, CarAim { train: t.id, car }));
                }
            }
        }
        best
    }

    /// The box around car `aim` in the world, as lowest corner and size.
    pub fn car_bounds(&self, aim: CarAim) -> Option<[f64; 6]> {
        let t = self.train_by_id(aim.train)?;
        let p = self.car_pose(t, aim.car, Vec3::ZERO)?;
        let top = if aim.car == 0 { LOCO_TOP } else { WAGON_TOP };
        let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
        for corner in 0..8 {
            let x = if corner & 1 == 0 { -HALF_W } else { HALF_W };
            let y = RAIL_TOP + if corner & 2 == 0 { 0.0 } else { top };
            let z = if corner & 4 == 0 { -HALF_L } else { HALF_L };
            let c = p.mid + p.right * x + p.up * y - p.fwd * z;
            for k in 0..3 {
                (lo[k], hi[k]) = (lo[k].min(c.get(k)), hi[k].max(c.get(k)));
            }
        }
        Some([lo[0], lo[1], lo[2], hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]])
    }

    /// The readout's title for a car.
    pub fn car_name(&self, aim: CarAim) -> String {
        match self.train_by_id(aim.train) {
            Some(t) if aim.car > 0 => format!("Wagon {} of {}", aim.car, t.cars),
            _ => "Locomotive".into(),
        }
    }

    /// The readout's lines for a car: a wagon's load (what of how many slots, the most of what), a locomotive's whole
    /// train, then what the train is doing.
    pub fn car_report(&self, aim: CarAim) -> String {
        let Some(t) = self.train_by_id(aim.train) else { return String::new() };
        let load = if aim.car == 0 {
            match t.cars {
                0 => "No wagons: hold a wagon and click the track just behind it".to_string(),
                n => {
                    let used = t.cargo.slots.iter().filter(|s| !s.is_empty()).count();
                    format!("{n} wagon{}: {used} of {} slots used", if n == 1 { "" } else { "s" }, t.cargo.slots.len())
                }
            }
        } else {
            let slots = &t.cargo.slots[wagon_slots(aim.car)];
            let used = slots.iter().filter(|s| !s.is_empty()).count();
            let mut kinds: Vec<(ItemId, u32)> = Vec::new();
            for s in slots.iter().filter(|s| !s.is_empty()) {
                match kinds.iter_mut().find(|k| k.0 == s.item) {
                    Some(k) => k.1 += s.count,
                    None => kinds.push((s.item, s.count)),
                }
            }
            crate::math::sort_small_by_key(&mut kinds, |k| u32::MAX - k.1);
            let named: Vec<String> = kinds
                .iter()
                .take(NAMED)
                .map(|&(i, n)| format!("{} {}", super::super::fmt_int(u64::from(n)), item::name(i)))
                .collect();
            let more = if kinds.len() > NAMED { " and more" } else { "" };
            match used {
                0 => format!("Empty: 0 of {CAR_SLOTS} slots used"),
                _ => format!("{used} of {CAR_SLOTS} slots used: {}{more}", named.join(", ")),
            }
        };
        format!("{load}\n{}\nRight-click: its cargo and schedule", self.train_state(t))
    }

    /// What a train is doing, in a few words.
    pub fn train_state(&self, t: &Train) -> String {
        if t.idle.is_some() {
            let dock = self.stop_node(t).and_then(|n| self.dock_at(n)).map(|i| self.processors[i].spec.pick);
            return match dock {
                Some(Pick::Load) => "Loading at a dock".into(),
                Some(Pick::Unload) => "Unloading at a dock".into(),
                _ => "Stopped at a dock".into(),
            };
        }
        if t.speed == 0 {
            return "Waiting for the track ahead".into();
        }
        match t.schedule.len() {
            0 if t.cars > 0 => "Running: stops at every dock it reaches".into(),
            0 => "Running".into(),
            n => format!("Running to stop {} of {n}", usize::from(t.next) % n + 1),
        }
    }

    /// The train panel's numbers for the train car `aim` belongs to: wagons, the next stop's index, the stops (x, y, z
    /// of each dock), then each wagon's used slots and items. Empty if the train is gone.
    pub fn train_panel(&self, aim: CarAim) -> Vec<i32> {
        let Some(t) = self.train_by_id(aim.train) else { return Vec::new() };
        let mut out = vec![i32::from(t.cars), i32::from(t.next), t.schedule.len() as i32];
        t.schedule.iter().for_each(|d| out.extend([d.x, d.y, d.z]));
        for car in 1..=t.cars {
            let slots = &t.cargo.slots[wagon_slots(car)];
            out.push(slots.iter().filter(|s| !s.is_empty()).count() as i32);
            out.push(slots.iter().map(|s| s.count).sum::<u32>() as i32);
        }
        out
    }

    /// The cargo slots of the wagon a cargo address names (`car_box`).
    pub(in crate::factory) fn car_slots(&self, pos: IVec3) -> Option<&[Stack]> {
        let aim = car_at(pos)?;
        let t = self.train_by_id(aim.train).filter(|t| (1..=t.cars).contains(&aim.car))?;
        Some(&t.cargo.slots[wagon_slots(aim.car)])
    }

    pub(in crate::factory) fn car_slots_mut(&mut self, pos: IVec3) -> Option<&mut [Stack]> {
        let aim = car_at(pos)?;
        let t = self.trains.iter_mut().find(|t| t.id == aim.train && (1..=t.cars).contains(&aim.car))?;
        Some(&mut t.cargo.slots[wagon_slots(aim.car)])
    }

    /// What wagon `car` of `t` heaps up: the layer of the item it holds most of, and how full it is (0..=1, each slot
    /// by its share of a full stack). None for the locomotive or an empty wagon.
    pub(super) fn heap(t: &Train, car: u8) -> Option<(u16, f64)> {
        if car == 0 {
            return None;
        }
        let slots = t.cargo.slots.get(wagon_slots(car))?;
        let mut main: Option<(ItemId, u32)> = None;
        let mut full = 0.0;
        for s in slots.iter().filter(|s| !s.is_empty()) {
            full += f64::from(s.count) / f64::from(stack_size(s.item));
            let n: u32 = slots.iter().filter(|o| o.item == s.item).map(|o| o.count).sum();
            if main.is_none_or(|m| n > m.1) {
                main = Some((s.item, n));
            }
        }
        let (item, _) = main?;
        Some((item::def(item)?.tex[0], (full / CAR_SLOTS as f64).min(1.0)))
    }

    /// Gives every train an id, numbering those without one (an older save) or with one taken twice (a damaged one).
    pub(super) fn number_trains(&mut self) {
        let mut seen = Vec::new();
        let mut next = self.trains.iter().map(|t| t.id).max().unwrap_or(0);
        for t in &mut self.trains {
            if t.id == 0 || seen.contains(&t.id) {
                next += 1;
                t.id = next;
            }
            seen.push(t.id);
        }
    }

    /// How far along `dir` the ray from `origin` enters car `car` of `t`, if it does.
    fn car_entry(&self, t: &Train, car: u8, origin: Vec3, dir: Vec3) -> Option<f64> {
        let p = self.car_pose(t, car, Vec3::ZERO)?;
        let rel = origin - (p.mid + p.up * RAIL_TOP);
        let o = [dot(rel, p.right), dot(rel, p.up), -dot(rel, p.fwd)];
        let d = [dot(dir, p.right), dot(dir, p.up), -dot(dir, p.fwd)];
        let top = if car == 0 { LOCO_TOP } else { WAGON_TOP };
        let (lo, hi) = ([-HALF_W, 0.0, -HALF_L], [HALF_W, top, HALF_L]);
        let (mut enter, mut leave) = (0.0_f64, f64::MAX);
        for k in 0..3 {
            if d[k].abs() < 1e-12 {
                if o[k] < lo[k] || o[k] > hi[k] {
                    return None;
                }
                continue;
            }
            let (a, b) = ((lo[k] - o[k]) / d[k], (hi[k] - o[k]) / d[k]);
            (enter, leave) = (enter.max(a.min(b)), leave.min(a.max(b)));
        }
        (enter <= leave).then_some(enter)
    }
}

fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// The cargo slots of wagon `car` (from 1).
fn wagon_slots(car: u8) -> Range<usize> {
    let start = (usize::from(car).max(1) - 1) * CAR_SLOTS;
    start..start + CAR_SLOTS
}

#[cfg(test)]
mod tests;

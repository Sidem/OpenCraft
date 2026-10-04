//! The factory's side of drone ports, for `drones/`: which ports there are, where their pads are, how
//! many drones sit at home, and the storage boxes touching each pad (a port's supply: drones take what
//! they build with from them and put what they gather into them). Everything here is exact integer or
//! position bookkeeping on existing machines; the flying and building are `drones/`. The box lookups
//! also serve the personal drone (`helpers/`).

use crate::inventory::Stack;
use crate::item::{ItemId, DRONE};
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::{Factory, DIRS};

/// A drone port as the drones see it.
pub struct PortInfo {
    pub anchor: IVec3,
    /// The pad's centre, at the height of its surface.
    pub centre: Vec3,
    /// How far from the centre its drones work, in blocks.
    pub reach: f64,
    /// Drones at home on the pad.
    pub home: u32,
    /// Whether it is wired to a grid that is not browned out completely.
    pub powered: bool,
}

impl Factory {
    /// Every drone port, in list order.
    pub fn ports(&self) -> Vec<PortInfo> {
        let mut ports = Vec::new();
        for p in &self.processors {
            let Some(tier) = p.hangar_tier() else { continue };
            let centre = p.pos.as_vec3() + Vec3::new(0.5, 0.0, 0.5) + p.spec.footprint.centre(p.dir);
            ports.push(PortInfo {
                anchor: p.pos,
                centre,
                reach: f64::from(tier.reach),
                home: p.drones_home(),
                powered: p.speed > 0,
            });
        }
        ports
    }

    /// Sets how many of the port's drones are out (it draws power while any are).
    pub fn set_port_flight(&mut self, anchor: IVec3, away: u32) {
        if let Some(Slot::Process(i)) = self.at.get(&anchor).copied() {
            let p = &mut self.processors[i as usize];
            p.hangar.away = away;
            p.hangar.busy = away > 0;
        }
    }

    /// The pad's centre of the port at `anchor`, if it is still there.
    pub fn port_centre(&self, anchor: IVec3) -> Option<Vec3> {
        self.ports().into_iter().find(|p| p.anchor == anchor).map(|p| p.centre)
    }

    /// Takes one drone from the port's pad.
    pub fn port_take_drone(&mut self, anchor: IVec3) -> bool {
        let Some(Slot::Process(i)) = self.at.get(&anchor).copied() else { return false };
        let p = &mut self.processors[i as usize];
        let home = p.hangar_tier().is_some() && p.input.count(DRONE) > 0;
        if home {
            p.input.remove(DRONE, 1);
        }
        home
    }

    /// Puts a drone back on the port's pad; false if there is no such port.
    pub fn port_land(&mut self, anchor: IVec3) -> bool {
        let Some(Slot::Process(i)) = self.at.get(&anchor).copied() else { return false };
        let p = &mut self.processors[i as usize];
        p.hangar_tier().is_some() && p.input.add(DRONE, 1) == 1
    }

    /// The storage boxes touching the port's pad (sharing a face with one of its cells), in a fixed order.
    pub fn port_boxes(&self, anchor: IVec3) -> Vec<IVec3> {
        let Some(Slot::Process(i)) = self.at.get(&anchor).copied() else { return Vec::new() };
        let cells = self.processors[i as usize].cells();
        let mut boxes = Vec::new();
        for &c in &cells {
            for d in DIRS {
                let n = c + d;
                if !cells.contains(&n) && !boxes.contains(&n) && matches!(self.at.get(&n), Some(Slot::Storage(_))) {
                    boxes.push(n);
                }
            }
        }
        boxes
    }

    /// How many `item` the box at `pos` holds.
    pub fn box_count(&self, pos: IVec3, item: ItemId) -> u32 {
        match self.at.get(&pos) {
            Some(&Slot::Storage(i)) => self.storages[i as usize].buf.count(item),
            _ => 0,
        }
    }

    /// The storage box nearest to `from` (within `range` blocks) that holds `item`, with its squared distance.
    /// Ties go to the box placed first.
    pub fn nearest_box_with(&self, from: IVec3, item: ItemId, range: i32) -> Option<(IVec3, i64)> {
        let reach = i64::from(range) * i64::from(range);
        let mut best: Option<(IVec3, i64)> = None;
        for s in self.storages.iter().filter(|s| s.buf.count(item) > 0) {
            let d = s.pos - from;
            let d2 = i64::from(d.x).pow(2) + i64::from(d.y).pow(2) + i64::from(d.z).pow(2);
            if d2 <= reach && best.is_none_or(|b| d2 < b.1) {
                best = Some((s.pos, d2));
            }
        }
        best
    }

    /// Takes up to `n` of `item` from the box at `pos`; how many came out.
    pub fn box_take_up_to(&mut self, pos: IVec3, item: ItemId, n: u32) -> u32 {
        let Some(&Slot::Storage(i)) = self.at.get(&pos) else { return 0 };
        let buf = &mut self.storages[i as usize].buf;
        let took = buf.count(item).min(n);
        if took > 0 {
            buf.remove(item, took);
        }
        took
    }

    /// How many items the box at `pos` holds.
    #[cfg(test)]
    pub(crate) fn box_total(&self, pos: IVec3) -> u32 {
        match self.at.get(&pos) {
            Some(&Slot::Storage(i)) => self.storages[i as usize].buf.total(),
            _ => 0,
        }
    }

    /// Whether any of `boxes` has room for at least one `item`.
    pub fn boxes_have_room(&self, boxes: &[IVec3], item: ItemId) -> bool {
        boxes.iter().any(
            |b| matches!(self.at.get(b), Some(&Slot::Storage(i)) if self.storages[i as usize].buf.space_for(item) > 0),
        )
    }

    /// Takes one `item` from the box at `pos`; false if it holds none.
    pub fn box_take(&mut self, pos: IVec3, item: ItemId) -> bool {
        let Some(&Slot::Storage(i)) = self.at.get(&pos) else { return false };
        let buf = &mut self.storages[i as usize].buf;
        let any = buf.count(item) > 0;
        if any {
            buf.remove(item, 1);
        }
        any
    }

    /// Puts `stack` into the first of `boxes` with room, spilling over into the next; returns what did not fit.
    pub fn store_in_boxes(&mut self, boxes: &[IVec3], mut stack: Stack) -> Stack {
        for &pos in boxes {
            let Some(&Slot::Storage(i)) = self.at.get(&pos) else { continue };
            stack.count -= self.storages[i as usize].buf.add(stack.item, stack.count);
            if stack.count == 0 {
                return Stack::default();
            }
        }
        stack
    }
}

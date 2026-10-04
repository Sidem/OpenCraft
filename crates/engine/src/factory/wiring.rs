//! Hand-made power connections. Nothing joins a grid by itself: the player wires a pole to another pole
//! or to a machine (`Action::Connect`), and each wire takes one of the `slots` of every pole it touches
//! (`POLE_TIERS`). A [`Hook`] names its ends by cell, so it survives the machine lists being reordered;
//! `relink` resolves them to indices (`Hooked`) for `power.rs`. Removing either end drops the hook.
//!
//! Who takes a wire (`takes_pole`): generators, miners, quarries, labs, pumps and the processors that run
//! on electricity or the sun. Belts, splitters, filters, boxes and burners need none. A machine has at most
//! one pole; wiring it to another moves it. Cables are not wired by hand (`pole.rs`).
//!
//! [`Factory::hookup`] tells the hands (`power_tools.rs`) what a click would do; `Factory::hook_by_reach`
//! wires saves from before version 24 the way the old rule did.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::math::IVec3;

use super::links::Slot;
use super::pipes::Part;
use super::pole::{dist2, linked, nearest_of, Pole, POLE_TIERS};
use super::process::{Energy, Processor};
use super::Factory;

/// A wire from the pole at `pole` to the machine or other pole whose anchor cell is `to`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hook {
    pub pole: IVec3,
    pub to: IVec3,
}

/// The hooks as pole indices, derived by `relink`: the pole each machine is wired to (by anchor cell) and
/// the pole pairs.
#[derive(Default)]
pub(super) struct Hooked {
    pub by_target: FxHashMap<IVec3, u32>,
    pub pairs: Vec<(u32, u32)>,
    /// Anchors of machines a sensor has switched off (sensor.rs): they hang on nothing.
    pub off: FxHashSet<IVec3>,
}

/// What a click on a cell would do with a pole selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hookup {
    /// Wire it to the pole.
    Connect,
    /// It hangs on another pole (this cell): wire it to this one instead.
    Move(IVec3),
    /// Already wired to this pole: cut the wire.
    Disconnect,
    /// Nothing here takes power from a pole.
    Nothing,
    TooFar,
    /// The pole has no free slot.
    PoleFull,
    /// The other pole has no free slot.
    OtherFull,
}

/// Whether a processor wants a pole: burners, machine recipes and boilers run without.
pub(super) fn takes_pole(p: &Processor) -> bool {
    !matches!(p.energy(), Energy::Burner | Energy::Recipe | Energy::Boiler)
}

impl Factory {
    /// How many of the pole at `pole`'s slots hold a wire.
    pub fn slots_used(&self, pole: IVec3) -> usize {
        self.hooks.iter().filter(|h| h.pole == pole || h.to == pole).count()
    }

    /// The pole at `pole`'s slot count, `None` when it is not a real pole.
    pub fn pole_slots(&self, pole: IVec3) -> Option<usize> {
        match self.at.get(&pole) {
            Some(&Slot::Pole(i)) => {
                Some(&self.poles[i as usize]).filter(|p| !p.is_cable()).map(|p| p.stats().slots as usize)
            }
            _ => None,
        }
    }

    /// The anchor cell of the machine or pole occupying `cell`.
    pub fn anchor_of(&self, cell: IVec3) -> Option<IVec3> {
        match *self.at.get(&cell)? {
            Slot::Process(i) => Some(self.processors[i as usize].pos),
            _ => Some(cell),
        }
    }

    /// The tier of the real pole at `pole`.
    pub fn pole_tier(&self, pole: IVec3) -> Option<u8> {
        self.pole_slots(pole).map(|_| self.tier_at(pole) as u8)
    }

    /// Every cell of the machine at `cell` (one, unless it is a multi-block processor).
    pub fn machine_cells(&self, cell: IVec3) -> Vec<IVec3> {
        match self.at.get(&cell) {
            Some(&Slot::Process(i)) => self.processors[i as usize].cells(),
            _ => vec![cell],
        }
    }

    /// The pole a new pole of `tier` at `pos` wires itself to: the nearest in link range with a free slot
    /// whose grid has a power source (a generator, turbine or solar panel), so it is powered at once. `None`
    /// while the grids are stale (a change waits for the next tick).
    pub fn auto_hook(&self, pos: IVec3, tier: u8) -> Option<IVec3> {
        if self.dirty {
            return None;
        }
        let mine = POLE_TIERS[tier as usize].link;
        let mut best: Option<(i32, IVec3)> = None;
        for (i, p) in self.poles.iter().enumerate().filter(|(_, p)| !p.is_cable() && p.pos != pos) {
            let (d, link) = (dist2(p.pos, pos), mine.max(p.stats().link));
            let open = self.slots_used(p.pos) < p.stats().slots as usize;
            if d <= link * link && open && best.is_none_or(|b| d < b.0) && self.grid_has_source(self.power.pole_grid[i])
            {
                best = Some((d, p.pos));
            }
        }
        best.map(|b| b.1)
    }

    /// Wires the new pole at `pos` to `auto_hook`'s pole.
    pub(super) fn hook_new_pole(&mut self, pos: IVec3, tier: u8) {
        if self.dirty {
            self.relink();
        }
        if let Some(pole) = self.auto_hook(pos, tier) {
            self.hooks.push(Hook { pole, to: pos });
            self.dirty = true;
        }
    }

    fn grid_has_source(&self, grid: u32) -> bool {
        let on = |p: &Option<u32>| p.is_some_and(|p| self.power.pole_grid[p as usize] == grid);
        let sun_or_steam = |m: &Processor| matches!(m.energy(), Energy::Turbine | Energy::Solar);
        self.power.gen_pole.iter().any(on)
            || self.processors.iter().zip(&self.power.process_pole).any(|(m, p)| sun_or_steam(m) && on(p))
    }

    /// The pole the machine at `anchor` is wired to, if any.
    pub fn wired_pole(&self, anchor: IVec3) -> Option<IVec3> {
        self.hooks.iter().find(|h| h.to == anchor).map(|h| h.pole)
    }

    /// Whether the machine at `cell` is on a grid: wired to a pole, or hanging on a cable.
    pub fn is_wired(&self, cell: IVec3) -> bool {
        let Some(&slot) = self.at.get(&cell) else { return false };
        match slot {
            Slot::Generator(i) => self.power.gen_pole.get(i as usize).copied().flatten().is_some(),
            Slot::Miner(i) => self.power.miner_pole.get(i as usize).copied().flatten().is_some(),
            Slot::Process(i) => self.power.process_pole.get(i as usize).copied().flatten().is_some(),
            Slot::Lab(i) => self.power.lab_pole.get(i as usize).copied().flatten().is_some(),
            Slot::Pipe(i) => self.power.pipe_pole.get(i as usize).copied().flatten().is_some(),
            Slot::Quarry(i) => self.power.quarry_pole.get(i as usize).copied().flatten().is_some(),
            _ => false,
        }
    }

    /// What a click on `cell` would do with the real pole at `pole` selected.
    pub fn hookup(&self, pole: IVec3, cell: IVec3) -> Hookup {
        let Some(slots) = self.pole_slots(pole) else { return Hookup::Nothing };
        let (Some(target), Some(anchor)) = (self.at.get(&cell), self.anchor_of(cell)) else { return Hookup::Nothing };
        if anchor == pole {
            return Hookup::Nothing;
        }
        let wired = |a, b| self.hooks.iter().any(|h| h.pole == a && h.to == b);
        let stats = &POLE_TIERS[self.tier_at(pole)];
        let (near, other_full) = match *target {
            Slot::Pole(i) => {
                let other = &self.poles[i as usize];
                if other.is_cable() {
                    return Hookup::Nothing;
                }
                if wired(pole, anchor) || wired(anchor, pole) {
                    return Hookup::Disconnect;
                }
                let link = stats.link.max(other.stats().link);
                let full = self.slots_used(anchor) >= other.stats().slots as usize;
                (dist2(pole, anchor) <= link * link, full)
            }
            slot => {
                let Some(cells) = self.powered_cells(slot) else { return Hookup::Nothing };
                if wired(pole, anchor) {
                    return Hookup::Disconnect;
                }
                (cells.iter().any(|&c| dist2(pole, c) <= stats.reach * stats.reach), false)
            }
        };
        if !near {
            return Hookup::TooFar;
        }
        if self.slots_used(pole) >= slots {
            return Hookup::PoleFull;
        }
        if other_full {
            return Hookup::OtherFull;
        }
        match (target, self.wired_pole(anchor)) {
            (Slot::Pole(_), _) | (_, None) => Hookup::Connect,
            (_, Some(from)) => Hookup::Move(from),
        }
    }

    /// Wires the pole at `pole` to what stands at `cell` (a click, `Action::Connect`): quietly nothing
    /// unless `hookup` says it fits. A machine on another pole moves to this one. Cutting an existing
    /// wire is `disconnect`.
    pub fn connect(&mut self, pole: IVec3, cell: IVec3) {
        let (Hookup::Connect | Hookup::Move(_)) = self.hookup(pole, cell) else { return };
        let Some(to) = self.anchor_of(cell) else { return };
        self.hooks.retain(|h| h.to != to || matches!(self.at.get(&to), Some(Slot::Pole(_))));
        self.hooks.push(Hook { pole, to });
        self.dirty = true;
    }

    /// Cuts the wire between the pole at `pole` and what stands at `cell`.
    pub fn disconnect(&mut self, pole: IVec3, cell: IVec3) {
        let Some(to) = self.anchor_of(cell) else { return };
        let before = self.hooks.len();
        self.hooks.retain(|h| !((h.pole == pole && h.to == to) || (h.pole == to && h.to == pole)));
        self.dirty |= self.hooks.len() != before;
    }

    /// Wires every machine and pole to what the old rule hung it on: poles within their link range, and
    /// each machine on the nearest pole in reach. How saves before version 24 keep their power (a pole
    /// may end up over its slots, which only stops new wires), and how tests that are not about wiring
    /// get power (`Factory::by_hand`); the caller relinks.
    pub fn hook_by_reach(&mut self) {
        self.hooks.clear();
        let reals: Vec<&Pole> = self.poles.iter().filter(|p| !p.is_cable()).collect();
        for (i, a) in reals.iter().enumerate() {
            for b in &reals[i + 1..] {
                if linked(a, b) {
                    self.hooks.push(Hook { pole: a.pos, to: b.pos });
                }
            }
        }
        let mut hooks = std::mem::take(&mut self.hooks);
        for slot in self.power_slots() {
            let Some(cells) = self.powered_cells(slot) else { continue };
            let near =
                |&c: &IVec3| nearest_of(&self.poles, c, false).map(|i| (dist2(self.poles[i as usize].pos, c), i));
            if let Some((_, i)) = cells.iter().filter_map(near).min() {
                hooks.push(Hook { pole: self.poles[i as usize].pos, to: cells[0] });
            }
        }
        self.hooks = hooks;
    }

    /// The hooks as pole indices (see [`Hooked`]).
    pub(super) fn resolve_hooks(&self) -> Hooked {
        let mut out = Hooked { off: self.switched_off(), ..Hooked::default() };
        for h in &self.hooks {
            let (Some(&Slot::Pole(p)), Some(target)) = (self.at.get(&h.pole), self.at.get(&h.to)) else { continue };
            match *target {
                Slot::Pole(q) => out.pairs.push((p.min(q), p.max(q))),
                _ => {
                    out.by_target.insert(h.to, p);
                }
            }
        }
        out
    }

    /// Drops hooks whose ends are gone (a machine removed, a pole replaced).
    pub(super) fn prune_hooks(&mut self) {
        if self.hooks.is_empty() {
            return;
        }
        let real =
            |f: &Factory, c: &IVec3| matches!(f.at.get(c), Some(&Slot::Pole(i)) if !f.poles[i as usize].is_cable());
        let keep: Vec<bool> = self.hooks.iter().map(|h| real(self, &h.pole) && self.at.contains_key(&h.to)).collect();
        let mut keep = keep.into_iter();
        self.hooks.retain(|_| keep.next().unwrap_or(false));
    }

    fn tier_at(&self, pole: IVec3) -> usize {
        match self.at.get(&pole) {
            Some(&Slot::Pole(i)) => self.poles[i as usize].tier as usize,
            _ => 0,
        }
    }

    /// Every slot that takes a wire, kind by kind in list order (so that anything built from it is deterministic).
    fn power_slots(&self) -> impl Iterator<Item = Slot> {
        let (g, m, p) = (self.generators.len(), self.miners.len(), self.processors.len());
        let (l, w, q) = (self.labs.len(), self.pipework.len(), self.quarries.len());
        let n = |n: usize| 0..n as u32;
        n(g).map(Slot::Generator)
            .chain(n(m).map(Slot::Miner))
            .chain(n(p).map(Slot::Process))
            .chain(n(l).map(Slot::Lab))
            .chain(n(w).map(Slot::Pipe))
            .chain(n(q).map(Slot::Quarry))
    }

    /// The cells of the machine in `slot` if it takes a wire, its anchor first.
    pub(super) fn powered_cells(&self, slot: Slot) -> Option<Vec<IVec3>> {
        match slot {
            Slot::Generator(i) => Some(vec![self.generators[i as usize].pos]),
            Slot::Miner(i) => Some(vec![self.miners[i as usize].pos]),
            Slot::Process(i) => {
                let p = &self.processors[i as usize];
                takes_pole(p).then(|| p.cells())
            }
            Slot::Lab(i) => Some(vec![self.labs[i as usize].pos]),
            Slot::Pipe(i) => {
                let p = &self.pipework[i as usize];
                (p.part == Part::Pump).then(|| vec![p.pos])
            }
            Slot::Quarry(i) => Some(vec![self.quarries[i as usize].pos]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;

//! Building docks by hand, part of the local player's hands (`interaction.rs`). A dock is a piece of rail
//! (`factory/rail/pieces.rs`): with one in hand, the straight rail nodes it can go onto within `SHOW_REACH` are
//! outlined, the aimed node green or red with the reason in the label (`DockFit`), and a click sends an ordinary
//! `PlaceBlock` onto the node, which the core turns into the dock or refuses. Aiming at the ground next to a node
//! takes that node (a rail is thin to hit).
//!
//! Laid in line: with a rail node selected (the one just placed with rails, `rail_tools.rs`) and no node aimed at,
//! the dock goes where the aim crosses that node's straight line, joined to it in the same action
//! (`PlaceBlock` with the heading as the facing and the node as `against`), and is selected so rails continue from
//! it. The outlines and label show what the rail hand would: the cells to fill or clear, or why not.
//!
//! The press is tracked with `RailTools::down`, which the rail and train hands share (only one of them is ever in hand).

use crate::action::Action;
use crate::block::{self, BlockId, AIR, LOADING_DOCK};
use crate::factory::{dir_of, fit_track, is_dock, rail_faults, write_track, Curve, DockFit, Fault, Fit};
use crate::math::{IVec3, Vec3};
use crate::rail_tools::{ground_why, why_not};
use crate::sound;
use crate::Game;

/// Straight nodes this near (blocks, on each axis) are outlined while a dock is in hand, at most `MAX_SHOWN`.
const SHOW_REACH: i32 = 24;
const MAX_SHOWN: usize = 48;
const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;
const FILL_AMBER: i32 = 0xfbbf24;
const ANCHOR_BLUE: i32 = 0x7dd3fc;
/// A node a dock could go onto, not aimed at.
const CAN_TEAL: i32 = 0x2dd4bf;
const UP: IVec3 = IVec3::new(0, 1, 0);
/// Heading steps between grid axes.
const AXIS: u8 = 64;
/// How far over the aim a dock laid in line looks for the ground (a rise along the line).
const RISE: i32 = 2;

/// A dock laid in line: its cell, the selected node it joins (with its heading, which the dock takes), the cells
/// the track needs filled or cleared, and why it cannot go there (`None`: a click lays it).
struct InLine {
    pos: IVec3,
    from: (IVec3, u8),
    faults: Vec<(IVec3, Fault)>,
    why: Option<String>,
}

impl Game {
    /// The dock block in hand.
    fn held_dock(&self) -> Option<BlockId> {
        let stack = self.inventory().selected_stack();
        stack.item.places().filter(|&b| !stack.is_empty() && is_dock(b))
    }

    /// The rail node (or dock) aimed at, and whether a dock fits there: the cell hit, or a node beside the ground hit.
    fn dock_aim(&self) -> Option<(IVec3, DockFit)> {
        let f = &self.sim.factory;
        let hit = self.target?;
        let near = hit.block + hit.normal;
        let cell = [hit.block, near]
            .into_iter()
            .chain(
                [IVec3::new(1, 0, 0), IVec3::new(-1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(0, 0, -1)]
                    .map(|d| near + d),
            )
            .find(|&c| f.rail_yaw(c).is_some())?;
        Some((cell, f.dock_fit(cell)))
    }

    /// With no node aimed at and one selected: the dock laid on the selected node's line where the aim crosses it.
    fn dock_in_line(&self) -> Option<InLine> {
        if self.dock_aim().is_some() {
            return None;
        }
        let (a, ya) = self.rail_anchor()?;
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
        let aim = hit.block + hit.normal;
        let mut line = InLine { pos: aim, from: (a, ya), faults: Vec::new(), why: None };
        if !ya.is_multiple_of(AXIS) {
            line.why =
                Some("the selected node's track runs on a diagonal: docks go on north-south or east-west track".into());
            return Some(line);
        }
        let (dx, dz) = dir_of(ya);
        let (dx, dz) = (dx.round() as i32, dz.round() as i32);
        let along = (aim.x - a.x) * dx + (aim.z - a.z) * dz;
        let (pos, free) = self.settle_node(IVec3::new(a.x + dx * along, aim.y, a.z + dz * along), RISE);
        line.pos = pos;
        let fit = self.sim.factory.lay_fit(a, pos, ya).unwrap_or_else(|| fit_track(a, ya, pos, ya));
        if !free {
            line.why = Some("needs clear ground to stand on: aim at the ground along the track".into());
        } else if fit != Fit::Ok {
            line.why = Some(format!("{} · move the aim", why_not(fit)));
        } else {
            let world = &self.sim.world;
            line.faults = rail_faults(&Curve::new(a, ya, pos, ya), |p| world.get_block(p).unwrap_or(AIR));
            line.why = (!line.faults.is_empty()).then(|| ground_why(&line.faults));
        }
        Some(line)
    }

    /// Runs the dock hand for one tick. True while a dock is in hand (the press is its own).
    pub(crate) fn update_dock_tools(&mut self) -> bool {
        if self.held_dock().is_none() {
            return false;
        }
        let edge = self.using && !self.rails.down;
        self.rails.down = self.using;
        if !edge {
            return true;
        }
        let slot = self.inventory().selected as u8;
        let pos = match (self.dock_aim(), self.dock_in_line()) {
            (Some((pos, DockFit::Ok)), _) => {
                self.act(Action::PlaceBlock { pos, slot, facing: 0, against: pos - UP });
                pos
            }
            (None, Some(InLine { pos, from: (a, ya), why: None, .. })) => {
                self.act(Action::PlaceBlock { pos, slot, facing: ya, against: a });
                self.select_rail_piece(pos, ya);
                pos
            }
            _ => return true,
        };
        self.play(sound::PLACE, block::sound::METAL, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.6);
        true
    }

    /// Outlines (7 numbers each): every node a dock could go onto nearby, and the aimed one green or red; laid in
    /// line, the selected node, the dock's cell and the cells to fill (amber) or clear (red).
    pub(crate) fn dock_boxes(&self) -> Vec<i32> {
        if self.held_dock().is_none() {
            return Vec::new();
        }
        let f = &self.sim.factory;
        let me = self.body().pos.floor();
        let mut out = Vec::new();
        let mut boxed = |p: IVec3, colour: i32| out.extend([p.x, p.y, p.z, p.x, p.y, p.z, colour]);
        let aim = self.dock_aim();
        let close = |p: IVec3| (p - me).x.abs().max((p - me).y.abs()).max((p - me).z.abs()) <= SHOW_REACH;
        for p in f.rail_nodes().filter(|&p| close(p) && Some(p) != aim.map(|a| a.0)).take(MAX_SHOWN) {
            if f.dock_fit(p) == DockFit::Ok {
                boxed(p, CAN_TEAL);
            }
        }
        if let Some((p, fit)) = aim {
            boxed(p, if fit == DockFit::Ok { GHOST_GREEN } else { BLOCKED_RED });
        }
        if let Some(line) = self.dock_in_line() {
            boxed(line.from.0, ANCHOR_BLUE);
            boxed(line.pos, if line.why.is_none() { GHOST_GREEN } else { BLOCKED_RED });
            for &(cell, fault) in &line.faults {
                boxed(cell, if fault == Fault::Floating { FILL_AMBER } else { BLOCKED_RED });
            }
        }
        out
    }

    /// The HUD lines while a dock is in hand: its name, then what a click does or why it cannot.
    pub(crate) fn dock_label(&self) -> String {
        let Some(dock) = self.held_dock() else { return String::new() };
        let (name, belts) = if dock == LOADING_DOCK {
            ("Loading Dock", "belts feed it from the sides")
        } else {
            ("Unloading Dock", "belts lead away from its sides")
        };
        let text = match (self.dock_aim(), self.dock_in_line()) {
            (Some((_, DockFit::Ok)), _) => format!("click to build it here: trains stop on it, {belts}"),
            (Some((p, _)), _) if self.sim.factory.dock_anchor(p).is_some() => "this is a dock already".to_string(),
            (Some((_, why)), _) => why_not_here(why).to_string(),
            (None, Some(InLine { pos, from: (a, _), why: None, .. })) => {
                let run = (pos - a).x.abs() + (pos - a).z.abs();
                format!("click to lay it in line: {run} blocks of track from the selected node · trains stop on it")
            }
            (None, Some(InLine { why: Some(why), .. })) => why,
            (None, None) => "aim at a rail node on straight track: the dock takes its place (nodes it fits are \
                             outlined), or along the track after the node you just placed to lay it in line"
                .to_string(),
        };
        format!("{name}\n{text}")
    }

    /// The track a dock laid in line would bring, drawn with the machines.
    pub(crate) fn write_dock_preview(&mut self, eye: Vec3) {
        if self.held_dock().is_none() {
            return;
        }
        if let Some(InLine { pos, from: (a, ya), why: None, .. }) = self.dock_in_line() {
            write_track(&mut self.instances, &Curve::new(a, ya, pos, ya), eye, 1.0e9);
        }
    }
}

fn why_not_here(fit: DockFit) -> &'static str {
    match fit {
        DockFit::Ok => "",
        DockFit::NoNode => "aim at a rail node",
        DockFit::Diagonal => "this track runs on a diagonal: docks go on north-south or east-west track",
        DockFit::Junction => "more than one track leaves an end here: docks go on plain track",
        DockFit::Signal => "a signal stands here: take it off first",
        DockFit::Train => "a train is on this track: wait for it to move on",
    }
}

#[cfg(test)]
mod tests;

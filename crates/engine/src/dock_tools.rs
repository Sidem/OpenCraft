//! Building docks by hand, part of the local player's hands (`interaction.rs`). A dock is a piece of rail
//! (`factory/rail/pieces.rs`): with one in hand, the straight rail nodes it can go onto within `SHOW_REACH` are
//! outlined, the aimed node green or red with the reason in the label (`DockFit`), and a click sends an ordinary
//! `PlaceBlock` onto the node, which the core turns into the dock or refuses. Aiming at the ground next to a node
//! takes that node (a rail is thin to hit). The press is tracked with `RailTools::down`, which the rail and train
//! hands share (only one of them is ever in hand).

use crate::action::Action;
use crate::block::{self, BlockId, LOADING_DOCK};
use crate::factory::{is_dock, DockFit};
use crate::math::{IVec3, Vec3};
use crate::sound;
use crate::Game;

/// Straight nodes this near (blocks, on each axis) are outlined while a dock is in hand, at most `MAX_SHOWN`.
const SHOW_REACH: i32 = 24;
const MAX_SHOWN: usize = 48;
const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;
/// A node a dock could go onto, not aimed at.
const CAN_TEAL: i32 = 0x2dd4bf;
const UP: IVec3 = IVec3::new(0, 1, 0);

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

    /// Runs the dock hand for one tick. True while a dock is in hand (the press is its own).
    pub(crate) fn update_dock_tools(&mut self) -> bool {
        if self.held_dock().is_none() {
            return false;
        }
        let edge = self.using && !self.rails.down;
        self.rails.down = self.using;
        let Some((pos, DockFit::Ok)) = self.dock_aim().filter(|_| edge) else { return true };
        let slot = self.inventory().selected as u8;
        self.act(Action::PlaceBlock { pos, slot, facing: 0, against: pos - UP });
        self.play(sound::PLACE, block::sound::METAL, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.6);
        true
    }

    /// Outlines (7 numbers each): every node a dock could go onto nearby, and the aimed one green or red.
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
        let text = match self.dock_aim() {
            None => "aim at a rail node on straight track: the dock takes its place (nodes it fits are outlined)"
                .to_string(),
            Some((_, DockFit::Ok)) => format!("click to build it here: trains stop on it, {belts}"),
            Some((p, _)) if self.sim.factory.dock_anchor(p).is_some() => "this is a dock already".to_string(),
            Some((_, why)) => why_not(why).to_string(),
        };
        format!("{name}\n{text}")
    }
}

fn why_not(fit: DockFit) -> &'static str {
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

//! Putting locomotives on the track, coupling wagons, taking trains off and setting their schedules, part of the
//! local player's hands (`interaction.rs`).
//! - A locomotive in hand: click a rail node that has track to put it on (`Action::PlaceTrain`); click a node with a
//!   train near it to select that train; with one selected, click a dock to add it to the train's schedule and
//!   crouch-click a dock to clear the schedule (`Action::TrainStop`).
//! - A wagon in hand: click a node beside a train to couple it on (Shift: every wagon in hand, up to `MAX_CARS`).
//! - A signal in hand: click a rail node to put it there, click it again to take it back (`Action::ToggleSignal`).
//! - A locomotive or wagon: crouch-click a node to pick up the train nearest it with all it carries (`Action::TakeTrain`).
//!
//! `Factory::train_spot` and `couple_spot` say what a click on a node would do; the outline is green where it
//! fits and red where it does not, the selected train's node is blue, and the label says why.
//! The press is tracked with `RailTools::down` (the rail and train hands are never in hand together); the
//! selected node is `RailTools::train`.

use crate::action::Action;
use crate::block;
use crate::factory::{Spot, MAX_CARS};
use crate::item::{ItemId, LOCOMOTIVE, RAIL_SIGNAL, WAGON};
use crate::math::{IVec3, Vec3};
use crate::sound;
use crate::Game;

const GHOST_GREEN: i32 = 0x4ade80;
const BLOCKED_RED: i32 = 0xff4a3d;
const SELECTED_BLUE: i32 = 0x7dd3fc;

/// What the train hand is aimed at.
#[derive(Clone, Copy)]
enum Aim {
    /// A rail node and what the stock in hand would do there.
    Node(IVec3, Spot),
    /// A dock (its anchor cell), with a locomotive in hand.
    Dock(IVec3),
    /// A rail node, with a signal in hand: a click puts the signal there or takes it back.
    Signal(IVec3),
}

impl Game {
    /// The locomotive, wagon or signal in hand.
    pub(crate) fn rolling_stock(&self) -> Option<ItemId> {
        let stack = self.inventory().selected_stack();
        (!stack.is_empty() && (stack.item == LOCOMOTIVE || stack.item == WAGON || stack.item == RAIL_SIGNAL))
            .then_some(stack.item)
    }

    fn train_aim(&self) -> Option<Aim> {
        let f = &self.sim.factory;
        let item = self.rolling_stock()?;
        let cell = self.target?.block;
        // A dock is a rail node too: with a train selected, a locomotive's click adds it to the schedule.
        if item == LOCOMOTIVE && self.rails.train.is_some() {
            if let Some(dock) = f.dock_anchor(cell) {
                return Some(Aim::Dock(dock));
            }
        }
        if f.rail_yaw(cell).is_some() {
            return Some(match item {
                RAIL_SIGNAL => Aim::Signal(cell),
                WAGON => Aim::Node(cell, f.couple_spot(cell)),
                _ => Aim::Node(cell, f.train_spot(cell)),
            });
        }
        f.dock_anchor(cell).filter(|_| item == LOCOMOTIVE).map(Aim::Dock)
    }

    /// Runs the train hand for one tick. True while rolling stock or a signal is in hand (the press is its own).
    pub(crate) fn update_train_tools(&mut self) -> bool {
        let Some(item) = self.rolling_stock() else { return false };
        let edge = self.using && !self.rails.down;
        self.rails.down = self.using;
        let Some(aim) = self.train_aim().filter(|_| edge) else { return true };
        let crouch = self.body().input.crouch;
        let at = match aim {
            Aim::Signal(node) => {
                let slot = self.inventory().selected as u8;
                self.act(Action::ToggleSignal { pos: node, slot });
                node
            }
            Aim::Node(node, _) if crouch => {
                self.act(Action::TakeTrain { pos: node });
                node
            }
            Aim::Node(node, spot) => {
                if spot == Spot::Ok {
                    // Shift with wagons in hand couples them all (the core stops at a full train or the track's end).
                    let all = item == WAGON && self.body().input.sprint;
                    let times = if all { self.inventory().selected_stack().count.min(u32::from(MAX_CARS)) } else { 1 };
                    let slot = self.inventory().selected as u8;
                    for _ in 0..times {
                        self.act(Action::PlaceTrain { pos: node, slot });
                    }
                } else if spot != Spot::Occupied {
                    return true;
                }
                if item == LOCOMOTIVE {
                    self.rails.train = Some(node);
                }
                node
            }
            Aim::Dock(dock) => {
                let Some(node) = self.rails.train else { return true };
                self.act(Action::TrainStop { node, dock, clear: crouch });
                dock
            }
        };
        self.play(sound::PLACE, block::sound::METAL, at.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.5);
        true
    }

    /// Outlines (7 numbers each: lowest and highest cell, colour): the aimed node or dock, and the selected train's node.
    pub(crate) fn train_boxes(&self) -> Vec<i32> {
        let cell = |p: IVec3, colour| [p.x, p.y, p.z, p.x, p.y, p.z, colour];
        let mut out = Vec::new();
        match self.train_aim() {
            Some(Aim::Node(p, spot)) => out.extend(cell(p, if spot == Spot::Ok { GHOST_GREEN } else { BLOCKED_RED })),
            Some(Aim::Signal(p)) => out.extend(cell(p, GHOST_GREEN)),
            Some(Aim::Dock(p)) => {
                out.extend(cell(p, if self.rails.train.is_some() { GHOST_GREEN } else { BLOCKED_RED }))
            }
            None => {}
        }
        if let Some(p) = self.rails.train.filter(|_| self.rolling_stock() == Some(LOCOMOTIVE)) {
            out.extend(cell(p, SELECTED_BLUE));
        }
        out
    }

    /// The HUD lines while a locomotive, wagon, or signal is in hand: title, status/schedule, and control chips.
    pub(crate) fn train_label(&self) -> String {
        let Some(item) = self.rolling_stock() else { return String::new() };
        let keys = crate::factory::hud_keys;
        if item == RAIL_SIGNAL {
            return match self.train_aim() {
                Some(Aim::Signal(p)) if self.sim.factory.signal_at(p) => {
                    format!("Rail Signal\n{}", keys(&[("RMB", "Remove signal")]))
                }
                Some(_) => format!("Rail Signal\n{}", keys(&[("RMB", "Place signal")])),
                None => "Rail Signal\nAim at a rail node · trains wait for clear track".to_string(),
            };
        }
        let wagon = item == WAGON;
        let (status, controls) = match (wagon, self.train_aim()) {
            (_, Some(Aim::Signal(_))) => ("", String::new()),
            (false, None) => ("Aim at a rail node or dock", String::new()),
            (true, None) => ("Aim at a rail node beside a train", String::new()),
            (false, Some(Aim::Dock(_))) if self.rails.train.is_none() => {
                ("Select a train on a rail node first", String::new())
            }
            (_, Some(Aim::Dock(_))) => ("", keys(&[("RMB", "Add dock"), ("Ctrl+RMB", "Clear schedule")])),
            (false, Some(Aim::Node(_, Spot::Ok))) => {
                ("", keys(&[("RMB", "Place train"), ("Ctrl+RMB", "Pick up nearest")]))
            }
            (true, Some(Aim::Node(_, Spot::Ok))) => {
                ("", keys(&[("RMB", "Couple 1"), ("Shift+RMB", "Couple all"), ("Ctrl+RMB", "Pick up train")]))
            }
            (_, Some(Aim::Node(_, Spot::NoTrack))) => {
                ("No track long enough · join to another node first", String::new())
            }
            (false, Some(Aim::Node(_, Spot::Occupied))) => {
                ("", keys(&[("RMB", "Select train"), ("Ctrl+RMB", "Pick up")]))
            }
            (true, Some(Aim::Node(_, Spot::Occupied))) => ("Train already here", keys(&[("Ctrl+RMB", "Pick up")])),
            (_, Some(Aim::Node(_, Spot::NoTrain))) => ("No train near this node", String::new()),
            (_, Some(Aim::Node(_, Spot::Full))) => ("Train already has 6 wagons (max)", String::new()),
            (_, Some(Aim::Node(_, Spot::NoRoom))) => {
                ("No track behind the train · lay more track first", String::new())
            }
        };
        let title = if wagon { "Wagon" } else { "Locomotive" };
        let mut lines = vec![title.to_string()];
        if !status.is_empty() {
            lines.push(status.to_string());
        }
        if let Some(node) = self.rails.train.filter(|_| !wagon) {
            lines.push(self.schedule_line(node));
        }
        if !controls.is_empty() {
            lines.push(controls);
        }
        lines.join("\n")
    }

    /// The schedule of the train near `node` as one line.
    fn schedule_line(&self, node: IVec3) -> String {
        let stops = self.sim.factory.schedule_near(node);
        if stops.is_empty() {
            return "Selected train: no schedule, it stops at every dock".to_string();
        }
        let named: Vec<String> = stops.iter().map(|d| format!("dock {}, {}", d.x, d.z)).collect();
        format!("Selected train: {}, then round again", named.join(" â†’ "))
    }
}

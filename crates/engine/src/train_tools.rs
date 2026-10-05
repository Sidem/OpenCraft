//! Putting locomotives on the track, coupling wagons, taking trains off and setting their schedules, part of the
//! local player's hands (`interaction.rs`).
//! - A locomotive in hand: click a rail node that has track to put it on (`Action::PlaceTrain`); click a node with a
//!   train near it to select that train; with one selected, click a dock to add it to the train's schedule and
//!   crouch-click a dock to clear the schedule (`Action::TrainStop`).
//! - A wagon in hand: click a node beside a train to couple it on.
//! - Either: crouch-click a node to pick up the train nearest it with all it carries (`Action::TakeTrain`).
//!
//! `Factory::train_spot` and `couple_spot` say what a click on a node would do; the outline is green where it
//! fits and red where it does not, the selected train's node is blue, and the label says why.
//! The press is tracked with `RailTools::down` (the rail and train hands are never in hand together); the
//! selected node is `RailTools::train`.

use crate::action::Action;
use crate::block;
use crate::factory::Spot;
use crate::item::{ItemId, LOCOMOTIVE, WAGON};
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
}

impl Game {
    /// The locomotive or wagon in hand.
    fn rolling_stock(&self) -> Option<ItemId> {
        let stack = self.inventory().selected_stack();
        (!stack.is_empty() && (stack.item == LOCOMOTIVE || stack.item == WAGON)).then_some(stack.item)
    }

    fn train_aim(&self) -> Option<Aim> {
        let f = &self.sim.factory;
        let item = self.rolling_stock()?;
        let cell = self.target?.block;
        if f.rail_yaw(cell).is_some() {
            return Some(Aim::Node(cell, if item == WAGON { f.couple_spot(cell) } else { f.train_spot(cell) }));
        }
        f.dock_anchor(cell).filter(|_| item == LOCOMOTIVE).map(Aim::Dock)
    }

    /// Runs the train hand for one tick. True while a locomotive or wagon is in hand (the press is its own).
    pub(crate) fn update_train_tools(&mut self) -> bool {
        let Some(item) = self.rolling_stock() else { return false };
        let edge = self.using && !self.rails.down;
        self.rails.down = self.using;
        let Some(aim) = self.train_aim().filter(|_| edge) else { return true };
        let crouch = self.body().input.crouch;
        let at = match aim {
            Aim::Node(node, _) if crouch => {
                self.act(Action::TakeTrain { pos: node });
                node
            }
            Aim::Node(node, spot) => {
                if spot == Spot::Ok {
                    let slot = self.inventory().selected as u8;
                    self.act(Action::PlaceTrain { pos: node, slot });
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

    /// The HUD lines while a locomotive or wagon is in hand: a title, then what a click does, then the selected
    /// train's schedule.
    pub(crate) fn train_label(&self) -> String {
        let Some(item) = self.rolling_stock() else { return String::new() };
        let wagon = item == WAGON;
        let text = match (wagon, self.train_aim()) {
            (false, None) => {
                "aim at a rail node to put it on the track, or a dock to add it to the selected train's schedule"
            }
            (true, None) => "aim at a rail node beside a train and click to couple the wagon on behind",
            (false, Some(Aim::Dock(_))) if self.rails.train.is_none() => {
                "click a node a train is on to select it first"
            }
            (_, Some(Aim::Dock(_))) => "click to add this dock to the schedule · crouch-click to clear the schedule",
            (false, Some(Aim::Node(_, Spot::Ok))) => {
                "click to put it on the track here · crouch-click to pick up the train nearest"
            }
            (true, Some(Aim::Node(_, Spot::Ok))) => {
                "click to couple it behind the train here · crouch-click to pick it up"
            }
            (_, Some(Aim::Node(_, Spot::NoTrack))) => {
                "this node has no track long enough: join it to another node first"
            }
            (false, Some(Aim::Node(_, Spot::Occupied))) => {
                "click to select the train here · crouch-click to pick it up"
            }
            (true, Some(Aim::Node(_, Spot::Occupied))) => "a train is already here · crouch-click to pick it up",
            (_, Some(Aim::Node(_, Spot::NoTrain))) => "no train is near this node",
            (_, Some(Aim::Node(_, Spot::Full))) => "this train has all the wagons it can pull (6)",
            (_, Some(Aim::Node(_, Spot::NoRoom))) => {
                "no track behind the train to stand a wagon on: lay more track first"
            }
        };
        let title = if wagon { "Wagon" } else { "Locomotive" };
        match self.rails.train.filter(|_| !wagon) {
            Some(node) => format!("{title}\n{text}\n{}", self.schedule_line(node)),
            None => format!("{title}\n{text}"),
        }
    }

    /// The schedule of the train near `node` as one line.
    fn schedule_line(&self, node: IVec3) -> String {
        let stops = self.sim.factory.schedule_near(node);
        if stops.is_empty() {
            return "Selected train: no schedule, it stops at every dock".to_string();
        }
        let named: Vec<String> = stops.iter().map(|d| format!("dock {}, {}", d.x, d.z)).collect();
        format!("Selected train: {}, then round again", named.join(" → "))
    }
}

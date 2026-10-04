//! Laying track by hand, part of the local player's hands (`interaction.rs`), built like the power poles
//! (`power_tools.rs`). With rails in hand the use button works on rail *nodes*; everything it builds is an ordinary
//! action (`PlaceBlock` with the heading as the facing, `Connect`, `Disconnect`), so the core, co-op and saves see
//! plain placements and tracks.
//!
//! - Aim at the ground and click: a node goes there, joined to the selected node (if any) by a curve, and is
//!   selected itself, so clicking along lays a railway. A first node faces where you look; a later one faces so the
//!   track bends in an arc from the one before (`Heading`). Holding Shift places at the full reach (`MAX_SPAN`)
//!   along the way you face instead.
//! - Click a node to select it (again to deselect). With one selected, clicking another node joins them; a
//!   crouch-click on a joined node cuts the track.
//! - `Factory::track_fit` decides what fits (span, grade, bend); the preview curve and the outline colours show it.
//!
//! The preview: ghost node and curve (`write_rail_preview`), outlines (`rail_boxes`) and label (`rail_label`).

use crate::action::Action;
use crate::block::{self, RAIL};
use crate::factory::{dir_of, fit_track, write_node, write_track, yaw_of, Curve, Fit, MAX_LINKS, MAX_SPAN};
use crate::math::{IVec3, Vec3};
use crate::power_tools::plan_pole;
use crate::sound;
use crate::Game;

/// Ticks a just-placed node counts as standing before the core has it (a co-op client's round trip).
const PENDING_TICKS: u32 = 90;
const UP: IVec3 = IVec3::new(0, 1, 0);
const GHOST_GREEN: i32 = 0x4ade80;
const ANCHOR_BLUE: i32 = 0x7dd3fc;
const BLOCKED_RED: i32 = 0xff4a3d;
const CUT_AMBER: i32 = 0xfbbf24;

/// The track hand's input state: never saved.
#[derive(Default)]
pub struct RailTools {
    /// The use button has been down since the last press was handled.
    down: bool,
    /// The node new nodes join.
    sel: Option<IVec3>,
    /// The last node this tool sent, its heading and the ticks since (it counts before the core has it).
    last: Option<(IVec3, u8, u32)>,
}

/// What a click would do now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    /// Aiming at a node: select it, or join it to the selected one (`fit`), or cut the track between them.
    Node { pos: IVec3, fit: Option<Fit> },
    /// Aiming at open ground: a node would go at `pos` facing `yaw`, joined to the selected one (`fit`).
    Place { pos: IVec3, yaw: u8, fit: Option<Fit>, free: bool },
}

impl Game {
    fn holds_rail(&self) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item.places() == Some(RAIL)
    }

    /// The selected node and its heading, while it stands (or has just been placed).
    fn rail_anchor(&self) -> Option<(IVec3, u8)> {
        let sel = self.rails.sel?;
        let pending = self.rails.last.filter(|l| l.0 == sel && l.2 < PENDING_TICKS).map(|l| l.1);
        Some((sel, self.sim.factory.rail_yaw(sel).or(pending)?))
    }

    /// The heading of a node placed at `pos` after the one at `from` facing `from_yaw`: the mirror of that
    /// heading in the chord, which makes the track an arc (straight when the chord lies along the heading).
    fn follow_yaw(from: IVec3, from_yaw: u8, pos: IVec3) -> u8 {
        let (cx, cz) = (f64::from(pos.x - from.x), f64::from(pos.z - from.z));
        let run = cx.hypot(cz);
        if run < 1e-9 {
            return from_yaw;
        }
        let (cx, cz) = (cx / run, cz / run);
        let (dx, dz) = dir_of(from_yaw);
        let dot = dx * cx + dz * cz;
        yaw_of(2.0 * dot * cx - dx, 2.0 * dot * cz - dz)
    }

    fn rail_plan(&self) -> Option<Plan> {
        if !self.holds_rail() {
            return None;
        }
        let f = &self.sim.factory;
        let anchor = self.rail_anchor();
        if let Some(hit) = self.target.filter(|h| f.rail_yaw(h.block).is_some()) {
            let fit =
                anchor.and_then(|(a, _)| f.track_fit(a, hit.block)).filter(|_| anchor.map(|a| a.0) != Some(hit.block));
            return Some(Plan::Node { pos: hit.block, fit });
        }
        let world = &self.sim.world;
        let look = self.body().look_dir();
        // Crouching starts a new line instead of continuing the selected one.
        let anchor = anchor.filter(|_| !self.body().input.crouch);
        let (pos, yaw) = match anchor {
            Some((a, ya)) => {
                let aim = self.aim_cell();
                let snap = self.body().input.sprint;
                let (pos, _) = plan_pole(|p| world.get_block(p), a, aim, look, MAX_SPAN, snap)?;
                (pos, Self::follow_yaw(a, ya, pos))
            }
            None => {
                let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
                (hit.block + hit.normal, yaw_of(look.x, look.z))
            }
        };
        let free = world.get_block(pos).is_some_and(block::replaceable);
        let fit =
            anchor.map(|(a, ya)| if f.track_count(a) >= MAX_LINKS { Fit::Full } else { fit_track(a, ya, pos, yaw) });
        Some(Plan::Place { pos, yaw, fit, free })
    }

    /// Runs the track hand for one tick. True while rails are in hand (the press is its own: a plain placement
    /// would put a node down with no heading and no track).
    pub(crate) fn update_rail_tools(&mut self) -> bool {
        if let Some(last) = self.rails.last.as_mut() {
            last.2 = last.2.saturating_add(1);
        }
        if !self.holds_rail() {
            self.rails.down = false;
            return false;
        }
        let edge = self.using && !self.rails.down;
        self.rails.down = self.using;
        if !edge {
            return true;
        }
        let crouch = self.body().input.crouch;
        let anchor = self.rail_anchor().map(|a| a.0);
        match self.rail_plan() {
            Some(Plan::Node { pos, fit }) => match (anchor, fit) {
                (Some(a), Some(Fit::Ok)) => {
                    self.act(Action::Connect { pole: a, to: pos });
                    self.rails.sel = Some(pos);
                }
                (Some(a), Some(Fit::Joined)) if crouch => self.act(Action::Disconnect { pole: a, to: pos }),
                _ => self.rails.sel = Some(pos).filter(|&p| anchor != Some(p)),
            },
            Some(Plan::Place { pos, yaw, fit, free: true }) if matches!(fit, None | Some(Fit::Ok)) => {
                let slot = self.inventory().selected as u8;
                self.act(Action::PlaceBlock { pos, slot, facing: yaw, against: pos - UP });
                if let (Some(a), Some(Fit::Ok)) = (anchor, fit) {
                    self.act(Action::Connect { pole: a, to: pos });
                }
                self.rails.last = Some((pos, yaw, 0));
                self.rails.sel = Some(pos);
            }
            _ => return true,
        }
        let at = self.target.map_or(self.body().eye(), |h| h.block.as_vec3());
        self.play(sound::PLACE, block::sound::METAL, at + Vec3::new(0.5, 0.5, 0.5), 0.4);
        true
    }

    /// Outline boxes (7 numbers each, as `power_boxes`): the selected node, the aimed node or the ghost's cell.
    pub(crate) fn rail_boxes(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut boxed = |p: IVec3, colour: i32| out.extend([p.x, p.y, p.z, p.x, p.y, p.z, colour]);
        let Some(plan) = self.rail_plan() else { return out };
        if let Some((a, _)) = self.rail_anchor() {
            boxed(a, ANCHOR_BLUE);
        }
        match plan {
            Plan::Node { pos, fit: Some(Fit::Ok) } => boxed(pos, GHOST_GREEN),
            Plan::Node { pos, fit: Some(Fit::Joined) } => boxed(pos, CUT_AMBER),
            Plan::Node { pos, fit: Some(_) } => boxed(pos, BLOCKED_RED),
            Plan::Node { .. } => {}
            Plan::Place { pos, fit, free, .. } => {
                boxed(pos, if free && matches!(fit, None | Some(Fit::Ok)) { GHOST_GREEN } else { BLOCKED_RED })
            }
        }
        out
    }

    /// The HUD lines while rails are in hand: a title, then what a click does (or what stops it).
    pub(crate) fn rail_label(&self) -> String {
        let Some(plan) = self.rail_plan() else { return String::new() };
        let anchor = self.rail_anchor();
        match plan {
            Plan::Node { pos, fit } => match (fit, anchor) {
                (_, Some((a, _))) if a == pos => "Rail node (selected)\nclick to deselect".to_string(),
                (Some(Fit::Ok), Some((a, ya))) => {
                    let yb = self.sim.factory.rail_yaw(pos).unwrap_or(0);
                    format!("Join to the selected node\nclick to lay {}", describe_span(a, ya, pos, yb))
                }
                (Some(Fit::Joined), _) => {
                    "Joined to the selected node\ncrouch-click to cut the track · click to select".to_string()
                }
                (Some(why), _) => format!("Rail node\n{} · click to select it instead", why_not(why)),
                (None, _) => "Rail node\nclick to select it, then join it to another node".to_string(),
            },
            Plan::Place { pos, yaw, fit, free } => match (fit, anchor) {
                _ if !free => "Rail node\nSomething is in the way".to_string(),
                (Some(Fit::Ok), Some((a, ya))) => {
                    let how = if self.body().input.sprint {
                        "release Shift: free placement"
                    } else {
                        "hold Shift: full reach"
                    };
                    format!("Rail node\nclick to place and lay {} · {how}", describe_span(a, ya, pos, yaw))
                }
                (Some(why), _) => format!("Rail node\n{} · move the aim", why_not(why)),
                (None, _) => "Rail node\nclick to place the first node: its track runs the way you face".to_string(),
            },
        }
    }

    /// Ghost node and curve, drawn with the machines.
    pub(crate) fn write_rail_preview(&mut self, eye: Vec3) {
        let Some(plan) = self.rail_plan() else { return };
        let anchor = self.rail_anchor();
        let curve = |a: (IVec3, u8), b: IVec3, yb: u8, fit: Option<Fit>| {
            let shows = matches!(fit, Some(Fit::Ok | Fit::TooSteep | Fit::TooSharp));
            shows.then(|| Curve::new(a.0, a.1, b, yb))
        };
        let lit = match plan {
            Plan::Place { pos, yaw, fit, free } if free => {
                let rel = pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
                write_node(&mut self.instances, rel, yaw, fit.is_none());
                anchor.and_then(|a| curve(a, pos, yaw, fit))
            }
            Plan::Node { pos, fit: Some(Fit::Ok) } => {
                anchor.zip(self.sim.factory.rail_yaw(pos)).and_then(|(a, yb)| curve(a, pos, yb, Some(Fit::Ok)))
            }
            _ => None,
        };
        if let Some(c) = lit {
            write_track(&mut self.instances, &c, eye, 1.0e9);
        }
    }
}

/// "a 14-block track" with its grade: the length of the curve between two nodes.
fn describe_span(a: IVec3, ya: u8, b: IVec3, yb: u8) -> String {
    let curve = Curve::new(a, ya, b, yb);
    let (d, rise) = (b - a, (b.y - a.y).abs());
    let run = f64::from(d.x).hypot(f64::from(d.z));
    let slope =
        if rise == 0 { String::new() } else { format!(", {}% slope", (f64::from(rise) / run * 100.0).round() as i32) };
    format!("{} blocks of track{slope}", curve.length().round() as i32)
}

fn why_not(fit: Fit) -> String {
    match fit {
        Fit::Ok => String::new(),
        Fit::Joined => "Already joined".to_string(),
        Fit::TooFar => format!("Too far: nodes join up to {MAX_SPAN} blocks apart"),
        Fit::TooClose => "Too close to the selected node".to_string(),
        Fit::TooSteep => "Too steep: track climbs at most 1 in 3".to_string(),
        Fit::TooSharp => "Bend too sharp: the node's heading or a radius under 8 blocks".to_string(),
        Fit::Full => format!("A node holds at most {MAX_LINKS} tracks"),
    }
}

#[cfg(test)]
mod tests;

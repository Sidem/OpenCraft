//! Rails (Milestone 9): track a train will run on, built like power poles. A rail is a *node*: a block on a cell
//! with a heading (a byte, `curve::dir_of`), and the player joins two nodes with a *track* (`Action::Connect`,
//! quietly nothing unless `track_fit` says the pair fits). A track is a smooth curve between its two nodes
//! (`curve.rs`), free of the voxel grid; only the nodes sit on cells. A node may hold up to `MAX_LINKS` tracks
//! (a junction needs three).
//!
//! Saved: each node's cell and heading, then the tracks as pairs of node cells (`state.rs`). Derived: each
//! node's `links` count (`relink`). Removing a node drops its tracks.
//!
//! To use the track: `Factory::tracks` and `curve_of` give the graph and its shapes (trains, step 9.4b).

mod curve;

pub use curve::{dir_of, fit, write_node, write_track, yaw_of, Curve, Fit, MAX_SPAN};

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::RAIL_SIGNAL;
use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::{Factory, Machine};

/// The most tracks one node holds.
pub const MAX_LINKS: u8 = 4;

/// A rail node: where it is and the way its track runs through it (the heading is an axis: the track may leave
/// either side).
pub struct Rail {
    pub pos: IVec3,
    pub yaw: u8,
    /// How many tracks end here (derived by `relink`).
    pub links: u8,
    /// A signal stands here (`trains/signals.rs`); saved since 35.
    pub signal: bool,
}

/// A stretch of track between the nodes at `a` and `b`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Track {
    pub a: IVec3,
    pub b: IVec3,
}

impl Rail {
    pub fn new(pos: IVec3, yaw: u8) -> Rail {
        Rail { pos, yaw, links: 0, signal: false }
    }
}

impl Track {
    fn has(&self, p: IVec3) -> bool {
        self.a == p || self.b == p
    }
}

impl Factory {
    /// The heading of the rail node at `pos`, if there is one.
    pub fn rail_yaw(&self, pos: IVec3) -> Option<u8> {
        match self.at.get(&pos) {
            Some(&Slot::Rail(i)) => Some(self.rails[i as usize].yaw),
            _ => None,
        }
    }

    /// Whether the nodes at `a` and `b` are joined.
    pub fn track_between(&self, a: IVec3, b: IVec3) -> bool {
        self.tracks.iter().any(|t| t.has(a) && t.has(b))
    }

    /// How many tracks end at the node at `p`.
    pub fn track_count(&self, p: IVec3) -> u8 {
        self.tracks.iter().filter(|t| t.has(p)).count() as u8
    }

    /// Whether the nodes at `a` and `b` can be joined, `None` unless both are rail nodes.
    pub fn track_fit(&self, a: IVec3, b: IVec3) -> Option<Fit> {
        let (ya, yb) = (self.rail_yaw(a)?, self.rail_yaw(b)?);
        Some(if self.track_between(a, b) {
            Fit::Joined
        } else if self.track_count(a) >= MAX_LINKS || self.track_count(b) >= MAX_LINKS {
            Fit::Full
        } else {
            fit(a, ya, b, yb)
        })
    }

    /// Joins the nodes at `a` and `b` (`Action::Connect`); quietly nothing unless they fit.
    pub fn lay_track(&mut self, a: IVec3, b: IVec3) {
        if self.track_fit(a, b) == Some(Fit::Ok) {
            self.tracks.push(Track { a, b });
            self.dirty = true;
        }
    }

    /// Cuts the track between the nodes at `a` and `b` (`Action::Disconnect`).
    pub fn cut_track(&mut self, a: IVec3, b: IVec3) {
        if self.train_on(a, b) {
            return; // a train is on it
        }
        let before = self.tracks.len();
        self.tracks.retain(|t| !(t.has(a) && t.has(b)));
        self.dirty |= self.tracks.len() != before;
    }

    /// The curve of `track`, `None` if an end is not a node.
    pub fn curve_of(&self, track: &Track) -> Option<Curve> {
        Some(Curve::new(track.a, self.rail_yaw(track.a)?, track.b, self.rail_yaw(track.b)?))
    }

    /// Every track laid. Tests for now: trains (9.4b) route along it.
    #[cfg(test)]
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Counts each node's tracks (called from `relink`).
    pub(super) fn link_rails(&mut self) {
        for r in &mut self.rails {
            r.links = 0;
        }
        for t in &self.tracks {
            for end in [t.a, t.b] {
                if let Some(&Slot::Rail(i)) = self.at.get(&end) {
                    self.rails[i as usize].links += 1;
                }
            }
        }
    }

    /// Drops tracks whose ends are gone.
    pub(super) fn prune_tracks(&mut self) {
        if !self.tracks.is_empty() {
            let at = &self.at;
            self.tracks.retain(|t| [t.a, t.b].iter().all(|p| matches!(at.get(p), Some(Slot::Rail(_)))));
        }
    }

    /// Draws every track near the camera (called by `write_instances`).
    pub(super) fn write_tracks(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        for t in &self.tracks {
            let mid = (t.a.as_vec3() + t.b.as_vec3()) * 0.5 + Vec3::new(0.5, 0.5, 0.5) - eye;
            if mid.length() <= range + f64::from(MAX_SPAN) {
                if let Some(curve) = self.curve_of(t) {
                    write_track(out, &curve, eye, range);
                }
            }
        }
    }
}

/// The heading's axis, in words.
fn axis(yaw: u8) -> &'static str {
    const AXES: [&str; 4] = ["north-south", "north-east to south-west", "east-west", "south-east to north-west"];
    AXES[((u32::from(yaw) % 128 + 16) / 32 % 4) as usize]
}

impl Machine for Rail {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.yaw);
        w.bool(self.signal);
    }

    fn read_state(r: &mut ByteReader) -> Option<Rail> {
        let pos = r.ivec3()?;
        let mut rail = Rail::new(pos, if r.version >= 31 { r.u8()? } else { 0 });
        rail.signal = r.version >= 35 && r.bool()?;
        Some(rail)
    }

    /// A signal comes back as its item when the node is taken.
    fn contents(&self) -> Vec<Stack> {
        if self.signal {
            vec![Stack { item: RAIL_SIGNAL, count: 1 }]
        } else {
            Vec::new()
        }
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return "Rail node".to_string();
        }
        let axis = axis(self.yaw);
        let signal = if self.signal { " · signal" } else { "" };
        match self.links {
            0 => format!("Rail node\nTrack runs {axis} here: join it to another node"),
            1 => format!("Rail node\nEnd of the track · runs {axis}{signal}"),
            n => format!("Rail node\n{n} of {MAX_LINKS} tracks · runs {axis}{signal}"),
        }
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        curve::write_node(out, rel, self.yaw, self.links == 0);
        if self.signal {
            curve::write_signal(out, rel, self.yaw);
        }
    }
}

#[cfg(test)]
mod tests;

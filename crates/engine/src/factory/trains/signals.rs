//! Signals: a rail node may carry one (`Rail::signal`, put there with `Action::ToggleSignal`). Signals cut the track
//! into *sections*: the tracks that join through nodes without a signal. A train passing a signal node may only go
//! on into a section no other train is in; at a junction it takes the straightest branch whose section is free, and
//! when none is it waits at the node (no state: it asks again every tick). Nodes without a signal never hold a train
//! back, so a railway without signals runs as before.
//!
//! A train occupies every edge of its `path`, so a long train holds the sections it straddles. Invariants: the
//! checks read only the track graph and the trains, in train order, so every peer agrees. A layout can still
//! deadlock (two trains each waiting for the section the other is in, as on a single line with a signal in the
//! middle); a passing loop between two signals is how trains meet and pass.

use crate::math::IVec3;

use super::super::links::Slot;
use super::{key, Factory, Train, MIN_AHEAD};

/// What a train that has reached the end of its last edge does next.
pub(super) enum Way {
    /// Go on along the track to this node.
    Go(IVec3),
    /// A signal holds it: every way on leads into a section another train is in.
    Wait,
    /// No way on: turn round.
    End,
}

type Edge = (IVec3, IVec3);

fn edge(a: IVec3, b: IVec3) -> Edge {
    if key(a) <= key(b) {
        (a, b)
    } else {
        (b, a)
    }
}

impl Factory {
    /// Whether the rail node at `pos` carries a signal.
    pub fn signal_at(&self, pos: IVec3) -> bool {
        match self.at.get(&pos) {
            Some(&Slot::Rail(i)) => self.rails[i as usize].signal,
            _ => false,
        }
    }

    /// Puts a signal on the rail node at `pos` or takes it off; false (nothing done) if it is not a rail node or
    /// already is as asked.
    pub fn set_signal(&mut self, pos: IVec3, on: bool) -> bool {
        match self.at.get(&pos) {
            Some(&Slot::Rail(i)) if self.rails[i as usize].signal != on => {
                self.rails[i as usize].signal = on;
                true
            }
            _ => false,
        }
    }

    /// The tracks of the section the track from `from` to `to` is in.
    fn section(&self, from: IVec3, to: IVec3) -> Vec<Edge> {
        let mut edges = vec![edge(from, to)];
        let mut i = 0;
        while i < edges.len() {
            let (a, b) = edges[i];
            i += 1;
            for end in [a, b].into_iter().filter(|&p| !self.signal_at(p)) {
                for q in self.neighbours(end) {
                    let e = edge(end, q);
                    if !edges.contains(&e) {
                        edges.push(e);
                    }
                }
            }
        }
        edges
    }

    /// Whether a train is on the section of the track from `from` to `to`. (While a train is being stepped its own
    /// place in `trains` is empty, so this is about the others.)
    fn section_taken(&self, from: IVec3, to: IVec3) -> bool {
        if self.trains.is_empty() {
            return false;
        }
        let section = self.section(from, to);
        self.trains.iter().any(|t| t.path.iter().any(|&(a, b)| section.contains(&edge(a, b))))
    }

    /// Where the train `t`, at the end of its edge from `prev` to `at`, goes on.
    pub(super) fn way(&self, t: &mut Train, prev: IVec3, at: IVec3) -> Way {
        let Some(first) = self.steer(t, prev, at) else { return Way::End };
        if !self.signal_at(at) || !self.section_taken(at, first) {
            return Way::Go(first);
        }
        // The way it would take is blocked: any other branch it may turn onto whose section is free.
        let Some(ahead) = self.run(prev, at).map(|c| c.heading(1.0)) else { return Way::Wait };
        let mut best: Option<(f64, IVec3)> = None;
        for q in self.neighbours(at).into_iter().filter(|&q| q != prev && q != first) {
            let Some((x, z)) = self.run(at, q).map(|c| c.heading(0.0)) else { continue };
            let dot = ahead.0 * x + ahead.1 * z;
            if dot >= MIN_AHEAD && best.is_none_or(|b| dot > b.0) && !self.section_taken(at, q) {
                best = Some((dot, q));
            }
        }
        best.map_or(Way::Wait, |b| Way::Go(b.1))
    }
}

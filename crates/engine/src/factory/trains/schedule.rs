//! Schedules: a train with a list of docks stops only at the next one on it, then the one after, round and round; one
//! with an empty list stops at every dock it reaches (`docks.rs`). At a node it goes on toward the scheduled dock by
//! the shortest route (`route`: a breadth-first search over the track the train may turn onto, within `MAX_STATES`
//! states), else as `next_from` does. Docks are kept by their anchor cell; one that is gone is skipped.
//! The hand edits a schedule with `Action::TrainStop` (`set_stop`).

use crate::math::IVec3;

use super::super::links::Slot;
use super::docks::near as near_dock;
use super::{key, Factory, Train, MIN_AHEAD};

/// Docks on one train's schedule at most.
pub const MAX_STOPS: usize = 8;
/// The most track states a route search looks at.
const MAX_STATES: usize = 512;

impl Factory {
    /// Adds the dock at `dock` to the end of the schedule of the train near the node at `node` (not twice in a row,
    /// at most `MAX_STOPS`), or with `clear` empties that schedule. False (nothing done) if no train is near or the
    /// cell is no dock.
    pub fn set_stop(&mut self, node: IVec3, dock: IVec3, clear: bool) -> bool {
        let Some(i) = self.train_near(node) else { return false };
        if clear {
            self.trains[i].schedule.clear();
            self.trains[i].next = 0;
            return true;
        }
        let Some(anchor) = self.dock_anchor(dock) else { return false };
        let t = &mut self.trains[i];
        if t.schedule.len() >= MAX_STOPS || t.schedule.last() == Some(&anchor) {
            return false;
        }
        t.schedule.push(anchor);
        true
    }

    /// The anchor of the dock that has a cell at `cell`.
    pub fn dock_anchor(&self, cell: IVec3) -> Option<IVec3> {
        match self.at.get(&cell) {
            Some(&Slot::Process(i)) if self.processors[i as usize].spec.pick.docks() => {
                Some(self.processors[i as usize].pos)
            }
            _ => None,
        }
    }

    /// The schedule of the train near the node at `node`, as dock anchors.
    pub fn schedule_near(&self, node: IVec3) -> Vec<IVec3> {
        self.train_near(node).map_or_else(Vec::new, |i| self.trains[i].schedule.clone())
    }

    /// The processor index of the dock `t` is bound for, skipping docks that are gone (`None`: no schedule or
    /// no dock left on it).
    pub(super) fn scheduled(&self, t: &mut Train) -> Option<usize> {
        let n = t.schedule.len();
        for _ in 0..n {
            let i = usize::from(t.next) % n;
            t.next = i as u8;
            match self.at.get(&t.schedule[i]) {
                Some(&Slot::Process(p)) if self.processors[p as usize].spec.pick.docks() => return Some(p as usize),
                _ => t.next = ((i + 1) % n) as u8,
            }
        }
        None
    }

    /// The dock a train reaching node `at` stops at, if it stops there at all.
    pub(super) fn stop_dock(&self, t: &mut Train, at: IVec3) -> Option<usize> {
        if t.schedule.is_empty() {
            return self.dock_at(at);
        }
        self.scheduled(t).filter(|&p| near_dock(&self.processors[p], at))
    }

    /// The node a train that came from `prev` to `at` goes on to: toward its next dock if it has one it can reach.
    pub(super) fn steer(&self, t: &mut Train, prev: IVec3, at: IVec3) -> Option<IVec3> {
        let goals: Vec<IVec3> = match self.scheduled(t) {
            Some(p) => self.rails.iter().map(|r| r.pos).filter(|&n| near_dock(&self.processors[p], n)).collect(),
            None => Vec::new(),
        };
        self.route(prev, at, &goals).or_else(|| self.next_from(prev, at))
    }

    /// The first node of the shortest way on from `at` (arrived from `prev`) to any of `goals`.
    fn route(&self, prev: IVec3, at: IVec3, goals: &[IVec3]) -> Option<IVec3> {
        if goals.is_empty() {
            return None;
        }
        // States are (came from, now at, the first node taken); a train never turns back on itself or past 60 degrees.
        let mut queue = vec![(prev, at, None::<IVec3>)];
        let mut seen = vec![(key(prev), key(at))];
        let mut head = 0;
        while head < queue.len() && queue.len() < MAX_STATES {
            let (from, here, first) = queue[head];
            head += 1;
            let Some(ahead) = self.run(from, here).map(|c| c.heading(1.0)) else { continue };
            for q in self.neighbours(here).into_iter().filter(|&q| q != from) {
                let Some((x, z)) = self.run(here, q).map(|c| c.heading(0.0)) else { continue };
                if ahead.0 * x + ahead.1 * z < MIN_AHEAD || seen.contains(&(key(here), key(q))) {
                    continue;
                }
                let first = first.or(Some(q));
                if goals.contains(&q) {
                    return first;
                }
                seen.push((key(here), key(q)));
                queue.push((here, q, first));
            }
        }
        None
    }
}

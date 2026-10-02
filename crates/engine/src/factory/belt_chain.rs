//! The belts joined to one belt, for upgrading a whole line at once (`upgrade_aim.rs`).
//!
//! A belt is joined to the belt it delivers to and to every belt delivering to it (`Belt::out`, so ramps,
//! lifts, underpasses and side joins count), but not through splitters, filters or machines. Only belts of
//! the starting belt's tier are followed. Reads the derived links, so it needs a relinked factory.

use crate::math::IVec3;

use super::links::{Link, Slot};
use super::Factory;

const NONE: u32 = u32::MAX;

impl Factory {
    /// Where the belts joined to the belt at `start` stand, all of its tier, nearest first (`start` is the
    /// first), at most `max`. Empty when no belt is there; just `start` while the links are out of date.
    pub fn belt_chain(&self, start: IVec3, max: usize) -> Vec<IVec3> {
        let Some(&Slot::Belt(first)) = self.at.get(&start) else { return Vec::new() };
        if self.dirty {
            return vec![start];
        }
        // `feeders[b]` heads a list (continued by `more`) of the belts whose output is belt `b`.
        let mut feeders = vec![NONE; self.belts.len()];
        let mut more = vec![NONE; self.belts.len()];
        for (i, belt) in self.belts.iter().enumerate() {
            if let Link::Belt { belt: to, .. } = belt.out {
                more[i] = feeders[to as usize];
                feeders[to as usize] = i as u32;
            }
        }
        let tier = self.belts[first as usize].tier;
        let mut seen = vec![false; self.belts.len()];
        seen[first as usize] = true;
        let mut found = vec![first];
        let mut next = 0;
        while next < found.len() && found.len() < max {
            let b = found[next] as usize;
            next += 1;
            let mut joined = Vec::with_capacity(4);
            if let Link::Belt { belt: to, .. } = self.belts[b].out {
                joined.push(to);
            }
            let mut f = feeders[b];
            while f != NONE {
                joined.push(f);
                f = more[f as usize];
            }
            for c in joined {
                if !seen[c as usize] && self.belts[c as usize].tier == tier {
                    seen[c as usize] = true;
                    found.push(c);
                }
            }
        }
        found.truncate(max);
        found.into_iter().map(|i| self.belts[i as usize].pos).collect()
    }
}

#[cfg(test)]
mod tests;

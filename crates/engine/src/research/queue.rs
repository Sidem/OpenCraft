//! The research queue: techs that follow the current one, in order, so the player picks several at once.
//!
//! - `Research::queue` holds the techs after `current`. When the current tech is done (`add_unit`) or given up,
//!   the first queued tech takes over (`advance`).
//! - Invariant (`prune` restores it after every change): a queued tech is not done, not current, listed once, and
//!   every prerequisite is done, current, or earlier in the queue. At most `MAX_QUEUE` long. An endless bonus tech
//!   never finishes, so it can only be last: nothing is queued behind one.
//! - Queueing a locked tech adds the prerequisites it is missing in front of it (`enqueue`). Choosing a tech to
//!   research now (`set_current`) puts the one it replaces at the front of the queue, and giving the current one
//!   up (`set_current(None)`) moves on to the next.
//!
//! To change how long the queue may be: `MAX_QUEUE`. The queue is core state, saved after the progress.

use crate::bytes::{ByteReader, ByteWriter};

use super::{is_bonus, Research, TechState, TECHS};

/// The most techs that wait behind the current one (room for the longest chain of prerequisites, with some to spare).
pub const MAX_QUEUE: usize = 32;

impl Research {
    /// Techs waiting behind the current one, next first.
    pub fn queue(&self) -> &[u8] {
        &self.queue
    }

    /// Adds `tech` to the end of the queue, with the prerequisites it is missing in front of it (a free lab line
    /// takes the first one at once). Ignored when it is done, already chosen or queued, would pass `MAX_QUEUE`,
    /// or would come after an endless tech.
    pub fn enqueue(&mut self, tech: u8) {
        let after_endless = self.queue.last().copied().or(self.current).is_some_and(is_bonus);
        if after_endless
            || (tech as usize) >= TECHS.len()
            || self.state(tech) == TechState::Done
            || self.is_chosen(tech)
        {
            return;
        }
        // The tech and the prerequisites it is missing, in index order (prerequisites always have lower indices).
        let mut missing = vec![tech];
        let mut i = 0;
        while i < missing.len() {
            for &n in TECHS[missing[i] as usize].needs {
                if self.state(n) != TechState::Done && !self.is_chosen(n) && !missing.contains(&n) {
                    missing.push(n);
                }
            }
            i += 1;
        }
        missing.sort_unstable();
        // The first one becomes current when nothing is, so it doesn't count against the queue.
        if self.queue.len() + missing.len() > MAX_QUEUE + usize::from(self.current.is_none()) {
            return;
        }
        self.queue.extend(missing);
        self.prune();
        if self.current.is_none() {
            self.advance();
        }
    }

    /// Takes `tech` and every queued tech that needs it out of the queue.
    pub fn dequeue(&mut self, tech: u8) {
        self.queue.retain(|&q| q != tech);
        self.prune();
    }

    /// Whether `tech` is the current tech or waits in the queue.
    pub fn is_chosen(&self, tech: u8) -> bool {
        self.current == Some(tech) || self.queue.contains(&tech)
    }

    /// Makes the first queued tech the current one (no-op while one is being researched).
    pub(super) fn advance(&mut self) {
        if self.current.is_none() && !self.queue.is_empty() {
            self.current = Some(self.queue.remove(0));
        }
    }

    /// Restores the queue invariant: drops what is done, current or repeated, what needs a tech that is no longer
    /// ahead of it, what comes after an endless tech, and what passes `MAX_QUEUE`.
    pub(super) fn prune(&mut self) {
        let mut kept: Vec<u8> = Vec::with_capacity(self.queue.len());
        for &q in &self.queue {
            let ready = TECHS.get(q as usize).is_some_and(|t| {
                t.needs
                    .iter()
                    .all(|&n| self.state(n) == TechState::Done || self.current == Some(n) || kept.contains(&n))
            });
            let behind_endless = kept.last().is_some_and(|&l| is_bonus(l));
            let wanted = self.state(q) != TechState::Done && self.current != Some(q) && !kept.contains(&q);
            if ready && wanted && !behind_endless && kept.len() < MAX_QUEUE {
                kept.push(q);
            }
        }
        self.queue = kept;
    }

    pub(super) fn write_queue(&self, w: &mut ByteWriter) {
        w.count(self.queue.len());
        self.queue.iter().for_each(|&q| w.u8(q));
    }

    /// Reads the queue and drops whatever no longer fits (a tech table that changed).
    pub(super) fn read_queue(&mut self, r: &mut ByteReader) -> Option<()> {
        let n = r.count()?;
        if n > MAX_QUEUE {
            return None;
        }
        for _ in 0..n {
            self.queue.push(r.u8()?);
        }
        self.prune();
        Some(())
    }
}

#[cfg(test)]
mod tests;

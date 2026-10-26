//! Merging resting loose items into stacks, so a belt ejector (or a long day of throwing and mining) leaves a few
//! piles on the ground, not hundreds of entities. Host-side presentation like the rest of `entities.rs`: the core
//! never sees loose items, so nothing here touches determinism or the state hash.
//!
//! Invariants (checked by `entities/tests.rs`): a merge never changes the total of an item, never makes a stack
//! larger than `item::stack_size`, never joins different items or tools (a tool's count is its uses, not a pile),
//! and only joins items that lie still: on the ground, or waiting in a chunk that is not loaded (there nothing ever
//! lands, so an ejector far away would otherwise pile up floating items). Items still in their pickup delay, in the
//! air, in water or flying to a player are left alone. The pile keeps the older item's id and place; its age is the
//! average of the two weighted by count, so adding to a pile never keeps it alive for long.
//!
//! To change how eagerly items merge: [`MERGE_SECONDS`] (how often the pass runs) and [`MERGE_CELL`] (how close).

use std::collections::HashMap;

use crate::item::stack_size;
use crate::math::Vec3;
use crate::tools;

use super::{ItemEntity, Items};

/// Seconds between merge passes.
pub(super) const MERGE_SECONDS: f32 = 0.25;
/// Side of the grid cell items must share to merge, in blocks.
pub const MERGE_CELL: f64 = 1.5;

type Cell = (i32, i32, i32, u16);

impl Items {
    /// Counts `dt` and, once per [`MERGE_SECONDS`], merges resting items of one kind that share a grid cell.
    pub(super) fn merge_resting(&mut self, dt: f32, loaded: &impl Fn(Vec3) -> bool) {
        self.merge_clock += dt;
        if self.merge_clock < MERGE_SECONDS {
            return;
        }
        self.merge_clock = 0.0;
        // The newest pile with room in each cell, by index (earlier items win ties).
        let mut piles: HashMap<Cell, usize> = HashMap::new();
        let mut merged = false;
        for i in 0..self.list.len() {
            if !can_merge(&self.list[i], loaded) {
                continue;
            }
            let cell = cell_of(&self.list[i]);
            let Some(&p) = piles.get(&cell) else {
                piles.insert(cell, i);
                continue;
            };
            let (head, tail) = self.list.split_at_mut(i);
            let (pile, e) = (&mut head[p], &mut tail[0]);
            let moved = e.count.min(stack_size(pile.item) - pile.count);
            let total = pile.count + moved;
            pile.age = (pile.age * pile.count as f32 + e.age * moved as f32) / total as f32;
            pile.count = total;
            e.count -= moved;
            merged = true;
            if e.count > 0 {
                // The pile is full: this one starts the next pile in the cell.
                piles.insert(cell, i);
            }
        }
        if merged {
            self.list.retain(|e| e.count > 0);
        }
    }
}

/// Whether `e` lies still, is past its pickup delay and has room to join (or start) a pile.
fn can_merge(e: &ItemEntity, loaded: &impl Fn(Vec3) -> bool) -> bool {
    let resting = e.on_ground || !loaded(e.pos);
    resting && e.age >= e.pickup_delay && tools::tool(e.item).is_none() && e.count < stack_size(e.item)
}

fn cell_of(e: &ItemEntity) -> Cell {
    let at = e.pos * (1.0 / MERGE_CELL);
    (at.x.floor() as i32, at.y.floor() as i32, at.z.floor() as i32, e.item.0)
}

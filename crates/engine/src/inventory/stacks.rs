//! Operations on stacks and slices of stacks, shared by the player inventory and storage boxes: topping up
//! (`add_to_slots`), moving every stack of an item (`move_all_of`), sorting, and the screen clicks on one slot with
//! the cursor stack: `click_stack` (pick up, put down, merge, swap), `split_stack` (right-click: half) and
//! `place_one` (shift-right-click). A tool's count is its wear, so tools never merge or split.

use super::Stack;
use crate::item::{stack_size, ItemId};
use crate::math::sort_small_by_key;
use crate::tools;

/// Adds up to `count` of `item` into `slots`, topping up matching stacks before filling empty
/// slots (in slice order). Returns what didn't fit.
pub fn add_to_slots(slots: &mut [Stack], item: ItemId, mut count: u32) -> u32 {
    if item == ItemId::NONE {
        return count;
    }
    let max = stack_size(item);
    for s in slots.iter_mut().filter(|s| !s.is_empty() && s.item == item) {
        let n = count.min(max.saturating_sub(s.count));
        s.count += n;
        count -= n;
    }
    for s in slots.iter_mut().filter(|s| s.is_empty()) {
        if count == 0 {
            break;
        }
        let n = count.min(max);
        *s = Stack { item, count: n };
        count -= n;
    }
    count
}

/// Moves every stack of `item` from `from` into `to` until `to` has no room for more; what doesn't fit
/// stays in `from`. Returns whether anything moved.
pub fn move_all_of(from: &mut [Stack], to: &mut [Stack], item: ItemId) -> bool {
    let mut moved = false;
    for s in from.iter_mut().filter(|s| s.item == item && !s.is_empty()) {
        let left = add_to_slots(to, item, s.count);
        moved |= left != s.count;
        *s = if left == 0 { Stack::default() } else { Stack { item, count: left } };
        if left > 0 {
            break; // the target is full of this item
        }
    }
    moved
}

/// Sorts `slots` in place: like items merged into full stacks (a tool's count is its wear, so tools
/// never merge), ordered by item id (blocks first) with the fullest stack first, empty slots last.
pub fn sort_stacks(slots: &mut [Stack]) {
    let mut sorted: Vec<Stack> = Vec::with_capacity(slots.len());
    for s in slots.iter().filter(|s| !s.is_empty()) {
        let mut left = s.count;
        if tools::tool(s.item).is_none() {
            for t in sorted.iter_mut().filter(|t| t.item == s.item) {
                let n = left.min(stack_size(s.item).saturating_sub(t.count));
                t.count += n;
                left -= n;
            }
        }
        if left > 0 {
            sorted.push(Stack { item: s.item, count: left });
        }
    }
    sort_small_by_key(&mut sorted, |s| (s.item.0, std::cmp::Reverse(s.count)));
    for (i, slot) in slots.iter_mut().enumerate() {
        *slot = sorted.get(i).copied().unwrap_or_default();
    }
}

/// A click on slot `s` holding the cursor stack `c` (inventory or box screen): pick up, put down,
/// merge or swap. Returns whether anything changed.
pub fn click_stack(s: &mut Stack, c: &mut Stack) -> bool {
    if c.is_empty() {
        if s.is_empty() {
            return false;
        }
        *c = std::mem::take(s);
    } else if s.is_empty() {
        *s = std::mem::take(c);
    } else if s.item == c.item {
        let n = c.count.min(stack_size(s.item).saturating_sub(s.count));
        s.count += n;
        c.count -= n;
        if c.count == 0 {
            *c = Stack::default();
        }
    } else {
        std::mem::swap(s, c);
    }
    true
}

/// A right-click on slot `s` with the cursor stack `c`: takes half of `s` (rounded up) onto an empty cursor, or half
/// of what is left onto a cursor holding the same item (up to a full stack). A tool, whose count is its wear, is
/// taken whole. Returns whether anything changed.
pub fn split_stack(s: &mut Stack, c: &mut Stack) -> bool {
    if s.is_empty() {
        return false;
    }
    let tool = tools::tool(s.item).is_some();
    let n = if c.is_empty() {
        if tool {
            s.count
        } else {
            s.count.div_ceil(2)
        }
    } else if s.item == c.item && !tool {
        s.count.div_ceil(2).min(stack_size(c.item).saturating_sub(c.count))
    } else {
        0
    };
    if n == 0 {
        return false;
    }
    *c = Stack { item: s.item, count: c.count + n };
    s.count -= n;
    if s.is_empty() {
        *s = Stack::default();
    }
    true
}

/// A shift-right-click on slot `s` with the cursor stack `c`: puts one item of `c` into `s`, if `s` is empty or holds
/// the same item with room (a tool goes whole, into an empty slot only). Returns whether anything changed.
pub fn place_one(s: &mut Stack, c: &mut Stack) -> bool {
    if c.is_empty() {
        return false;
    }
    let tool = tools::tool(c.item).is_some();
    let n = if tool { c.count } else { 1 };
    if s.is_empty() {
        *s = Stack { item: c.item, count: n };
    } else if !tool && s.item == c.item && s.count < stack_size(s.item) {
        s.count += 1;
    } else {
        return false;
    }
    c.count -= n;
    if c.is_empty() {
        *c = Stack::default();
    }
    true
}

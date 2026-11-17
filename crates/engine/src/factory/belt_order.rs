//! The order belts step in (`relink` sets it; `tick.rs` steps belts in it): every belt after the one it feeds, so an item
//! moves at most one belt a tick, whatever order the belts were built in. Derived from the belts' links, never saved.

use super::belt::Belt;
use super::links::Link;

/// Fills `order` with every belt's index, each after the belt it feeds. Each belt has at most one belt downstream, so
/// walking the chain from every unvisited belt and appending it reversed does it.
pub(super) fn belt_order(belts: &[Belt], order: &mut Vec<u32>) {
    let mut visited = vec![false; belts.len()];
    let mut path = Vec::new();
    order.clear();
    for start in 0..belts.len() {
        let mut cur = start;
        path.clear();
        while !visited[cur] {
            visited[cur] = true;
            path.push(cur as u32);
            match belts[cur].out {
                Link::Belt { belt, .. } => cur = belt as usize,
                _ => break,
            }
        }
        order.extend(path.iter().rev());
    }
}

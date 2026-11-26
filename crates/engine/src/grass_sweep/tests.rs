//! The sweep finds dirt that could green in a column, and nothing else.

use super::*;
use crate::block::{AIR, GRASS};

/// A hillside step: grass on top at y 10 for x ≤ 0, one step down to grass at y 8 for x > 0, dirt under both. The
/// dirt at (0, 9) and (0, 8) shows a bare face towards +x.
fn hillside(p: IVec3) -> BlockId {
    let top = if p.x <= 0 { 10 } else { 8 };
    match p.y {
        y if y > top => AIR,
        y if y == top => GRASS,
        _ => DIRT,
    }
}

#[test]
fn the_sweep_finds_the_bare_step_of_a_hillside_and_not_buried_dirt() {
    let mut block = hillside;
    assert_eq!(seed_in_column(&mut block, IVec3::new(0, 30, 0)), Some(IVec3::new(0, 9, 0)));
    assert_eq!(seed_in_column(&mut block, IVec3::new(-3, 30, 0)), None, "dirt under a flat lawn is buried");
    assert_eq!(seed_in_column(&mut block, IVec3::new(0, 30 - 2 * DEPTH - 1, 0)), None, "the column ends");
}

#[test]
fn columns_spread_over_the_reach() {
    let span = (2 * REACH + 1) as u64;
    // 40,000 ticks (11 minutes) over 9,409 columns: a fair hash leaves about 1.4% unseen.
    let cells: std::collections::BTreeSet<_> = (0..40_000u64).map(|t| (mix(t) % span, (mix(t) >> 20) % span)).collect();
    assert!(cells.len() as u64 > span * span * 97 / 100, "{} of {}", cells.len(), span * span);
}

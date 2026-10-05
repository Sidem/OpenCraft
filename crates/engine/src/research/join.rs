//! Joins the tech tables into the one array [`techs::TECHS`](super::techs::TECHS), at compile time.

use super::Tech;

/// The `parts` one after another as one array of `N` = their lengths added.
pub(super) const fn join<const N: usize>(parts: &[&[Tech]]) -> [Tech; N] {
    let mut out = [parts[0][0]; N];
    let (mut at, mut p) = (0, 0);
    while p < parts.len() {
        let mut i = 0;
        while i < parts[p].len() {
            out[at] = parts[p][i];
            at += 1;
            i += 1;
        }
        p += 1;
    }
    assert!(at == N);
    out
}

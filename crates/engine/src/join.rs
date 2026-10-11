//! Joins const tables into one array at compile time (the tech table, the machine recipe table), so each table can
//! live in a file of its own and stay under the size budget.

/// The `parts` one after another as one array of `N` = their lengths added (checked at compile time).
pub(crate) const fn join<T: Copy, const N: usize>(parts: &[&[T]]) -> [T; N] {
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

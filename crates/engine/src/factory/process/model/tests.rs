use super::super::specs::SPECS;
use super::*;

/// The faces of two parts that lie in one plane, face the same way and overlap z-fight: the GPU can't tell which is in
/// front, so they flicker. Turned parts are skipped (their faces aren't axis-aligned in the model's frame).
#[test]
fn no_model_has_two_faces_in_one_plane() {
    let mut clashes = Vec::new();
    for spec in SPECS {
        let parts: Vec<&Part> = spec.parts.iter().filter(|p| p.turn == 0.0 && !matches!(p.look, Look::Smoke)).collect();
        for (i, a) in parts.iter().enumerate() {
            for (j, b) in parts.iter().enumerate().skip(i + 1) {
                if let Some(axis) = shared_face(a, b) {
                    clashes.push(format!("block {} parts {i} and {j} along {axis}", spec.block));
                }
            }
        }
    }
    assert!(clashes.is_empty(), "coplanar faces:\n{}", clashes.join("\n"));
}

/// The axis (0 x, 1 y, 2 z) on which `a` and `b` have a face in one plane, facing the same way and overlapping.
fn shared_face(a: &Part, b: &Part) -> Option<usize> {
    const EPS: f32 = 1e-3;
    let lo = |p: &Part, k: usize| p.at[k] - p.size[k] / 2.0;
    let hi = |p: &Part, k: usize| p.at[k] + p.size[k] / 2.0;
    for axis in 0..3 {
        let same_plane = (lo(a, axis) - lo(b, axis)).abs() < EPS || (hi(a, axis) - hi(b, axis)).abs() < EPS;
        let overlap = (0..3).filter(|&k| k != axis).all(|k| hi(a, k).min(hi(b, k)) - lo(a, k).max(lo(b, k)) > EPS);
        if same_plane && overlap {
            return Some(axis);
        }
    }
    None
}

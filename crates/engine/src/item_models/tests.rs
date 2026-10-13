use super::*;

#[test]
fn manufactured_parts_have_bounded_distinct_assemblies() {
    for item in [IRON_INGOT, COPPER_INGOT, IRON_PLATE, IRON_ROD, SCREW, COPPER_WIRE, RED_PACK, GREEN_PACK] {
        let model = parts(item);
        assert!(model.len() >= 2, "{item:?} needs a silhouette beyond one textured cube");
        for p in model {
            for axis in 0..3 {
                assert!(p.size[axis] > 0.0);
                assert!(p.center[axis].abs() + p.size[axis] * 0.5 <= 0.501, "{item:?} extends beyond its item cell");
            }
            assert!(p.tex.iter().all(|&t| (t as usize) < tex::COUNT));
        }
    }
    assert!(parts(ItemId::block(crate::block::STONE)).is_empty(), "blocks keep their existing cube model");
}

#[test]
fn the_cog_is_round_and_stays_inside_its_cell() {
    let model = parts(GEAR);
    assert!(model.len() >= 12, "a cog is turned boxes, not one cube");
    for p in model {
        // The farthest corner of a part turned by its yaw.
        let (s, c) = p.yaw.sin_cos();
        for (sx, sz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
            let (lx, lz) = (sx * p.size[0], sz * p.size[2]);
            let (x, z) = (p.center[0] + c * lx - s * lz, p.center[2] + s * lx + c * lz);
            assert!((x * x + z * z).sqrt() <= 0.46, "{x} {z} is outside the cog's radius");
        }
        assert!(p.center[1].abs() + p.size[1] * 0.5 <= 0.501);
    }
    assert!(model.iter().any(|p| p.yaw > 1.0), "the teeth run at several angles");
}

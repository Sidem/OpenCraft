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

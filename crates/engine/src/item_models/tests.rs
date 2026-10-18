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

/// Where a part's corners reach, turned by its yaw: (min x, max x, min z, max z).
fn reach(p: &Part) -> [f32; 4] {
    let (s, c) = p.yaw.sin_cos();
    let mut r = [f32::MAX, f32::MIN, f32::MAX, f32::MIN];
    for (sx, sz) in [(-0.5, -0.5), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
        let (lx, lz) = (sx * p.size[0], sz * p.size[2]);
        let (x, z) = (p.center[0] + c * lx - s * lz, p.center[2] + s * lx + c * lz);
        r = [r[0].min(x), r[1].max(x), r[2].min(z), r[3].max(z)];
    }
    r
}

#[test]
fn every_item_that_is_not_a_block_is_a_model_resting_on_its_floor() {
    for id in 256..crate::item::ITEM_COUNT as u16 {
        let item = ItemId(id);
        let def = crate::item::def(item).unwrap();
        if def.size == [1.0; 3]
            && !matches!(item, crate::item::MINER_MK3 | crate::item::MINER_MK4 | crate::item::MINER_MK5)
        {
            continue; // a machine tier: drawn as its block, with a tier chip on the icon
        }
        let model = parts(item);
        assert!(model.len() >= 2, "{} is still one textured box", def.name);
        let mut bottom = f32::MAX;
        for p in model {
            assert!(p.size.iter().all(|&s| s > 0.0), "{} has an empty part", def.name);
            assert!(p.tex.iter().all(|&t| (t as usize) < tex::COUNT));
            let [x0, x1, z0, z1] = reach(p);
            assert!(x0 >= -0.501 && x1 <= 0.501 && z0 >= -0.501 && z1 <= 0.501, "{} leaves its cell", def.name);
            // The miners' fan hub stands a hair above the cube.
            assert!(p.center[1].abs() + p.size[1] * 0.5 <= 0.52, "{} leaves its cell", def.name);
            bottom = bottom.min(p.center[1] - p.size[1] * 0.5);
        }
        let floor = -def.size[1] * 0.5;
        assert!((bottom - floor).abs() <= 0.06, "{} rests at {bottom}, its floor is {floor}", def.name);
    }
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

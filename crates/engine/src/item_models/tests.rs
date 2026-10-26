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

#[test]
fn every_power_pole_is_a_post_with_a_crossarm_inside_its_cell_and_a_tier_wears_its_stripe() {
    use crate::block::{LONG_POLE, POLE};
    use crate::item::{POLE_MK2, POLE_MK3, POLE_MK4};
    let poles = [ItemId::block(POLE), POLE_MK2, POLE_MK3, POLE_MK4, ItemId::block(LONG_POLE)];
    for item in poles {
        let model = parts(item);
        assert!(model.len() >= 8, "{item:?} is still a striped cube");
        for p in model {
            assert!(p.size.iter().all(|&s| s > 0.0) && p.tex.iter().all(|&t| (t as usize) < tex::COUNT));
            for axis in 0..3 {
                assert!(p.center[axis].abs() + p.size[axis] * 0.5 <= 0.501, "{item:?} leaves its cell");
            }
        }
    }
    let band = |item| parts(item)[2].tex[0];
    assert_ne!(band(ItemId::block(POLE)), band(POLE_MK2));
    assert_ne!(band(POLE_MK2), band(POLE_MK3));
    assert_ne!(band(POLE_MK3), band(POLE_MK4));
}

const GRID: usize = 96;

/// The silhouette of a model as its icon draws it (`web/src/ui/hud.ts` `itemIcon`: x runs right-down, z left-down, y up,
/// models standing on their cell's floor centred up and down), on a `GRID`² grid over [-0.6, 0.6]².
fn silhouette(model: &[Part]) -> Vec<bool> {
    assert!(model.iter().all(|p| p.yaw == 0.0), "the silhouette only handles upright boxes");
    let (low, high) = model.iter().fold((1.0f32, -1.0f32), |(l, h), p| {
        (l.min(p.center[1] - p.size[1] * 0.5), h.max(p.center[1] + p.size[1] * 0.5))
    });
    let lift = if low <= -0.49 { -(low + high) * 0.5 } else { 0.0 };
    let proj = |x: f32, y: f32, z: f32| [(x - z) * 0.5, (x + z) * 0.25 - y * 0.5];
    let edge = |a: [f32; 2], b: [f32; 2]| [b[0] - a[0], b[1] - a[1]];
    // The three faces a viewer above and in front of +x, +z sees: a corner and two edges each.
    let mut faces = Vec::new();
    for p in model {
        let (c, h) = ([p.center[0], p.center[1] + lift, p.center[2]], p.size.map(|s| s * 0.5));
        let top = proj(c[0] - h[0], c[1] + h[1], c[2] - h[2]);
        faces.push((
            top,
            edge(top, proj(c[0] + h[0], c[1] + h[1], c[2] - h[2])),
            edge(top, proj(c[0] - h[0], c[1] + h[1], c[2] + h[2])),
        ));
        let x = proj(c[0] + h[0], c[1] + h[1], c[2] - h[2]);
        faces.push((
            x,
            edge(x, proj(c[0] + h[0], c[1] + h[1], c[2] + h[2])),
            edge(x, proj(c[0] + h[0], c[1] - h[1], c[2] - h[2])),
        ));
        let z = proj(c[0] - h[0], c[1] + h[1], c[2] + h[2]);
        faces.push((
            z,
            edge(z, proj(c[0] + h[0], c[1] + h[1], c[2] + h[2])),
            edge(z, proj(c[0] - h[0], c[1] - h[1], c[2] + h[2])),
        ));
    }
    let mut mask = vec![false; GRID * GRID];
    for (i, cell) in mask.iter_mut().enumerate() {
        let p = [
            -0.6 + 1.2 * ((i % GRID) as f32 + 0.5) / GRID as f32,
            -0.6 + 1.2 * ((i / GRID) as f32 + 0.5) / GRID as f32,
        ];
        *cell = faces.iter().any(|&(o, u, v)| {
            let (d, det) = ([p[0] - o[0], p[1] - o[1]], u[0] * v[1] - u[1] * v[0]);
            let (a, b) = ((d[0] * v[1] - d[1] * v[0]) / det, (u[0] * d[1] - u[1] * d[0]) / det);
            det.abs() > 1e-6 && (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b)
        });
    }
    mask
}

fn iou(a: &[bool], b: &[bool]) -> f32 {
    let both = a.iter().zip(b).filter(|(x, y)| **x && **y).count();
    let either = a.iter().zip(b).filter(|(x, y)| **x || **y).count();
    both as f32 / either as f32
}

/// Rows of the grid a silhouette spans.
fn tall(mask: &[bool]) -> usize {
    let rows: Vec<usize> = (0..GRID).filter(|&r| mask[r * GRID..(r + 1) * GRID].iter().any(|&m| m)).collect();
    rows.last().unwrap() - rows[0] + 1
}

#[test]
fn every_belt_family_icon_is_a_model_inside_its_cell() {
    use crate::block::{BELT, BELT_EJECTOR, FILTER, LIFT, SPLITTER, UNDERPASS_IN};
    use crate::factory::tiers::item_of;
    let mut items: Vec<ItemId> = (0..4).flat_map(|t| [item_of(BELT, t), item_of(UNDERPASS_IN, t)]).flatten().collect();
    items.extend([SPLITTER, FILTER, LIFT, BELT_EJECTOR].map(ItemId::block));
    assert_eq!(items.len(), 12);
    for item in items {
        let model = parts(item);
        assert!(model.len() >= 4, "{item:?} is still a cube");
        for p in model {
            assert!(p.size.iter().all(|&s| s > 0.0) && p.tex.iter().all(|&t| (t as usize) < tex::COUNT));
            for axis in 0..3 {
                assert!(p.center[axis].abs() + p.size[axis] * 0.5 <= 0.501, "{item:?} leaves its cell");
            }
        }
    }
}

#[test]
fn a_belt_model_is_as_low_as_the_placed_belt_and_wears_its_tier() {
    use crate::block::BELT;
    use crate::factory::tiers::item_of;
    use crate::factory::{BELT_HEIGHT, BELT_TIERS};
    for tier in 0..4u8 {
        let model = parts(item_of(BELT, tier).unwrap());
        let slab = &model[1];
        assert_eq!(slab.tex[0], BELT_TIERS[usize::from(tier)].top, "the top is the tier's belt");
        assert!(
            (slab.center[1] + slab.size[1] * 0.5 - (-0.5 + BELT_HEIGHT)).abs() < 1e-4,
            "as high as the placed belt"
        );
        assert!(model.iter().all(|p| p.center[1] + p.size[1] * 0.5 <= -0.5 + 0.3), "a belt is flat");
        assert!(model.iter().any(|p| p.tex[1] == tex::stripe(tier)), "the rails wear the tier stripe");
    }
}

#[test]
fn belts_lifts_underpasses_and_ejectors_have_different_silhouettes_and_a_belt_is_flat() {
    use crate::block::{BELT, BELT_EJECTOR, LIFT, UNDERPASS_IN};
    let shape = |b| silhouette(parts(ItemId::block(b)));
    let (belt, lift, pass, eject) = (shape(BELT), shape(LIFT), shape(UNDERPASS_IN), shape(BELT_EJECTOR));
    let cube = silhouette(&[part([0.0; 3], [1.0; 3], 0)]);
    let ratio = |m: &[bool]| tall(m) as f32 / tall(&cube) as f32;
    eprintln!(
        "heights belt {:.2} lift {:.2} underpass {:.2} ejector {:.2}; iou belt/lift {:.2} belt/underpass {:.2} belt/ejector {:.2} lift/underpass {:.2} lift/ejector {:.2} underpass/ejector {:.2}",
        ratio(&belt), ratio(&lift), ratio(&pass), ratio(&eject), iou(&belt, &lift), iou(&belt, &pass), iou(&belt, &eject),
        iou(&lift, &pass), iou(&lift, &eject), iou(&pass, &eject)
    );
    assert!(ratio(&belt) <= 0.68, "a belt is flat: {:.2} of a cube's height", ratio(&belt));
    assert!(ratio(&lift) >= 0.95, "a lift stands tall");
    for (name, a, b) in [
        ("belt and lift", &belt, &lift),
        ("belt and underpass", &belt, &pass),
        ("belt and ejector", &belt, &eject),
        ("lift and underpass", &lift, &pass),
        ("lift and ejector", &lift, &eject),
        ("underpass and ejector", &pass, &eject),
    ] {
        assert!(iou(a, b) < 0.8, "{name} look alike: {:.2}", iou(a, b));
    }
}

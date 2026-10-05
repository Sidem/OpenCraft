use crate::block::AIR;
use crate::chunk::CHUNK_VOLUME;
use crate::math::IVec3;
use crate::worldgen::WorldGen;

/// Air cells at least 6 blocks under the ground in the chunks of heights 32..95, over 20 × 20 columns: all caves.
fn cave_cells(g: &mut WorldGen) -> usize {
    let mut n = 0;
    for cz in -10..10 {
        for cx in -10..10 {
            let heights: Vec<i32> = (0..1024).map(|i| g.height_at(cx * 32 + (i & 31), cz * 32 + (i >> 5))).collect();
            for cy in 1..=2 {
                let c = g.generate(IVec3::new(cx, cy, cz));
                n += (0..CHUNK_VOLUME)
                    .filter(|&i| {
                        let (x, y, z) = (i & 31, i >> 10, (i >> 5) & 31);
                        c.get(x, y, z) == AIR && cy * 32 + (y as i32) <= heights[z * 32 + x] - 6
                    })
                    .count();
            }
        }
    }
    n
}

#[test]
fn version_6_has_far_fewer_caves() {
    let (old, new) = (cave_cells(&mut WorldGen::with_version(1337, 5)), cave_cells(&mut WorldGen::new(1337)));
    println!("cave cells: version 5 {old}, version 6 {new}");
    assert!(old > 20_000, "version 5 is full of caves: {old}");
    assert!(new > 0, "but there are still some");
    assert!(new * 6 < old, "version 6 {new} against version 5 {old}");
}

#[test]
fn caves_gather_in_zones() {
    // Where the zone noise is low no chunk has any cave; the field is not even built.
    let g = WorldGen::new(1337);
    let mut empty = 0;
    let mut built = 0;
    for cz in -20..20 {
        for cx in -20..20 {
            match super::CaveField::new(&g, IVec3::new(cx * 32, 32, cz * 32)) {
                None => empty += 1,
                Some(_) => built += 1,
            }
        }
    }
    println!("chunk columns without a cave field: {empty}, with: {built}");
    assert!(empty > built, "most ground has no cave zone");
    assert!(built > 0, "some ground does");
    assert!(
        super::CaveField::new(&WorldGen::with_version(1337, 5), IVec3::new(0, 32, 0)).is_some(),
        "older worlds always build it"
    );
}

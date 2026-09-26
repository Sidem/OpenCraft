use super::*;

#[test]
fn every_layer_is_generated() {
    let px = generate();
    assert_eq!(px.len(), tex::COUNT * 16 * 16 * 4);
    for layer in 0..tex::COUNT {
        let start = layer * 16 * 16 * 4;
        let slice = &px[start..start + 16 * 16 * 4];
        let magenta = slice.chunks(4).filter(|c| c == &[255, 0, 255, 255]).count();
        assert_eq!(magenta, 0, "layer {layer} fell through to the placeholder");
    }
}

#[test]
fn natural_surfaces_have_no_tile_border() {
    // Compare wrap-edge jumps with interior pixel jumps, not identical first/last pixels:
    // neighbouring texels should remain neighbours when a greedy quad repeats the texture.
    for layer in [
        tex::STONE,
        tex::DIRT,
        tex::GRASS_TOP,
        tex::SAND,
        tex::LEAVES,
        tex::BEDROCK,
        tex::SPENT_ROCK,
        tex::COAL_ORE,
        tex::IRON_ORE,
        tex::COPPER_ORE,
    ] {
        let mut interior = 0u32;
        let mut seam = 0u32;
        let difference = |a: [u8; 4], b: [u8; 4]| (0..3).map(|i| a[i].abs_diff(b[i]) as u32).sum::<u32>();
        for y in 0..16 {
            for x in 0..16 {
                let p = pixel(layer, x, y);
                for (xx, yy, edge) in [((x + 1) % 16, y, x == 15), (x, (y + 1) % 16, y == 15)] {
                    let d = difference(p, pixel(layer, xx, yy));
                    if edge {
                        seam += d;
                    } else {
                        interior += d;
                    }
                }
            }
        }
        assert!(seam * 15 <= interior * 2 + 480, "layer {layer} has a visible wrap border");
    }
}

#[test]
fn ore_marks_read_without_hue() {
    let luma = |c: [u8; 4]| (u32::from(c[0]) * 54 + u32::from(c[1]) * 183 + u32::from(c[2]) * 19) / 256;
    let mut ranges = Vec::new();
    for layer in [tex::COAL_ORE, tex::IRON_ORE, tex::COPPER_ORE] {
        let values: Vec<_> = (0..16).flat_map(|y| (0..16).map(move |x| luma(pixel(layer, x, y)))).collect();
        let dark = values.iter().filter(|&&v| v < 55).count();
        let light = values.iter().filter(|&&v| v > 170).count();
        ranges.push((dark, light));
    }
    assert!(ranges[0].0 > 20, "coal needs broad dark seams");
    assert!(ranges[1].1 > 20, "iron needs broad pale blooms");
    assert!(
        ranges[2].0 < ranges[0].0 && ranges[2].1 < ranges[1].1,
        "copper's fine medium-value veins must sit between coal and iron"
    );
}

#[test]
fn foliage_keeps_cutouts_and_colour_for_mipmaps() {
    let mut holes = 0;
    for y in 0..16 {
        for x in 0..16 {
            let c = pixel(tex::LEAVES, x, y);
            assert!(c[3] == 0 || c[3] == 255);
            if c[3] == 0 {
                holes += 1;
                assert!(c[1] > c[0] && c[1] > c[2], "transparent texels must retain leaf RGB");
            }
        }
    }
    assert!((25..=80).contains(&holes), "foliage needs 10–31% open area, got {holes}/256");
}

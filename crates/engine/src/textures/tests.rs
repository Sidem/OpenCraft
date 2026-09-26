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
    for layer in [tex::STONE, tex::DIRT, tex::GRASS_TOP, tex::SAND, tex::LEAVES, tex::BEDROCK, tex::SPENT_ROCK] {
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

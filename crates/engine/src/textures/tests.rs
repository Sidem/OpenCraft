use super::*;

fn layer_px(layer: u16) -> Vec<[u8; 4]> {
    (0..16).flat_map(|y| (0..16).map(move |x| pixel(layer, x, y))).collect()
}

/// A layer and its alternates.
fn looks(layer: u16) -> Vec<u16> {
    let first = tex::alternates(layer);
    std::iter::once(layer).chain(first..first + tex::ALTERNATES).collect()
}

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
fn alternates_map_back_to_their_base() {
    for base in tex::WITH_ALTERNATES {
        for (i, layer) in looks(base).into_iter().enumerate() {
            assert_eq!(tex::look(layer), (base, i as u16));
        }
    }
    assert_eq!(usize::from(tex::FIRST_ALT + 6 * tex::ALTERNATES), usize::from(tex::RUSTY_GRASS_TOP));
    assert_eq!(tex::look(tex::STONE), (tex::STONE, 0));
}

#[test]
fn natural_surfaces_have_no_tile_border() {
    // Compare wrap-edge jumps with interior pixel jumps, not identical first/last pixels:
    // neighbouring texels should remain neighbours when a greedy quad repeats the texture.
    let mut layers = vec![
        tex::STONE,
        tex::DIRT,
        tex::GRASS_TOP,
        tex::SAND,
        tex::BEDROCK,
        tex::SPENT_ROCK,
        tex::GRANITE,
        tex::SANDSTONE,
        tex::BASALT,
    ];
    layers.extend(tex::RUSTY_SOIL..=tex::PALE_SOIL);
    layers.extend(tex::RUSTY_GRASS_TOP..=tex::PALE_GRASS_TOP);
    layers.extend(tex::RUSTY_SAND..=tex::PALE_SAND);
    layers.extend(tex::WITH_ALTERNATES.iter().flat_map(|&l| looks(l)));
    for layer in layers {
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
fn surface_hints_are_a_shade_off_their_plain_surface() {
    let pairs = [
        (tex::DIRT, tex::RUSTY_SOIL),
        (tex::GRASS_TOP, tex::RUSTY_GRASS_TOP),
        (tex::GRASS_SIDE, tex::RUSTY_GRASS_SIDE),
        (tex::SAND, tex::RUSTY_SAND),
    ];
    for (plain, first) in pairs {
        for hint in first..first + 4 {
            let delta: u32 = layer_px(plain)
                .into_iter()
                .zip(layer_px(hint))
                .map(|(a, b)| (0..3).map(|i| a[i].abs_diff(b[i]) as u32).sum::<u32>())
                .sum();
            let average = delta / (16 * 16 * 3);
            assert!((4..=24).contains(&average), "hint layer {hint} differs from {plain} by {average} levels");
        }
    }
}

#[test]
fn ores_read_by_value_and_hue_in_every_look() {
    let luma = |c: [u8; 4]| (u32::from(c[0]) * 54 + u32::from(c[1]) * 183 + u32::from(c[2]) * 19) / 256;
    let count = |layer: u16, f: &dyn Fn([u8; 4]) -> bool| layer_px(layer).into_iter().filter(|&c| f(c)).count();
    for layer in looks(tex::COAL_ORE) {
        assert!(count(layer, &|c| luma(c) < 50) > 25, "coal look {layer} needs dark lumps");
    }
    for layer in looks(tex::IRON_ORE) {
        assert!(count(layer, &|c| c[0] as i32 > c[1] as i32 + 40) > 20, "iron look {layer} needs rust nodules");
    }
    for layer in looks(tex::COPPER_ORE) {
        assert!(count(layer, &|c| c[1] as i32 > c[0] as i32 + 40) > 15, "copper look {layer} needs green crusts");
    }
    for layer in looks(tex::QUARTZ_ORE) {
        assert!(count(layer, &|c| luma(c) > 200) > 20, "quartz look {layer} needs bright crystals");
    }
    for base in tex::WITH_ALTERNATES {
        let base_px = layer_px(base);
        for alt in looks(base).into_iter().skip(1) {
            let differ = layer_px(alt).iter().zip(&base_px).filter(|(a, b)| a != b).count();
            assert!(differ > 30, "alternate {alt} looks too much like {base}");
        }
    }
}

#[test]
fn foliage_keeps_cutouts_and_colour_for_mipmaps() {
    for layer in looks(tex::LEAVES) {
        let mut holes = 0;
        for c in layer_px(layer) {
            assert!(c[3] == 0 || c[3] == 255);
            if c[3] == 0 {
                holes += 1;
                assert!(c[1] > c[0] && c[1] > c[2], "transparent texels must retain leaf RGB");
            }
        }
        assert!((25..=80).contains(&holes), "foliage needs 10–31% open area, got {holes}/256 in {layer}");
    }
}

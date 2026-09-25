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

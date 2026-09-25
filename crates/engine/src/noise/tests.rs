use super::*;

#[test]
fn deterministic_and_bounded() {
    let a = Perlin::new(42);
    let b = Perlin::new(42);
    for i in 0..1000 {
        let (x, y, z) = (i as f64 * 0.37, i as f64 * 0.11 - 20.0, i as f64 * 0.53);
        assert_eq!(a.noise3(x, y, z), b.noise3(x, y, z));
        assert!(a.noise2(x, y).abs() <= 1.01);
        assert!(a.noise3(x, y, z).abs() <= 1.01);
    }
    assert_ne!(Perlin::new(1).noise2(0.5, 0.5), Perlin::new(2).noise2(0.5, 0.5));
}

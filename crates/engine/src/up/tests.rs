use super::*;

#[test]
fn a_flat_world_is_up_along_y_and_falls_along_minus_y_exactly() {
    for p in [Vec3::ZERO, Vec3::new(1e6, -40.0, -3.5e5), Vec3::new(-0.25, 255.9, 17.0)] {
        assert_eq!(up_at(p), Vec3::new(0.0, 1.0, 0.0));
        let g = gravity_at(p, 28.0);
        assert_eq!((g.x, g.y, g.z), (0.0, -28.0, 0.0));
        let v = Vec3::new(3.25, -7.125, 0.5);
        assert_eq!(along_up(p, v).to_bits(), v.y.to_bits(), "the component along up is y, bit for bit");
    }
}

#[test]
fn falling_by_the_vector_matches_the_old_hard_coded_y_bit_for_bit() {
    // What player.rs and entities.rs did before (vel.y - G * dt) against what they do now (vel.y + g.y * dt).
    let mut seed = 0x2545_F491_u64;
    for _ in 0..10_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let vy = (seed >> 11) as f64 / (1u64 << 53) as f64 * 80.0 - 40.0;
        let dt = 1.0 / (60.0 + (seed % 200) as f64);
        for strength in [28.0, 22.0, 2.8000000000000003] {
            let g = gravity_at(Vec3::new(vy, vy, vy), strength);
            assert_eq!((vy - strength * dt).to_bits(), (vy + g.y * dt).to_bits());
        }
    }
}

use super::*;
use crate::block::{tex, UNDERPASS_IN};
use crate::factory::render::INSTANCE_FLOATS;
use crate::factory::tiers::FAMILIES;
use crate::factory::Factory;
use crate::math::IVec3;
use crate::world::World;

#[test]
fn a_row_has_one_stud_per_mk_centred() {
    for tier in 0..MAX_STUDS {
        let row: Vec<f32> = offsets(tier).collect();
        assert_eq!(row.len(), tier as usize + 1);
        assert!((row[0] + row[row.len() - 1]).abs() < 1e-6, "Mk{} is centred", tier + 1);
        assert!(row[row.len() - 1] - row[0] + STUD <= ROW_MAX + 1e-6);
    }
}

/// Every tiered family's machine shows its Mk as studs in its tier's paint near the camera, a whole number of rows of
/// `tier + 1`, and none past `DETAIL`. A new family without marks fails here.
#[test]
fn every_tiered_machine_shows_its_mk_near_and_none_far() {
    for family in FAMILIES {
        for tier in 0..family.items.len() as u8 {
            let (mut world, mut f) = (World::new(1, 2), Factory { by_hand: true, ..Factory::default() });
            // A belt shows its Mk on one cell in `BELT_EVERY`: this cell is one.
            let pos = IVec3::new(BELT_EVERY * 3, 100, 0);
            f.place(&mut world, family.block, pos, 0, pos - IVec3::new(0, 1, 0), tier);
            if family.block == UNDERPASS_IN {
                f.place(&mut world, UNDERPASS_IN, pos + IVec3::new(0, 0, -3), 0, pos, tier);
            }
            f.update(&mut world, 0, &mut Vec::new());
            let studs = |eye: Vec3| {
                let mut out = Vec::new();
                f.write_instances(&mut out, eye, 0.0, 500.0);
                out.chunks_exact(INSTANCE_FLOATS).filter(|b| b[9] as u16 == tex::paint(tier)).count()
            };
            let here = pos.as_vec3() + Vec3::new(0.5, 2.0, 4.0);
            let near = studs(here);
            let name = crate::item::name(family.items[tier as usize]);
            assert!(near > 0 && near % (tier as usize + 1) == 0, "{name}: {near} studs");
            assert_eq!(studs(here + Vec3::new(DETAIL + 10.0, 0.0, 0.0)), 0, "{name} far away");
        }
    }
}

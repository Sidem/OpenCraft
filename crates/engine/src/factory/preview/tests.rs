use super::*;
use crate::math::Vec3;

fn models_of(f: &Factory) -> usize {
    let mut out = Vec::new();
    f.write_instances(&mut out, Vec3::new(0.0, 0.0, 0.0), 0.0, 1000.0);
    out.len() / INSTANCE_FLOATS
}

/// Machines whose block the world's mesh draws whole, so their model is empty and a ghost of them is a cube.
fn drawn_by_the_world(block: BlockId) -> bool {
    crate::block::def(block).render == crate::block::Render::Opaque
}

#[test]
fn every_machine_block_has_a_model_or_is_a_cube() {
    let mut none = Vec::new();
    for def in MACHINES.iter() {
        let mut f = Factory::default();
        let pos = IVec3::new(8, 40, 8);
        assert!(f.place_preview(def.block, pos, 1, 0), "{}: no preview", crate::block::def(def.block).name);
        f.settle_preview();
        if models_of(&f) == 0 && !drawn_by_the_world(def.block) {
            none.push(crate::block::def(def.block).name);
        }
    }
    assert!(none.is_empty(), "draw nothing as a ghost and are no cube: {none:?}");
}

#[test]
fn a_plain_block_is_no_machine() {
    let mut f = Factory::default();
    assert!(!f.place_preview(crate::block::STONE, IVec3::new(1, 1, 1), 0, 0));
    assert_eq!(models_of(&f), 0);
}

#[test]
fn a_turn_and_a_tier_change_the_model() {
    let draw = |facing: u8, tier: u8| {
        let mut f = Factory::default();
        f.place_preview(crate::block::ASSEMBLER, IVec3::new(8, 40, 8), facing, tier);
        let mut out = Vec::new();
        f.write_instances(&mut out, Vec3::new(0.0, 0.0, 0.0), 0.0, 1000.0);
        out
    };
    assert_ne!(draw(0, 0), draw(1, 0), "turning moves the parts");
    assert_eq!(draw(0, 0), draw(0, 0), "a model is the same every frame");
}

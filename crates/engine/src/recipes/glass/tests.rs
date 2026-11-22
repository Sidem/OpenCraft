use std::collections::HashSet;

use super::{STAINED_GLASS, STAINED_GLASS_RAW};
use crate::block::{
    self, is_glass, is_stained_glass, tex, BlockId, Render, AIR, GLASS, GLASS_KILN, LAMP, QUARTZ_ORE, SAND,
    STAINED_GLASS_COUNT, STAINED_GLASS_FIRST,
};
use crate::chunk::Chunk;
use crate::factory::process_spec;
use crate::light::{self, Lighting};
use crate::mesher::Mesher;
use crate::recipes::{
    b, Category, GLASS_FIRST, GLASS_PRIMARY_COUNT, GLASS_RAW_FIRST, GLASS_RAW_ROWS, GLASS_ROWS, MACHINE_RECIPES,
    RECIPES,
};
use crate::research::{Research, Unlock, TECHS};

fn stained_ids() -> impl Iterator<Item = BlockId> {
    STAINED_GLASS_FIRST..STAINED_GLASS_FIRST + STAINED_GLASS_COUNT as BlockId
}

#[test]
fn stained_glass_blocks_and_recipes_match_glass_kiln_with_pigments() {
    assert_eq!(STAINED_GLASS.len(), GLASS_ROWS as usize);
    assert_eq!(STAINED_GLASS.len(), STAINED_GLASS_COUNT);
    assert_eq!(STAINED_GLASS_RAW.len(), GLASS_RAW_ROWS as usize);
    let gas = crate::recipes::gas::GAS_MACHINES.len();
    let seis = crate::recipes::refit::REFIT_SEISMIC.len();
    assert_eq!(
        MACHINE_RECIPES.len(),
        GLASS_FIRST as usize + STAINED_GLASS.len() + gas + seis + STAINED_GLASS_RAW.len()
    );
    let kiln_spec = process_spec(GLASS_KILN).expect("Glass Kiln spec");
    let mut seen_inputs = HashSet::new();
    for (i, id) in stained_ids().enumerate() {
        let def = block::def(id);
        assert!(is_stained_glass(id) && is_glass(id));
        assert_eq!((def.render, def.solid, def.drop, def.placeable), (Render::Translucent, true, id, true));
        assert_eq!(def.faces, [tex::stained_glass_layer(i as u16); 6]);
        assert!(def.faces[0] < 512, "cube layer must fit in 9 bits");
        let pigment = crate::colour::pigment_of(i as u8 + 1).expect("matching pigment");
        for (row, expected_silica) in [
            (&STAINED_GLASS[i], (b(GLASS), 4)),
            (&STAINED_GLASS_RAW[i], (b(SAND), 4)),
            (&STAINED_GLASS_RAW[STAINED_GLASS_COUNT + i], (b(QUARTZ_ORE), 2)),
        ] {
            assert_eq!(row.category, Category::Glassmaking);
            assert_eq!(row.inputs, &[expected_silica, (pigment, 1)][..]);
            assert_eq!(row.outputs, &[(b(id), 4)][..]);
            assert!(kiln_spec.categories.contains(&row.category));
            assert!(seen_inputs.insert(row.inputs), "duplicate glass kiln recipe inputs at {i}");
        }
        assert!(!RECIPES.iter().any(|r| r.output == b(id)), "{} is machine-made only", def.name);
    }
    assert!(!is_stained_glass(GLASS) && is_glass(GLASS));
}

#[test]
fn architecture_unlocks_primary_stained_glass_and_pigments_unlocks_mixed_hues() {
    let arch = TECHS.iter().position(|t| t.name == "Architecture").unwrap() as u8;
    let pig = TECHS.iter().position(|t| t.name == "Pigments").unwrap() as u8;
    let mut research = Research::default();
    for i in 0..GLASS_PRIMARY_COUNT {
        for row in [GLASS_FIRST + i, GLASS_RAW_FIRST + i, GLASS_RAW_FIRST + GLASS_ROWS + i] {
            assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), Some(arch));
        }
    }
    for i in GLASS_PRIMARY_COUNT..GLASS_ROWS {
        for row in [GLASS_FIRST + i, GLASS_RAW_FIRST + i, GLASS_RAW_FIRST + GLASS_ROWS + i] {
            assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), Some(pig));
        }
    }
    for t in TECHS[usize::from(arch)].needs.iter().copied().chain([arch]) {
        (0..TECHS[usize::from(t)].units).for_each(|_| research.add_unit(t));
    }
    for i in 0..GLASS_PRIMARY_COUNT {
        for row in [GLASS_FIRST + i, GLASS_RAW_FIRST + i, GLASS_RAW_FIRST + GLASS_ROWS + i] {
            assert_eq!(research.locked_by(Unlock::MachineRecipe(row)), None);
        }
    }
}

#[test]
fn stained_glass_lets_sky_and_block_light_through() {
    for id in stained_ids() {
        assert!(light::passes_sky(id), "{} must pass straight sky light", block::def(id).name);
    }
    let air = Chunk::uniform(AIR);
    let mut centre = Chunk::uniform(AIR);
    // Lamp at (10, 10, 10), stained glass wall at x = 12, check light reaches (14, 10, 10).
    centre.set(10, 10, 10, LAMP);
    for z in 8..=12 {
        for y in 8..=12 {
            centre.set(12, y, z, STAINED_GLASS_FIRST);
        }
    }
    let mut chunks = [&air; 27];
    chunks[13] = &centre;
    let mut lighting = Lighting::default();
    lighting.light(&chunks, &[&[]; 9]);
    let idx = |x: usize, y: usize, z: usize| ((y + 1) * 34 + (z + 1)) * 34 + (x + 1);
    let past_wall = lighting.pad[idx(14, 10, 10)] >> 4;
    assert_eq!(past_wall, 15, "lamp light passes right through stained glass");
}

#[test]
fn mesher_puts_stained_glass_in_translucent_pass_and_culls_same_colour_faces() {
    let air = Chunk::uniform(AIR);
    let mut centre = Chunk::uniform(AIR);
    // Two adjacent red panes hide their shared face (10 faces total, greedy merged into 6 quads);
    // a blue pane beside them keeps the boundary faces between red and blue.
    centre.set(10, 10, 10, STAINED_GLASS_FIRST);
    centre.set(11, 10, 10, STAINED_GLASS_FIRST);
    let light = vec![0x0Fu8; 34 * 34 * 34];
    let mut mesher = Mesher::new();
    let two_same = {
        let mut chunks = [&air; 27];
        chunks[13] = &centre;
        mesher.mesh(&chunks, &light, None)
    };
    assert_eq!((two_same.opaque_quads, two_same.cutout_quads, two_same.liquid_quads), (0, 0, 0));
    assert_eq!(two_same.translucent_quads, 6, "two adjacent identical panes merge cleanly to 6 outer quads");

    centre.set(11, 10, 10, STAINED_GLASS_FIRST + 5); // Blue next to Red
    let mut chunks = [&air; 27];
    chunks[13] = &centre;
    let two_diff = mesher.mesh(&chunks, &light, None);
    assert_eq!(two_diff.translucent_quads, 12, "two different colours keep every face including their divider");
}

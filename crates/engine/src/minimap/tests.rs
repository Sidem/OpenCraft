use super::*;
use crate::block::{BlockId, STONE};
use crate::tests::run_until_ready;
use crate::worldgen::WORLD_HEIGHT;
use crate::Game;

/// Height and id of the top non-air loaded block of a column.
fn top_of(g: &Game, x: i32, z: i32) -> (i32, BlockId) {
    (0..WORLD_HEIGHT)
        .rev()
        .find_map(|y| g.sim.world.get_block(IVec3::new(x, y, z)).filter(|&b| b != AIR).map(|b| (y, b)))
        .expect("a loaded column")
}

/// What the map should show for column (x, z).
fn expected(g: &Game, x: i32, z: i32) -> [u8; 4] {
    let ((h, b), (nh, _)) = (top_of(g, x, z), top_of(g, x - 1, z - 1));
    let k = 256 + (h - nh).clamp(-SHADE_MAX_STEP, SHADE_MAX_STEP) * SHADE_PER_BLOCK;
    let c = g.minimap.colors[b as usize].map(|v| ((v as i32 * k) >> 8).min(255) as u8);
    [c[0], c[1], c[2], 255]
}

/// The pixel `d` blocks east and south of the player's column.
fn pixel(g: &Game, dx: i32, dz: i32) -> [u8; 4] {
    let p = ((HALF + dz) as usize * MAP_SIZE + (HALF + dx) as usize) * 4;
    g.minimap.pixels[p..p + 4].try_into().unwrap()
}

#[test]
fn map_shows_the_top_block_of_each_column_north_up() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    assert!(g.minimap_redraw());
    let c = g.body().pos.floor();
    for (dx, dz) in [(0, 0), (5, -7), (-20, 13), (40, 30), (-63, -63), (63, 63)] {
        assert_eq!(pixel(&g, dx, dz), expected(&g, c.x + dx, c.z + dz), "column offset {dx}, {dz}");
    }
    // Grass is green, and the colours come from the textures.
    let [r, gr, b] = g.minimap.colors[crate::block::GRASS as usize];
    assert!(gr > r && gr > b);
}

#[test]
fn redraws_only_after_a_move_or_a_change_in_range() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    assert!(g.minimap_redraw());
    assert!(!g.minimap_redraw(), "nothing changed");

    // A block placed on a column in range shows once its chunk remeshes.
    let c = g.body().pos.floor();
    let (x, z) = (c.x + 9, c.z - 4);
    let (h, _) = top_of(&g, x, z);
    assert!(g.sim.world.set_block(IVec3::new(x, h + 1, z), STONE));
    while g.next_event() != 0 {}
    assert!(g.minimap_redraw());
    assert_eq!(pixel(&g, 9, -4), expected(&g, x, z));
    assert_eq!(top_of(&g, x, z).1, STONE);

    // A change far outside the map doesn't redraw it.
    g.minimap.touch(IVec3::new(c.x / 32 + 20, 1, c.z / 32));
    assert!(!g.minimap_redraw());

    // Moving one block does, and the map follows.
    let p = g.body().pos;
    g.teleport(p.x + 1.0, p.y, p.z);
    assert!(g.minimap_redraw());
    assert_eq!(pixel(&g, 8, -4), expected(&g, x, z));
}

/// Streams the world in and lets the explored map take in every meshed chunk column.
fn explored() -> Game {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    while g.next_event() != 0 {}
    for _ in 0..200 {
        g.minimap.atlas.refresh_some(&g.sim.world);
    }
    g
}

#[test]
fn the_explored_map_outlives_the_chunks_and_round_trips() {
    let g = explored();
    let tiles = g.minimap.atlas.len();
    assert!(tiles >= 25, "{tiles} tiles");
    let c = g.body().pos.floor();
    let (x, z) = (c.x + 20, c.z - 30);
    let seen = g.minimap.atlas.column(x, z);
    assert_eq!(((seen >> 8) as i32, (seen & 0xff) as BlockId), top_of(&g, x, z));

    // A fresh game of the same world, far from here, shows it from the bytes alone.
    let mut other = Game::new(2024, 3);
    other.set_explored_map(&g.explored_map());
    assert_eq!(other.minimap.atlas.len(), tiles);
    assert_eq!(other.minimap.atlas.column(x, z), seen);
    // At 2 blocks a pixel, pixel (5, 5) shows the column 10 blocks east and south of the corner,
    // shaded by the step from the column 2 blocks north-west.
    other.world_map_draw(x - 10, z - 10, 2, 16, 16);
    let nw = other.minimap.atlas.column(x - 2, z - 2);
    let k = 256 + ((seen >> 8) as i32 - (nw >> 8) as i32).clamp(-SHADE_MAX_STEP, SHADE_MAX_STEP) * SHADE_PER_BLOCK;
    let [r, gr, b] = g.minimap.colors[(seen & 0xff) as usize].map(|v| ((v as i32 * k) >> 8).min(255) as u8);
    let p = (5 * 16 + 5) * 4;
    assert_eq!(other.minimap.world_pixels[p..p + 4], [r, gr, b, 255]);
    other.set_explored_map(&[9, 1, 2]);
    assert_eq!(other.minimap.atlas.len(), tiles, "unknown bytes add nothing");
}

#[test]
fn ore_seen_at_the_surface_is_marked_and_coloured() {
    let g = explored();
    let marks = g.world_map_marks(-200, -200, 200, 200);
    let ore_marks: Vec<&[i32]> = marks.chunks_exact(MARK_FIELDS).filter(|m| m[3] == marks::MARK_ORE).collect();
    assert!(!ore_marks.is_empty(), "the starter patches show within the loaded area");
    let mut shown = 0;
    for z in -96..96 {
        for x in -96..96 {
            let v = g.minimap.atlas.column(x, z);
            let block = (v & 0xff) as BlockId;
            if v == 0 || !crate::block::is_ore(block) {
                continue;
            }
            shown += 1;
            let colour = ore_color(block);
            let near = |m: &&[i32]| m[2] == colour && (m[0] >> 3, m[1] >> 3) == (x >> 3, z >> 3);
            assert!(ore_marks.iter().any(near), "ore at {x}, {z} has a mark in its 8×8 square");
            assert_eq!(g.minimap.colors[block as usize], colour.to_be_bytes()[1..], "ore is drawn in its mark colour");
        }
    }
    assert!(shown > 0);
}

#[test]
fn other_players_are_marked_relative_to_the_local_one() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    assert!(g.minimap_players().is_empty());
    let id = g.add_player().unwrap() as usize;
    let at = g.body().pos;
    g.bodies[id].as_mut().unwrap().pos = at + Vec3::new(10.0, 0.0, -5.0);
    assert_eq!(g.minimap_players(), vec![10.0, -5.0, g.bodies[id].as_ref().unwrap().yaw as f32]);
}

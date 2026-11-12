use super::*;

#[test]
fn sides_are_seen_from_the_player() {
    // Facing north (DIRS[0]): north is far, east right, south near, west left; turning the player turns them all.
    assert_eq!((0..4).map(|s| seen_side(s, 0)).collect::<Vec<_>>(), [0, 1, 2, 3]);
    assert_eq!((0..4).map(|s| seen_side(s, 1)).collect::<Vec<_>>(), [3, 0, 1, 2]);
}

#[test]
fn every_turning_machine_turns_its_ports_with_it() {
    let blocks = footprint_blocks();
    assert!(blocks.len() >= 20, "{} machines", blocks.len());
    for &b in &blocks {
        let faces: Vec<_> = (0..4).map(|d| port_marks::port_faces(b, IVec3::ZERO, d)).collect();
        for d in 1..4 {
            // Turned a quarter, every face looks out one step further round, doing the same.
            let mut want: Vec<(u8, u8)> = faces[d - 1].iter().map(|&(_, s, f)| ((s + 1) % 4, f as u8)).collect();
            let mut got: Vec<(u8, u8)> = faces[d].iter().map(|&(_, s, f)| (s, f as u8)).collect();
            want.sort();
            got.sort();
            assert_eq!(got, want, "{}", block::def(b).name);
        }
    }
}

#[test]
fn the_hud_names_the_sides_as_the_player_sees_them() {
    let mut g = Game::new(1, 2);
    g.place_turn = 0;
    // Facing north, an assembler takes in at its back and sides and gives out at its front, towards the player.
    let yaw = g.body().yaw;
    assert_eq!(factory::dir_from_yaw(yaw), 0, "a new player faces north");
    assert_eq!(g.ports_line(block::ASSEMBLER), "in: far, right, left · out: near");
    g.place_turn = 1;
    assert_eq!(g.ports_line(block::ASSEMBLER), "in: far, right, near · out: left");
    assert_eq!(g.ports_line(block::BLAST_FURNACE), "in: far, right · out: left · byproduct: near");
}

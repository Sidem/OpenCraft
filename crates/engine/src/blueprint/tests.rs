use super::*;
use crate::action::Action;
use crate::block::{ASSEMBLER, BELT, STORAGE};
use crate::factory::footprint;
use crate::sim::PlayerId;

const P: PlayerId = PlayerId(0);

fn spot() -> IVec3 {
    IVec3::new(500, 200, -500)
}

fn build(sim: &mut Sim, block: BlockId, pos: IVec3, facing: u8) {
    sim.apply(P, Action::Give { item: block.into(), count: 1 });
    let slot = sim.player(P).unwrap().inventory.slots.iter().position(|s| s.item == ItemId::block(block)).unwrap();
    sim.apply(P, Action::PlaceBlock { pos, slot: slot as u8, facing, against: pos - IVec3::new(0, 1, 0) });
}

/// Two belts east, then a box, then an assembler two cells further.
fn factory() -> Sim {
    let mut sim = Sim::new(7, 2);
    build(&mut sim, BELT, spot(), 1);
    build(&mut sim, BELT, spot() + IVec3::new(1, 0, 0), 1);
    build(&mut sim, STORAGE, spot() + IVec3::new(2, 0, 0), 0);
    build(&mut sim, ASSEMBLER, spot() + IVec3::new(4, 0, 0), 1);
    sim
}

fn whole_box(sim: &Sim) -> Blueprint {
    let far = spot() + IVec3::new(12, 4, 12);
    Blueprint::copy(sim, spot() - IVec3::new(0, 0, 8), far, "Test".into()).unwrap()
}

#[test]
fn copy_takes_machine_anchors_with_facing() {
    let sim = factory();
    let bp = whole_box(&sim);
    let blocks: Vec<BlockId> = bp.entries.iter().map(|e| e.block).collect();
    assert_eq!(blocks, vec![BELT, BELT, STORAGE, ASSEMBLER], "one entry per machine, not per cell");
    assert_eq!(bp.entries[0].off, IVec3::new(0, 0, 8));
    assert_eq!((bp.entries[0].facing, bp.entries[3].facing), (1, 1));
    assert_eq!(bp.needs().len(), 3);
    let empty = Blueprint::copy(&sim, spot() + IVec3::new(0, 20, 0), spot() + IVec3::new(2, 22, 2), String::new());
    assert_eq!(empty, Err("No machines in that box"));
    let huge = Blueprint::copy(&sim, spot(), spot() + IVec3::new(MAX_SPAN, 0, 0), String::new());
    assert_eq!(huge, Err("That box is too big to copy"));
}

#[test]
fn the_plan_shows_every_piece_from_above() {
    let bp = whole_box(&factory());
    let plan = bp.plan();
    assert_eq!(&plan[..3], &[bp.size.x, bp.size.y, bp.size.z]);
    let pieces: Vec<&[i32]> = plan[3..].chunks_exact(PLAN_FIELDS).collect();
    assert_eq!(pieces.len(), bp.entries.len(), "one piece per entry");
    let of = |item: ItemId| pieces.iter().filter(|p| p[5] == i32::from(item.0)).count() as u32;
    for (item, n) in bp.needs() {
        assert_eq!(of(item), n, "the plan holds what building it needs");
    }
    let belts: Vec<&&[i32]> = pieces.iter().filter(|p| p[5] == i32::from(ItemId::block(BELT).0)).collect();
    assert!(belts.iter().all(|p| (p[6], p[7]) == (1, 1)), "belts run east and show which way: {belts:?}");
    let asm = pieces.last().unwrap();
    assert_eq!(asm[5], i32::from(ItemId::block(ASSEMBLER).0), "the tallest piece is drawn last");
    assert_eq!((asm[2] - asm[0], asm[3] - asm[1], asm[7]), (1, 1, 0), "a 2×2 machine covers 2×2 columns");
    for p in &pieces {
        assert!(p[0] >= 0 && p[2] < bp.size.x && p[1] >= 0 && p[3] < bp.size.z, "{p:?} inside the box");
    }
}

#[test]
fn four_turns_come_back_and_each_turn_keeps_the_shape() {
    let bp = whole_box(&factory());
    assert_eq!(bp.turned(4), (bp.size, bp.entries.clone()));
    let (size, entries) = bp.turned(1);
    assert_eq!(size, IVec3::new(bp.size.z, bp.size.y, bp.size.x));
    for e in &entries {
        assert!((0..size.x).contains(&e.off.x) && (0..size.z).contains(&e.off.z), "{e:?} inside {size:?}");
    }
    // The belt line ran east; a quarter turn makes it run south, one cell per step.
    assert_eq!((entries[1].off - entries[0].off, entries[0].facing), (IVec3::new(0, 0, 1), 2));
}

#[test]
fn a_turned_multi_block_machine_covers_the_turned_cells() {
    let sim = factory();
    let bp = whole_box(&sim);
    let anchor = spot() + IVec3::new(4, 0, 0);
    let fp = footprint::of(ASSEMBLER).unwrap();
    let rel: Vec<IVec3> = fp.cells(anchor, 1).iter().map(|&c| c - (spot() - IVec3::new(0, 0, 8))).collect();
    let origin = IVec3::new(100, 50, 100);
    let ghost = bp.ghosts_at(origin, 1).into_iter().find(|g| g.block == ASSEMBLER).unwrap();
    let mut got: Vec<IVec3> = ghost.cells().iter().map(|&c| c - origin).collect();
    let mut want: Vec<IVec3> = rel.iter().map(|c| IVec3::new(bp.size.z - 1 - c.z, c.y, c.x)).collect();
    let key = |v: &IVec3| (v.x, v.y, v.z);
    got.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(got, want);
}

#[test]
fn stamping_plants_ghosts_and_building_clears_them() {
    let mut sim = factory();
    let bp = whole_box(&sim);
    let origin = IVec3::new(520, 200, -520);
    for g in bp.ghosts_at(origin, 0) {
        sim.apply(P, Action::PlantGhost { pos: g.pos, block: g.block, facing: g.facing, tier: g.tier });
    }
    assert_eq!(sim.ghosts.len(), 4);
    assert_eq!(sim.ghosts.covering(origin + IVec3::new(0, 0, 8)).map(|g| g.block), Some(BELT));
    build(&mut sim, BELT, origin + IVec3::new(0, 0, 8), 0);
    assert_eq!(sim.ghosts.len(), 3, "the belt's ghost is built");
}

/// A bent belt's bend rides in its facing's upper bits through the copy, a turn, the bytes, a ghost and the build.
#[test]
fn a_bent_belt_is_copied_turned_and_built_bent() {
    use crate::factory::{belt_facing, BEND_LEFT};
    let mut sim = Sim::new(7, 2);
    build(&mut sim, BELT, spot(), belt_facing(1, BEND_LEFT));
    assert_eq!(sim.factory.bend_at(spot()), Some(BEND_LEFT), "placed bent");
    let bp = whole_box(&sim);
    assert_eq!(bp.entries[0].facing, belt_facing(1, BEND_LEFT));
    assert_eq!(bp.turned(1).1[0].facing, belt_facing(2, BEND_LEFT), "a turn keeps the bend");
    assert_eq!(import(&export(std::slice::from_ref(&bp))), Some(vec![bp.clone()]));
    let origin = IVec3::new(520, 200, -520);
    let planned = bp.ghosts_at(origin, 3)[0];
    sim.apply(P, Action::PlantGhost { pos: planned.pos, block: BELT, facing: planned.facing, tier: 0 });
    let ghost = *sim.ghosts.covering(planned.pos).expect("the belt's ghost");
    assert_eq!(ghost.facing, belt_facing(0, BEND_LEFT));
    let mut w = ByteWriter::default();
    sim.ghosts.write_state(&mut w);
    assert!(Ghosts::read_state(&mut ByteReader::new(&w.bytes)).is_some(), "a bent ghost reads back");
    build(&mut sim, BELT, ghost.pos, 1);
    assert_eq!(sim.factory.placed_as(ghost.pos).map(|p| p.0), Some(belt_facing(0, BEND_LEFT)), "built as the ghost");
}

#[test]
fn bytes_round_trip_and_damage_is_refused() {
    let bp = whole_box(&factory());
    let bytes = export(&[bp.clone(), Blueprint { name: "Second".into(), ..bp.clone() }]);
    let back = import(&bytes).unwrap();
    assert_eq!((back.len(), &back[0]), (2, &bp));
    assert_eq!(back[1].name, "Second");
    assert_eq!(import(&bytes[..bytes.len() - 1]), None, "short");
    assert_eq!(import(&[9, 0]), None, "unknown format");
    let mut bad = bytes.clone();
    *bad.last_mut().unwrap() = 200; // a tier byte is fine, so damage the facing of the last entry instead
    let n = bad.len();
    bad[n - 2] = 9;
    assert_eq!(import(&bad), None, "facing out of range");
}

#[test]
fn rename_and_delete_keep_the_held_one_straight() {
    let bp = whole_box(&factory());
    let mut lib = Library { list: vec![bp.clone(), bp.clone(), bp], held: Some(2), ..Library::default() };
    lib.rename(1, "  Smelter\nrow  ");
    assert_eq!(lib.list[1].name, "Smelterrow");
    lib.rename(1, "   ");
    assert_eq!(lib.list[1].name, "Smelterrow", "an empty name is ignored");
    lib.delete(0);
    assert_eq!(lib.held, Some(1));
    lib.delete(1);
    assert_eq!(lib.held, None);
    lib.delete(9);
    assert_eq!(lib.list.len(), 1);
}

#[test]
fn the_hands_mark_copy_and_stamp_through_the_game() {
    use crate::raycast::RayHit;
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let base = g.body().eye().floor() + IVec3::new(3, 3, 3);
    let aim = |g: &mut crate::Game, block: IVec3| g.target = Some(RayHit { block, normal: IVec3::new(0, 1, 0), id: 0 });
    for i in 0..3 {
        build(&mut g.sim, BELT, base + IVec3::new(i, 0, 0), 1);
    }

    g.mark_blueprint_corner();
    assert!(g.library.corner.is_none(), "marking needs ghost mode");
    g.toggle_ghost_mode();
    aim(&mut g, base);
    g.mark_blueprint_corner();
    aim(&mut g, base + IVec3::new(2, 1, 0));
    g.mark_blueprint_corner();
    assert!(g.library.selection.is_some());
    assert!(g.blueprint_label().unwrap().starts_with("Box selected"));
    g.copy_blueprint();
    assert_eq!((g.library.list.len(), g.library.held, g.library.selection), (1, Some(0), None));
    assert_eq!(g.library.list[0].entries.len(), 3);

    aim(&mut g, base + IVec3::new(10, 0, 10)); // stamps against the face above this block
    g.using = true;
    g.update_ghosts();
    g.run_ticks(1);
    assert_eq!(g.sim.ghosts.len(), 3);
    assert!(g.sim.ghosts.covering(base + IVec3::new(10, 1, 10)).is_some());
    let label = g.blueprint_label().unwrap();
    assert!(label.contains("Missing 3 Conveyor Belt"), "{label}");
    // The tick re-aims at whatever the eye sees (open sky, on this terrain): aim again for the preview boxes.
    aim(&mut g, base + IVec3::new(10, 0, 10));
    assert!(!g.blueprint_boxes().is_empty());

    g.mark_blueprint_corner();
    assert_eq!(g.library.held, None, "Z puts it away");
}

#[test]
fn a_third_copy_of_a_machine_offers_a_blueprint_once() {
    use crate::block::{CONSTRUCTOR, SMELTER};
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let base = g.body().eye().floor() + IVec3::new(3, 3, 3);
    let place = |g: &mut crate::Game, block: BlockId, at: IVec3| {
        build(&mut g.sim, block, base + at, 1);
        g.run_ticks(1);
    };
    place(&mut g, SMELTER, IVec3::ZERO);
    place(&mut g, SMELTER, IVec3::new(4, 0, 0));
    for i in 0..3 {
        place(&mut g, BELT, IVec3::new(i, 0, 2)); // logistics never count
    }
    assert_eq!(g.repeat_prompt(), "", "two copies are not a pattern yet");
    place(&mut g, SMELTER, IVec3::new(8, 0, 0));
    let tip = g.repeat_prompt();
    assert!(tip.starts_with("Building the same again") && tip.contains("third *Smelter*"), "{tip}");
    let (lo, hi) = g.library.selection.expect("a box is preset");
    let previous = base + IVec3::new(4, 0, 0);
    assert!(lo.x <= previous.x && previous.x <= hi.x && lo.z <= previous.z && previous.z <= hi.z);
    assert!(hi.x < base.x + 8, "the box holds the previous copy, not the new one");

    g.dismiss_repeat_prompt();
    assert_eq!(g.repeat_prompt(), "");
    place(&mut g, SMELTER, IVec3::new(12, 0, 0));
    assert_eq!(g.repeat_prompt(), "", "once per kind a session");

    // A world with a blueprint already knows the tool.
    g.library.selection = None;
    g.library.list.push(Blueprint { name: "Line".into(), size: IVec3::new(1, 1, 1), entries: Vec::new() });
    for i in 0..3 {
        place(&mut g, CONSTRUCTOR, IVec3::new(4 * i, 0, 6));
    }
    assert_eq!((g.repeat_prompt(), g.library.selection), (String::new(), None));
}

/// A game with three smelters in a row (4 apart), so the third brought the prompt and a preset box.
fn prompted_game() -> (crate::Game, IVec3) {
    use crate::block::SMELTER;
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let base = g.body().eye().floor() + IVec3::new(3, 3, 3);
    for i in 0..3 {
        build(&mut g.sim, SMELTER, base + IVec3::new(4 * i, 0, 0), 1);
        g.run_ticks(1);
    }
    assert!(g.repeat_prompt().contains("[Z] marks new corners"));
    (g, base)
}

fn aim_at(g: &mut crate::Game, block: IVec3) {
    g.target = Some(crate::raycast::RayHit { block, normal: IVec3::new(0, 1, 0), id: 0 });
}

#[test]
fn following_the_prompt_word_for_word_copies_a_blueprint() {
    // "Press [B]: a box is already marked … and [Enter] copies it."
    let (mut g, _) = prompted_game();
    g.toggle_ghost_mode();
    g.copy_blueprint();
    assert_eq!((g.library.list.len(), g.library.held), (1, Some(0)), "B then Enter copies the preset box");

    // "([Z] marks new corners)": two presses mark a box of the player's own, whatever was preset.
    let (mut g, base) = prompted_game();
    g.toggle_ghost_mode();
    aim_at(&mut g, base);
    g.mark_blueprint_corner();
    assert!(g.blueprint_label().unwrap().starts_with("Marking a box"), "{:?}", g.blueprint_label());
    aim_at(&mut g, base + IVec3::new(8, 1, 0));
    g.mark_blueprint_corner();
    g.copy_blueprint();
    assert_eq!(g.library.list.len(), 1, "Z, Z, Enter copies");
    assert_eq!(g.library.list[0].entries.len(), 3, "the new box holds all three smelters, not the preset one");
}

#[test]
fn leaving_ghost_mode_forgets_a_half_marked_box() {
    let (mut g, base) = prompted_game();
    g.library.selection = None; // no preset box: the first Z marks a corner
    g.toggle_ghost_mode();
    aim_at(&mut g, base + IVec3::new(20, 0, 20));
    g.mark_blueprint_corner();
    assert!(g.library.corner.is_some());
    g.toggle_ghost_mode();
    assert_eq!((g.library.corner, g.library.selection), (None, None), "B forgets the corner and the box");
    g.toggle_ghost_mode();
    assert_eq!(g.blueprint_label(), None, "nothing is half done");
    aim_at(&mut g, base);
    g.mark_blueprint_corner();
    aim_at(&mut g, base + IVec3::new(4, 1, 0));
    g.mark_blueprint_corner();
    g.copy_blueprint();
    assert_eq!(g.library.list[0].entries.len(), 2, "the box is the two new corners, not a stale one");
}

#[test]
fn the_label_always_says_what_z_does_next() {
    let (mut g, base) = prompted_game();
    g.toggle_ghost_mode();
    let says = |g: &crate::Game, what: &str| {
        let label = g.ghost_label();
        assert!(label.contains(what), "{what:?} missing from {label:?}");
    };
    says(&g, "Z starts a new box"); // the preset box
    aim_at(&mut g, base);
    g.mark_blueprint_corner();
    says(&g, "press Z");
    aim_at(&mut g, base + IVec3::new(4, 1, 0));
    g.mark_blueprint_corner();
    says(&g, "Z starts a new box");
    g.target = None;
    g.mark_blueprint_corner(); // Z at the sky clears the box
    assert_eq!(g.library.selection, None);
    says(&g, "Z marks a box to copy");
    for corner in [base, base + IVec3::new(4, 1, 0)] {
        aim_at(&mut g, corner);
        g.mark_blueprint_corner();
    }
    g.copy_blueprint();
    says(&g, "Z puts it away");
}

#[test]
fn the_blueprint_prompt_ends_by_itself() {
    use crate::block::GENERATOR;
    let mut g = crate::Game::new(2024, 3);
    crate::tests::run_until_ready(&mut g);
    let base = g.body().eye().floor() + IVec3::new(3, 3, 3);
    for i in 0..3 {
        build(&mut g.sim, GENERATOR, base + IVec3::new(2 * i, 0, 0), 0);
        g.run_ticks(1);
    }
    assert!(!g.repeat_prompt().is_empty());
    g.sim.tick += super::repeat::SHOW_TICKS;
    assert_eq!(g.repeat_prompt(), "");
}

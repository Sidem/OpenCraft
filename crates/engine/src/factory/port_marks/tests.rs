use super::*;
use crate::factory::links::{Link, Slot};
use crate::world::World;

/// The marks show exactly the faces that work: for every machine with ports at every turn, a belt pointing into a
/// marked inlet links into the machine, a belt leading away from a marked outlet takes its product (a byproduct outlet
/// its byproduct), and on every other face of its bottom layer neither happens.
#[test]
fn marks_show_exactly_the_faces_that_work() {
    let mut world = World::new(1, 2);
    let anchor = IVec3::new(0, 64, 0);
    let (mut machines, mut faces_checked, mut marked) = (0, 0, 0);
    for b in 1..=255u8 {
        if process::spec(b).is_none_or(|s| s.footprint.is_single()) {
            continue;
        }
        machines += 1;
        for dir in 0..4 {
            let faces = port_faces(b, anchor, dir);
            marked += faces.len();
            let mut f = Factory::default();
            f.place(&mut world, b, anchor, dir, anchor, 0);
            let Some(&Slot::Process(i)) = f.at.get(&anchor) else { panic!("no machine") };
            let cells = f.processors[i as usize].cells();
            let name = crate::block::def(b).name;
            for &c in cells.iter().filter(|c| c.y == anchor.y) {
                for s in 0..4u8 {
                    let stub = c + DIRS[s as usize];
                    if cells.contains(&stub) {
                        continue;
                    }
                    let mark = |flow| faces.contains(&(c, s, flow));
                    let belt = |f: &Factory| match f.at.get(&stub) {
                        Some(&Slot::Belt(j)) => j,
                        _ => panic!("no belt"),
                    };
                    f.add_belt(stub, opposite(s));
                    f.relink();
                    let fed = matches!(f.belts[belt(&f) as usize].out, Link::Machine(Slot::Process(k)) if k == i);
                    assert_eq!(fed, mark(Flow::In), "{name} turned {dir}, cell {c:?} side {s}: a belt pointing in");
                    f.remove(stub);
                    f.add_belt(stub, s);
                    f.relink();
                    let (j, p) = (belt(&f), &f.processors[i as usize]);
                    assert_eq!(p.outs.contains(&j), mark(Flow::Out), "{name} turned {dir}, {c:?} side {s}: product");
                    let side = p.side_outs.contains(&j);
                    assert_eq!(side, mark(Flow::Byproduct), "{name} turned {dir}, {c:?} side {s}: byproduct");
                    f.remove(stub);
                    faces_checked += 1;
                }
            }
        }
    }
    assert!(machines >= 20 && marked >= 200 && faces_checked >= 1000, "{machines} {marked} {faces_checked}");
}

/// A ghost draws its model see-through and its marks: as many arrows as its ports call for, the belt stubs only on
/// open cells.
#[test]
fn a_ghost_draws_its_model_and_one_stub_per_open_port() {
    let anchor = IVec3::new(0, 64, 0);
    let count = |free: bool| {
        let (mut solid, mut ghost) = (Vec::new(), Vec::new());
        draw_ghost(crate::block::ASSEMBLER, anchor, 0, Vec3::ZERO, 0.0, |_| free, &mut solid, &mut ghost);
        (solid.len() / super::super::INSTANCE_FLOATS, ghost.len() / super::super::INSTANCE_FLOATS)
    };
    let (solid_open, ghost_open) = count(true);
    let (solid_shut, ghost_shut) = count(false);
    // The assembler: three inlet sides and an outlet side, two cells each, two arrows of two arms per face.
    let faces = port_faces(crate::block::ASSEMBLER, anchor, 0).len();
    assert_eq!(faces, 8);
    assert!(solid_open >= faces * 4, "arrows and riding items: {solid_open}");
    assert_eq!(solid_open, solid_shut, "arrows and items show either way");
    assert!(ghost_open > ghost_shut && ghost_shut > 10, "model {ghost_shut}, with stubs {ghost_open}");
}

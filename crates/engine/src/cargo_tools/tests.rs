use super::*;
use crate::block::DRONE_PORT;
use crate::item::ItemId;
use crate::raycast::RayHit;
use crate::sim::PlayerId;

const P: PlayerId = PlayerId(0);

fn v(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn port_at(g: &mut Game, pos: IVec3) {
    g.sim.apply(P, Action::Give { item: DRONE_PORT.into(), count: 1 });
    let slot =
        g.sim.player(P).unwrap().inventory.slots.iter().position(|s| s.item == ItemId::from(DRONE_PORT)).unwrap();
    g.sim.apply(P, Action::PlaceBlock { pos, slot: slot as u8, facing: 0, against: pos });
}

fn aim_at(g: &mut Game, block: IVec3) {
    g.target = Some(RayHit { block, normal: IVec3::ZERO, id: block::AIR });
}

fn click(g: &mut Game) {
    for on in [true, false] {
        g.using = on;
        g.update_cargo_tools();
        g.sim.step();
    }
}

/// Two ports `gap` blocks apart in the air and a cargo drone in hand; returns (game, first port, second port).
fn two_ports(gap: i32) -> (Game, IVec3, IVec3) {
    let mut g = Game::new(2024, 3);
    let (a, b) = (v(900, 200, -900), v(900 + gap, 200, -900));
    port_at(&mut g, a);
    port_at(&mut g, b);
    g.give(CARGO_DRONE.0, 1);
    g.run_ticks(3);
    let slot = g.inventory().slots.iter().position(|s| s.item == CARGO_DRONE).expect("given") as u32;
    g.select_slot(slot);
    g.run_ticks(2);
    (g, a, b)
}

#[test]
fn clicking_two_ports_sets_a_route_and_crouch_clicking_clears_it() {
    let (mut g, a, b) = two_ports(80);
    assert!(g.cargo_label().starts_with("Cargo Drone\naim at a drone port"), "{}", g.cargo_label());
    aim_at(&mut g, a);
    assert!(g.cargo_label().contains("start a route"), "{}", g.cargo_label());
    assert_eq!(g.cargo_boxes()[6], GHOST_GREEN);
    click(&mut g);
    assert_eq!(g.cargo_from, Some(a));
    aim_at(&mut g, b);
    assert!(g.cargo_label().contains("80 blocks, 2 batteries a trip"), "{}", g.cargo_label());
    let boxes = g.cargo_boxes();
    assert_eq!((boxes[6], boxes[13]), (GHOST_GREEN, SELECTED_BLUE), "the aimed port and the first one");
    click(&mut g);
    assert_eq!(g.sim.cargo.route_from(a), Some(b));
    assert_eq!(g.cargo_from, None);
    aim_at(&mut g, a);
    assert!(g.cargo_label().contains("Route: to the port at"), "{}", g.cargo_label());
    assert_eq!(g.cargo_boxes()[13], ROUTE_AMBER, "the destination is outlined");
    g.body_mut().input.crouch = true;
    click(&mut g);
    g.body_mut().input.crouch = false;
    assert_eq!(g.sim.cargo.route_from(a), None);
}

#[test]
fn a_port_out_of_range_is_red_and_only_a_cargo_drone_in_hand_uses_the_hand() {
    let (mut g, a, b) = two_ports(250);
    aim_at(&mut g, a);
    click(&mut g);
    aim_at(&mut g, b);
    assert!(g.cargo_label().contains("too far"), "{}", g.cargo_label());
    assert_eq!(g.cargo_boxes()[6], BLOCKED_RED);
    click(&mut g);
    assert_eq!(g.sim.cargo.route_from(a), None);
    g.select_slot(8);
    g.run_ticks(2);
    assert!(g.cargo_boxes().is_empty() && g.cargo_label().is_empty());
    assert!(!g.update_cargo_tools());
    assert_eq!(g.cargo_from, None, "putting the drone away forgets the first port");
}

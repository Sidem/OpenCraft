//! Aiming at a train car from the player's eye: the readout, the outline, the right-click and the box clicks on its
//! cargo, through `Game` as the host sees it.

use super::*;
use crate::block::{AIR, RAIL, STONE};
use crate::factory::car_box;
use crate::item::{IRON_INGOT, LOCOMOTIVE};
use crate::math::{IVec3, Vec3};

#[test]
fn aiming_at_a_wagon_reads_it_and_a_right_click_opens_its_cargo_as_a_box() {
    let mut g = Game::new(2024, 3);
    run_until_ready(&mut g);
    let y = g.sim.world.generator().height_at(0, 0);
    for x in -4..=30 {
        for z in -6..=6 {
            for h in y - 2..=y + 6 {
                g.sim.world.set_block_anywhere_later(IVec3::new(x, h, z), if h <= y { STONE } else { AIR });
            }
        }
    }
    let (a, b) = (IVec3::new(0, y + 1, 0), IVec3::new(24, y + 1, 0));
    for p in [a, b] {
        g.sim.world.set_block_anywhere(p, RAIL);
        g.sim.factory.place(&mut g.sim.world, RAIL, p, 64, p - IVec3::new(0, 1, 0), 0);
    }
    g.sim.factory.lay_track(a, b);
    assert!(g.sim.factory.place_train(a) && g.sim.factory.couple(a) && g.sim.factory.couple(a));
    let id = 1;
    assert!(g.sim.factory.train_by_id(id).is_some(), "the first train is number 1");
    let wagon = car_box(id, 1);
    g.sim.factory.box_slots_mut(wagon).unwrap()[0] = crate::inventory::Stack { item: IRON_INGOT, count: 30 };

    // Stand beside the first wagon (2.5 to 5 m behind the front, at x 3.5..6) and look at it.
    let front = g.sim.factory.train_by_id(id).unwrap().head as f64 / 1000.0 + 0.5;
    let at = Vec3::new(front - 3.75, f64::from(y + 1) + 0.3, 0.5);
    g.body_mut().pos = Vec3::new(at.x, f64::from(y + 1), 3.5);
    let eye = g.body().eye();
    let d = at - eye;
    g.set_look(d.x.atan2(-d.z), (d.y / d.length()).asin());
    g.update_target();
    assert!(g.has_target() && g.target.is_none(), "a car stands in for the block behind it");
    assert_eq!(g.target_name(), "Wagon 1 of 2");
    assert!(g.target_detail().starts_with("1 of 24 slots used: 30 "), "{}", g.target_detail());
    assert_eq!(g.target_fit().len(), 6, "the car is outlined");

    // A locomotive in hand aims at the rail nodes under a train instead.
    g.give(LOCOMOTIVE.0, 1);
    g.run_ticks(1);
    let slot = g.inventory().slots.iter().position(|s| s.item == LOCOMOTIVE).unwrap() as u32;
    g.select_slot(slot);
    g.run_ticks(1);
    g.update_target();
    assert!(g.target_car.is_none());
    g.select_slot((slot + 1) % 9);
    g.run_ticks(1);
    g.update_target();

    // Right-click: the panel request is the wagon's cargo address, which reads and clicks like a box.
    g.using = true;
    g.update_placing(1.0 / 60.0);
    let request = g.take_panel_request();
    assert_eq!(request, vec![wagon.x, wagon.y, wagon.z]);
    assert_eq!(g.train_panel(wagon.x, wagon.y, wagon.z)[0], 2, "two wagons");
    assert_eq!(&g.box_slots(wagon.x, wagon.y, wagon.z)[..2], &[u32::from(IRON_INGOT.0), 30]);
    g.click_box(wagon.x, wagon.y, wagon.z, 0, true);
    g.run_ticks(1);
    assert!(g.sim.factory.train_by_id(id).unwrap().cargo.slots[0].is_empty(), "shift-click moved it to the inventory");
    assert!(g.inventory().slots.iter().any(|s| s.item == IRON_INGOT && s.count >= 30));
}

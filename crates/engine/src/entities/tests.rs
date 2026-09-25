use super::*;

#[test]
fn item_falls_and_is_collected() {
    let mut items = Items::default();
    let mut inv = Inventory::default();
    items.spawn(Vec3::new(0.5, 3.0, 0.5), Vec3::ZERO, 1, 2, 0.0);
    let mut floor = |_x: i32, y: i32, _z: i32| y < 0;
    let loaded = |_p: Vec3| true;
    let far = Vec3::new(50.0, 0.0, 50.0);
    for _ in 0..240 {
        items.update(1.0 / 60.0, far, &mut floor, &loaded, &mut inv, |_, _| {});
    }
    assert!((items.list[0].pos.y - HALF).abs() < 1e-6, "rests on the floor");

    let mut got = 0;
    for _ in 0..120 {
        items.update(1.0 / 60.0, Vec3::new(1.5, 0.9, 0.5), &mut floor, &loaded, &mut inv, |_, n| got += n);
    }
    assert!(items.list.is_empty());
    assert_eq!(got, 2);
    assert_eq!(inv.slots[0].count, 2);
}

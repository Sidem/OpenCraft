use super::*;

const STROKES: [Stroke; 5] = [Stroke::Pick, Stroke::Axe, Stroke::Dig, Stroke::Sweep, Stroke::Punch];

fn at(stroke: Stroke, t: f64) -> Pose {
    pose(stroke, t * TAU, 1.0)
}

fn values(p: Pose) -> [f64; 6] {
    [p.swing, p.lean, p.sweep, p.roll, p.thrust, p.lift]
}

#[test]
fn every_stroke_is_smooth_and_loops_back_to_rest() {
    for stroke in STROKES {
        assert!(values(at(stroke, 0.0)).iter().all(|v| v.abs() < 1e-9), "{stroke:?} starts at rest");
        assert!(values(at(stroke, 1.0)).iter().all(|v| v.abs() < 1e-9), "{stroke:?} ends at rest");
        // No jumps between frames at 60 fps.
        let steps = 2000;
        for i in 0..steps {
            let (a, b) =
                (values(at(stroke, i as f64 / steps as f64)), values(at(stroke, (i + 1) as f64 / steps as f64)));
            assert!(a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.015), "{stroke:?} jumps at {i}");
        }
        // Not mining: still.
        assert!(values(pose(stroke, 2.0, 0.0)).iter().all(|v| *v == 0.0));
    }
}

#[test]
fn pickaxe_winds_up_overhead_strikes_fast_and_recoils() {
    let raised = (0..=100)
        .map(|i| i as f64 / 100.0)
        .min_by(|a, b| at(Stroke::Pick, *a).swing.total_cmp(&at(Stroke::Pick, *b).swing))
        .unwrap();
    let struck = (0..=100)
        .map(|i| i as f64 / 100.0)
        .max_by(|a, b| at(Stroke::Pick, *a).swing.total_cmp(&at(Stroke::Pick, *b).swing))
        .unwrap();
    assert!(raised < struck, "the strike comes after the wind-up");
    assert!(struck - raised < raised * 0.5, "the strike is much faster than the wind-up");
    assert!(at(Stroke::Pick, raised).lift > 0.0 && at(Stroke::Pick, struck).lift < 0.0, "hand rises then drops");
    assert!(at(Stroke::Pick, struck).thrust > 0.05, "hand drives forward onto the rock");
    assert!(at(Stroke::Pick, 0.83).swing < at(Stroke::Pick, struck).swing, "recoil bounces back after impact");
}

#[test]
fn axe_cleaves_diagonally_distinct_from_pickaxe() {
    let windup = at(Stroke::Axe, 0.50);
    let hit = at(Stroke::Axe, 0.70);
    let pick_windup = at(Stroke::Pick, 0.52);
    assert!(windup.sweep > 0.25 && hit.sweep < -0.30, "axe swings wide right-to-left: {windup:?} -> {hit:?}");
    assert!(windup.roll < -0.4 && hit.roll < -0.3, "axe rolls its blade diagonally into the cut plane");
    assert!(
        windup.sweep - hit.sweep > (pick_windup.sweep - at(Stroke::Pick, 0.72).sweep) * 2.5,
        "axe has a much wider lateral chop than the vertical pickaxe"
    );
}

#[test]
fn a_dig_stabs_in_then_prises_the_blade_up() {
    let stabbed = at(Stroke::Dig, 0.36);
    let prised = at(Stroke::Dig, 0.66);
    assert!(stabbed.thrust > 0.1 && prised.thrust > 0.05, "the blade stays in while it prises");
    assert!(prised.lean > stabbed.lean + 0.3, "the blade levers up to scoop");
}

#[test]
fn bare_hand_punch_cocks_back_and_strikes_forward() {
    let cock = at(Stroke::Punch, 0.36);
    let hit = at(Stroke::Punch, 0.58);
    assert!(cock.thrust < -0.02, "fist cocks back before punching");
    assert!(hit.thrust > 0.10, "fist drives forward into the block");
    assert!(hit.sweep < cock.sweep - 0.2, "fist strikes inward toward the crosshair");
}

#[test]
fn the_arm_follows_each_stroke() {
    use crate::item::{IRON_AXE, IRON_PICKAXE, IRON_SHOVEL};
    for held in [IRON_PICKAXE, IRON_AXE, IRON_SHOVEL, ItemId::NONE] {
        let all: Vec<f64> = (0..=200).map(|i| arm(held, i as f64 / 200.0 * TAU)).collect();
        assert!(all.iter().all(|a| (-1.0..=1.0).contains(a)), "{held:?} stays in range");
        assert!(all.iter().cloned().fold(f64::MIN, f64::max) > 0.95, "{held:?} reaches its strike");
    }
    assert!(arm(IRON_PICKAXE, 0.52 * TAU) < -0.95 && arm(IRON_PICKAXE, 0.72 * TAU) > 0.95);
    assert!(arm(IRON_AXE, 0.50 * TAU) < -0.95 && arm(IRON_AXE, 0.70 * TAU) > 0.95);
}

use super::*;

const STROKES: [Stroke; 3] = [Stroke::Chop, Stroke::Dig, Stroke::Sweep];

fn at(stroke: Stroke, t: f64) -> Pose {
    pose(stroke, t * TAU, 1.0)
}

fn values(p: Pose) -> [f64; 5] {
    [p.swing, p.lean, p.sweep, p.thrust, p.lift]
}

#[test]
fn every_stroke_is_smooth_and_loops_back_to_rest() {
    for stroke in STROKES {
        assert!(values(at(stroke, 0.0)).iter().all(|v| v.abs() < 1e-9), "{stroke:?} starts at rest");
        assert!(values(at(stroke, 1.0)).iter().all(|v| v.abs() < 1e-9), "{stroke:?} ends at rest");
        // No jumps between frames at 60 fps (a stroke is 0.48 s: about 29 frames).
        let steps = 2000;
        for i in 0..steps {
            let (a, b) =
                (values(at(stroke, i as f64 / steps as f64)), values(at(stroke, (i + 1) as f64 / steps as f64)));
            assert!(a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.01), "{stroke:?} jumps at {i}");
        }
        // Not mining: still.
        assert!(values(pose(stroke, 2.0, 0.0)).iter().all(|v| *v == 0.0));
    }
}

#[test]
fn a_chop_winds_up_slowly_and_strikes_fast() {
    let raised = (0..=100)
        .map(|i| i as f64 / 100.0)
        .min_by(|a, b| at(Stroke::Chop, *a).swing.total_cmp(&at(Stroke::Chop, *b).swing));
    let struck = (0..=100)
        .map(|i| i as f64 / 100.0)
        .max_by(|a, b| at(Stroke::Chop, *a).swing.total_cmp(&at(Stroke::Chop, *b).swing));
    let (raised, struck) = (raised.unwrap(), struck.unwrap());
    assert!(raised < struck, "the strike comes after the wind-up");
    assert!(struck - raised < raised * 0.5, "the strike is much faster than the wind-up");
    assert!(at(Stroke::Chop, raised).lift > 0.0 && at(Stroke::Chop, struck).lift < 0.0, "the hand rises, then drops");
}

#[test]
fn a_dig_stabs_in_then_prises_the_blade_up() {
    let stabbed = at(Stroke::Dig, 0.3);
    let prised = at(Stroke::Dig, 0.65);
    assert!(stabbed.thrust > 0.1 && prised.thrust > 0.05, "the blade stays in while it prises");
    // Held at the top, levering the blade up turns it forward about the hand.
    assert!(prised.lean > stabbed.lean + 0.2, "the blade levers up");
}

#[test]
fn the_arm_follows_each_stroke() {
    use crate::item::{IRON_AXE, IRON_PICKAXE, IRON_SHOVEL};
    for held in [IRON_PICKAXE, IRON_AXE, IRON_SHOVEL, ItemId::NONE] {
        let all: Vec<f64> = (0..=200).map(|i| arm(held, i as f64 / 200.0 * TAU)).collect();
        assert!(all.iter().all(|a| (-1.0..=1.0).contains(a)), "{held:?} stays in range");
        assert!(all.iter().cloned().fold(f64::MIN, f64::max) > 0.95, "{held:?} reaches its strike");
    }
    // A pick's arm is raised while the pick winds up, and struck when it strikes.
    assert!(arm(IRON_PICKAXE, 0.55 * TAU) < -0.95 && arm(IRON_PICKAXE, 0.75 * TAU) > 0.95);
}

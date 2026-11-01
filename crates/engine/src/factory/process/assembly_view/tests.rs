use super::*;
use crate::recipes::{Category, MACHINE_RECIPES};

/// Samples of a batch the checks step through.
const SAMPLES: usize = 4000;

fn assembly() -> impl Iterator<Item = &'static MachineRecipe> {
    MACHINE_RECIPES.iter().filter(|r| matches!(r.category, Category::Assembly))
}

fn gap(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn every_input_kind_is_carried() {
    assert!(assembly().count() > 3);
    for r in assembly() {
        let plan = plan(r);
        assert!(plan.n > 0);
        for &(item, _) in r.inputs {
            assert!(plan.picks[..plan.n].iter().any(|k| k.item == item), "{:?} misses {item:?}", r.outputs);
        }
        // Every pick is set down before the cloud comes.
        for k in &plan.picks[..plan.n] {
            assert!(k.start + k.len <= GATHER + 1e-9);
        }
    }
}

#[test]
fn arms_reach_their_wrists_and_items_never_jump() {
    for r in assembly() {
        let plan = plan(r);
        let mut last: Vec<Option<[f64; 3]>> = vec![None; plan.n];
        let mut wrists = [rest(0), rest(1)];
        for i in 0..=SAMPLES {
            let t = i as f64 / SAMPLES as f64;
            for (arm, prev) in wrists.iter_mut().enumerate() {
                let w = wrist(&plan, arm, t, 0.0);
                let s = BASES[arm];
                let reached = elbow([s[0], s[1] + SHOULDER, s[2]], w).1;
                assert!(gap(reached, w) < 1e-6, "arm {arm} can't reach {w:?} at {t}");
                assert!(gap(*prev, w) < 0.03, "arm {arm} jumps at {t}");
                *prev = w;
            }
            for (k, prev) in plan.picks[..plan.n].iter().zip(last.iter_mut()) {
                let now = input_at(&plan, k, t, 0.0).map(|i| i.0);
                if let (Some(a), Some(b)) = (*prev, now) {
                    assert!(gap(a, b) < 0.03, "an input of {:?} jumps at {t}", r.outputs);
                }
                *prev = now;
            }
        }
    }
}

#[test]
fn the_cloud_hides_the_swap_and_the_product_rides_out() {
    assert!(cloud(SWAP) > 0.99);
    assert!(product_at(SWAP - 1e-6).is_none() && product_at(SWAP).is_some());
    let mut prev = product_at(SWAP).unwrap();
    assert_eq!(prev, FIXTURE);
    for i in 0..=SAMPLES {
        let t = SWAP + (1.0 - SWAP) * i as f64 / SAMPLES as f64;
        let now = product_at(t).unwrap();
        assert!(gap(prev, now) < 0.03 && now[2] >= prev[2]);
        prev = now;
    }
    assert!((prev[2] - EXIT_Z).abs() < 1e-9 && prev[1] == 0.0);
}

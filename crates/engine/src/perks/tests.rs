use super::*;

fn finish(r: &mut Research, tech: usize) {
    (0..TECHS[tech].units).for_each(|_| r.add_unit(tech as u8));
}

/// The techs that list a tier of `stat`, in tier order.
fn tiers(stat: Stat) -> Vec<(usize, u8)> {
    let mut found = Vec::new();
    for (i, t) in TECHS.iter().enumerate() {
        for u in t.unlocks {
            if let Unlock::Perk(s, tier) = *u {
                if s == stat {
                    found.push((i, tier));
                }
            }
        }
    }
    found
}

#[test]
fn crafting_has_three_tiers_in_order_each_needing_the_last() {
    let found = tiers(Stat::Crafting);
    assert_eq!(found.iter().map(|f| f.1).collect::<Vec<_>>(), [1, 2, 3]);
    for pair in found.windows(2) {
        assert!(TECHS[pair[1].0].needs.contains(&(pair[0].0 as u8)), "{} needs the tier before", TECHS[pair[1].0].name);
    }
}

#[test]
fn each_finished_tier_adds_a_quarter_linearly() {
    let mut r = Research::default();
    assert_eq!(player_bonus(&r, Stat::Crafting), 1000);
    let found = tiers(Stat::Crafting);
    finish(&mut r, found[0].0);
    assert_eq!(player_bonus(&r, Stat::Crafting), 1250);
    r.add_unit(found[1].0 as u8);
    assert_eq!(player_bonus(&r, Stat::Crafting), 1250, "a tier counts only when done");
    finish(&mut r, found[1].0);
    finish(&mut r, found[2].0);
    assert_eq!(player_bonus(&r, Stat::Crafting), 1750);
    let mut all = Research::default();
    all.complete_all();
    assert_eq!(player_bonus(&all, Stat::Crafting), 1750, "creative worlds craft at the top speed");
}

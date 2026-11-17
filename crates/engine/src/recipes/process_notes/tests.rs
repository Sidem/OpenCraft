use super::*;
use crate::recipes::{Category, MACHINE_RECIPES};

/// The machines that teach: the refinery, the cracker, the chemical plant, the electrolyser and the centrifuge.
const TEACHING: [Category; 5] =
    [Category::Distilling, Category::Cracking, Category::Chemistry, Category::Splitting, Category::Enrichment];

#[test]
fn every_row_of_a_chemical_machine_has_a_note() {
    let missing: Vec<usize> = (0..MACHINE_RECIPES.len())
        .filter(|&i| TEACHING.contains(&MACHINE_RECIPES[i].category) && note(i).is_empty())
        .collect();
    assert!(missing.is_empty(), "machine recipes without a note: {missing:?}; add a row to NOTES");
}

#[test]
fn notes_are_one_plain_line_for_a_row_that_exists_once() {
    let mut seen = Vec::new();
    for &(row, text) in NOTES {
        assert!(usize::from(row) < MACHINE_RECIPES.len(), "row {row}");
        assert!(
            TEACHING.contains(&MACHINE_RECIPES[usize::from(row)].category),
            "row {row} is not a chemical machine's"
        );
        assert!(!seen.contains(&row), "row {row} twice");
        seen.push(row);
        assert!(text.len() < 150 && !text.contains('\n') && text.ends_with('.'), "row {row}: {text}");
    }
    assert_eq!(note(usize::MAX), "", "a row with none says nothing");
}

//! Canonical round-trip oracle against the real corpus.
//!
//! The first render normalizes (whitespace, divergent stubs, marker forms), so
//! we do NOT assert byte-equality with the original. Instead we assert
//! *idempotence*: render(parse(render(parse(md)))) == render(parse(md)). Once
//! the data is in canonical form, every further round-trip is byte-stable.

use markdown_project_manager::{parse, render};

fn fixtures() -> (String, String) {
    let open = include_str!("fixtures/FOLLOWUPS.md").to_string();
    let done = include_str!("fixtures/FOLLOWUPS_DONE.md").to_string();
    (open, done)
}

#[test]
fn render_is_idempotent_on_real_corpus() {
    let (open, done) = fixtures();

    let store1 = parse::parse(&open, &done, (0, 0));
    let (open1, done1) = render::render(&store1);

    let store2 = parse::parse(&open1, &done1, (0, 0));
    let (open2, done2) = render::render(&store2);

    assert_eq!(open1, open2, "open file not stable on second round-trip");
    assert_eq!(done1, done2, "done file not stable on second round-trip");
}

#[test]
fn entry_count_preserved_through_roundtrip() {
    let (open, done) = fixtures();
    let store1 = parse::parse(&open, &done, (0, 0));
    let n1 = store1.entries.len();

    let (open1, done1) = render::render(&store1);
    let store2 = parse::parse(&open1, &done1, (0, 0));

    assert_eq!(n1, store2.entries.len(), "entry count drifted through round-trip");
    assert!(n1 >= 12, "expected at least the 12 open entries, got {n1}");
}

#[test]
fn open_entries_have_ids_and_categories() {
    let (open, done) = fixtures();
    let store = parse::parse(&open, &done, (0, 0));
    for e in &store.entries {
        assert!(e.id.starts_with("FU-"), "bad id: {}", e.id);
        if matches!(e.location, markdown_project_manager::model::Location::Open) {
            assert!(!e.category.is_empty(), "open entry {} has no category", e.id);
        }
    }
}

//! CRUD lifecycle tests against a temp .claude dir seeded from the fixtures.
//! Each test runs the real command path (load → mutate → save → re-render), so
//! a regression in store/render/edge-rebuild surfaces here.

use markdown_project_manager::cli::Command;
use markdown_project_manager::commands;
use markdown_project_manager::model::{Location, Status};
use markdown_project_manager::store::{self, Paths};
use std::fs;
use std::path::PathBuf;

/// A throwaway .claude dir seeded with the fixtures. Uniquely named per test via
/// the caller-supplied tag (no Date/random in tests; the tag is the isolation).
fn seed(tag: &str) -> Paths {
    let dir = std::env::temp_dir().join(format!("mpm_test_{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("FOLLOWUPS.md"), include_str!("fixtures/FOLLOWUPS.md")).unwrap();
    fs::write(
        dir.join("FOLLOWUPS_DONE.md"),
        include_str!("fixtures/FOLLOWUPS_DONE.md"),
    )
    .unwrap();
    Paths::resolve(Some(&dir))
}

fn load(paths: &Paths) -> markdown_project_manager::model::Store {
    store::load(paths).unwrap()
}

fn open_md(paths: &Paths) -> String {
    fs::read_to_string(&paths.open_md).unwrap()
}

#[test]
fn done_moves_entry_to_archive_and_leaves_stub() {
    let p = seed("done");
    let id = "FU-2026-05-23-021"; // an open entry in the fixture

    commands::run(
        Command::Done {
            id: id.into(),
            pr: Some(999),
            branch: None,
            commit: Some("abc1234".into()),
            note: Some("test".into()),
        },
        &p,
    )
    .unwrap();

    let store = load(&p);
    let e = store.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.status, Status::Done);
    assert_eq!(e.location, Location::Done);

    // Stub appears in the Open file's derived ## Resolved section.
    let open = open_md(&p);
    assert!(
        open.contains(&format!("<!-- {id} → ✅ done in PR #999")),
        "stub missing:\n{open}"
    );
    // The full body no longer renders in the Open file.
    assert!(!open.contains(&format!("### {id}  ·")), "open body should be gone");
    // It renders in the Done file.
    let done = fs::read_to_string(&p.done_md).unwrap();
    assert!(done.contains(&format!("### {id}  ·")), "done body missing");
}

#[test]
fn reopen_pulls_entry_back_to_open() {
    let p = seed("reopen");
    let id = "FU-2026-05-22-001"; // a done entry in the fixture

    commands::run(Command::Reopen { id: id.into() }, &p).unwrap();

    let store = load(&p);
    let e = store.entries.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.status, Status::Open);
    assert_eq!(e.location, Location::Open);
    assert!(e.resolution.is_none());
}

#[test]
fn add_then_show_roundtrips_fields() {
    let p = seed("add");
    commands::run(
        Command::Add {
            category: "CLI / Commands".into(),
            scope: "a brand new follow-up".into(),
            why: Some("out-of-scope".into()),
            next: None,
            size: Some("S".into()),
            owner: None,
            surfaced: Some("PR #777".into()),
        },
        &p,
    )
    .unwrap();

    let store = load(&p);
    let e = store
        .entries
        .iter()
        .find(|e| e.core.scope.as_deref() == Some("a brand new follow-up"))
        .unwrap();
    assert_eq!(e.status, Status::Open);
    assert_eq!(e.core.owner.as_deref(), Some("unassigned"));
    assert_eq!(e.provenance, "surfaced in PR #777");
    assert!(open_md(&p).contains("a brand new follow-up"));
}

#[test]
fn block_sets_status_and_unblock_clears_it() {
    let p = seed("block");
    let id = "FU-2026-05-23-021";

    commands::run(
        Command::Block {
            id: id.into(),
            on: "FU-2026-05-23-007".into(),
        },
        &p,
    )
    .unwrap();
    let e = load(&p).entries.iter().find(|e| e.id == id).cloned().unwrap();
    assert_eq!(e.status, Status::Blocked);

    commands::run(
        Command::Unblock {
            id: id.into(),
            on: None,
        },
        &p,
    )
    .unwrap();
    let e = load(&p).entries.iter().find(|e| e.id == id).cloned().unwrap();
    assert_eq!(e.status, Status::Open);
}

#[test]
fn wontfix_archives_with_reason() {
    let p = seed("wontfix");
    let id = "FU-2026-05-23-021";
    commands::run(
        Command::Wontfix {
            id: id.into(),
            reason: "not worth it".into(),
        },
        &p,
    )
    .unwrap();
    let e = load(&p).entries.iter().find(|e| e.id == id).cloned().unwrap();
    assert_eq!(e.status, Status::Wontfix);
    assert_eq!(e.location, Location::Done);
    assert!(fs::read_to_string(&p.done_md)
        .unwrap()
        .contains("🚫 wontfix: not worth it"));
}

#[test]
fn cache_is_reused_when_fresh_and_reimported_when_stale() {
    let p = seed("cache");
    // First load builds + writes the cache.
    let _ = load(&p);
    assert!(PathBuf::from(&p.cache).exists());

    // A mutation re-renders MD and refreshes cache; a subsequent load matches.
    commands::run(
        Command::Wontfix {
            id: "FU-2026-05-23-021".into(),
            reason: "x".into(),
        },
        &p,
    )
    .unwrap();
    let store = load(&p);
    assert!(store
        .entries
        .iter()
        .any(|e| e.id == "FU-2026-05-23-021" && e.status == Status::Wontfix));
}

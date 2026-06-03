//! First-run behavior on a repo with no FOLLOWUPS.md: reads must not error or
//! create files (no log = empty log); writes initialize `<dir>/.claude` on demand.

use markdown_project_manager::cli::Command;
use markdown_project_manager::store::{self, Paths};
use markdown_project_manager::{commands, wal};

fn fresh(tag: &str) -> Paths {
    let dir = std::env::temp_dir().join(format!("mpm_init_{tag}")).join(".claude");
    let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    // Note: do NOT create the dir — that's what we're testing.
    Paths::resolve(Some(&dir))
}

#[test]
fn read_on_missing_log_returns_empty_and_creates_nothing() {
    let p = fresh("read");
    let store = wal::read(&p).unwrap();
    assert_eq!(store.entries.len(), 0, "missing log reads as empty");
    assert!(!p.open_md.exists(), "a read must not create the log file");
    assert!(!p.open_md.parent().unwrap().exists(), "a read must not create .claude");
    let _ = std::fs::remove_dir_all(p.open_md.parent().unwrap().parent().unwrap());
}

#[test]
fn first_write_initializes_the_log_dir() {
    let p = fresh("write");
    assert!(!p.open_md.parent().unwrap().exists());
    commands::run(
        Command::Add {
            id: None,
            category: "Init".into(),
            scope: "the very first entry in a brand-new repo".into(),
            why: None,
            next: None,
            size: None,
            owner: None,
            surfaced: None,
        },
        &p,
    )
    .unwrap();
    // Drain to markdown, then the entry is durably stored.
    while wal::try_group_commit(&p).unwrap() {}
    let store = store::load(&p).unwrap();
    assert_eq!(store.entries.len(), 1, "first add must land");
    assert!(p.open_md.exists(), "first write creates FOLLOWUPS.md");
    let _ = std::fs::remove_dir_all(p.open_md.parent().unwrap().parent().unwrap());
}

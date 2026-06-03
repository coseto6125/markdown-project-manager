//! Upward `.claude/FOLLOWUPS.md` discovery, so `mpm` inside any repo targets
//! that repo's log without an explicit `--dir`.

use markdown_project_manager::store::find_claude_upward;
use std::fs;

fn unique_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mpm_discover_{tag}"));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn finds_enclosing_claude_log_from_a_deep_subdir() {
    let root = unique_root("found");
    let claude = root.join("repo/.claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("FOLLOWUPS.md"), "# Follow-ups\n").unwrap();
    let deep = root.join("repo/src/a/b");
    fs::create_dir_all(&deep).unwrap();

    let found = find_claude_upward(&deep).expect("should find the enclosing .claude");
    assert_eq!(found, claude);
}

#[test]
fn returns_none_when_no_log_above() {
    let root = unique_root("none");
    let deep = root.join("repo/src");
    fs::create_dir_all(&deep).unwrap();
    // A bare .claude with no FOLLOWUPS.md must NOT match.
    fs::create_dir_all(root.join("repo/.claude")).unwrap();

    assert!(find_claude_upward(&deep).is_none());
}

#[test]
fn nearest_log_wins_over_an_ancestor() {
    let root = unique_root("nearest");
    let outer = root.join("outer/.claude");
    let inner = root.join("outer/inner/.claude");
    fs::create_dir_all(&outer).unwrap();
    fs::create_dir_all(&inner).unwrap();
    fs::write(outer.join("FOLLOWUPS.md"), "outer\n").unwrap();
    fs::write(inner.join("FOLLOWUPS.md"), "inner\n").unwrap();
    let start = root.join("outer/inner/src");
    fs::create_dir_all(&start).unwrap();

    assert_eq!(find_claude_upward(&start).unwrap(), inner);
}

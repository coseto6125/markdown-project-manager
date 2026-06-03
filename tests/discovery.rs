//! Path discovery: with no explicit `--dir`, mpm must locate the log by walking
//! up from the current directory to the nearest `.claude/FOLLOWUPS.md` (git-style),
//! never a hardcoded absolute path. This is what makes mpm portable across
//! machines and operating systems (the previous Unix-only fallback broke Windows
//! and every non-author checkout).

use markdown_project_manager::store::Paths;
use std::fs;

/// A throwaway nested dir tree seeded with a `.claude/FOLLOWUPS.md`, returned
/// alongside a deep subdir to resolve from. Uniquely named per test via the tag.
fn seed_tree(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("mpm_disc_{tag}"));
    let _ = fs::remove_dir_all(&root);
    let claude = root.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("FOLLOWUPS.md"), "# Follow-ups\n\n## Open\n").unwrap();
    let deep = root.join("crates").join("inner").join("src");
    fs::create_dir_all(&deep).unwrap();
    (root, deep)
}

#[test]
fn resolve_walks_up_to_nearest_claude_followups() {
    let (root, deep) = seed_tree("walkup");
    let paths = Paths::resolve_from(None, &deep);
    assert_eq!(paths.open_md, root.join(".claude").join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn resolve_explicit_dir_overrides_discovery() {
    let (root, deep) = seed_tree("explicit");
    let elsewhere = root.join("other");
    fs::create_dir_all(&elsewhere).unwrap();
    let paths = Paths::resolve_from(Some(&elsewhere), &deep);
    assert_eq!(paths.open_md, elsewhere.join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn resolve_no_log_found_falls_back_to_cwd_dot_claude_not_hardcoded() {
    let root = std::env::temp_dir().join("mpm_disc_none");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let paths = Paths::resolve_from(None, &root);
    assert_eq!(paths.open_md, root.join(".claude").join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

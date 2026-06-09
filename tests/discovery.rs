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

#[test]
fn resolve_stops_at_git_repo_boundary_not_crossing_into_outer_repo() {
    // Mirrors the real incident: an inner repo (its own `.git`, no log) nested
    // under an outer repo that DOES have a log. Discovery must stop at the inner
    // repo root and NOT bleed into the outer repo's log.
    let outer = std::env::temp_dir().join("mpm_disc_boundary");
    let _ = fs::remove_dir_all(&outer);
    let outer_claude = outer.join(".claude");
    fs::create_dir_all(&outer_claude).unwrap();
    fs::write(outer_claude.join("FOLLOWUPS.md"), "# outer\n").unwrap();

    let inner = outer.join("vendored").join("inner-repo");
    let inner_src = inner.join("src");
    fs::create_dir_all(inner.join(".git")).unwrap();
    fs::create_dir_all(&inner_src).unwrap();

    let paths = Paths::resolve_from(None, &inner_src);
    // Falls back to the inner repo's own <cwd>/.claude, never the outer log.
    assert_ne!(paths.open_md, outer_claude.join("FOLLOWUPS.md"));
    assert_eq!(paths.open_md, inner_src.join(".claude").join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&outer);
}

#[test]
fn resolve_finds_log_at_git_root_before_stopping() {
    // The `.git` boundary stops *upward* search, but a log living at the repo
    // root (alongside `.git`) is still found.
    let repo = std::env::temp_dir().join("mpm_disc_gitroot");
    let _ = fs::remove_dir_all(&repo);
    fs::create_dir_all(repo.join(".git")).unwrap();
    let claude = repo.join(".claude");
    fs::create_dir_all(&claude).unwrap();
    fs::write(claude.join("FOLLOWUPS.md"), "# root\n").unwrap();
    let deep = repo.join("a").join("b");
    fs::create_dir_all(&deep).unwrap();

    let paths = Paths::resolve_from(None, &deep);
    assert_eq!(paths.open_md, claude.join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&repo);
}

#[test]
fn resolve_follows_linked_worktree_pointer_to_main_tree_log() {
    // A linked git worktree has a `.git` FILE (gitdir pointer), not a dir.
    // The repo boundary must not treat it as a foreign repo: the canonical
    // log lives in the MAIN working tree's `.claude`. Mirrors the real
    // incident where `mpm list` from an ecp worktree returned an empty log.
    let root = std::env::temp_dir().join("mpm_disc_wt_pointer");
    let _ = fs::remove_dir_all(&root);
    let main = root.join("main");
    let main_claude = main.join(".claude");
    fs::create_dir_all(main.join(".git").join("worktrees").join("wt")).unwrap();
    fs::create_dir_all(&main_claude).unwrap();
    fs::write(main_claude.join("FOLLOWUPS.md"), "# main log\n").unwrap();

    // Worktree placed OUTSIDE the main tree so plain upward walking cannot
    // find the log — only following the pointer can.
    let wt = root.join("wt");
    let wt_deep = wt.join("src");
    fs::create_dir_all(&wt_deep).unwrap();
    fs::write(
        wt.join(".git"),
        format!("gitdir: {}\n", main.join(".git").join("worktrees").join("wt").display()),
    )
    .unwrap();

    let paths = Paths::resolve_from(None, &wt_deep);
    assert_eq!(paths.open_md, main_claude.join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn resolve_worktree_with_no_log_anywhere_falls_back_to_main_claude() {
    // First-run `add` from a worktree must seed the log in the MAIN tree's
    // `.claude` (one log per repo), not split-brain into the worktree's own.
    let root = std::env::temp_dir().join("mpm_disc_wt_fallback");
    let _ = fs::remove_dir_all(&root);
    let main = root.join("main");
    fs::create_dir_all(main.join(".git").join("worktrees").join("wt")).unwrap();

    let wt = root.join("wt");
    fs::create_dir_all(&wt).unwrap();
    fs::write(
        wt.join(".git"),
        format!("gitdir: {}\n", main.join(".git").join("worktrees").join("wt").display()),
    )
    .unwrap();

    let paths = Paths::resolve_from(None, &wt);
    assert_eq!(paths.open_md, main.join(".claude").join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn resolve_follows_relative_gitdir_pointer() {
    // Portable checkouts may write a RELATIVE gitdir pointer; it resolves
    // against the worktree root.
    let root = std::env::temp_dir().join("mpm_disc_wt_relative");
    let _ = fs::remove_dir_all(&root);
    let main = root.join("main");
    let main_claude = main.join(".claude");
    fs::create_dir_all(main.join(".git").join("worktrees").join("wt")).unwrap();
    fs::create_dir_all(&main_claude).unwrap();
    fs::write(main_claude.join("FOLLOWUPS.md"), "# main log\n").unwrap();

    let wt = root.join("wt");
    fs::create_dir_all(&wt).unwrap();
    fs::write(wt.join(".git"), "gitdir: ../main/.git/worktrees/wt\n").unwrap();

    let paths = Paths::resolve_from(None, &wt);
    assert_eq!(paths.open_md, main_claude.join("FOLLOWUPS.md"));
    let _ = fs::remove_dir_all(&root);
}

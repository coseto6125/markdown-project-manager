//! `start` / `stop` lifecycle: an entry is marked in progress by the PR or branch
//! that is fixing it, and `done` later resolves it from that marker.

use clap::Parser;
use markdown_project_manager::cli::{Cli, Command};
use markdown_project_manager::index::Index;
use markdown_project_manager::model::{Edge, Entry, Location, Status, WorkRef};
use markdown_project_manager::store::{self, Paths};
use markdown_project_manager::{commands, parse};
use std::fs;

const OPEN_ID: &str = "FU-2026-05-23-021";
/// Open entry whose fields include a non-core key (`renumber-note`).
const EXTRA_ID: &str = "FU-2026-05-23-042";
const DONE_ID: &str = "FU-2026-05-22-001";
const BLOCKER_ID: &str = "FU-2026-05-23-048";
/// Appears in both fixture files: a duplicated id.
const DUPLICATE_ID: &str = "FU-2026-05-23-007";

fn seed(tag: &str) -> Paths {
    let dir = std::env::temp_dir().join(format!("mpm_test_in_progress_{tag}"));
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

fn entry(paths: &Paths, id: &str) -> Entry {
    store::load(paths)
        .unwrap()
        .entries
        .into_iter()
        .find(|e| e.id == id)
        .unwrap_or_else(|| panic!("entry {id} missing"))
}

fn start(paths: &Paths, id: &str, pr: Option<u32>, branch: Option<&str>) -> anyhow::Result<()> {
    commands::run(
        Command::Start {
            id: id.into(),
            pr,
            branch: branch.map(str::to_string),
        },
        paths,
    )
}

fn stop(paths: &Paths, id: &str) -> anyhow::Result<()> {
    commands::run(Command::Stop { id: id.into() }, paths)
}

fn done_inferred(paths: &Paths, id: &str) -> anyhow::Result<()> {
    commands::run(
        Command::Done {
            id: id.into(),
            pr: None,
            branch: None,
            commit: None,
            note: None,
        },
        paths,
    )
}

fn marker(e: &Entry) -> Option<&str> {
    parse::in_progress_value(e)
}

fn done_with_pr(paths: &Paths, id: &str, pr: u32) -> anyhow::Result<()> {
    commands::run(
        Command::Done {
            id: id.into(),
            pr: Some(pr),
            branch: None,
            commit: None,
            note: None,
        },
        paths,
    )
}

fn set(paths: &Paths, id: &str, field: &str, value: &str) {
    commands::run(
        Command::Set {
            id: id.into(),
            field: field.into(),
            value: value.into(),
            append: false,
        },
        paths,
    )
    .unwrap();
}

/// Insert `line` right after `id`'s heading in the Open markdown.
fn insert_after_heading(paths: &Paths, id: &str, line: &str) {
    let md = fs::read_to_string(&paths.open_md).unwrap();
    let heading = format!("### {id}  ·  surfaced in PR #366 (ci tier split)\n");
    let edited = md.replacen(&heading, &format!("{heading}{line}\n"), 1);
    assert_ne!(md, edited, "fixture heading changed; update the test");
    fs::write(&paths.open_md, edited).unwrap();
}

#[test]
fn test_start_with_pr_sets_in_progress_status_field_and_edge() {
    let p = seed("start_pr");
    start(&p, OPEN_ID, Some(912), None).unwrap();

    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::InProgress);
    assert_eq!(e.location, Location::Open);
    assert_eq!(marker(&e), Some("PR #912"));
    assert!(e.edges.contains(&Edge::InProgressIn(WorkRef::Pr(912))));
    assert!(fs::read_to_string(&p.open_md)
        .unwrap()
        .contains("- **in-progress**: PR #912"));
}

#[test]
fn test_start_branch_then_pr_replaces_the_marker() {
    let p = seed("start_upgrade");
    start(&p, OPEN_ID, None, Some("fix/x")).unwrap();
    assert_eq!(marker(&entry(&p, OPEN_ID)), Some("branch fix/x"));

    start(&p, OPEN_ID, Some(913), None).unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(marker(&e), Some("PR #913"));
    assert_eq!(e.extra.iter().filter(|(k, _)| k == "in-progress").count(), 1);
    assert!(!e
        .edges
        .iter()
        .any(|x| matches!(x, Edge::InProgressIn(WorkRef::Branch(_)))));
}

#[test]
fn test_index_build_after_start_pr_lists_entry_by_pr_and_status() {
    let p = seed("index");
    start(&p, OPEN_ID, Some(914), None).unwrap();
    let store = store::load(&p).unwrap();
    let idx = Index::build(&store);
    let hits = idx.by_pr.get(&914).cloned().unwrap_or_default();
    assert_eq!(hits.len(), 1);
    assert_eq!(store.entries[hits[0]].id, OPEN_ID);
    let in_progress = idx.by_status.get(&Status::InProgress).cloned().unwrap_or_default();
    assert_eq!(in_progress, hits);
}

#[test]
fn test_done_without_flags_after_start_pr_resolves_with_that_pr() {
    let p = seed("done_pr");
    start(&p, OPEN_ID, Some(915), None).unwrap();
    done_inferred(&p, OPEN_ID).unwrap();

    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Done);
    assert_eq!(e.location, Location::Done);
    assert!(e.edges.iter().any(|x| matches!(x, Edge::DoneInPr { pr: 915, .. })));
    assert_eq!(marker(&e), None, "a resolved entry keeps no in-progress marker");
    assert!(fs::read_to_string(&p.open_md)
        .unwrap()
        .contains(&format!("<!-- {OPEN_ID} → ✅ done in PR #915")));
}

#[test]
fn test_done_without_flags_after_start_branch_resolves_with_that_branch() {
    let p = seed("done_branch");
    start(&p, OPEN_ID, None, Some("fix/y")).unwrap();
    done_inferred(&p, OPEN_ID).unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Done);
    assert!(e
        .edges
        .iter()
        .any(|x| matches!(x, Edge::DoneInBranch { branch, .. } if branch == "fix/y")));
}

#[test]
fn test_done_without_flags_or_start_errors() {
    let p = seed("done_none");
    let err = done_inferred(&p, OPEN_ID).unwrap_err().to_string();
    assert!(err.contains("--pr") && err.contains("mpm start"), "{err}");
    assert_eq!(entry(&p, OPEN_ID).status, Status::Open);
}

#[test]
fn test_stop_after_start_returns_to_open_and_keeps_other_fields() {
    let p = seed("stop_open");
    start(&p, EXTRA_ID, Some(916), None).unwrap();
    commands::run(
        Command::Set {
            id: EXTRA_ID.into(),
            field: "later-note".into(),
            value: "kept".into(),
            append: false,
        },
        &p,
    )
    .unwrap();
    stop(&p, EXTRA_ID).unwrap();

    let e = entry(&p, EXTRA_ID);
    assert_eq!(e.status, Status::Open);
    assert_eq!(marker(&e), None);
    assert!(!e.edges.iter().any(|x| matches!(x, Edge::InProgressIn(_))));
    let md = fs::read_to_string(&p.open_md).unwrap();
    assert!(
        md.contains("- **later-note**: kept"),
        "a field after the removed marker still renders"
    );
    assert!(
        md.contains("- **renumber-note**:"),
        "a field before the removed marker still renders"
    );
    assert!(!md.contains("- **in-progress**: PR #916"));
}

#[test]
fn test_stop_on_blocked_entry_returns_to_blocked() {
    let p = seed("stop_blocked");
    commands::run(
        Command::Block {
            id: OPEN_ID.into(),
            on: BLOCKER_ID.into(),
        },
        &p,
    )
    .unwrap();
    start(&p, OPEN_ID, Some(917), None).unwrap();
    assert_eq!(
        entry(&p, OPEN_ID).status,
        Status::InProgress,
        "in progress outranks blocked"
    );

    stop(&p, OPEN_ID).unwrap();
    assert_eq!(entry(&p, OPEN_ID).status, Status::Blocked);
}

#[test]
fn test_block_and_unblock_while_in_progress_keep_in_progress() {
    let p = seed("block_during");
    start(&p, OPEN_ID, Some(918), None).unwrap();
    commands::run(
        Command::Block {
            id: OPEN_ID.into(),
            on: BLOCKER_ID.into(),
        },
        &p,
    )
    .unwrap();
    assert_eq!(entry(&p, OPEN_ID).status, Status::InProgress);
    commands::run(
        Command::Unblock {
            id: OPEN_ID.into(),
            on: None,
        },
        &p,
    )
    .unwrap();
    assert_eq!(entry(&p, OPEN_ID).status, Status::InProgress);
}

#[test]
fn test_stop_without_start_errors() {
    let p = seed("stop_none");
    let err = stop(&p, OPEN_ID).unwrap_err().to_string();
    assert!(err.contains("not in progress"), "{err}");
}

#[test]
fn test_start_on_resolved_entry_errors() {
    let p = seed("start_done");
    let err = start(&p, DONE_ID, Some(919), None).unwrap_err().to_string();
    assert!(err.contains("resolved"), "{err}");
    assert_eq!(marker(&entry(&p, DONE_ID)), None);
}

#[test]
fn test_wontfix_after_start_clears_the_marker() {
    let p = seed("wontfix");
    start(&p, OPEN_ID, Some(920), None).unwrap();
    commands::run(
        Command::Wontfix {
            id: OPEN_ID.into(),
            reason: "abandoned".into(),
        },
        &p,
    )
    .unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Wontfix);
    assert_eq!(marker(&e), None);
}

#[test]
fn test_load_hand_written_marker_imports_as_in_progress() {
    let p = seed("import");
    insert_after_heading(&p, OPEN_ID, "- **in-progress**: PR #921");

    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::InProgress);
    assert!(e.edges.contains(&Edge::InProgressIn(WorkRef::Pr(921))));
}

#[test]
fn test_set_malformed_marker_gives_in_progress_without_edge() {
    let p = seed("malformed");
    set(&p, OPEN_ID, "in-progress", "PR #abc");
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::InProgress);
    assert!(!e.edges.iter().any(|x| matches!(x, Edge::InProgressIn(_))));

    let err = done_inferred(&p, OPEN_ID).unwrap_err().to_string();
    assert!(
        err.contains("`PR #abc`") && err.contains("names no PR or branch"),
        "{err}"
    );
}

#[test]
fn test_load_marker_with_text_after_pr_number_keeps_the_pr() {
    let p = seed("pr_suffix");
    set(&p, OPEN_ID, "in-progress", "PR #922 (draft)");
    assert!(entry(&p, OPEN_ID).edges.contains(&Edge::InProgressIn(WorkRef::Pr(922))));
}

#[test]
fn test_start_with_blank_or_spaced_branch_errors() {
    for bad in ["", "   ", "fix x", "fix\nx"] {
        let p = seed("bad_branch");
        let err = start(&p, OPEN_ID, None, Some(bad)).unwrap_err().to_string();
        assert!(err.contains("--branch"), "{bad:?}: {err}");
        assert_eq!(marker(&entry(&p, OPEN_ID)), None, "{bad:?}");
    }
}

#[test]
fn test_stop_with_duplicate_markers_removes_all() {
    let p = seed("dup_marker");
    insert_after_heading(&p, OPEN_ID, "- **in-progress**: PR #1\n- **in-progress**: PR #2");
    assert_eq!(entry(&p, OPEN_ID).status, Status::InProgress);

    stop(&p, OPEN_ID).unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Open);
    assert_eq!(marker(&e), None);
    assert!(!fs::read_to_string(&p.open_md).unwrap().contains("- **in-progress**:"));
    assert!(fs::read_to_string(&p.open_md).unwrap().contains("- **scope**:"));
}

#[test]
fn test_start_on_resolved_entry_in_open_file_errors() {
    let p = seed("start_open_resolved");
    let md = fs::read_to_string(&p.open_md).unwrap();
    let heading = format!("### {OPEN_ID}  ·  surfaced in PR #366 (ci tier split)");
    let edited = md.replacen(&heading, &format!("{heading}  ·  ✅ done in PR #12"), 1);
    assert_ne!(md, edited, "fixture heading changed; update the test");
    fs::write(&p.open_md, edited).unwrap();
    assert_eq!(entry(&p, OPEN_ID).status, Status::Done);

    let err = start(&p, OPEN_ID, Some(923), None).unwrap_err().to_string();
    assert!(err.contains("resolved"), "{err}");
    assert_eq!(marker(&entry(&p, OPEN_ID)), None);
}

#[test]
fn test_load_marker_on_archived_entry_gives_no_edge_and_no_pr_hit() {
    // An mpm older than `start` resolves an entry and leaves its marker behind.
    let p = seed("archived_marker");
    set(&p, DONE_ID, "in-progress", "PR #924");
    let store = store::load(&p).unwrap();
    let e = store.entries.iter().find(|e| e.id == DONE_ID).unwrap();
    assert_eq!(e.status, Status::Done);
    assert!(!e.edges.iter().any(|x| matches!(x, Edge::InProgressIn(_))));
    assert!(!Index::build(&store).by_pr.contains_key(&924));
}

#[test]
fn test_done_with_explicit_pr_overrides_the_marker() {
    let p = seed("done_override");
    start(&p, OPEN_ID, Some(925), None).unwrap();
    done_with_pr(&p, OPEN_ID, 926).unwrap();
    let e = entry(&p, OPEN_ID);
    assert!(e.edges.iter().any(|x| matches!(x, Edge::DoneInPr { pr: 926, .. })));
    assert_eq!(marker(&e), None);
}

#[test]
fn test_supersede_after_start_clears_the_marker() {
    let p = seed("supersede");
    start(&p, OPEN_ID, Some(927), None).unwrap();
    commands::run(
        Command::Supersede {
            id: OPEN_ID.into(),
            by: BLOCKER_ID.into(),
        },
        &p,
    )
    .unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Superseded);
    assert_eq!(marker(&e), None);
}

#[test]
fn test_supersede_by_duplicated_id_errors_instead_of_a_silent_no_op() {
    let p = seed("supersede_dup");
    let err = commands::run(
        Command::Supersede {
            id: OPEN_ID.into(),
            by: DUPLICATE_ID.into(),
        },
        &p,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("ambiguous id"), "{err}");
}

#[test]
fn test_reopen_after_start_and_done_returns_to_open() {
    let p = seed("reopen");
    start(&p, OPEN_ID, Some(928), None).unwrap();
    done_inferred(&p, OPEN_ID).unwrap();
    commands::run(Command::Reopen { id: OPEN_ID.into() }, &p).unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Open);
    assert_eq!(marker(&e), None);
}

#[test]
fn test_index_build_with_branch_marker_adds_no_pr_hit() {
    let p = seed("branch_index");
    start(&p, OPEN_ID, None, Some("fix/929")).unwrap();
    let prs_of = |store: &markdown_project_manager::model::Store| -> Vec<u32> {
        let idx = Index::build(store);
        let mut prs: Vec<u32> = idx
            .by_pr
            .iter()
            .filter(|(_, hits)| hits.iter().any(|&i| store.entries[i].id == OPEN_ID))
            .map(|(pr, _)| *pr)
            .collect();
        prs.sort_unstable();
        prs
    };
    let before = prs_of(&store::load(&seed("branch_index_before")).unwrap());
    assert_eq!(prs_of(&store::load(&p).unwrap()), before);
}

#[test]
fn test_list_status_in_progress_and_blocked_include_a_blocked_started_entry() {
    let p = seed("list_filters");
    commands::run(
        Command::Block {
            id: OPEN_ID.into(),
            on: BLOCKER_ID.into(),
        },
        &p,
    )
    .unwrap();
    start(&p, OPEN_ID, Some(930), None).unwrap();
    let dir = p.open_md.parent().unwrap();
    for args in [&["list", "--status", "in-progress"][..], &["list", "--blocked"][..]] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_mpm"))
            .arg("--dir")
            .arg(dir)
            .args(args)
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains(OPEN_ID), "{args:?}");
    }
}

#[test]
fn test_set_on_resolved_entry_in_open_file_keeps_the_resolution() {
    let p = seed("open_resolved_set");
    let md = fs::read_to_string(&p.open_md).unwrap();
    let heading = format!("### {OPEN_ID}  ·  surfaced in PR #366 (ci tier split)");
    fs::write(
        &p.open_md,
        md.replacen(&heading, &format!("{heading}  ·  ✅ done in PR #12"), 1),
    )
    .unwrap();
    set(&p, OPEN_ID, "owner", "probe");

    // Re-import from the markdown alone: the cache must not mask a dropped heading segment.
    fs::remove_file(p.open_md.with_file_name(".followups.cache")).unwrap();
    let e = entry(&p, OPEN_ID);
    assert_eq!(e.status, Status::Done);
    assert!(e.edges.iter().any(|x| matches!(x, Edge::DoneInPr { pr: 12, .. })));
}

#[test]
fn test_cli_done_rejects_both_pr_and_branch() {
    assert!(Cli::try_parse_from(["mpm", "done", OPEN_ID, "--pr", "1", "--branch", "b"]).is_err());
    assert!(Cli::try_parse_from(["mpm", "done", OPEN_ID, "--branch", "b", "--commit", "abc"]).is_ok());
}

#[test]
fn test_cli_start_requires_exactly_one_of_pr_or_branch() {
    assert!(Cli::try_parse_from(["mpm", "start", OPEN_ID]).is_err());
    assert!(Cli::try_parse_from(["mpm", "start", OPEN_ID, "--pr", "1", "--branch", "b"]).is_err());
    assert!(Cli::try_parse_from(["mpm", "start", OPEN_ID, "--pr", "1"]).is_ok());
    assert!(Cli::try_parse_from(["mpm", "start", OPEN_ID, "--branch", "fix/z"]).is_ok());
}

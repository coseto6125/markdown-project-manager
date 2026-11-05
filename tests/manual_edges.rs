//! `block` / `link` edges live in the `blocked-on` / `links-to` fields, so a
//! re-import from markdown alone (after a hand edit invalidates the cache)
//! keeps them, and every edge derives from the markdown.

use markdown_project_manager::cli::Command;
use markdown_project_manager::commands;
use markdown_project_manager::model::{Edge, Entry, Status, Store, FIELDLESS_EDGES_VERSION};
use markdown_project_manager::render;
use markdown_project_manager::store::{self, Paths};
use std::fs;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

const A: &str = "FU-2026-05-23-021";
const B: &str = "FU-2026-05-23-042";
const C: &str = "FU-2026-05-29-001";

fn seed_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mpm_test_manual_edges_{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn seed(tag: &str) -> Paths {
    let dir = seed_dir(tag);
    fs::write(dir.join("FOLLOWUPS.md"), include_str!("fixtures/FOLLOWUPS.md")).unwrap();
    fs::write(
        dir.join("FOLLOWUPS_DONE.md"),
        include_str!("fixtures/FOLLOWUPS_DONE.md"),
    )
    .unwrap();
    Paths::resolve(Some(&dir))
}

fn find(store: Store, id: &str) -> Entry {
    store.entries.into_iter().find(|e| e.id == id).unwrap()
}

fn load(paths: &Paths, id: &str) -> Entry {
    find(store::load(paths).unwrap(), id)
}

/// Load from the markdown alone, as after a hand edit invalidates the cache.
fn reimport(paths: &Paths, id: &str) -> Entry {
    let _ = fs::remove_file(&paths.cache);
    load(paths, id)
}

fn run(paths: &Paths, cmd: Command) {
    commands::run(cmd, paths).unwrap();
}

fn block(paths: &Paths, id: &str, on: &str) {
    run(
        paths,
        Command::Block {
            id: id.into(),
            on: on.into(),
        },
    );
}

fn unblock(paths: &Paths, id: &str, on: Option<&str>) {
    run(
        paths,
        Command::Unblock {
            id: id.into(),
            on: on.map(Into::into),
        },
    );
}

fn link(paths: &Paths, from: &str, to: &str) {
    run(
        paths,
        Command::Link {
            from: from.into(),
            to: to.into(),
        },
    );
}

fn set(paths: &Paths, id: &str, field: &str, value: &str, append: bool) {
    run(
        paths,
        Command::Set {
            id: id.into(),
            field: field.into(),
            value: value.into(),
            append,
        },
    );
}

fn open_md(paths: &Paths) -> String {
    fs::read_to_string(&paths.open_md).unwrap()
}

/// The lines of `id`'s section in FOLLOWUPS.md, heading excluded.
fn section(paths: &Paths, id: &str) -> Vec<String> {
    open_md(paths)
        .lines()
        .skip_while(|l| !l.starts_with(&format!("### {id} ")))
        .skip(1)
        .take_while(|l| !l.starts_with("### ") && !l.starts_with("## "))
        .map(str::to_string)
        .collect()
}

/// Hand-edit FOLLOWUPS.md: add `line` right under `id`'s heading.
fn hand_add_line(paths: &Paths, id: &str, line: &str) {
    let md = open_md(paths);
    let heading = md.lines().find(|l| l.starts_with(&format!("### {id} "))).unwrap();
    fs::write(&paths.open_md, md.replacen(heading, &format!("{heading}\n{line}"), 1)).unwrap();
}

fn blocked_on(e: &Entry) -> Vec<&str> {
    e.edges
        .iter()
        .filter_map(|x| match x {
            Edge::BlockedOn(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

fn links_to(e: &Entry) -> Vec<&str> {
    e.edges
        .iter()
        .filter_map(|x| match x {
            Edge::LinksTo(t) => Some(t.as_str()),
            _ => None,
        })
        .collect()
}

/// Rewrite the cache as a pre-field mpm left it: schema 1, `edge` on entry
/// `id`, and no field recording it. The markdown is untouched, so the
/// cache stays fresh.
fn plant_cache_only_edge(paths: &Paths, id: &str, edge: Edge) {
    let mut s: Store = rmp_serde::from_slice(&fs::read(&paths.cache).unwrap()).unwrap();
    s.schema_version = FIELDLESS_EDGES_VERSION;
    let i = s.entries.iter().position(|e| e.id == id).unwrap();
    s.entries[i].edges.push(edge);
    fs::write(&paths.cache, rmp_serde::to_vec_named(&s).unwrap()).unwrap();
}

#[test]
fn test_link_then_reimport_keeps_the_edge() {
    let p = seed("link");
    link(&p, A, B);
    let e = reimport(&p, A);
    assert!(e.edges.contains(&Edge::LinksTo(B.into())));
    assert!(open_md(&p).contains(&format!("- **links-to**: [[{B}]]")));
}

#[test]
fn test_link_twice_writes_the_target_once() {
    let p = seed("link_twice");
    link(&p, A, B);
    link(&p, A, B);
    assert_eq!(open_md(&p).matches(&format!("[[{B}]]")).count(), 1);
}

#[test]
fn test_add_with_a_non_fu_id_is_refused() {
    let p = seed("add_bad_id");
    let refused = commands::run(
        Command::Add {
            id: Some("legacy-7".into()),
            category: "Parser & Schema".into(),
            scope: "an id the parser cannot read back".into(),
            why: None,
            next: None,
            size: None,
            owner: None,
            surfaced: None,
        },
        &p,
    );
    assert!(refused.is_err());
    assert!(!open_md(&p).contains("legacy-7"));
}

#[test]
fn test_block_then_reimport_stays_blocked() {
    let p = seed("block");
    block(&p, A, B);
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Blocked);
    assert_eq!(blocked_on(&e), vec![B]);
}

#[test]
fn test_block_on_an_id_the_body_already_names_survives_a_body_rewrite() {
    let p = seed("block_body");
    set(&p, A, "next-action", &format!("blocked: {B}"), false);
    block(&p, A, B);
    set(&p, A, "next-action", "go ahead", false);
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Blocked);
    assert_eq!(blocked_on(&e), vec![B]);
}

#[test]
fn test_link_to_an_id_the_body_already_names_survives_a_body_rewrite() {
    let p = seed("link_body");
    set(&p, A, "scope", &format!("see [[{B}]]"), false);
    link(&p, A, B);
    set(&p, A, "scope", "rewritten", false);
    assert_eq!(links_to(&reimport(&p, A)), vec![B]);
}

#[test]
fn test_set_append_on_blocked_on_gives_two_blocks() {
    let p = seed("append");
    set(&p, A, "blocked-on", B, true);
    set(&p, A, "blocked-on", C, true);
    assert!(open_md(&p).contains(&format!("- **blocked-on**: {B}; {C}")));
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Blocked);
    assert_eq!(blocked_on(&e), vec![B, C]);
}

#[test]
fn test_blocked_on_placeholder_word_is_not_a_block() {
    let p = seed("placeholder");
    set(&p, A, "blocked-on", "none", false);
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Open);
    assert!(blocked_on(&e).is_empty());
}

#[test]
fn test_unblock_one_of_two_keeps_the_other_text_and_place() {
    let p = seed("unblock_prose");
    block(&p, A, B);
    set(&p, A, "blocked-on", "waits on parser, then lexer", true);
    let before = section(&p, A);
    let pos = before.iter().position(|l| l.starts_with("- **blocked-on**:")).unwrap();
    unblock(&p, A, Some(B));
    let after = section(&p, A);
    assert_eq!(after[pos], "- **blocked-on**: waits on parser, then lexer");
    assert_eq!(after.len(), before.len());
    assert_eq!(reimport(&p, A).status, Status::Open);
}

#[test]
fn test_unblock_on_an_unlisted_id_leaves_the_field_text_alone() {
    let p = seed("unblock_miss");
    set(&p, A, "blocked-on", &format!("{B}, {C}"), false);
    unblock(&p, A, Some("FU-2026-05-23-048"));
    assert!(open_md(&p).contains(&format!("- **blocked-on**: {B}, {C}")));
}

#[test]
fn test_unblock_one_of_two_then_reimport_keeps_the_other() {
    let p = seed("unblock_two");
    block(&p, A, B);
    block(&p, A, C);
    assert!(open_md(&p).contains(&format!("- **blocked-on**: {B}; {C}")));
    unblock(&p, A, Some(B));
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Blocked);
    assert_eq!(blocked_on(&e), vec![C]);
}

#[test]
fn test_unblock_with_two_hand_written_lines_edits_the_matching_line() {
    let p = seed("unblock_lines");
    hand_add_line(&p, A, &format!("- **blocked-on**: {B}"));
    hand_add_line(&p, A, &format!("- **blocked-on**: {C}"));
    let _ = fs::remove_file(&p.cache);
    unblock(&p, A, Some(B));
    assert_eq!(blocked_on(&reimport(&p, A)), vec![C]);
    unblock(&p, A, Some(C));
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Open);
    assert!(blocked_on(&e).is_empty());
}

#[test]
fn test_unblock_of_a_blocked_on_wikilink_drops_its_link_too() {
    let p = seed("unblock_wikilink");
    set(&p, A, "blocked-on", &format!("[[{B}]]"), false);
    assert_eq!(links_to(&load(&p, A)), vec![B]);
    unblock(&p, A, Some(B));
    assert!(links_to(&load(&p, A)).is_empty());
    assert!(links_to(&reimport(&p, A)).is_empty());
}

#[test]
fn test_unblock_all_then_reimport_is_open_with_no_field() {
    let p = seed("unblock_all");
    block(&p, A, B);
    unblock(&p, A, None);
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Open);
    assert!(blocked_on(&e).is_empty());
    assert!(!open_md(&p).contains("- **blocked-on**:"));
}

#[test]
fn test_done_after_block_drops_the_blocked_on_field() {
    let p = seed("done_blocked");
    block(&p, A, B);
    run(
        &p,
        Command::Done {
            id: A.into(),
            pr: Some(7),
            branch: None,
            commit: None,
            note: None,
        },
    );
    assert!(!fs::read_to_string(&p.done_md).unwrap().contains("- **blocked-on**:"));
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Done);
    assert!(blocked_on(&e).is_empty());
}

#[test]
fn test_reopen_after_wontfix_of_a_blocked_entry_is_open() {
    let p = seed("wontfix_blocked");
    block(&p, A, B);
    run(
        &p,
        Command::Wontfix {
            id: A.into(),
            reason: "not worth it".into(),
        },
    );
    run(&p, Command::Reopen { id: A.into() });
    let e = reimport(&p, A);
    assert_eq!(e.status, Status::Open);
    assert!(blocked_on(&e).is_empty());
}

#[test]
fn test_set_that_removes_a_body_wikilink_drops_the_edge() {
    let p = seed("set_drops");
    set(&p, A, "scope", &format!("see [[{B}]]"), false);
    assert_eq!(links_to(&load(&p, A)), vec![B]);
    set(&p, A, "scope", "no link any more", false);
    assert!(links_to(&load(&p, A)).is_empty());
}

#[test]
fn test_set_links_to_empty_clears_the_link() {
    let p = seed("set_clears");
    link(&p, A, B);
    set(&p, A, "links-to", "", false);
    assert!(links_to(&load(&p, A)).is_empty());
    assert!(links_to(&reimport(&p, A)).is_empty());
}

#[test]
fn test_unclosed_bracket_before_a_wikilink_keeps_the_link() {
    let p = seed("unclosed");
    set(&p, A, "scope", &format!("a stray [[ then [[{B}]]"), false);
    assert_eq!(links_to(&reimport(&p, A)), vec![B]);
}

#[test]
fn test_cache_only_edges_migrate_into_fields_on_the_next_write() {
    let p = seed("legacy_cache");
    set(&p, C, "scope", "write the cache", false);
    plant_cache_only_edge(&p, A, Edge::LinksTo(B.into()));
    plant_cache_only_edge(&p, A, Edge::BlockedOn(C.into()));
    set(&p, C, "scope", "an unrelated write", false);
    let md = open_md(&p);
    assert_eq!(md.matches(&format!("- **links-to**: [[{B}]]")).count(), 1);
    assert_eq!(md.matches(&format!("- **blocked-on**: {C}")).count(), 1);
    let e = reimport(&p, A);
    assert_eq!(links_to(&e), vec![B]);
    assert_eq!(blocked_on(&e), vec![C]);
    assert_eq!(e.status, Status::Blocked);
}

#[test]
fn test_cache_only_edge_that_no_field_can_hold_is_not_written() {
    let p = seed("legacy_unholdable");
    set(&p, C, "scope", "write the cache", false);
    plant_cache_only_edge(&p, A, Edge::LinksTo("FU-zz and [[FU-b".into()));
    set(&p, C, "scope", "first write", false);
    let first = section(&p, A);
    set(&p, C, "scope", "second write", false);
    assert_eq!(section(&p, A), first);
    assert!(!first.iter().any(|l| l.starts_with("- **links-to**:")));
}

#[test]
fn test_cache_only_edge_shows_in_the_render_before_any_write() {
    let p = seed("legacy_drift");
    set(&p, C, "scope", "write the cache", false);
    plant_cache_only_edge(&p, A, Edge::LinksTo(B.into()));
    let (open_now, _) = render::render(&store::load(&p).unwrap());
    assert!(open_now.contains(&format!("- **links-to**: [[{B}]]")));
}

/// `source_mtimes` that mpm 0.1.4 stored in `fixtures/legacy-0.1.4/followups.cache`.
const LEGACY_MTIME: u64 = 1_793_858_952;

fn set_mtime(path: &Path, secs: u64) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(UNIX_EPOCH + Duration::from_secs(secs))
        .unwrap();
}

/// The log and cache mpm 0.1.4 wrote after `block aaa --on bbb`,
/// `link aaa ccc`, and a `set` that gave ccc the edge `FU-zz and [[FU-…bbb`.
fn seed_legacy_014(tag: &str) -> Paths {
    let dir = seed_dir(tag);
    fs::write(
        dir.join("FOLLOWUPS.md"),
        include_str!("fixtures/legacy-0.1.4/FOLLOWUPS.md"),
    )
    .unwrap();
    fs::write(
        dir.join("FOLLOWUPS_DONE.md"),
        include_str!("fixtures/legacy-0.1.4/FOLLOWUPS_DONE.md"),
    )
    .unwrap();
    fs::write(
        dir.join(".followups.cache"),
        include_bytes!("fixtures/legacy-0.1.4/followups.cache"),
    )
    .unwrap();
    set_mtime(&dir.join("FOLLOWUPS.md"), LEGACY_MTIME);
    set_mtime(&dir.join("FOLLOWUPS_DONE.md"), LEGACY_MTIME);
    Paths::resolve(Some(&dir))
}

#[test]
fn test_cache_written_by_0_1_4_migrates_its_edges_once() {
    const AAA: &str = "FU-2026-01-01-aaa";
    const BBB: &str = "FU-2026-01-01-bbb";
    const CCC: &str = "FU-2026-01-01-ccc";
    let p = seed_legacy_014("legacy_014");
    // The cache, not the markdown, is what holds aaa's block and link.
    let e = load(&p, AAA);
    assert_eq!(blocked_on(&e), vec![BBB]);
    assert_eq!(links_to(&e), vec![CCC]);
    set(&p, BBB, "scope", "first write", false);
    let first = section(&p, AAA);
    assert!(first.contains(&format!("- **blocked-on**: {BBB}")));
    assert!(first.contains(&format!("- **links-to**: [[{CCC}]]")));
    assert!(!section(&p, CCC).iter().any(|l| l.starts_with("- **links-to**:")));
    set(&p, BBB, "scope", "second write", false);
    assert_eq!(section(&p, AAA), first);
    let aaa = reimport(&p, AAA);
    assert_eq!(aaa.status, Status::Blocked);
    assert_eq!(blocked_on(&aaa), vec![BBB]);
    assert_eq!(links_to(&aaa), vec![CCC]);
    assert_eq!(links_to(&reimport(&p, CCC)), vec![BBB]);
}

#[test]
fn test_set_on_an_aliased_core_field_keeps_one_line() {
    let p = seed("alias");
    let md = open_md(&p);
    let line = section(&p, A)
        .into_iter()
        .find(|l| l.starts_with("- **scope**:"))
        .unwrap();
    let aliased = line.replacen("- **scope**:", "- **original-scope**:", 1);
    fs::write(&p.open_md, md.replacen(&line, &aliased, 1)).unwrap();
    let _ = fs::remove_file(&p.cache);
    set(&p, A, "scope", "rewritten", false);
    let scope_lines: Vec<String> = section(&p, A)
        .into_iter()
        .filter(|l| l.starts_with("- **scope**:") || l.starts_with("- **original-scope**:"))
        .collect();
    assert_eq!(scope_lines, vec!["- **original-scope**: rewritten".to_string()]);
}

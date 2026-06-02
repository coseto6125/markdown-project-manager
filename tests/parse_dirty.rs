//! Targeted tests for each real-corpus parsing anomaly, so a regression in the
//! parser surfaces as a named failure rather than a round-trip diff.

use markdown_project_manager::model::{Location, ResKind, Status};
use markdown_project_manager::parse;

fn parse_done(heading_and_body: &str) -> markdown_project_manager::model::Entry {
    let done = format!("# Follow-ups · Done archive\n\n{heading_and_body}\n");
    let store = parse::parse("# Follow-ups · Open\n", &done, (0, 0));
    store.entries.into_iter().next().expect("one entry")
}

#[test]
fn id_first_heading_with_trailing_resolution() {
    let e = parse_done(
        "### FU-2026-05-22-003  ·  surfaced in PR #345  ·  ✅ done in PR #372 (merged as fdfdd6ea, 2026-05-23)\n- **owner**: x",
    );
    assert_eq!(e.id, "FU-2026-05-22-003");
    assert_eq!(e.provenance, "surfaced in PR #345");
    assert_eq!(e.status, Status::Done);
    let r = e.resolution.unwrap();
    assert_eq!(r.kind, ResKind::Done);
    assert_eq!(r.pr, Some(372));
    assert_eq!(r.commit.as_deref(), Some("fdfdd6ea"));
}

#[test]
fn resolution_first_heading_extracts_id_from_middle() {
    let e = parse_done(
        "### ✅ done in PR #450 · FU-2026-05-25-005 · surfaced in FU-003 root-cause + PR #448 session\n- **owner**: x",
    );
    assert_eq!(e.id, "FU-2026-05-25-005");
    assert_eq!(e.provenance, "surfaced in FU-003 root-cause + PR #448 session");
    assert_eq!(e.resolution.unwrap().pr, Some(450));
}

#[test]
fn emoji_id_emdash_resolution_heading() {
    let e =
        parse_done("### ✅ FU-2026-05-25-003 — resolved 2026-05-25 (NOT a code bug: stale index)\n- **owner**: x");
    assert_eq!(e.id, "FU-2026-05-25-003");
    assert_eq!(e.status, Status::Done);
    assert!(e.resolution.unwrap().raw.starts_with("resolved 2026-05-25"));
}

#[test]
fn wontfix_heading() {
    let e = parse_done("### FU-2026-05-23-037  ·  surfaced in PR #384  ·  🚫 wontfix 2026-05-24\n- **owner**: x");
    assert_eq!(e.status, Status::Wontfix);
    let r = e.resolution.unwrap();
    assert_eq!(r.kind, ResKind::Wontfix);
}

#[test]
fn original_scope_alias_maps_to_scope_slot_but_renders_original_key() {
    let e = parse_done(
        "### FU-2026-05-22-001  ·  surfaced in PR #345  ·  ✅ done in PR #370\n- **original-scope**: the thing",
    );
    assert_eq!(e.core.scope.as_deref(), Some("the thing"));
    let (o, d) = markdown_project_manager::render::render(&parse::parse(
        "# Follow-ups · Open\n",
        &format!("# Follow-ups · Done archive\n\n{}", entry_md(&e)),
        (0, 0),
    ));
    let _ = o;
    assert!(
        d.contains("- **original-scope**: the thing"),
        "alias key spelling lost:\n{d}"
    );
}

fn entry_md(e: &markdown_project_manager::model::Entry) -> String {
    // Reconstruct minimal markdown for re-parse in the alias test.
    let mut s = format!("### {}  ·  {}", e.id, e.provenance);
    if let Some(r) = &e.resolution {
        s.push_str(&format!("  ·  {} {}", r.marker(), r.raw));
    }
    s.push('\n');
    if let Some(v) = &e.core.scope {
        s.push_str(&format!("- **original-scope**: {v}\n"));
    }
    s
}

#[test]
fn blocked_entry_stays_open_with_blocked_status() {
    let open = "# Follow-ups · Open\n\n## CLI / Commands\n\n### FU-2026-06-01-001  ·  surfaced in PR #999\n- **owner**: x\n- **next-action**: blocked: FU-2026-05-23-013 then proceed\n";
    let store = parse::parse(open, "# Follow-ups · Done archive\n", (0, 0));
    let e = &store.entries[0];
    assert_eq!(e.status, Status::Blocked);
    assert_eq!(e.location, Location::Open);
    assert!(e.edges.iter().any(
        |edge| matches!(edge, markdown_project_manager::model::Edge::BlockedOn(t) if t == "FU-2026-05-23-013")
    ));
}

#[test]
fn wikilinks_become_links_to_edges() {
    let open = "# Follow-ups · Open\n\n## Parser & Schema\n\n### FU-2026-06-01-002  ·  surfaced in PR #1\n- **links**: see [[FU-2026-05-26-002]] and [[FU-2026-05-23-013]]\n";
    let store = parse::parse(open, "# Follow-ups · Done archive\n", (0, 0));
    let e = &store.entries[0];
    let targets: Vec<_> = e
        .edges
        .iter()
        .filter_map(|edge| match edge {
            markdown_project_manager::model::Edge::LinksTo(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert!(targets.contains(&"FU-2026-05-26-002"));
    assert!(targets.contains(&"FU-2026-05-23-013"));
}

#[test]
fn in_flight_marker_is_not_a_terminal_resolution() {
    // The Cypher-engine `pushdown approach in-flight PR #426` stub must NOT be
    // parsed as a done-resolution. It lives as a floating note, not an entry,
    // so here we assert the resolution parser rejects the fragment.
    let e = parse_done("### FU-2026-06-01-003  ·  pushdown approach in-flight PR #426\n- **owner**: x");
    // Heading has no terminal resolution segment → done by virtue of Location
    // only, no Resolution struct.
    assert!(e.resolution.is_none(), "in-flight wrongly parsed as resolution");
}

//! Index + graph-traversal tests on the real corpus.

use markdown_project_manager::index::Index;
use markdown_project_manager::parse;

fn corpus() -> markdown_project_manager::model::Store {
    parse::parse(
        include_str!("fixtures/FOLLOWUPS.md"),
        include_str!("fixtures/FOLLOWUPS_DONE.md"),
        (0, 0),
    )
}

#[test]
fn carried_over_from_edge_is_indexed() {
    // FU-2026-05-23-048 carries over from FU-008 (legacy short id, dangling).
    let store = corpus();
    let idx = Index::build(&store);
    let i = store.entries.iter().position(|e| e.id == "FU-2026-05-23-048").unwrap();
    // FU-008 is not a real entry id → recorded as dangling, not a resolved edge.
    assert!(idx.dangling.iter().any(|(src, t)| *src == i && t == "FU-008"));
}

#[test]
fn wikilink_edges_resolve_to_real_entries_or_dangle() {
    let store = corpus();
    let idx = Index::build(&store);
    // FU-2026-05-29-001 links to FU-2026-05-26-002 (a real done entry).
    let src = store.entries.iter().position(|e| e.id == "FU-2026-05-29-001").unwrap();
    let target = store.entries.iter().position(|e| e.id == "FU-2026-05-26-002");
    if let Some(t) = target {
        assert!(
            idx.links_to.get(&src).map(|v| v.contains(&t)).unwrap_or(false),
            "links_to edge missing"
        );
        assert!(
            idx.linked_from.get(&t).map(|v| v.contains(&src)).unwrap_or(false),
            "reverse edge missing"
        );
    }
}

#[test]
fn by_pr_index_finds_entries_resolved_in_a_pr() {
    let store = corpus();
    let idx = Index::build(&store);
    // Many entries are done in PR #385 / #447 etc. (surfaced + done edges).
    assert!(
        idx.by_pr.contains_key(&498),
        "expected PR #498 in by_pr (surfaced-in edge)"
    );
}

#[test]
fn by_status_partitions_all_entries() {
    let store = corpus();
    let idx = Index::build(&store);
    let total: usize = idx.by_status.values().map(|v| v.len()).sum();
    assert_eq!(
        total,
        store.entries.len(),
        "every entry must land in exactly one status bucket"
    );
}

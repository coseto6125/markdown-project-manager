//! Entry → JSON projection for the `toon`/`json` output paths. Signal-dense:
//! only fields that are present are emitted, edges are flattened to their most
//! query-useful shape.

use crate::model::*;
use serde_json::{json, Map, Value};

/// One entry as a compact JSON object. `full` includes the body fields; the
/// list view omits them for a triage-friendly summary.
pub fn entry_json(entry: &Entry, full: bool) -> Value {
    let mut m = Map::new();
    m.insert("id".into(), json!(entry.id));
    m.insert("status".into(), json!(entry.status.as_str()));
    m.insert("category".into(), json!(entry.category));
    // `location` is omitted: it's a total function of status (done/wontfix/
    // superseded → archive, open/blocked → active), so emitting it wastes bytes.
    if let Some(sz) = &entry.core.size {
        m.insert("size".into(), json!(sz));
    }
    if !entry.provenance.is_empty() {
        m.insert("provenance".into(), json!(entry.provenance));
    }

    let edges = edges_json(entry);
    if !edges.is_empty() {
        m.insert("edges".into(), Value::Object(edges));
    }

    if full {
        let mut fields = Map::new();
        for (k, v) in ordered_fields(entry) {
            fields.insert(k, json!(v));
        }
        if !fields.is_empty() {
            m.insert("fields".into(), Value::Object(fields));
        }
        if let Some(r) = &entry.resolution {
            m.insert("resolution".into(), json!(r.raw));
        }
    } else if let Some(scope) = entry.core.scope.as_ref() {
        // List view: a one-line scope teaser.
        m.insert("scope".into(), json!(first_line(scope)));
    }

    Value::Object(m)
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

/// Fields in their original render order, keys with their original spelling.
fn ordered_fields(entry: &Entry) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for fref in &entry.field_order {
        match fref {
            FieldRef::Core(k) => {
                if let Some(Some(v)) = entry.core.slot(k) {
                    out.push((k.clone(), v.clone()));
                }
            }
            FieldRef::CoreAlias { key, slot } => {
                if let Some(Some(v)) = entry.core.slot(slot) {
                    out.push((key.clone(), v.clone()));
                }
            }
            FieldRef::Extra(i) => {
                if let Some((k, v)) = entry.extra.get(*i) {
                    out.push((k.clone(), v.clone()));
                }
            }
        }
    }
    out
}

/// Edges grouped by relation, ids/PRs only — the shape an agent traverses.
fn edges_json(entry: &Entry) -> Map<String, Value> {
    let mut m = Map::new();
    let mut blocked_on = Vec::new();
    let mut links_to = Vec::new();
    let mut superseded_by = None;
    let mut carried_from = None;
    let mut done_pr = None;
    let mut surfaced_pr = None;

    for e in &entry.edges {
        match e {
            Edge::BlockedOn(t) => blocked_on.push(t.clone()),
            Edge::LinksTo(t) => links_to.push(t.clone()),
            Edge::SupersededBy(t) => superseded_by = Some(t.clone()),
            Edge::CarriedOverFrom(t) => carried_from = Some(t.clone()),
            Edge::DoneInPr { pr, .. } => done_pr = Some(*pr),
            Edge::SurfacedInPr(pr) => surfaced_pr = Some(*pr),
            Edge::DoneInBranch { .. } | Edge::Wontfix { .. } => {}
        }
    }
    if !blocked_on.is_empty() {
        m.insert("blocked_on".into(), json!(blocked_on));
    }
    if !links_to.is_empty() {
        m.insert("links_to".into(), json!(links_to));
    }
    if let Some(t) = superseded_by {
        m.insert("superseded_by".into(), json!(t));
    }
    if let Some(t) = carried_from {
        m.insert("carried_from".into(), json!(t));
    }
    if let Some(pr) = done_pr {
        m.insert("done_pr".into(), json!(pr));
    }
    if let Some(pr) = surfaced_pr {
        m.insert("surfaced_pr".into(), json!(pr));
    }
    m
}

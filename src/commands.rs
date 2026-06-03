//! Command implementations. Every mutation follows the same path: load store →
//! mutate → `store::save` (which re-renders both markdown files + refreshes the
//! cache). Reads build the index and project entries to toon/json.

use crate::cli::Command;
use crate::index::Index;
use crate::model::*;
use crate::output::{emit, Format};
use crate::store::{self, Paths};
use crate::{render, view};
use anyhow::{bail, Result};
use serde_json::{json, Value};

pub fn run(cmd: Command, paths: &Paths) -> Result<()> {
    match cmd {
        Command::Add {
            category,
            scope,
            why,
            next,
            size,
            owner,
            surfaced,
        } => add(paths, category, scope, why, next, size, owner, surfaced),
        Command::Show { id, json } => show(paths, &id, json),
        Command::Stub { id } => stub(paths, &id),
        Command::List {
            status,
            category,
            size,
            blocked,
            pr,
            json,
        } => list(paths, status, category, size, blocked, pr, json),
        Command::Set {
            id,
            field,
            value,
            append,
        } => set(paths, &id, &field, value, append),
        Command::Move { id, category } => move_category(paths, &id, category),
        Command::Done {
            id,
            pr,
            branch,
            commit,
            note,
        } => done(paths, &id, pr, branch, commit, note),
        Command::Wontfix { id, reason } => wontfix(paths, &id, reason),
        Command::Supersede { id, by } => supersede(paths, &id, &by),
        Command::Block { id, on } => block(paths, &id, &on),
        Command::Unblock { id, on } => unblock(paths, &id, on.as_deref()),
        Command::Reopen { id } => reopen(paths, &id),
        Command::Link { from, to } => link(paths, &from, &to),
        Command::Graph {
            id,
            direction,
            depth,
            json,
        } => graph(paths, &id, &direction, depth, json),
        Command::Query { filters, json } => query(paths, &filters, json),
        Command::Render { check } => render_cmd(paths, check),
        Command::Import { dry_run } => import_cmd(paths, dry_run),
        Command::Validate { json } => validate(paths, json),
        Command::NextId => {
            // Hash ids carry no "next" sequence: mint one now (the entry it would
            // belong to is created later by `add`, which mints its own).
            println!("{}", crate::id::mint(now_nanos()));
            Ok(())
        }
        // Intercepted in main() before Paths resolution (install targets agent
        // hosts, not a follow-ups log).
        Command::Install { .. } => unreachable!("Install is handled in main"),
    }
}

fn fmt(json: bool) -> Format {
    Format::from_json_flag(json)
}

/// Find an entry index by id, erroring if absent or ambiguous.
fn find(store: &Store, id: &str) -> Result<usize> {
    let matches: Vec<usize> = store
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.id == id)
        .map(|(i, _)| i)
        .collect();
    match matches.as_slice() {
        [i] => Ok(*i),
        [] => bail!("no entry with id {id}"),
        many => bail!("ambiguous id {id}: {} entries share it", many.len()),
    }
}

/// Nanoseconds since the unix epoch — the entropy source for id minting.
fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
fn add(
    paths: &Paths,
    category: String,
    scope: String,
    why: Option<String>,
    next: Option<String>,
    size: Option<String>,
    owner: Option<String>,
    surfaced: Option<String>,
) -> Result<()> {
    let mut store = store::load(paths)?;
    let id = crate::id::mint(now_nanos());
    let provenance = surfaced.map(|s| format!("surfaced in {s}")).unwrap_or_default();

    let mut entry = Entry {
        id: id.clone(),
        location: Location::Open,
        category,
        status: Status::Open,
        provenance,
        core: CoreFields {
            owner: Some(owner.unwrap_or_else(|| "unassigned".into())),
            scope: Some(scope),
            why_deferred: why,
            next_action: next,
            size,
            links: None,
        },
        extra: Vec::new(),
        field_order: Vec::new(),
        edges: Vec::new(),
        resolution: None,
    };
    // Canonical field order for a fresh entry.
    for k in ["owner", "scope", "why-deferred", "next-action", "size", "links"] {
        if entry.core.slot(k).map(|s| s.is_some()).unwrap_or(false) {
            entry.field_order.push(FieldRef::Core(k.to_string()));
        }
    }
    rebuild_edges(&mut entry);
    store.entries.push(entry);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn show(paths: &Paths, id: &str, json: bool) -> Result<()> {
    let store = store::load(paths)?;
    let i = find(&store, id)?;
    println!("{}", emit(&view::entry_json(&store.entries[i], true), fmt(json))?);
    Ok(())
}

fn stub(paths: &Paths, id: &str) -> Result<()> {
    let store = store::load(paths)?;
    let i = find(&store, id)?;
    match store.entries[i].stub() {
        Some(s) => println!("{s}"),
        None => bail!("entry {id} is not resolved (no stub)"),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn list(
    paths: &Paths,
    status: Option<String>,
    category: Option<String>,
    size: Option<String>,
    blocked: bool,
    pr: Option<u32>,
    json: bool,
) -> Result<()> {
    let store = store::load(paths)?;
    let idx = Index::build(&store);

    let mut candidates: Vec<usize> = (0..store.entries.len()).collect();
    if let Some(s) = &status {
        let st = parse_status(s)?;
        candidates.retain(|&i| store.entries[i].status == st);
    }
    if let Some(c) = &category {
        candidates.retain(|&i| store.entries[i].category == *c);
    }
    if let Some(sz) = &size {
        let want = sz.to_uppercase();
        candidates.retain(|&i| size_bucket(store.entries[i].core.size.as_deref()) == want);
    }
    if blocked {
        candidates.retain(|&i| store.entries[i].status == Status::Blocked);
    }
    if let Some(p) = pr {
        let hits = idx.by_pr.get(&p).cloned().unwrap_or_default();
        candidates.retain(|i| hits.contains(i));
    }
    candidates.sort_by(|&a, &b| store.entries[a].id.cmp(&store.entries[b].id));

    let items: Vec<Value> = candidates
        .iter()
        .map(|&i| view::entry_json(&store.entries[i], false))
        .collect();
    println!(
        "{}",
        emit(&json!({ "count": items.len(), "entries": items }), fmt(json))?
    );
    Ok(())
}

fn set(paths: &Paths, id: &str, field: &str, value: String, append: bool) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    apply_field(&mut store.entries[i], field, value, append);
    rebuild_edges(&mut store.entries[i]);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

/// Set or append a field, preserving canonical-vs-alias spelling + order.
fn apply_field(entry: &mut Entry, key: &str, value: String, append: bool) {
    // Existing core slot?
    if CORE_KEYS.contains(&key) {
        let slot = entry.core.slot_mut(key).unwrap();
        *slot = Some(merge(slot.take(), value, append));
        if !entry
            .field_order
            .iter()
            .any(|f| matches!(f, FieldRef::Core(k) if k == key))
        {
            entry.field_order.push(FieldRef::Core(key.to_string()));
        }
        return;
    }
    // Existing extra key?
    if let Some(pos) = entry.extra.iter().position(|(k, _)| k == key) {
        let cur = std::mem::take(&mut entry.extra[pos].1);
        entry.extra[pos].1 = merge(Some(cur), value, append);
        return;
    }
    // New extra key.
    let idx = entry.extra.len();
    entry.extra.push((key.to_string(), value));
    entry.field_order.push(FieldRef::Extra(idx));
}

fn merge(existing: Option<String>, value: String, append: bool) -> String {
    match (existing, append) {
        (Some(cur), true) if !cur.is_empty() => format!("{cur}; {value}"),
        _ => value,
    }
}

fn move_category(paths: &Paths, id: &str, category: String) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    store.entries[i].category = category;
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn done(
    paths: &Paths,
    id: &str,
    pr: Option<u32>,
    branch: Option<String>,
    commit: Option<String>,
    note: Option<String>,
) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    let resolution = match (pr, branch) {
        (Some(pr), _) => Resolution::done_pr(pr, commit, note),
        (None, Some(b)) => Resolution::done_branch(b, commit, note),
        (None, None) => bail!("done requires --pr or --branch"),
    };
    let e = &mut store.entries[i];
    e.resolution = Some(resolution);
    e.status = Status::Done;
    e.location = Location::Done;
    // Drop blocked-on edges; the entry is resolved.
    e.edges.retain(|edge| !matches!(edge, Edge::BlockedOn(_)));
    rebuild_edges(e);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn wontfix(paths: &Paths, id: &str, reason: String) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    let e = &mut store.entries[i];
    e.resolution = Some(Resolution::wontfix(reason));
    e.status = Status::Wontfix;
    e.location = Location::Done;
    rebuild_edges(e);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn supersede(paths: &Paths, id: &str, by: &str) -> Result<()> {
    let mut store = store::load(paths)?;
    find(&store, by)?; // target must exist
    let i = find(&store, id)?;
    let e = &mut store.entries[i];
    e.resolution = Some(Resolution::superseded(by.to_string()));
    e.status = Status::Superseded;
    e.location = Location::Done;
    rebuild_edges(e);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn block(paths: &Paths, id: &str, on: &str) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    let e = &mut store.entries[i];
    if !e.edges.iter().any(|edge| matches!(edge, Edge::BlockedOn(t) if t == on)) {
        e.edges.push(Edge::BlockedOn(on.to_string()));
    }
    if e.status == Status::Open {
        e.status = Status::Blocked;
    }
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn unblock(paths: &Paths, id: &str, on: Option<&str>) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    let e = &mut store.entries[i];
    e.edges.retain(|edge| match (edge, on) {
        (Edge::BlockedOn(t), Some(target)) => t != target,
        (Edge::BlockedOn(_), None) => false,
        _ => true,
    });
    if e.status == Status::Blocked && !e.edges.iter().any(|edge| matches!(edge, Edge::BlockedOn(_))) {
        e.status = Status::Open;
    }
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn reopen(paths: &Paths, id: &str) -> Result<()> {
    let mut store = store::load(paths)?;
    let i = find(&store, id)?;
    let e = &mut store.entries[i];
    e.resolution = None;
    e.status = Status::Open;
    e.location = Location::Open;
    rebuild_edges(e);
    store::save(paths, &mut store)?;
    println!("{id}");
    Ok(())
}

fn link(paths: &Paths, from: &str, to: &str) -> Result<()> {
    let mut store = store::load(paths)?;
    find(&store, to)?;
    let i = find(&store, from)?;
    let e = &mut store.entries[i];
    if !e.edges.iter().any(|edge| matches!(edge, Edge::LinksTo(t) if t == to)) {
        e.edges.push(Edge::LinksTo(to.to_string()));
    }
    store::save(paths, &mut store)?;
    println!("{from}");
    Ok(())
}

fn graph(paths: &Paths, id: &str, direction: &str, depth: usize, json: bool) -> Result<()> {
    let store = store::load(paths)?;
    let idx = Index::build(&store);
    let start = find(&store, id)?;

    let down = matches!(direction, "down" | "both");
    let up = matches!(direction, "up" | "both");

    // BFS over blocked_on/links_to/superseded/carried edges.
    let mut visited = vec![false; store.entries.len()];
    let mut layers: Vec<Vec<Value>> = Vec::new();
    let mut frontier = vec![start];
    visited[start] = true;

    for _ in 0..depth {
        let mut next_frontier = Vec::new();
        let mut layer = Vec::new();
        for &node in &frontier {
            let mut neighbors: Vec<(&str, usize)> = Vec::new();
            if down {
                collect(&idx.blocked_on, node, "blocked_on", &mut neighbors);
                collect(&idx.links_to, node, "links_to", &mut neighbors);
                if let Some(&t) = idx.superseded_by.get(&node) {
                    neighbors.push(("superseded_by", t));
                }
                if let Some(&t) = idx.carried_from.get(&node) {
                    neighbors.push(("carried_from", t));
                }
            }
            if up {
                collect(&idx.blocks, node, "blocks", &mut neighbors);
                collect(&idx.linked_from, node, "linked_from", &mut neighbors);
                collect(&idx.supersedes, node, "supersedes", &mut neighbors);
            }
            for (rel, t) in neighbors {
                if !visited[t] {
                    visited[t] = true;
                    next_frontier.push(t);
                    layer.push(json!({
                        "from": store.entries[node].id,
                        "rel": rel,
                        "to": store.entries[t].id,
                        "status": store.entries[t].status.as_str(),
                    }));
                }
            }
        }
        if layer.is_empty() {
            break;
        }
        layers.push(layer);
        frontier = next_frontier;
    }

    let edges: Vec<Value> = layers.into_iter().flatten().collect();
    println!("{}", emit(&json!({ "root": id, "edges": edges }), fmt(json))?);
    Ok(())
}

fn collect(
    map: &std::collections::HashMap<usize, Vec<usize>>,
    node: usize,
    rel: &'static str,
    out: &mut Vec<(&'static str, usize)>,
) {
    if let Some(ts) = map.get(&node) {
        for &t in ts {
            out.push((rel, t));
        }
    }
}

fn query(paths: &Paths, filters: &[String], json: bool) -> Result<()> {
    let store = store::load(paths)?;
    let idx = Index::build(&store);
    let mut candidates: Vec<usize> = (0..store.entries.len()).collect();

    for f in filters {
        let (key, val) = f
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("filter must be key:value, got `{f}`"))?;
        match key {
            "status" => {
                let st = parse_status(val)?;
                candidates.retain(|&i| store.entries[i].status == st);
            }
            "category" => candidates.retain(|&i| store.entries[i].category == val),
            "size" => {
                let want = val.to_uppercase();
                candidates.retain(|&i| size_bucket(store.entries[i].core.size.as_deref()) == want);
            }
            "pr" => {
                let p: u32 = val.parse()?;
                let hits = idx.by_pr.get(&p).cloned().unwrap_or_default();
                candidates.retain(|i| hits.contains(i));
            }
            "blocked-on" => candidates.retain(|&i| {
                store.entries[i]
                    .edges
                    .iter()
                    .any(|e| matches!(e, Edge::BlockedOn(t) if t == val))
            }),
            "links-to" => candidates.retain(|&i| {
                store.entries[i]
                    .edges
                    .iter()
                    .any(|e| matches!(e, Edge::LinksTo(t) if t == val))
            }),
            "owner" => candidates.retain(|&i| {
                store.entries[i]
                    .core
                    .owner
                    .as_deref()
                    .map(|o| o.contains(val))
                    .unwrap_or(false)
            }),
            other => bail!("unknown filter key `{other}`"),
        }
    }
    candidates.sort_by(|&a, &b| store.entries[a].id.cmp(&store.entries[b].id));
    let items: Vec<Value> = candidates
        .iter()
        .map(|&i| view::entry_json(&store.entries[i], false))
        .collect();
    println!(
        "{}",
        emit(&json!({ "count": items.len(), "entries": items }), fmt(json))?
    );
    Ok(())
}

fn render_cmd(paths: &Paths, check: bool) -> Result<()> {
    let mut store = store::load(paths)?;
    if check {
        let (open_now, done_now) = render::render(&store);
        let open_disk = std::fs::read_to_string(&paths.open_md).unwrap_or_default();
        let done_disk = std::fs::read_to_string(&paths.done_md).unwrap_or_default();
        if open_now == open_disk && done_now == done_disk {
            println!("ok: markdown matches store");
            Ok(())
        } else {
            eprintln!("drift: markdown differs from store render");
            std::process::exit(1);
        }
    } else {
        store::save(paths, &mut store)?;
        println!("rendered");
        Ok(())
    }
}

fn import_cmd(paths: &Paths, dry_run: bool) -> Result<()> {
    let store = store::import(paths)?;
    if dry_run {
        let (open_now, _) = render::render(&store);
        let open_disk = std::fs::read_to_string(&paths.open_md).unwrap_or_default();
        let changed = open_now != open_disk;
        println!(
            "imported {} entries (open file would {})",
            store.entries.len(),
            if changed { "change on render" } else { "be unchanged" }
        );
    } else {
        println!("imported {} entries", store.entries.len());
    }
    Ok(())
}

fn validate(paths: &Paths, json: bool) -> Result<()> {
    let store = store::load(paths)?;
    let idx = Index::build(&store);

    let dangling: Vec<Value> = idx
        .dangling
        .iter()
        .map(|(i, target)| json!({ "from": store.entries[*i].id, "missing_target": target }))
        .collect();

    let mut seen = std::collections::HashMap::<&str, usize>::new();
    for e in &store.entries {
        *seen.entry(e.id.as_str()).or_insert(0) += 1;
    }
    let dup_ids: Vec<Value> = seen
        .iter()
        .filter(|(_, &c)| c > 1)
        .map(|(id, c)| json!({ "id": id, "count": c }))
        .collect();

    let report = json!({
        "dangling": dangling,
        "duplicate_ids": dup_ids,
        "ok": idx.dangling.is_empty() && dup_ids.is_empty(),
    });
    println!("{}", emit(&report, fmt(json))?);
    Ok(())
}

fn parse_status(s: &str) -> Result<Status> {
    Ok(match s {
        "open" => Status::Open,
        "done" => Status::Done,
        "wontfix" => Status::Wontfix,
        "blocked" => Status::Blocked,
        "superseded" => Status::Superseded,
        other => bail!("unknown status `{other}`"),
    })
}

/// Recompute body-derived edges (wikilinks, blocked-on, surfaced/carried) after
/// a field or resolution mutation, preserving manually-added link/block edges.
fn rebuild_edges(entry: &mut Entry) {
    crate::parse::rebuild_entry_edges(entry);
}

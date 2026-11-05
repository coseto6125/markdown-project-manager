//! Markdown → [`Store`]. The inverse of [`crate::render`].
//!
//! Parsing is line-oriented and tolerant of the real corpus's irregularities
//! (mixed stub markers, orphan comments, multi-line field bodies, double-`·`
//! DONE headings, legacy short ids). Anything not recognized as structure is
//! preserved verbatim so the round-trip stays faithful after the first
//! canonicalizing render.

use crate::model::*;

const MIDDOT: &str = "  ·  ";

/// Parse both markdown files into a single store. `open_md` / `done_md` are the
/// full file contents; mtimes are recorded for staleness detection.
pub fn parse(open_md: &str, done_md: &str, mtimes: (i64, i64)) -> Store {
    let mut store = Store {
        schema_version: SCHEMA_VERSION,
        source_mtimes: mtimes,
        ..Store::default()
    };
    parse_file(open_md, Location::Open, &mut store);
    parse_file(done_md, Location::Done, &mut store);
    store
}

/// Parse one file, appending entries + floating notes to `store`.
fn parse_file(md: &str, location: Location, store: &mut Store) {
    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;
    // The `## Resolved` section's stubs are a fully-derived view; we skip them
    // on import (the entries they point at live in the DONE file) to avoid
    // double-counting. Their per-category stubs are also skipped — status comes
    // from the entry's own resolution, not the stub.
    let mut category = String::new();
    let mut in_resolved_section = false;

    while i < lines.len() {
        let line = lines[i];

        if let Some(rest) = line.strip_prefix("## ") {
            in_resolved_section = rest.trim_start().starts_with("Resolved");
            if !in_resolved_section {
                category = rest.trim().to_string();
            }
            i += 1;
            continue;
        }

        if let Some(rest) = line.strip_prefix("### ") {
            let (entry, next) = parse_entry(&lines, i, rest, location, &category);
            store.entries.push(entry);
            i = next;
            continue;
        }

        // A `<!-- FU-… → … -->` stub is derived; skip. A non-FU `<!-- … -->`
        // comment inside a category (not the protocol header, not Resolved) is
        // a floating note to preserve.
        if !in_resolved_section
            && is_comment(line)
            && !category.is_empty()
            && stub_id(line).is_none()
            && !is_protocol_comment(line)
        {
            store.floating_notes.push(FloatingNote {
                category: category.clone(),
                text: line.to_string(),
            });
        }

        i += 1;
    }
}

fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("<!--")
}

/// The `<!-- Protocol: … -->` / `<!-- One-line stubs … -->` header comments,
/// which the renderer emits from a fixed template rather than storing.
fn is_protocol_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("<!-- Protocol:") || t.starts_with("<!-- One-line stubs") || t.starts_with("<!-- Resolved (")
}

/// Extract the FU id from a `<!-- FU-… → … -->` stub line, if it is one.
fn stub_id(line: &str) -> Option<&str> {
    let inner = line.trim_start().strip_prefix("<!--")?.trim_start();
    let id = inner.split_whitespace().next()?;
    (id.starts_with("FU-") && inner.contains('→')).then_some(id)
}

/// Parse a `### …` entry starting at `lines[start]` (`heading` = text after `### `).
/// Returns the entry and the index of the first line past it.
fn parse_entry(lines: &[&str], start: usize, heading: &str, location: Location, category: &str) -> (Entry, usize) {
    let (id, provenance, resolution_from_heading) = parse_heading(heading);

    let mut entry = Entry {
        id,
        location,
        category: category.to_string(),
        status: if location == Location::Done {
            Status::Done
        } else {
            Status::Open
        },
        provenance,
        core: CoreFields::default(),
        extra: Vec::new(),
        field_order: Vec::new(),
        edges: Vec::new(),
        resolution: resolution_from_heading,
    };

    // Collect field blocks until the next heading or EOF.
    let mut i = start + 1;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("### ") || line.starts_with("## ") {
            break;
        }
        if let Some((key, first_val)) = parse_field_start(line) {
            // Gather continuation lines (indented or blank-then-indented) until
            // the next `- **field**:` / heading / EOF.
            let mut value = first_val.to_string();
            let mut j = i + 1;
            while j < lines.len() {
                let l = lines[j];
                if parse_field_start(l).is_some() || l.starts_with("### ") || l.starts_with("## ") {
                    break;
                }
                value.push('\n');
                value.push_str(l);
                j += 1;
            }
            // Trim trailing blank continuation lines that actually belong to the
            // inter-block spacing, keeping internal structure intact.
            let value = value.trim_end_matches('\n').to_string();
            store_field(&mut entry, &key, value);
            i = j;
        } else {
            i += 1;
        }
    }

    derive_edges(&mut entry);
    derive_status(&mut entry);
    (entry, i)
}

/// Split a `### ` heading into (id, provenance, optional resolution).
/// Open: `FU-…  ·  surfaced in PR #379 …` → provenance is the suffix.
/// Done: `FU-…  ·  surfaced in PR #345  ·  ✅ done in PR #370 (merged as …)` →
/// provenance is the first `·` segment; the trailing `· ✅ done …` is parsed
/// into a resolution and re-derived on render.
fn parse_heading(heading: &str) -> (String, String, Option<Resolution>) {
    // The corpus uses several heading shapes:
    //   `FU-…  ·  surfaced …  ·  ✅ done in PR #N`   (id first, resolution last)
    //   `✅ done in PR #N · FU-… · surfaced …`        (resolution first, id middle)
    //   `✅ FU-… — resolved 2026-05-25 (note)`         (emoji+id, em-dash separator)
    // Normalize the single-space ` · ` separator to the canonical `  ·  `, then
    // split into segments and classify each by content rather than by position.
    let normalized = heading.replace(" · ", MIDDOT);
    let segments: Vec<&str> = normalized
        .split(MIDDOT)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let mut id = String::new();
    let mut provenance_segs: Vec<String> = Vec::new();
    let mut resolution = None;

    for seg in segments {
        if id.is_empty() {
            if let Some((found_id, leftover)) = split_id_segment(seg) {
                id = found_id;
                // A trailing `— resolved …` after the id in the same segment.
                if let Some(res) = leftover.and_then(parse_resolution_text) {
                    resolution = Some(res);
                }
                continue;
            }
        }
        if resolution.is_none() {
            if let Some(res) = parse_resolution_text(seg) {
                resolution = Some(res);
                continue;
            }
        }
        provenance_segs.push(seg.to_string());
    }

    (id, provenance_segs.join(MIDDOT), resolution)
}

/// If `seg` carries the `FU-…` id, return (id, optional leftover text after the
/// id within the same segment, e.g. `— resolved 2026-05-25 (note)`).
fn split_id_segment(seg: &str) -> Option<(String, Option<&str>)> {
    let idx = seg.find("FU-")?;
    let after = &seg[idx + 3..];
    let id_tail: String = after
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    let id = format!("FU-{id_tail}");
    let rest = seg[idx + 3 + id_tail.len()..].trim();
    let leftover = rest.trim_start_matches(['—', '-', ' ']).trim();
    Some((id, (!leftover.is_empty()).then_some(leftover)))
}

/// Parse a `done in …` / `wontfix: …` / `superseded by …` fragment into a
/// [`Resolution`]. Returns `None` if the text is not a terminal resolution
/// (e.g. `in-flight PR #426`, which is a non-terminal note).
fn parse_resolution_text(text: &str) -> Option<Resolution> {
    let raw = text.trim().trim_start_matches(['✅', '🚫', ' ']).trim().to_string();

    // Non-terminal in-flight markers are notes, not resolutions.
    if raw.contains("in-flight") || raw.contains("pushdown") {
        return None;
    }

    if let Some(rest) = raw.strip_prefix("wontfix") {
        let reason = rest.trim_start_matches([':', ' ']).trim();
        let mut r = Resolution::wontfix(reason.to_string());
        r.raw = raw.clone();
        r.wontfix_reason = (!reason.is_empty()).then(|| reason.to_string());
        return Some(r);
    }
    if let Some(rest) = raw.strip_prefix("superseded by") {
        let target = rest.split_whitespace().next().unwrap_or("").to_string();
        let mut r = Resolution::superseded(target.clone());
        r.raw = raw.clone();
        return Some(r);
    }

    // Done family: `done in …`, `resolved …`, `fixed in …`. Anything starting
    // with one of these verbs is a terminal done-resolution; we keep `raw`
    // verbatim for render and extract pr/commit/branch for indexing.
    let is_done = raw.starts_with("done") || raw.starts_with("resolved") || raw.starts_with("fixed");
    if !is_done {
        return None;
    }

    let done_body = raw
        .strip_prefix("done in ")
        .or_else(|| raw.strip_prefix("resolved in "))
        .or_else(|| raw.strip_prefix("fixed in "))
        .unwrap_or("");
    let (pr, commit, branch, note) = parse_done_body(done_body);
    Some(Resolution {
        kind: ResKind::Done,
        raw,
        pr,
        commit,
        branch,
        note,
        superseded_by: None,
        wontfix_reason: None,
        date: None,
    })
}

/// Parse `PR #370 (merged as dbb71278) — note` or `feat/x commit 7c13046c — note`.
fn parse_done_body(body: &str) -> (Option<u32>, Option<String>, Option<String>, Option<String>) {
    // Split off a trailing note after the em-dash.
    let (head, note) = match body.split_once(" — ") {
        Some((h, n)) => (h.trim(), Some(n.trim().to_string())),
        None => (body.trim(), None),
    };

    let mut pr = None;
    let mut commit = None;
    let mut branch = None;

    if let Some(after) = head.strip_prefix("PR #") {
        let num: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        pr = num.parse().ok();
        commit = extract_merged_as(head);
    } else {
        // `<branch> commit <sha>` form.
        if let Some((b, rest)) = head.split_once(" commit ") {
            branch = Some(b.trim().to_string());
            let sha: String = rest.trim().chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            commit = (!sha.is_empty()).then_some(sha);
        } else {
            branch = Some(head.to_string());
        }
    }
    (pr, commit, branch, note)
}

fn extract_merged_as(s: &str) -> Option<String> {
    let idx = s.find("merged as ")? + "merged as ".len();
    let sha: String = s[idx..].chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    (!sha.is_empty()).then_some(sha)
}

/// `- **key**: value` → (key, value). Returns `None` for non-field lines.
fn parse_field_start(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("- **")?;
    let (key, after) = rest.split_once("**:")?;
    Some((key.to_string(), after.strip_prefix(' ').unwrap_or(after).to_string()))
}

/// Canonical key for a non-canonical alias that maps onto a core slot.
fn canon_alias(key: &str) -> Option<&'static str> {
    match key {
        "original-scope" => Some("scope"),
        _ => None,
    }
}

/// Store a parsed field into the right slot, recording render order + spelling.
fn store_field(entry: &mut Entry, key: &str, value: String) {
    if CORE_KEYS.contains(&key) {
        if let Some(slot) = entry.core.slot_mut(key) {
            *slot = Some(value);
            entry.field_order.push(FieldRef::Core(key.to_string()));
            return;
        }
    }
    if let Some(canon) = canon_alias(key) {
        if let Some(slot) = entry.core.slot_mut(canon) {
            *slot = Some(value);
            entry.field_order.push(FieldRef::CoreAlias {
                key: key.to_string(),
                slot: canon.to_string(),
            });
            return;
        }
    }
    let idx = entry.extra.len();
    entry.extra.push((key.to_string(), value));
    entry.field_order.push(FieldRef::Extra(idx));
}

/// Recompute every edge and the status from the entry's fields after a
/// mutation. `block` and `link` write their fields, so the markdown carries
/// every edge.
pub fn rebuild_entry_edges(entry: &mut Entry) {
    entry.edges.clear();
    derive_edges(entry);
    derive_status(entry);
}

/// Upgrade an entry from a [`FIELDLESS_EDGES_VERSION`] cache. Such a cache
/// holds `block` / `link` edges that no field records. Write each one that a
/// field can carry into its field, then re-derive every edge and the status,
/// so the entry matches what its markdown will derive.
pub fn migrate_cache_only_edges(entry: &mut Entry) {
    let mut derived = entry.clone();
    rebuild_entry_edges(&mut derived);
    let missing: Vec<(&str, String)> = entry
        .edges
        .iter()
        .filter(|e| !derived.edges.contains(e))
        .filter_map(|e| match e {
            Edge::BlockedOn(t) if crate::id::is_valid(t) => Some((BLOCKED_ON_KEY, t.clone())),
            Edge::LinksTo(t) if crate::id::is_valid(t) => Some((LINKS_TO_KEY, format!("[[{t}]]"))),
            _ => None,
        })
        .collect();
    for (key, item) in missing {
        entry.set_field(key, item, true);
    }
    rebuild_entry_edges(entry);
}

/// The id in one item of a `blocked-on` / `links-to` field, `[[…]]` stripped.
/// `None` for an item that is not exactly one id, such as prose or `none`.
pub fn list_item_id(item: &str) -> Option<&str> {
    let id = item.trim().trim_start_matches("[[").trim_end_matches("]]").trim();
    crate::id::is_valid(id).then_some(id)
}

/// The ids in every `key` field, in field order.
pub fn list_field_ids<'e>(entry: &'e Entry, key: &str) -> Vec<&'e str> {
    entry
        .extra
        .iter()
        .filter(|(k, _)| k == key)
        .flat_map(|(_, v)| own_text(v).split(LIST_SEPARATORS))
        .filter_map(list_item_id)
        .collect()
}

/// `value` without each item that names `id` and the separator after it.
/// `None` when no item names `id`, so a caller leaves the text as it was.
pub fn remove_list_item(value: &str, id: &str) -> Option<String> {
    let items: Vec<&str> = value.split_inclusive(LIST_SEPARATORS).collect();
    let is_target = |item: &&str| list_item_id(item.trim_end_matches(LIST_SEPARATORS)) == Some(id);
    if !items.iter().any(is_target) {
        return None;
    }
    let last_removed = items.last().is_some_and(is_target);
    let kept: String = items.into_iter().filter(|item| !is_target(item)).collect();
    let kept = kept.trim();
    // Removing the last item leaves the separator that preceded it.
    let kept = match last_removed {
        true => kept.strip_suffix(LIST_SEPARATORS).unwrap_or(kept).trim_end(),
        false => kept,
    };
    Some(kept.to_string())
}

/// The `next-action` mention `derive_edges` reads as a block: the first
/// `blocked: ` (else `blocked-on `) that starts a word, when an `FU-` id
/// follows it. Returns the id and the byte range from the phrase through the
/// id and the `]` brackets that close it.
fn blocked_mention(text: &str) -> Option<(&str, std::ops::Range<usize>)> {
    ["blocked: ", "blocked-on "].into_iter().find_map(|needle| {
        let at = text
            .match_indices(needle)
            .map(|(i, _)| i)
            .find(|&i| !text[..i].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '-'))?;
        let after = at + needle.len();
        let id = fu_id_at(text, after)?;
        let opened = text[after..id.start].matches('[').count();
        let closed = text[id.end..].chars().take_while(|&c| c == ']').count().min(opened);
        Some((&text[id.clone()], at..id.end + closed))
    })
}

/// The byte range of the `FU-` id at `from`, after any `[` and spaces.
fn fu_id_at(text: &str, from: usize) -> Option<std::ops::Range<usize>> {
    let at = text.len() - text[from..].trim_start_matches(['[', ' ']).len();
    let len = text[at..]
        .find(|c: char| !crate::id::is_id_char(c))
        .unwrap_or(text.len() - at);
    crate::id::is_valid(&text[at..at + len]).then_some(at..at + len)
}

/// `text` without the mentions `derive_edges` reads as a block on `on` (on
/// any id when `None`), or `None` when it reads none. A removal also takes
/// the separator and spaces after the mention (else the separator and
/// spaces before it), and its line when the mention is the whole line: `see
/// blocked: FU-x, first` becomes `see first`. An id that a comma and another
/// id follow leaves the phrase to that id: `blocked: FU-x, FU-y` becomes
/// `blocked: FU-y`. The next mention then becomes the block, as a re-import reads it.
pub fn remove_blocked_mentions(text: &str, on: Option<&str>) -> Option<String> {
    let mut out = text.to_string();
    let mut removed = false;
    while let Some((id, range)) = blocked_mention(&out) {
        if on.is_some_and(|t| t != id) {
            break;
        }
        removed = true;
        let rest = &out[range.end..];
        let next = rest
            .trim_start_matches(' ')
            .trim_start_matches(LIST_SEPARATORS)
            .trim_start_matches(' ');
        let mut tail = rest.len() - next.len();
        let start = if tail == 0 {
            let before = out[..range.start].trim_end_matches(' ');
            before
                .strip_suffix(LIST_SEPARATORS)
                .unwrap_or(before)
                .trim_end_matches(' ')
                .len()
        } else if rest.trim_start_matches(' ').starts_with(',') && fu_id_at(next, 0).is_some() {
            range.start + out[range.start..].find(' ').map_or(0, |i| i + 1)
        } else {
            range.start
        };
        let whole_line = start == 0 || out[..start].ends_with('\n');
        if whole_line && out[range.end + tail..].starts_with('\n') {
            tail += 1;
        }
        out.replace_range(start..range.end + tail, "");
        // Only a removal that reached the end leaves whitespace to trim.
        if start == out.len() {
            out.truncate(out.trim_end().len());
        }
    }
    removed.then_some(out)
}

fn owned_ids(entry: &Entry, key: &str) -> Vec<String> {
    list_field_ids(entry, key).into_iter().map(str::to_string).collect()
}

fn push_edge(entry: &mut Entry, edge: Edge) {
    if !entry.edges.contains(&edge) {
        entry.edges.push(edge);
    }
}

/// Build outbound edges from heading provenance, body links, and resolution.
fn derive_edges(entry: &mut Entry) {
    // First `PR #N` anywhere in provenance is the surfacing PR. Provenance
    // phrasings vary ("surfaced in PR #379" vs "surfaced in audit (…, PR #498)").
    if let Some(pr) = first_pr(&entry.provenance) {
        entry.edges.push(Edge::SurfacedInPr(pr));
    }
    // `carried over from FU-…` from provenance.
    if let Some(target) = find_fu_after(&entry.provenance, "carried over from ") {
        entry.edges.push(Edge::CarriedOverFrom(target));
    }
    // `[[FU-id]]` wikilinks anywhere in the body.
    for (_, val) in field_values(entry) {
        for target in wikilinks(&val) {
            push_edge(entry, Edge::LinksTo(target));
        }
    }
    for target in owned_ids(entry, LINKS_TO_KEY) {
        push_edge(entry, Edge::LinksTo(target));
    }
    // `blocked: FU-…` mentions in next-action keep the entry in Open.
    if let Some(na) = &entry.core.next_action {
        if let Some((target, _)) = blocked_mention(own_text(na)) {
            entry.edges.push(Edge::BlockedOn(target.to_string()));
        }
    }
    for target in owned_ids(entry, BLOCKED_ON_KEY) {
        push_edge(entry, Edge::BlockedOn(target));
    }
    // A resolved entry is never in progress, even when an old mpm left the marker.
    if entry.resolution.is_none() && entry.location == Location::Open {
        if let Some(w) = work_ref(entry) {
            entry.edges.push(Edge::InProgressIn(w));
        }
    }
    // Resolution-derived edges.
    if let Some(r) = &entry.resolution {
        match r.kind {
            ResKind::Done => match (r.pr, &r.branch) {
                (Some(pr), _) => entry.edges.push(Edge::DoneInPr {
                    pr,
                    commit: r.commit.clone(),
                }),
                (None, Some(b)) => entry.edges.push(Edge::DoneInBranch {
                    branch: b.clone(),
                    commit: r.commit.clone(),
                }),
                _ => {}
            },
            ResKind::Wontfix => {
                if let Some(reason) = &r.wontfix_reason {
                    entry.edges.push(Edge::Wontfix { reason: reason.clone() });
                }
            }
            ResKind::Superseded => {
                if let Some(t) = &r.superseded_by {
                    entry.edges.push(Edge::SupersededBy(t.clone()));
                }
            }
        }
    }
}

/// Set `entry.status` from the whole entry: its resolution, then its location,
/// then an `in-progress` field, then a `BlockedOn` edge. Call it after the
/// edges are derived.
fn derive_status(entry: &mut Entry) {
    if let Some(r) = &entry.resolution {
        entry.status = match r.kind {
            ResKind::Done => Status::Done,
            ResKind::Wontfix => Status::Wontfix,
            ResKind::Superseded => Status::Superseded,
        };
    } else if entry.location == Location::Done {
        entry.status = Status::Done;
    } else {
        entry.status = if in_progress_value(entry).is_some() {
            Status::InProgress
        } else if entry.edges.iter().any(|e| matches!(e, Edge::BlockedOn(_))) {
            Status::Blocked
        } else {
            Status::Open
        };
    }
}

/// The `in-progress` field value without its trailing comments, if the entry
/// carries one.
pub fn in_progress_value(entry: &Entry) -> Option<&str> {
    entry
        .extra
        .iter()
        .find(|(k, _)| k == IN_PROGRESS_KEY)
        .map(|(_, v)| own_text(v))
}

/// The PR or branch the `in-progress` field names. `None` when the field is
/// absent or malformed.
pub fn work_ref(entry: &Entry) -> Option<WorkRef> {
    in_progress_value(entry).and_then(WorkRef::parse)
}

fn field_values(entry: &Entry) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for k in CORE_KEYS {
        if let Some(Some(v)) = entry.core.slot(k) {
            out.push((k.to_string(), v.clone()));
        }
    }
    for (k, v) in &entry.extra {
        out.push((k.clone(), v.clone()));
    }
    out
}

/// First `PR #N` token anywhere in `haystack`.
fn first_pr(haystack: &str) -> Option<u32> {
    let idx = haystack.find("PR #")? + "PR #".len();
    let num: String = haystack[idx..].chars().take_while(|c| c.is_ascii_digit()).collect();
    num.parse().ok()
}

fn find_fu_after(haystack: &str, needle: &str) -> Option<String> {
    let idx = haystack.find(needle)? + needle.len();
    fu_id_at(haystack, idx).map(|id| haystack[id].to_string())
}

/// Collect `[[FU-id]]` targets from text. Each `]]` pairs with the nearest
/// `[[` before it, so an unclosed `[[` does not swallow a later link.
fn wikilinks(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(close) = rest.find("]]") {
        if let Some(open) = rest[..close].rfind("[[") {
            let inner = &rest[open + 2..close];
            if crate::id::is_valid(inner) {
                out.push(inner.to_string());
            }
        }
        rest = &rest[close + 2..];
    }
    out
}

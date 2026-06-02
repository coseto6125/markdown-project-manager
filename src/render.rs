//! [`Store`] → markdown. The inverse of [`crate::parse`].
//!
//! Produces a *canonical* form: fixed protocol header, an id-sorted derived
//! `## Resolved` stub section, then category sections holding their open
//! entries + floating notes. The first import→render is a one-time normalizing
//! diff; every round-trip after that is byte-stable.

use crate::model::*;

const MIDDOT: &str = "  ·  ";

/// Category order for the Open file. Categories not listed render after these,
/// in first-seen order (preserves any future sections).
const CATEGORY_ORDER: &[&str] = &[
    "Parser & Schema",
    "Build / Engine / CI",
    "CLI / Commands",
    "Cypher engine",
    "Review / Impact",
    "Docs / Distribution",
];

/// Render both files. Returns `(open_md, done_md)`.
pub fn render(store: &Store) -> (String, String) {
    (render_open(store), render_done(store))
}

fn render_open(store: &Store) -> String {
    let mut out = String::new();
    out.push_str("# Follow-ups · Open\n\n");
    out.push_str(
        "<!-- Protocol: see `.claude/CLAUDE.md` \"Follow-ups protocol\" section.\n     \
         Resolved / wontfix entries: see `.claude/FOLLOWUPS_DONE.md`.\n     \
         This file is gitignored; the canonical path is the absolute\n     \
         `/home/enor/code-graph-nexus/.claude/FOLLOWUPS.md` regardless of\n     \
         which worktree the agent is editing from. -->\n\n",
    );

    // Derived `## Resolved` section: one canonical stub per resolved entry,
    // sorted by id.
    out.push_str("## Resolved → see FOLLOWUPS_DONE.md\n");
    out.push_str("<!-- One-line stubs for entries archived elsewhere. Sorted by FU id. -->\n\n");
    let mut resolved: Vec<&Entry> = store.entries.iter().filter(|e| e.location == Location::Done).collect();
    resolved.sort_by(|a, b| a.id.cmp(&b.id));
    for e in &resolved {
        if let Some(stub) = e.stub() {
            out.push_str(&stub);
            out.push('\n');
        }
    }

    // Category sections, in canonical order then first-seen for unknowns.
    for cat in ordered_categories(store, Location::Open) {
        out.push('\n');
        out.push_str(&format!("## {cat}\n\n"));

        // Floating notes for this category first (they precede entries in the
        // real file's section bodies).
        for note in store.floating_notes.iter().filter(|n| n.category == cat) {
            out.push_str(&note.text);
            out.push('\n');
        }

        let mut entries: Vec<&Entry> = store
            .entries
            .iter()
            .filter(|e| e.location == Location::Open && e.category == cat)
            .collect();
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        for e in &entries {
            out.push('\n');
            out.push_str(&render_entry(e, Location::Open));
        }
    }
    out
}

fn render_done(store: &Store) -> String {
    let mut out = String::new();
    out.push_str("# Follow-ups · Done archive\n\n");
    out.push_str(
        "<!-- Resolved (`✅ done in PR #N`) and wontfix (`🚫 wontfix: <reason>`)\n     \
         entries live here so the active `.claude/FOLLOWUPS.md` stays\n     \
         token-light. Append-only; never delete. -->\n",
    );

    let mut entries: Vec<&Entry> = store.entries.iter().filter(|e| e.location == Location::Done).collect();
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    for e in &entries {
        out.push('\n');
        out.push_str(&render_entry(e, Location::Done));
    }
    out
}

/// Categories present in `store` for `location`, canonical order first.
fn ordered_categories(store: &Store, location: Location) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let push = |c: &str, seen: &mut Vec<String>| {
        if !c.is_empty() && !seen.iter().any(|s| s == c) {
            seen.push(c.to_string());
        }
    };
    // Include categories that have either entries or floating notes.
    let has_content = |cat: &str| {
        store
            .entries
            .iter()
            .any(|e| e.location == location && e.category == cat)
            || (location == Location::Open && store.floating_notes.iter().any(|n| n.category == cat))
    };
    for c in CATEGORY_ORDER {
        if has_content(c) {
            push(c, &mut seen);
        }
    }
    for e in &store.entries {
        if e.location == location {
            push(&e.category, &mut seen);
        }
    }
    if location == Location::Open {
        for n in &store.floating_notes {
            push(&n.category, &mut seen);
        }
    }
    seen
}

/// Render one entry's heading + ordered field block.
fn render_entry(entry: &Entry, location: Location) -> String {
    let mut out = String::new();
    out.push_str("### ");
    out.push_str(&entry.id);
    if !entry.provenance.is_empty() {
        out.push_str(MIDDOT);
        out.push_str(&entry.provenance);
    }
    // DONE entries re-append the resolution segment to the heading.
    if location == Location::Done {
        if let Some(seg) = heading_resolution_segment(entry) {
            out.push_str(MIDDOT);
            out.push_str(&seg);
        }
    }
    out.push('\n');

    for fref in &entry.field_order {
        let (key, value) = match fref {
            FieldRef::Core(k) => match entry.core.slot(k).and_then(|s| s.as_ref()) {
                Some(v) => (k.as_str(), v.as_str()),
                None => continue,
            },
            FieldRef::CoreAlias { key, slot } => match entry.core.slot(slot).and_then(|s| s.as_ref()) {
                Some(v) => (key.as_str(), v.as_str()),
                None => continue,
            },
            FieldRef::Extra(i) => match entry.extra.get(*i) {
                Some((k, v)) => (k.as_str(), v.as_str()),
                None => continue,
            },
        };
        out.push_str(&format!("- **{key}**: {value}\n"));
    }
    out
}

/// The trailing `✅ done in PR #N …` segment for a DONE heading, rendered from
/// the resolution's verbatim `raw` text plus its marker glyph.
fn heading_resolution_segment(entry: &Entry) -> Option<String> {
    let r = entry.resolution.as_ref()?;
    let marker = r.marker();
    let sep = if marker.is_empty() { "" } else { " " };
    Some(format!("{marker}{sep}{}", r.raw))
}

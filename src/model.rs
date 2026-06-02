//! Data model for the follow-ups store.
//!
//! Source of truth is the two markdown files; the msgpack cache is a disposable
//! derived snapshot (guarded by mtime + `SCHEMA_VERSION`). `field_order` is the
//! load-bearing render-fidelity mechanism: it captures the exact key spelling and
//! ordering seen on import so a round-trip re-emits each entry's body verbatim.

use serde::{Deserialize, Serialize};

/// Bump on any breaking change to the cached structs; a mismatch forces a
/// transparent re-import from the markdown.
pub const SCHEMA_VERSION: u16 = 1;

/// The six known core fields. Everything else lands in [`Entry::extra`].
/// Spelling here is the canonical render spelling; aliases (e.g. `original-scope`
/// rendering into the `scope` slot) are tracked via [`FieldRef::CoreAlias`].
pub const CORE_KEYS: &[&str] = &["owner", "scope", "why-deferred", "next-action", "size", "links"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    pub entries: Vec<Entry>,
    /// Orphan `<!-- ... -->` comments that belong to a section but no entry
    /// (e.g. the duplicated `routes.rs 同款 6 sites` CJK note). Preserved verbatim.
    pub floating_notes: Vec<FloatingNote>,
    pub schema_version: u16,
    /// (open_md_mtime, done_md_mtime) at import time — detects manual edits.
    pub source_mtimes: (i64, i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// `FU-2026-05-23-042` or legacy `FU-001`. Opaque string.
    pub id: String,
    pub location: Location,
    /// The `## <Category>` section this entry renders under.
    pub category: String,
    pub status: Status,
    /// Raw heading suffix after `### {id}  ·  ` (provenance text). For DONE
    /// entries the trailing `· ✅ done in …` segment is stripped on import and
    /// re-derived from `resolution` on render.
    pub provenance: String,
    pub core: CoreFields,
    /// Ordered key→value for non-core fields, keeping the exact key spelling.
    pub extra: Vec<(String, String)>,
    /// Render order spanning core + extra — the fidelity key.
    pub field_order: Vec<FieldRef>,
    /// Outbound edges; reverse adjacency is built in the index, not stored.
    pub edges: Vec<Edge>,
    /// Populated for resolved entries; drives the canonical stub + DONE heading.
    pub resolution: Option<Resolution>,
}

/// Known core fields. `None` means the field is absent and is not rendered.
/// Values are stored verbatim, including embedded newlines (multi-line
/// `next-action` / `resolution` bodies) and leading-space continuation lines.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreFields {
    pub owner: Option<String>,
    pub scope: Option<String>,
    pub why_deferred: Option<String>,
    pub next_action: Option<String>,
    /// Free string: `S`, `M`, `L`, `unknown`, or `S (option A) / M (option B)`.
    pub size: Option<String>,
    pub links: Option<String>,
}

impl CoreFields {
    /// Map a canonical core key to its slot. Returns `None` for non-core keys.
    pub fn slot_mut(&mut self, canon_key: &str) -> Option<&mut Option<String>> {
        match canon_key {
            "owner" => Some(&mut self.owner),
            "scope" => Some(&mut self.scope),
            "why-deferred" => Some(&mut self.why_deferred),
            "next-action" => Some(&mut self.next_action),
            "size" => Some(&mut self.size),
            "links" => Some(&mut self.links),
            _ => None,
        }
    }

    pub fn slot(&self, canon_key: &str) -> Option<&Option<String>> {
        match canon_key {
            "owner" => Some(&self.owner),
            "scope" => Some(&self.scope),
            "why-deferred" => Some(&self.why_deferred),
            "next-action" => Some(&self.next_action),
            "size" => Some(&self.size),
            "links" => Some(&self.links),
            _ => None,
        }
    }
}

/// Drives render order while preserving the exact key spelling seen on import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldRef {
    /// A core field, rendered with its canonical key (one of [`CORE_KEYS`]).
    Core(String),
    /// A non-canonical key that maps onto a core slot — e.g. `original-scope`
    /// stored in the `scope` slot but rendered back as `original-scope`.
    CoreAlias { key: String, slot: String },
    /// Index into [`Entry::extra`].
    Extra(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Location {
    Open,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Status {
    Open,
    Done,
    Wontfix,
    /// Stays in the Open file; `next-action` notes the dependency.
    Blocked,
    Superseded,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Done => "done",
            Status::Wontfix => "wontfix",
            Status::Blocked => "blocked",
            Status::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    DoneInPr { pr: u32, commit: Option<String> },
    DoneInBranch { branch: String, commit: Option<String> },
    Wontfix { reason: String },
    BlockedOn(String),
    SupersededBy(String),
    Supersedes(String),
    LinksTo(String),
    CarriedOverFrom(String),
    SurfacedInPr(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatingNote {
    pub category: String,
    /// Verbatim `<!-- ... -->` line.
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResKind {
    Done,
    Wontfix,
    Superseded,
}

/// Produces the canonical one-line stub and the DONE heading suffix.
///
/// `raw` holds the verbatim resolution text (everything after the `✅`/`🚫`
/// marker), captured on import. It is the render source of truth so the wide
/// variety of resolution phrasings (`done in PR #N`, `resolved 2026-05-25 — …`,
/// `fixed in PR #N`, `done in working tree …`) round-trips faithfully. The
/// structured fields (`pr`/`commit`/…) are extracted from `raw` for indexing
/// and for synthesizing a resolution when a mutation creates one from scratch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub kind: ResKind,
    pub raw: String,
    pub pr: Option<u32>,
    pub commit: Option<String>,
    pub branch: Option<String>,
    /// Trailing `— <short note>` on the stub.
    pub note: Option<String>,
    /// `superseded by FU-…` target, when kind is Superseded.
    pub superseded_by: Option<String>,
    pub wontfix_reason: Option<String>,
    pub date: Option<String>,
}

impl Resolution {
    /// The marker glyph for this resolution kind.
    pub fn marker(&self) -> &'static str {
        match self.kind {
            ResKind::Done => "✅",
            ResKind::Wontfix => "🚫",
            ResKind::Superseded => "",
        }
    }

    /// Build a resolution from structured inputs (used by `done`/`wontfix`/
    /// `supersede` mutations), synthesizing the canonical `raw` text.
    pub fn done_pr(pr: u32, commit: Option<String>, note: Option<String>) -> Self {
        let mut raw = format!("done in PR #{pr}");
        if let Some(c) = &commit {
            raw.push_str(&format!(" (merged as {c})"));
        }
        if let Some(n) = &note {
            raw.push_str(&format!(" — {n}"));
        }
        Resolution {
            kind: ResKind::Done,
            raw,
            pr: Some(pr),
            commit,
            branch: None,
            note,
            superseded_by: None,
            wontfix_reason: None,
            date: None,
        }
    }

    pub fn done_branch(branch: String, commit: Option<String>, note: Option<String>) -> Self {
        let mut raw = format!("done in {branch}");
        if let Some(c) = &commit {
            raw.push_str(&format!(" commit {c}"));
        }
        if let Some(n) = &note {
            raw.push_str(&format!(" — {n}"));
        }
        Resolution {
            kind: ResKind::Done,
            raw,
            pr: None,
            commit,
            branch: Some(branch),
            note,
            superseded_by: None,
            wontfix_reason: None,
            date: None,
        }
    }

    pub fn wontfix(reason: String) -> Self {
        Resolution {
            kind: ResKind::Wontfix,
            raw: format!("wontfix: {reason}"),
            pr: None,
            commit: None,
            branch: None,
            note: None,
            superseded_by: None,
            wontfix_reason: Some(reason),
            date: None,
        }
    }

    pub fn superseded(target: String) -> Self {
        Resolution {
            kind: ResKind::Superseded,
            raw: format!("superseded by {target}"),
            pr: None,
            commit: None,
            branch: None,
            note: None,
            superseded_by: Some(target),
            wontfix_reason: None,
            date: None,
        }
    }
}

/// Derive a coarse size bucket from the free-form size string (first S/M/L token),
/// so `--size L` filtering works despite `"S (option A) / M (option B)"`.
pub fn size_bucket(size: Option<&str>) -> &'static str {
    match size.map(str::trim).and_then(|s| s.chars().next()) {
        Some('S') | Some('s') => "S",
        Some('M') | Some('m') => "M",
        Some('L') | Some('l') => "L",
        _ => "unknown",
    }
}

impl Entry {
    /// The canonical resolution stub line, e.g.
    /// `<!-- FU-… → ✅ done in PR #370 (merged as abc1234) — note -->`. Rendered
    /// from the verbatim `raw` resolution text so every phrasing round-trips.
    pub fn stub(&self) -> Option<String> {
        let r = self.resolution.as_ref()?;
        let marker = r.marker();
        let sep = if marker.is_empty() { "" } else { " " };
        Some(format!("<!-- {} → {marker}{sep}{} -->", self.id, r.raw))
    }
}

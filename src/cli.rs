//! Command-line surface. Mutations print the bare id (so an agent captures it
//! from stdout); reads default to toon, `--json` switches to machine output.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mpm",
    version,
    about = "Structured CRUD + graph queries over markdown follow-up logs"
)]
pub struct Cli {
    /// The `.claude` directory holding FOLLOWUPS.md / FOLLOWUPS_DONE.md.
    /// Omit to auto-discover the nearest one by walking up from the cwd.
    #[arg(long, global = true)]
    pub dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Mint a new follow-up; prints its id.
    Add {
        #[arg(long)]
        category: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        why: Option<String>,
        #[arg(long)]
        next: Option<String>,
        #[arg(long)]
        size: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        /// Provenance text, e.g. "PR #520" → "surfaced in PR #520".
        #[arg(long)]
        surfaced: Option<String>,
    },
    /// Print one entry (toon by default).
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Print just the canonical one-line resolution stub for an entry.
    Stub { id: String },
    /// List entries with optional filters.
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        size: Option<String>,
        #[arg(long)]
        blocked: bool,
        #[arg(long)]
        pr: Option<u32>,
        #[arg(long)]
        json: bool,
    },
    /// Set a single field on an entry. Use --append for links / multi-line bodies.
    Set {
        id: String,
        #[arg(long)]
        field: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        append: bool,
    },
    /// Move an entry to a different category.
    Move {
        id: String,
        #[arg(long)]
        category: String,
    },
    /// Resolve an entry as done (moves it to the Done archive, leaves a stub).
    Done {
        id: String,
        #[arg(long)]
        pr: Option<u32>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        note: Option<String>,
    },
    /// Resolve an entry as wontfix.
    Wontfix {
        id: String,
        #[arg(long)]
        reason: String,
    },
    /// Mark an entry superseded by another.
    Supersede {
        id: String,
        #[arg(long)]
        by: String,
    },
    /// Add a blocked-on edge (keeps the entry Open, status=blocked).
    Block {
        id: String,
        #[arg(long)]
        on: String,
    },
    /// Remove a blocked-on edge (specific target, or all).
    Unblock {
        id: String,
        #[arg(long)]
        on: Option<String>,
    },
    /// Pull a resolved entry back to Open.
    Reopen { id: String },
    /// Add a links-to edge between two entries.
    Link { from: String, to: String },
    /// Traverse relationships from an entry.
    Graph {
        id: String,
        #[arg(long, default_value = "both")]
        direction: String,
        #[arg(long, default_value_t = 2)]
        depth: usize,
        #[arg(long)]
        json: bool,
    },
    /// Flat key:value AND-filter query, e.g. `status:open size:L` / `pr:385`.
    Query {
        filters: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Re-render the markdown from the store; --check reports drift instead.
    Render {
        #[arg(long)]
        check: bool,
    },
    /// Parse the markdown into the store (migration / absorb manual edits).
    Import {
        #[arg(long)]
        dry_run: bool,
    },
    /// Report dangling edge targets, duplicate ids, and orphan stubs.
    Validate {
        #[arg(long)]
        json: bool,
    },
    /// Mint and print the next id for today without creating an entry.
    NextId,
}

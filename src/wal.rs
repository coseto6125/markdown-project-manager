//! Write-ahead intent log for concurrent mutations.
//!
//! Dozens of agents can mutate the same follow-ups log at once. A plain
//! load→mutate→save would lose updates (last writer wins on the whole file) and
//! serialize every full-file re-render. Instead each mutation appends a tiny JSON
//! *intent op* to `.followups.wal` (under a short append lock), then whoever can
//! grab the commit lock rotates the WAL aside, replays it into the store, renders
//! once, and drops the rotated batch (group commit). Reads replay the WAL in
//! memory so they see not-yet-committed ops.
//!
//! Two locks, never collapsed (see [[project_mpm_wal_concurrency]]):
//! - **append lock** (`wal_lock`): circles only "write one line", so dozens of
//!   appends serialize at sub-ms cost without blocking on a commit.
//! - **commit lock** (`commit_lock`): held by the group-committer while it
//!   rotates + replays + renders.
//!
//! The committer ROTATES (renames) the WAL aside under the append lock rather than
//! truncating it: appends that race the commit land in a fresh WAL and are never
//! swallowed (the truncate-the-whole-file bug). Crash safety: each line is a
//! self-contained JSON op; replay skips any unparseable/half line (writer killed
//! mid-append). The rotated batch is removed only after the render lands, and a
//! leftover rotated batch from a crashed commit is folded into the next one — so
//! committed ops are at-least-once, never lost.

use crate::commands;
use crate::model::Store;
use crate::store::{self, Paths};
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

/// A pending mutation. Mirrors the `apply_*` ops in `commands` one-to-one; the
/// committer/reader replays these through those same functions, so there is no
/// duplicate mutation logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op {
    /// `id` is minted at append time (hash ids are computed locally), so the id
    /// shown to the user is exactly the one that lands on commit.
    Add {
        id: String,
        category: String,
        scope: String,
        why: Option<String>,
        next: Option<String>,
        size: Option<String>,
        owner: Option<String>,
        surfaced: Option<String>,
    },
    Set {
        id: String,
        field: String,
        value: String,
        append: bool,
    },
    Move {
        id: String,
        category: String,
    },
    Done {
        id: String,
        pr: Option<u32>,
        branch: Option<String>,
        commit: Option<String>,
        note: Option<String>,
    },
    Wontfix {
        id: String,
        reason: String,
    },
    Supersede {
        id: String,
        by: String,
    },
    Block {
        id: String,
        on: String,
    },
    Unblock {
        id: String,
        on: Option<String>,
    },
    Reopen {
        id: String,
    },
    Link {
        from: String,
        to: String,
    },
}

/// Apply one op to the store via the shared `commands::apply_*`. Returns the
/// affected id. An op whose target no longer exists (concurrent resolve, etc.)
/// returns its error so the caller can skip it without aborting the batch.
pub fn apply_op(store: &mut Store, op: Op) -> Result<String> {
    match op {
        Op::Add {
            id,
            category,
            scope,
            why,
            next,
            size,
            owner,
            surfaced,
        } => commands::apply_add(store, id, category, scope, why, next, size, owner, surfaced),
        Op::Set {
            id,
            field,
            value,
            append,
        } => commands::apply_set(store, &id, &field, value, append),
        Op::Move { id, category } => commands::apply_move_category(store, &id, category),
        Op::Done {
            id,
            pr,
            branch,
            commit,
            note,
        } => commands::apply_done(store, &id, pr, branch, commit, note),
        Op::Wontfix { id, reason } => commands::apply_wontfix(store, &id, reason),
        Op::Supersede { id, by } => commands::apply_supersede(store, &id, &by),
        Op::Block { id, on } => commands::apply_block(store, &id, &on),
        Op::Unblock { id, on } => commands::apply_unblock(store, &id, on.as_deref()),
        Op::Reopen { id } => commands::apply_reopen(store, &id),
        Op::Link { from, to } => commands::apply_link(store, &from, &to),
    }
}

/// Append one op as a JSON line, holding the append lock for just the write.
pub fn append(paths: &Paths, op: &Op) -> Result<()> {
    let line = serde_json::to_string(op).context("encode WAL op")?;
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&paths.wal_lock)
        .with_context(|| format!("open {}", paths.wal_lock.display()))?;
    lock.lock_exclusive().context("lock WAL for append")?;
    let res = (|| {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&paths.wal)
            .with_context(|| format!("open {}", paths.wal.display()))?;
        f.write_all(line.as_bytes())
            .and_then(|_| f.write_all(b"\n"))
            .context("append WAL line")
    })();
    let _ = FileExt::unlock(&lock);
    res
}

/// Replay every parseable line of `wal_text` into `store`, applying ops through
/// `apply_op`. Unparseable lines (crash half-writes) and ops that error
/// (vanished target) are skipped. Returns how many ops applied successfully.
pub fn replay_into(store: &mut Store, wal_text: &str) -> usize {
    let mut applied = 0;
    for line in wal_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<Op>(line) {
            Ok(op) => {
                if apply_op(store, op).is_ok() {
                    applied += 1;
                }
            }
            Err(_) => continue, // half-written / corrupt line — skip
        }
    }
    applied
}

/// Load the committed store and replay any pending WAL ops in memory (no write),
/// so reads observe concurrent not-yet-committed mutations. This is the read entry
/// point; it also opportunistically group-commits if it can grab the commit lock.
pub fn read(paths: &Paths) -> Result<Store> {
    // Opportunistic drain so the WAL never grows unbounded under read-heavy load.
    let _ = try_group_commit(paths);
    let mut store = store::load(paths)?;
    // Replay a rotated batch a concurrent committer hasn't finished, then the live
    // WAL — so this read still observes both.
    let rotated = paths.wal.with_extension("wal.committing");
    if let Ok(text) = std::fs::read_to_string(&rotated) {
        replay_into(&mut store, &text);
    }
    if let Ok(text) = std::fs::read_to_string(&paths.wal) {
        replay_into(&mut store, &text);
    }
    Ok(store)
}

/// Try to become the group-committer: non-blocking grab of the commit lock, then
/// replay the entire WAL into the committed store, render both markdown files, and
/// truncate the WAL. If the lock is held by another process, returns Ok(false)
/// (someone else will drain it). Returns Ok(true) if this call committed.
pub fn try_group_commit(paths: &Paths) -> Result<bool> {
    // Cheap pre-check: skip grabbing the lock when nothing is pending.
    match std::fs::read_to_string(&paths.wal) {
        Ok(t) if !t.trim().is_empty() => {}
        _ => return Ok(false),
    }
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&paths.commit_lock)
        .with_context(|| format!("open {}", paths.commit_lock.display()))?;
    if lock.try_lock_exclusive().is_err() {
        return Ok(false); // another committer is draining it
    }
    let res = drain_locked(paths);
    let _ = FileExt::unlock(&lock);
    res
}

/// The write entry point: append the op, then opportunistically group-commit.
pub fn submit(paths: &Paths, op: Op) -> Result<()> {
    append(paths, &op)?;
    let _ = try_group_commit(paths);
    Ok(())
}

/// Run a whole-store maintenance operation (migrate-ids, render) under the commit
/// lock so it cannot race a group commit's `store::save` (last-writer-wins file
/// clobber). Drains any pending WAL into markdown FIRST so the maintenance op sees
/// committed state, then runs `f` while still holding the lock. Blocks for the
/// lock (these are rare, human-driven commands — correctness over latency).
pub fn with_commit_lock<T>(paths: &Paths, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&paths.commit_lock)
        .with_context(|| format!("open {}", paths.commit_lock.display()))?;
    lock.lock_exclusive().context("lock for maintenance op")?;
    let res = (|| {
        // Fold pending ops into markdown before the maintenance op reads it. We
        // hold the commit lock, so inline the drain rather than recursing into
        // try_group_commit (which would deadlock trying to re-lock).
        drain_locked(paths)?;
        f()
    })();
    let _ = FileExt::unlock(&lock);
    res
}

/// Replay+commit the WAL assuming the commit lock is ALREADY held. Shared by
/// `try_group_commit` and `with_commit_lock`. Returns whether anything committed.
///
/// Rotates the WAL aside under the append lock (so appends racing the commit land
/// in a fresh WAL, never swallowed), replays the rotated batch, renders, and drops
/// it only after the render lands (at-least-once on crash). A leftover rotated
/// batch from a crashed prior commit is folded into this one.
fn drain_locked(paths: &Paths) -> Result<bool> {
    let rotated = paths.wal.with_extension("wal.committing");
    {
        let alock = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&paths.wal_lock)
            .with_context(|| format!("open {}", paths.wal_lock.display()))?;
        alock.lock_exclusive().context("lock WAL for rotate")?;
        let rotate_res = (|| {
            if rotated.exists() {
                if let (Ok(mut prev), Ok(cur)) = (std::fs::read(&rotated), std::fs::read(&paths.wal)) {
                    prev.extend_from_slice(&cur);
                    std::fs::write(&rotated, &prev)?;
                    std::fs::write(&paths.wal, b"")?;
                    return Ok::<bool, anyhow::Error>(true);
                }
            }
            match std::fs::rename(&paths.wal, &rotated) {
                Ok(_) => Ok(true),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
                Err(e) => Err(e.into()),
            }
        })();
        let _ = FileExt::unlock(&alock);
        if !rotate_res? {
            return Ok(false);
        }
    }
    let text = std::fs::read_to_string(&rotated).unwrap_or_default();
    if text.trim().is_empty() {
        let _ = std::fs::remove_file(&rotated);
        return Ok(false);
    }
    let mut store = store::load(paths)?;
    replay_into(&mut store, &text);
    store::save(paths, &mut store)?;
    std::fs::remove_file(&rotated).with_context(|| format!("remove {}", rotated.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(tag: &str) -> Paths {
        let dir = std::env::temp_dir().join(format!("mpm_wal_{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("FOLLOWUPS.md"), "# Follow-ups\n\n## Open\n").unwrap();
        std::fs::write(dir.join("FOLLOWUPS_DONE.md"), "# Done\n\n## Resolved\n").unwrap();
        Paths::resolve(Some(&dir))
    }

    fn add_op(id: &str, scope: &str) -> Op {
        Op::Add {
            id: id.into(),
            category: "Test".into(),
            scope: scope.into(),
            why: None,
            next: None,
            size: None,
            owner: None,
            surfaced: None,
        }
    }

    #[test]
    fn add_replayed_twice_is_idempotent() {
        // Models the crash-refold double-apply: the same Add op replayed twice
        // must yield ONE entry, not a duplicate (which would make `show` ambiguous).
        let p = paths("idem_add");
        let mut store = store::load(&p).unwrap();
        let line = serde_json::to_string(&add_op("FU-dup", "once")).unwrap();
        let doubled = format!("{line}\n{line}\n");
        replay_into(&mut store, &doubled);
        assert_eq!(store.entries.iter().filter(|e| e.id == "FU-dup").count(), 1);
        let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
    }

    #[test]
    fn submit_then_read_sees_the_op() {
        let p = paths("submit_read");
        wal_submit_and_clear_dir(&p, "single");
    }

    fn wal_submit_and_clear_dir(p: &Paths, _t: &str) {
        submit(p, add_op("FU-x", "hello")).unwrap();
        let store = read(p).unwrap();
        assert!(store.entries.iter().any(|e| e.id == "FU-x"));
        let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
    }

    #[test]
    fn replay_skips_corrupt_lines() {
        let p = paths("corrupt");
        let mut store = store::load(&p).unwrap();
        let wal = "{not json}\n".to_string()
            + &serde_json::to_string(&add_op("FU-ok", "good")).unwrap()
            + "\n{\"Add\":{\"id\""; // trailing half-line
        let applied = replay_into(&mut store, &wal);
        assert_eq!(applied, 1, "only the one valid op applies");
        assert!(store.entries.iter().any(|e| e.id == "FU-ok"));
        let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
    }

    #[test]
    fn group_commit_drains_wal_into_markdown() {
        let p = paths("commit");
        append(&p, &add_op("FU-1", "one")).unwrap();
        append(&p, &add_op("FU-2", "two")).unwrap();
        let committed = try_group_commit(&p).unwrap();
        assert!(committed);
        // WAL drained (rotated away → absent, or present-but-empty) and the
        // rotated batch cleaned up; entries now in committed markdown.
        let wal_empty = std::fs::read_to_string(&p.wal)
            .map(|t| t.trim().is_empty())
            .unwrap_or(true);
        assert!(wal_empty, "WAL should be drained after commit");
        assert!(
            !p.wal.with_extension("wal.committing").exists(),
            "rotated batch should be removed"
        );
        let md = std::fs::read_to_string(&p.open_md).unwrap();
        assert!(md.contains("FU-1") && md.contains("FU-2"));
        let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
    }

    #[test]
    fn many_appends_then_one_commit_keeps_all() {
        // The lost-update regression: N ops all survive one group commit.
        let p = paths("manyappend");
        for i in 0..50 {
            append(&p, &add_op(&format!("FU-{i:02}"), "s")).unwrap();
        }
        try_group_commit(&p).unwrap();
        let store = store::load(&p).unwrap();
        assert_eq!(store.entries.len(), 50, "every appended op must survive");
        let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
    }
}

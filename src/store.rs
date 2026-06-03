//! Load/save the store. The markdown files are the source of truth; the msgpack
//! cache is a disposable snapshot guarded by `(mtime, schema_version)`. If the
//! cache is missing, version-mismatched, or older than either markdown file, the
//! store is transparently re-imported from markdown.

use crate::model::{Store, SCHEMA_VERSION};
use crate::{parse, render};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Fixed canonical paths for the follow-ups log (see `.claude/CLAUDE.md`).
pub struct Paths {
    pub open_md: PathBuf,
    pub done_md: PathBuf,
    pub cache: PathBuf,
}

impl Paths {
    /// Resolve from an explicit base dir (the `.claude` directory), or the
    /// canonical default when `base` is `None`.
    pub fn resolve(base: Option<&Path>) -> Self {
        let dir = base
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("/home/enor/code-graph-nexus/.claude"));
        Paths {
            open_md: dir.join("FOLLOWUPS.md"),
            done_md: dir.join("FOLLOWUPS_DONE.md"),
            cache: dir.join(".followups.cache"),
        }
    }
}

/// File mtime in nanoseconds since the epoch. Nanosecond granularity (vs whole
/// seconds) is what lets the cache guard notice a manual edit that lands in the
/// same wall-clock second as the prior `mpm` write — otherwise that edit would
/// be silently masked by a stale cache, breaking the markdown-is-source-of-truth
/// contract.
fn mtime(p: &Path) -> i64 {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

/// Write `bytes` to `path` atomically: write a sibling `.tmp` then rename over
/// the target, so a mid-write fault can never leave a half-written markdown file.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Load the store, using the cache when fresh, else re-importing from markdown.
pub fn load(paths: &Paths) -> Result<Store> {
    let cur = (mtime(&paths.open_md), mtime(&paths.done_md));
    if let Ok(bytes) = fs::read(&paths.cache) {
        if let Ok(store) = rmp_serde::from_slice::<Store>(&bytes) {
            if store.schema_version == SCHEMA_VERSION && store.source_mtimes == cur {
                return Ok(store);
            }
        }
    }
    import(paths)
}

/// Parse both markdown files into a fresh store and write the cache.
pub fn import(paths: &Paths) -> Result<Store> {
    let open = fs::read_to_string(&paths.open_md).with_context(|| format!("read {}", paths.open_md.display()))?;
    let done = fs::read_to_string(&paths.done_md).with_context(|| format!("read {}", paths.done_md.display()))?;
    let store = parse::parse(&open, &done, (mtime(&paths.open_md), mtime(&paths.done_md)));
    write_cache(paths, &store)?;
    Ok(store)
}

fn write_cache(paths: &Paths, store: &Store) -> Result<()> {
    let bytes = rmp_serde::to_vec_named(store).context("encode cache")?;
    write_atomic(&paths.cache, &bytes)
}

/// Persist a mutated store: re-render both markdown files, then refresh the
/// cache with the new mtimes. This is the single write path for every mutation.
pub fn save(paths: &Paths, store: &mut Store) -> Result<()> {
    let (open_md, done_md) = render::render(store);
    // Done first: if a fault strikes between the two renames, a resolved entry
    // is duplicated (archive + still-open) — recoverable — rather than vanishing.
    write_atomic(&paths.done_md, done_md.as_bytes())?;
    write_atomic(&paths.open_md, open_md.as_bytes())?;
    store.source_mtimes = (mtime(&paths.open_md), mtime(&paths.done_md));
    write_cache(paths, store)?;
    Ok(())
}

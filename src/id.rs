//! Follow-up id minting. Ids are `FU-<date>-<hex12>` where the suffix is derived
//! from a high-entropy source (creation nanos + pid + a per-process counter), NOT
//! a daily sequence number. Content is never hashed, so two agents filing the
//! same scope still get distinct ids.
//!
//! Why hash-style over sequential `FU-<date>-NNN`: under dozens of concurrent
//! `mpm add` calls a sequence number is a contention point — minting it without
//! collisions forces serialization against the committed store + the WAL. A
//! locally-derived suffix lets every agent mint its own id with zero coordination
//! (see [[project_mpm_wal_concurrency]]). Ordering is recovered from each entry's
//! creation timestamp, not from the id.

use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Mix three entropy words into one 48-bit value (splitmix64 finalizer), so the
/// 12-hex suffix is uniform even when inputs are near-sequential nanos.
fn mix48(nanos: u128, pid: u32, counter: u64) -> u64 {
    let mut x = (nanos as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((pid as u64).wrapping_mul(0xD1B5_4A32_D192_ED03))
        .wrapping_add(counter.wrapping_mul(0xFF51_AFD7_ED55_8CCD));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    x & 0x0000_FFFF_FFFF_FFFF // 48 bits → 12 hex
}

/// `FU-<date>-<hex12>` for the given date (`YYYY-MM-DD`), using `nanos`/`pid` plus
/// a fresh per-process counter tick. Pure given its inputs — the live mint wraps
/// it with the real clock/pid.
pub fn mint_with(date: &str, nanos: u128, pid: u32) -> String {
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("FU-{date}-{:012x}", mix48(nanos, pid, counter))
}

/// Live mint: today's date + the current creation nanos + this process's pid.
pub fn mint(nanos: u128) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    mint_with(&date, nanos, std::process::id())
}

/// True for a legacy sequential id (`FU-2026-05-23-042` or short `FU-001`) — used
/// by `migrate-ids` only for reporting; migration re-mints every entry regardless.
pub fn is_sequential(id: &str) -> bool {
    id.rsplit_once('-')
        .map(|(_, tail)| !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_with_has_fu_date_hex12_shape() {
        let id = mint_with("2026-06-03", 1_700_000_000_000_000_000, 42);
        let suffix = id.strip_prefix("FU-2026-06-03-").expect("prefix");
        assert_eq!(suffix.len(), 12);
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn successive_mints_same_inputs_differ_via_counter() {
        // Same nanos + pid, but the per-process counter must keep them distinct —
        // this is what protects two adds within the same nanosecond tick.
        let a = mint_with("2026-06-03", 123, 7);
        let b = mint_with("2026-06-03", 123, 7);
        assert_ne!(a, b);
    }

    #[test]
    fn distinct_nanos_give_distinct_ids() {
        let a = mint_with("2026-06-03", 1000, 7);
        let b = mint_with("2026-06-03", 2000, 7);
        assert_ne!(a, b);
    }

    #[test]
    fn no_collisions_across_a_burst() {
        // Simulate a burst of adds in one process (counter varies, nanos static).
        use std::collections::HashSet;
        let ids: HashSet<String> = (0..10_000).map(|_| mint_with("2026-06-03", 555, 1)).collect();
        assert_eq!(ids.len(), 10_000, "burst produced a collision");
    }

    #[test]
    fn is_sequential_classifies_legacy_and_hash() {
        assert!(is_sequential("FU-2026-05-23-042"));
        assert!(is_sequential("FU-001"));
        assert!(!is_sequential("FU-2026-06-03-a1b2c3d4e5f6"));
    }
}

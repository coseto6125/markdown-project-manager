//! Concurrency regression: dozens of threads submitting mutations to ONE log must
//! lose nothing. This is the failure the WAL exists to prevent (a plain
//! load→mutate→save loses updates under concurrency). Uses temp small files +
//! bounded thread count — never the shared corpus.

use markdown_project_manager::store::{self, Paths};
use markdown_project_manager::wal::{self, Op};
use std::sync::Arc;
use std::thread;

fn seed(tag: &str) -> Paths {
    let dir = std::env::temp_dir().join(format!("mpm_conc_{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("FOLLOWUPS.md"), "# Follow-ups\n\n## Open\n").unwrap();
    std::fs::write(dir.join("FOLLOWUPS_DONE.md"), "# Done\n\n## Resolved\n").unwrap();
    Paths::resolve(Some(&dir))
}

fn add_op(id: &str) -> Op {
    Op::Add {
        id: id.into(),
        category: "Test".into(),
        scope: "concurrent add".into(),
        why: None,
        next: None,
        size: None,
        owner: None,
        surfaced: None,
    }
}

#[test]
fn concurrent_adds_lose_nothing() {
    let p = Arc::new(seed("adds"));
    const THREADS: usize = 24;
    const PER: usize = 20;

    let handles: Vec<_> = (0..THREADS)
        .map(|t| {
            let p = Arc::clone(&p);
            thread::spawn(move || {
                for i in 0..PER {
                    // Distinct id per (thread, i) so we assert exact survival.
                    wal::submit(&p, add_op(&format!("FU-{t:02}-{i:02}"))).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }

    // Drain anything still pending, then the committed store must hold every op.
    while wal::try_group_commit(&p).unwrap() {}
    let store = store::load(&p).unwrap();
    assert_eq!(
        store.entries.len(),
        THREADS * PER,
        "every concurrent add must survive — got {} of {}",
        store.entries.len(),
        THREADS * PER
    );
    let _ = std::fs::remove_dir_all(p.wal.parent().unwrap());
}

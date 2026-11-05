//! `mpm migrate-ids` — re-mint every entry's id as a hash id and rewrite every
//! reference to it, including ids embedded in free text. A follow-up id is not
//! just a primary key: it is referenced by other entries' edges, by
//! `resolution.superseded_by`, and informally inside scope / provenance / notes
//! (`[[FU-…]]` wikilinks and bare `FU-…`). Renaming an id without rewriting those
//! would orphan edges and break the graph, so migration rewrites them in one pass
//! over the whole store.
//!
//! `--dry-run` (the default) reports the id mapping and every textual replacement
//! without touching anything; `--commit` writes.

use crate::id;
use crate::model::*;
use std::collections::HashMap;

/// One old→new id rename.
pub struct Rename {
    pub old: String,
    pub new: String,
}

/// Re-mint a fresh hash id for every entry. `nanos` seeds the mint; callers pass
/// a monotonically increasing value so a single migration run yields distinct ids
/// even for entries created in the same wall-clock instant.
pub fn build_mapping(store: &Store, mut nanos: u128) -> Vec<Rename> {
    store
        .entries
        .iter()
        .map(|e| {
            nanos += 1;
            Rename {
                old: e.id.clone(),
                new: id::mint(nanos),
            }
        })
        .collect()
}

/// Replace every `FU-<token>` occurrence in `s` whose id is a key in `map`,
/// returning the rewritten string and the replacement count. Ids absent from the
/// map (e.g. references to already-deleted entries) are left untouched.
pub fn replace_ids_in_text(s: &str, map: &HashMap<&str, &str>) -> (String, usize) {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        // Require a word boundary before `FU-` so `MYFU-001` is not mistaken for
        // the id `FU-001` (no look-behind would corrupt the preceding word).
        let prev_is_ident = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'-');
        if !prev_is_ident && bytes[i] == b'F' && s[i..].starts_with("FU-") {
            // Consume the id token: FU- followed by [A-Za-z0-9-].
            let start = i;
            let mut j = i + 3;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-') {
                j += 1;
            }
            let token = &s[start..j];
            match map.get(token) {
                Some(new) => {
                    out.push_str(new);
                    count += 1;
                }
                None => out.push_str(token),
            }
            i = j;
        } else {
            // Push one UTF-8 char (bytes may be multibyte; advance by char len).
            let ch = s[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    (out, count)
}

/// Apply the renames in-place across all structured fields and free text. Returns
/// the total number of textual references rewritten (excludes the entry-id
/// rewrites themselves).
pub fn apply(store: &mut Store, renames: &[Rename]) -> usize {
    let map: HashMap<&str, &str> = renames.iter().map(|r| (r.old.as_str(), r.new.as_str())).collect();
    let mut refs = 0;

    fn bump(opt: &mut Option<String>, map: &HashMap<&str, &str>) -> usize {
        match opt {
            Some(v) => {
                let (new, n) = replace_ids_in_text(v, map);
                *v = new;
                n
            }
            None => 0,
        }
    }

    for e in &mut store.entries {
        // Free-text fields.
        refs += bump(&mut e.core.owner, &map);
        refs += bump(&mut e.core.scope, &map);
        refs += bump(&mut e.core.why_deferred, &map);
        refs += bump(&mut e.core.next_action, &map);
        refs += bump(&mut e.core.size, &map);
        refs += bump(&mut e.core.links, &map);
        let (prov, n) = replace_ids_in_text(&e.provenance, &map);
        e.provenance = prov;
        refs += n;
        for (_k, v) in &mut e.extra {
            let (nv, n) = replace_ids_in_text(v, &map);
            *v = nv;
            refs += n;
        }

        // Structured edge targets.
        for edge in &mut e.edges {
            match edge {
                Edge::BlockedOn(t)
                | Edge::SupersededBy(t)
                | Edge::Supersedes(t)
                | Edge::LinksTo(t)
                | Edge::CarriedOverFrom(t) => {
                    if let Some(new) = map.get(t.as_str()) {
                        *t = (*new).to_string();
                        refs += 1;
                    }
                }
                _ => {}
            }
        }

        // Resolution: structured target + free-text raw/note.
        if let Some(res) = &mut e.resolution {
            if let Some(sb) = &mut res.superseded_by {
                if let Some(new) = map.get(sb.as_str()) {
                    *sb = (*new).to_string();
                    refs += 1;
                }
            }
            let (raw, n) = replace_ids_in_text(&res.raw, &map);
            res.raw = raw;
            refs += n;
            refs += bump(&mut res.note, &map);
            refs += bump(&mut res.wontfix_reason, &map);
        }
    }

    for note in &mut store.floating_notes {
        let (t, n) = replace_ids_in_text(&note.text, &map);
        note.text = t;
        refs += n;
    }

    // Finally the entry ids themselves (after text scans, so an entry's own id in
    // its own free text was already mapped above).
    for e in &mut store.entries {
        if let Some(new) = map.get(e.id.as_str()) {
            e.id = (*new).to_string();
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map<'a>(pairs: &'a [(&'a str, &'a str)]) -> HashMap<&'a str, &'a str> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn replaces_bare_and_wikilink_ids_but_not_unmapped() {
        let m = map(&[("FU-001", "FU-2026-06-03-aaaa")]);
        let (out, n) = replace_ids_in_text("see [[FU-001]] and FU-001, but not FU-999", &m);
        assert_eq!(out, "see [[FU-2026-06-03-aaaa]] and FU-2026-06-03-aaaa, but not FU-999");
        assert_eq!(n, 2);
    }

    #[test]
    fn does_not_replace_substring_in_longer_token() {
        // FU-0011 must not be matched by an FU-001 key (token boundary).
        let m = map(&[("FU-001", "FU-NEW")]);
        let (out, n) = replace_ids_in_text("FU-0011", &m);
        assert_eq!(out, "FU-0011");
        assert_eq!(n, 0);
    }

    #[test]
    fn handles_multibyte_text_around_ids() {
        let m = map(&[("FU-1", "FU-X")]);
        let (out, n) = replace_ids_in_text("延後：FU-1 之類的", &m);
        assert_eq!(out, "延後：FU-X 之類的");
        assert_eq!(n, 1);
    }

    #[test]
    fn does_not_match_fu_inside_a_larger_word() {
        // `MYFU-001` is not a reference to `FU-001`; a missing look-behind would
        // corrupt it into `MYFU-NEW`.
        let m = map(&[("FU-001", "FU-NEW")]);
        let (out, n) = replace_ids_in_text("MYFU-001 and x-FU-001", &m);
        assert_eq!(out, "MYFU-001 and x-FU-001");
        assert_eq!(n, 0);
        // But a real boundary (space/start/punct that isn't ident) still matches.
        let (out2, n2) = replace_ids_in_text("(FU-001)", &m);
        assert_eq!(out2, "(FU-NEW)");
        assert_eq!(n2, 1);
    }

    #[test]
    fn rewrites_id_inside_wontfix_reason() {
        let mut store = Store::default();
        let mut e = entry("FU-A");
        e.resolution = Some(Resolution::wontfix("overlaps with FU-B, skip".into()));
        store.entries.push(e);
        store.entries.push(entry("FU-B"));
        let renames = vec![
            Rename {
                old: "FU-A".into(),
                new: "FU-2026-06-03-1111".into(),
            },
            Rename {
                old: "FU-B".into(),
                new: "FU-2026-06-03-2222".into(),
            },
        ];
        apply(&mut store, &renames);
        assert_eq!(
            store.entries[0].resolution.as_ref().unwrap().wontfix_reason.as_deref(),
            Some("overlaps with FU-2026-06-03-2222, skip")
        );
    }

    #[test]
    fn build_mapping_remaps_every_entry_to_distinct_hash() {
        let mut store = Store::default();
        for id in ["FU-001", "FU-002", "FU-2026-05-01-abc"] {
            store.entries.push(entry(id));
        }
        let renames = build_mapping(&store, 1000);
        assert_eq!(renames.len(), 3);
        let news: std::collections::HashSet<_> = renames.iter().map(|r| r.new.clone()).collect();
        assert_eq!(news.len(), 3, "new ids must be distinct");
        assert!(renames.iter().all(|r| r.new.contains("FU-")));
    }

    #[test]
    fn apply_rewrites_edge_target_and_entry_id() {
        let mut store = Store::default();
        store.entries.push(entry("FU-A"));
        let mut b = entry("FU-B");
        b.edges.push(Edge::BlockedOn("FU-A".into()));
        b.core.scope = Some("blocked on FU-A until merge".into());
        store.entries.push(b);

        let renames = vec![
            Rename {
                old: "FU-A".into(),
                new: "FU-2026-06-03-1111".into(),
            },
            Rename {
                old: "FU-B".into(),
                new: "FU-2026-06-03-2222".into(),
            },
        ];
        let refs = apply(&mut store, &renames);

        assert_eq!(store.entries[0].id, "FU-2026-06-03-1111");
        assert_eq!(store.entries[1].id, "FU-2026-06-03-2222");
        match &store.entries[1].edges[0] {
            Edge::BlockedOn(t) => assert_eq!(t, "FU-2026-06-03-1111"),
            _ => panic!("edge kind changed"),
        }
        assert_eq!(
            store.entries[1].core.scope.as_deref(),
            Some("blocked on FU-2026-06-03-1111 until merge")
        );
        assert!(refs >= 2, "edge target + free-text ref counted");
    }

    fn entry(id: &str) -> Entry {
        Entry {
            id: id.into(),
            location: Location::Open,
            category: "Test".into(),
            status: Status::Open,
            provenance: String::new(),
            core: CoreFields::default(),
            extra: Vec::new(),
            field_order: Vec::new(),
            edges: Vec::new(),
            resolution: None,
        }
    }
}

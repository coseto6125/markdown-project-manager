//! In-memory reverse indexes rebuilt on every load. At a few hundred entries the
//! full rebuild is sub-millisecond, so the indexes are never persisted.

use crate::model::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Index {
    pub by_id: HashMap<String, usize>,
    pub by_status: HashMap<Status, Vec<usize>>,
    pub by_category: HashMap<String, Vec<usize>>,
    pub by_size: HashMap<String, Vec<usize>>,
    pub by_pr: HashMap<u32, Vec<usize>>,
    pub blocked_on: HashMap<usize, Vec<usize>>,
    pub blocks: HashMap<usize, Vec<usize>>,
    pub links_to: HashMap<usize, Vec<usize>>,
    pub linked_from: HashMap<usize, Vec<usize>>,
    pub superseded_by: HashMap<usize, usize>,
    pub supersedes: HashMap<usize, Vec<usize>>,
    pub carried_from: HashMap<usize, usize>,
    /// (entry idx, raw target id) for edge targets not present in `by_id`.
    pub dangling: Vec<(usize, String)>,
}

impl Index {
    pub fn build(store: &Store) -> Self {
        let mut idx = Index::default();
        for (i, e) in store.entries.iter().enumerate() {
            idx.by_id.insert(e.id.clone(), i);
        }
        for (i, e) in store.entries.iter().enumerate() {
            idx.by_status.entry(e.status).or_default().push(i);
            idx.by_category.entry(e.category.clone()).or_default().push(i);
            idx.by_size
                .entry(size_bucket(e.core.size.as_deref()).to_string())
                .or_default()
                .push(i);

            for edge in &e.edges {
                idx.add_edge(store, i, edge);
            }
        }
        idx
    }

    fn add_edge(&mut self, store: &Store, src: usize, edge: &Edge) {
        let resolve = |id: &str| store.entries.iter().position(|e| e.id == id);
        match edge {
            Edge::DoneInPr { pr, .. } => self.by_pr.entry(*pr).or_default().push(src),
            Edge::SurfacedInPr(pr) => self.by_pr.entry(*pr).or_default().push(src),
            Edge::BlockedOn(id) => match resolve(id) {
                Some(t) => {
                    self.blocked_on.entry(src).or_default().push(t);
                    self.blocks.entry(t).or_default().push(src);
                }
                None => self.dangling.push((src, id.clone())),
            },
            Edge::LinksTo(id) => match resolve(id) {
                Some(t) => {
                    self.links_to.entry(src).or_default().push(t);
                    self.linked_from.entry(t).or_default().push(src);
                }
                None => self.dangling.push((src, id.clone())),
            },
            Edge::SupersededBy(id) => match resolve(id) {
                Some(t) => {
                    self.superseded_by.insert(src, t);
                    self.supersedes.entry(t).or_default().push(src);
                }
                None => self.dangling.push((src, id.clone())),
            },
            Edge::CarriedOverFrom(id) => match resolve(id) {
                Some(t) => {
                    self.carried_from.insert(src, t);
                }
                None => self.dangling.push((src, id.clone())),
            },
            Edge::Supersedes(_) | Edge::DoneInBranch { .. } | Edge::Wontfix { .. } => {}
        }
    }
}

//! Bounded block-coordinate output credit through the actual shared runtime.
//! A direct-answer bootstrap is explicit; later reader/update credit uses final
//! suffix answers. This is not cold joint-gradient training or new grammar.
use super::{
    runtime as binding,
    schedule_data::Example,
    schedule_learning::target,
    scheduling::{self as runtime, Artifact, Control, Probe},
};
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    recurrent_text::runtime::Action,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
fn source_digest() -> [u8; 32] {
    *blake3::hash(
        concat!(
            include_str!("joint_learning.rs"),
            include_str!("scheduling.rs"),
            include_str!("schedule_data.rs")
        )
        .as_bytes(),
    )
    .as_bytes()
}
/// Rebind every changed nested artifact from the leaves upward. Frozen geometry,
/// writer and scheduling parameters are retained, while training identity changes.
pub fn rebind(a: &mut Artifact, train: &[Example]) -> Result<()> {
    let data = *blake3::hash(&serde_json::to_vec(train).map_err(|_| Error::Artifact)?).as_bytes();
    let source = source_digest();
    a.parent.parent.source_digest = source;
    a.parent.parent.data_digest = data;
    a.parent.parent.training="Reader rule induction from actual final-answer suffix probes; direct bootstrap then mixed credit. No source/path labels.".into();
    a.parent.parent.parent_digest = *blake3::hash(&a.parent.parent.parent.encode()?).as_bytes();
    a.parent.parent_digest = *blake3::hash(&a.parent.parent.encode()?).as_bytes();
    a.parent.source_digest = source;
    a.parent.data_digest = data;
    a.parent.training="Update rule induction from actual final-answer suffix probes under current fitted reader. No intermediate/position labels.".into();
    a.parent_digest = *blake3::hash(&a.parent.encode()?).as_bytes();
    a.source_digest = source;
    a.data_digest = data;
    a.training="Four fixed blocks: direct reader bootstrap, updater, mixed reader final credit, updater refresh. Learned scheduler and byte writer frozen. Exact forward generation validates each block; no gold source path or intermediate and no claimed gradient learning.".into();
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProbeResult {
    pub tokens: Vec<u16>,
    pub exhausted: bool,
    pub committed_queries: Vec<Vec<u8>>,
    pub selected_path: Vec<[usize; 2]>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub id: String,
    pub clause: usize,
    pub query: Vec<u8>,
    pub features: Vec<u32>,
    pub compatible: Vec<bool>,
    pub occurrences: Vec<[usize; 2]>,
    pub payloads: Vec<Vec<u8>>,
    pub results: Vec<ProbeResult>,
    pub nonlocal_positive: usize,
    pub local_shortcut_positive: usize,
}
fn probe(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    p: &Probe,
) -> Result<(bool, ProbeResult)> {
    let out = runtime::generate_probe(a, g, m, &e.records, &e.prompt, Control::Full, Some(p))?;
    // A reader intervention remains attached to the entire observed word span.
    let mut fired = false;
    for step in &out.steps {
        if let Probe::Read {
            clause,
            query,
            source,
            word,
        } = p
        {
            if step.before.clause == *clause && step.before.core.query.bytes == *query {
                fired = true;
                if step
                    .observation
                    .route
                    .selected
                    .as_ref()
                    .map(|v| [v.source, v.word])
                    != Some([*source, *word])
                {
                    return Err(Error::State);
                }
            }
        } else if let Probe::Update { clause, word } = p {
            if step.before.clause == *clause {
                fired = true;
                if step.observation.update.as_ref().map(|v| v.word) != Some(*word) {
                    return Err(Error::State);
                }
            }
        }
    }
    if !fired {
        return Err(Error::State);
    }
    let good = !out.exhausted && out.tokens == target(e);
    let receipt = ProbeResult {
        tokens: out.tokens,
        exhausted: out.exhausted,
        committed_queries: out
            .steps
            .iter()
            .filter(|s| s.action == Action::Read && !s.after.core.exhausted)
            .map(|s| s.after.core.query.bytes.clone())
            .collect(),
        selected_path: out
            .steps
            .iter()
            .filter(|s| s.before.core.cursor == 0)
            .filter_map(|s| {
                s.observation
                    .route
                    .selected
                    .as_ref()
                    .map(|v| [v.source, v.word])
            })
            .collect(),
    };
    Ok((good, receipt))
}
fn reader_sites(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    mixed: bool,
) -> Result<Vec<Site>> {
    let mut sites = Vec::new();
    for e in train {
        let qs = runtime::clauses(&e.prompt)?;
        if !mixed && qs.len() != 1 {
            continue;
        }
        let mut reached = vec![(0, qs[0].clone())];
        if mixed {
            let baseline = runtime::generate(a, g, m, &e.records, &e.prompt, Control::Full)?;
            for step in baseline.steps {
                let item = (step.before.clause, step.before.core.query.bytes);
                if item.0 > 0 && step.before.core.cursor == 0 && !reached.contains(&item) {
                    reached.push(item);
                }
            }
        }
        for (clause, query) in reached {
            let candidates = reader::candidates(
                &a.parent.parent,
                g,
                m,
                &e.records,
                &query,
                reader::Control::Full,
            )?;
            if candidates.len() > 64 {
                return Err(Error::Shape);
            }
            let mut site = Site {
                id: e.id.clone(),
                clause,
                query: query.clone(),
                features: vec![],
                compatible: vec![],
                occurrences: vec![],
                payloads: vec![],
                results: vec![],
                nonlocal_positive: 0,
                local_shortcut_positive: 0,
            };
            for c in candidates {
                let p = Probe::Read {
                    clause,
                    query: query.clone(),
                    source: c.source,
                    word: c.word,
                };
                let (good, out) = probe(a, g, m, e, &p)?;
                if good && clause == 0 && qs.len() > 1 {
                    if c.bytes == e.answer {
                        site.local_shortcut_positive += 1;
                    } else {
                        site.nonlocal_positive += 1;
                    }
                }
                site.features.push(c.features);
                site.compatible.push(good);
                site.occurrences.push([c.source, c.word]);
                site.payloads.push(c.bytes);
                site.results.push(out);
            }
            sites.push(site);
        }
    }
    if sites.len() > 6144 {
        return Err(Error::Shape);
    }
    Ok(sites)
}
fn update_sites(
    a: &Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
) -> Result<Vec<Site>> {
    let mut sites = Vec::new();
    for e in train {
        let qs = runtime::clauses(&e.prompt)?;
        if qs.len() != 2 {
            continue;
        }
        let first = reader::route(
            &a.parent.parent,
            g,
            m,
            &e.records,
            &qs[0],
            reader::Control::Full,
        )?;
        let value = first.selected.as_ref().ok_or(Error::State)?;
        let candidates = binding::updates(
            &a.parent,
            g,
            m,
            &e.records,
            &qs[1],
            &value.bytes,
            binding::Control::Full,
        )?;
        let mut site = Site {
            id: e.id.clone(),
            clause: 0,
            query: qs[1].clone(),
            features: vec![],
            compatible: vec![],
            occurrences: vec![],
            payloads: vec![],
            results: vec![],
            nonlocal_positive: 0,
            local_shortcut_positive: 0,
        };
        for c in candidates {
            let (good, out) = probe(
                a,
                g,
                m,
                e,
                &Probe::Update {
                    clause: 0,
                    word: c.word,
                },
            )?;
            site.features.push(c.features);
            site.compatible.push(good);
            site.occurrences.push([0, c.word]);
            site.payloads.push(c.question);
            site.results.push(out);
        }
        sites.push(site);
    }
    if sites.len() > 2048 {
        return Err(Error::Shape);
    }
    Ok(sites)
}
fn selected(rules: &[u32], site: &Site, extra: u32) -> (usize, bool) {
    let mut n = 0;
    let mut good = true;
    for (&f, &yes) in site.features.iter().zip(&site.compatible) {
        if rules.iter().any(|&r| f & r == r) || (extra != 0 && f & extra == extra) {
            n += 1;
            good &= yes;
        }
    }
    (n, good)
}
fn proposals(start: usize, n: usize, left: usize, mask: u32, out: &mut Vec<u32>) {
    if mask != 0 {
        out.push(mask);
    }
    if left == 0 {
        return;
    }
    for i in start..n {
        proposals(i + 1, n, left - 1, mask | (1 << i), out);
    }
}
/// Same bounded monotone-conjunction cover used by the retained operator
/// learners, now sharing final-output labels across reader and update sites.
fn induce(sites: &[Site], features: usize, literals: usize) -> Result<(Vec<u32>, usize)> {
    if sites.is_empty()
        || sites
            .iter()
            .any(|s| s.features.is_empty() || s.features.len() != s.compatible.len())
    {
        return Err(Error::Shape);
    }
    let mut pool = Vec::new();
    proposals(0, features, literals, 0, &mut pool);
    pool.sort_by_key(|r| (r.count_ones(), *r));
    let mut rules = Vec::new();
    let mut covered = 0;
    for _ in 0..8 {
        let old: Vec<_> = sites.iter().map(|s| selected(&rules, s, 0)).collect();
        let mut best = 0;
        let mut best_count = covered;
        for &rule in &pool {
            let mut valid = true;
            let mut count = 0;
            for (s, &(old_n, old_good)) in sites.iter().zip(&old) {
                let (n, good) = selected(&rules, s, rule);
                if !good || (old_n == 1 && old_good && n != 1) {
                    valid = false;
                    break;
                }
                count += usize::from(n == 1 && good);
            }
            if valid && count > best_count {
                best = rule;
                best_count = count;
            }
        }
        if best == 0 {
            break;
        }
        rules.push(best);
        covered = best_count;
        if covered == sites.len() {
            break;
        }
    }
    Ok((rules, covered))
}
fn actual(a: &Artifact, g: &BoundGeometry, m: &Metric, train: &[Example]) -> Result<Vec<bool>> {
    train
        .iter()
        .map(|e| {
            let out = runtime::generate(a, g, m, &e.records, &e.prompt, Control::Full)?;
            Ok(!out.exhausted && out.tokens == target(e))
        })
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub name: String,
    pub before_exact: usize,
    pub after_exact: usize,
    pub reader_rules: Vec<u32>,
    pub update_rules: Vec<u32>,
    pub sites: usize,
    pub nonlocal_positive: usize,
    pub local_shortcut_positive: usize,
    pub covered: usize,
    pub preserved: bool,
}
pub struct Outcome {
    pub initial: Artifact,
    pub bootstrap: Artifact,
    pub candidate: Artifact,
    pub blocks: Vec<Block>,
}
fn write(path: &Path, name: &str, value: &impl Serialize) -> Result<()> {
    std::fs::write(
        path.join(name),
        serde_json::to_vec_pretty(value).map_err(|_| Error::Artifact)?,
    )
    .map_err(|_| Error::Artifact)
}
pub fn run(
    parent: Artifact,
    g: &BoundGeometry,
    m: &Metric,
    train: &[Example],
    output: &Path,
) -> Result<Outcome> {
    let mut a = parent;
    a.parent.parent.rules.clear();
    a.parent.rules.clear();
    rebind(&mut a, train)?;
    a.validate(g)?;
    let initial = a.clone();
    std::fs::write(output.join("initial.json"), initial.encode()?).map_err(|_| Error::Artifact)?;
    let mut prior = actual(&a, g, m, train)?;
    let mut blocks = Vec::new();
    let mut bootstrap = None;
    for (name, reader, mixed) in [
        ("reader-bootstrap", true, false),
        ("updater-bootstrap", false, false),
        ("reader-mixed-credit", true, true),
        ("updater-refresh", false, true),
    ] {
        let sites = if reader {
            reader_sites(&a, g, m, train, mixed)?
        } else {
            update_sites(&a, g, m, train)?
        };
        let (rules, covered) = induce(
            &sites,
            if reader { 18 } else { 8 },
            if reader { 4 } else { 3 },
        )?;
        let mut next = a.clone();
        if reader {
            next.parent.parent.rules = rules;
        } else {
            next.parent.rules = rules;
        }
        rebind(&mut next, train)?;
        next.validate(g)?;
        let now = actual(&next, g, m, train)?;
        let preserved = prior.iter().zip(&now).all(|(&was, &is)| !was || is);
        let block = Block {
            name: name.into(),
            before_exact: prior.iter().filter(|&&v| v).count(),
            after_exact: now.iter().filter(|&&v| v).count(),
            reader_rules: next.parent.parent.rules.clone(),
            update_rules: next.parent.rules.clone(),
            sites: sites.len(),
            nonlocal_positive: sites.iter().map(|s| s.nonlocal_positive).sum(),
            local_shortcut_positive: sites.iter().map(|s| s.local_shortcut_positive).sum(),
            covered,
            preserved,
        };
        std::fs::write(
            output.join(format!("{name}-candidate.json")),
            next.encode()?,
        )
        .map_err(|_| Error::Artifact)?;
        write(output, &format!("{name}-sites.json"), &sites)?;
        write(output, &format!("{name}-block.json"), &block)?;
        if !preserved || covered != sites.len() {
            return Err(Error::State);
        }
        blocks.push(block);
        a = next;
        prior = now;
        if name == "updater-bootstrap" {
            bootstrap = Some(a.clone());
        }
    }
    Ok(Outcome {
        initial,
        bootstrap: bootstrap.ok_or(Error::State)?,
        candidate: a,
        blocks,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_induction_refuses_erased_distinctions_and_preserves_latent_positives() -> Result<()> {
        let site = |features, compatible| Site {
            id: "test".into(),
            clause: 0,
            query: vec![],
            features,
            compatible,
            occurrences: vec![],
            payloads: vec![],
            results: vec![],
            nonlocal_positive: 0,
            local_shortcut_positive: 0,
        };
        let (_, covered) = induce(&[site(vec![1, 1], vec![true, false])], 2, 2)?;
        assert_eq!(covered, 0);
        let (rules, covered) = induce(
            &[
                site(vec![1, 2], vec![true, true]),
                site(vec![1, 2], vec![true, false]),
            ],
            2,
            2,
        )?;
        assert_eq!(rules, vec![1]);
        assert_eq!(covered, 2);
        Ok(())
    }
}

//! Finite word-role induction from direct answer text. No grammar lexicon is
//! supplied: a word seen in any answer cannot become a context-only barrier.
use super::span_data::Example;
use crate::native_geometric::{
    addressed_attention::artifact::BoundGeometry,
    hamming_refinement::metric::Metric,
    ordered_state::runtime as ordered,
    relational_attention::runtime::{Error, Result},
    relative_language::runtime as reader,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_CONTEXT_WORDS: usize = 64;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Word {
    pub bytes: Vec<u8>,
    pub geometry: ordered::Query,
}
pub fn validate(g: &BoundGeometry, words: &[Word]) -> Result<()> {
    if words.len() > MAX_CONTEXT_WORDS {
        return Err(Error::Artifact);
    }
    let mut previous: Option<&[u8]> = None;
    for word in words {
        if word.bytes.is_empty()
            || word.bytes.len() > 16
            || !word.bytes.iter().all(u8::is_ascii_lowercase)
            || previous.is_some_and(|p| p >= word.bytes.as_slice())
            || ordered::Query::encode(g, &word.bytes, ordered::CANONICAL)? != word.geometry
        {
            return Err(Error::Artifact);
        }
        previous = Some(&word.bytes);
    }
    Ok(())
}
pub fn learn(g: &BoundGeometry, train: &[Example]) -> Result<Vec<Word>> {
    let mut seen = BTreeSet::new();
    let mut positive = BTreeSet::new();
    for e in train {
        for word in reader::words(g, &e.answer, ordered::CANONICAL)? {
            positive.insert(word.bytes);
        }
        for raw in &e.records {
            for word in reader::words(g, raw, ordered::CANONICAL)? {
                seen.insert(word.bytes);
            }
        }
    }
    let result: Result<Vec<_>> = seen
        .difference(&positive)
        .map(|bytes| {
            Ok(Word {
                bytes: bytes.clone(),
                geometry: ordered::Query::encode(g, bytes, ordered::CANONICAL)?,
            })
        })
        .collect();
    let result = result?;
    validate(g, &result)?;
    Ok(result)
}
pub fn contains(m: &Metric, context: &[Word], word: &ordered::Query, exact: bool) -> Result<bool> {
    for known in context {
        if if exact {
            known.geometry.occurrences == word.occurrences
        } else {
            ordered::distance(m, &known.geometry, word, false)? == 0
        } {
            return Ok(true);
        }
    }
    Ok(false)
}
pub fn complete_run(barriers: &[bool], first: usize, end: usize) -> bool {
    first < end
        && end <= barriers.len()
        && !barriers[first..end].iter().any(|&b| b)
        && (first == 0 || barriers[first - 1])
        && (end == barriers.len() || barriers[end])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn answer_usage_overrides_context_role_and_extent_cannot_cross_barrier() -> Result<()> {
        let g = BoundGeometry::canonical().map_err(|_| Error::Geometry)?;
        let e = Example {
            id: "role".into(),
            family: "role".into(),
            depth: 1,
            variant: "baseline".into(),
            records: std::array::from_fn(|_| b"today amber did help ruby.".to_vec()),
            prompt: b"who did amber help?".to_vec(),
            answer: b"ruby amber".to_vec(),
            expected_path: vec![],
            expected_span: [0; 3],
            expected_bounds: [0; 2],
        };
        let words = learn(&g, &[e])?;
        assert!(!words
            .iter()
            .any(|w| w.bytes == b"amber" || w.bytes == b"ruby"));
        assert!(words.iter().any(|w| w.bytes == b"today"));
        assert!(complete_run(&[true, false, false, true], 1, 3));
        assert!(!complete_run(&[true, false, false, true], 1, 2));
        assert!(!complete_run(&[true, false, false, true], 0, 3));
        let mut bad = words.clone();
        bad[0].geometry.prefixes[0][0] ^= 1;
        assert!(validate(&g, &bad).is_err());
        Ok(())
    }
}
/// Attribute context evidence only to source occurrences that can produce the
/// answer. Intersection retains uncertainty among all compatible latent spans.
/// An unrelated unused record supplies no negative lexical evidence.
pub fn learn_from_compatible(
    g: &BoundGeometry,
    train: &[Example],
    prepared: &[super::span_learning::Prepared],
) -> Result<Vec<Word>> {
    if train.len() != prepared.len() {
        return Err(Error::Shape);
    }
    let mut evidence = BTreeSet::new();
    let mut positive = BTreeSet::new();
    for (e, p) in train.iter().zip(prepared) {
        if p.candidates.len() != p.compatible.len() {
            return Err(Error::Shape);
        }
        for word in reader::words(g, &e.answer, ordered::CANONICAL)? {
            positive.insert(word.bytes);
        }
        let mut common: Option<BTreeSet<Vec<u8>>> = None;
        for (c, &yes) in p.candidates.iter().zip(&p.compatible) {
            if !yes {
                continue;
            }
            let raw = e.records.get(c.value.source).ok_or(Error::Shape)?;
            let outside: BTreeSet<_> = reader::words(g, raw, ordered::CANONICAL)?
                .into_iter()
                .filter(|w| w.end <= c.value.start || w.start >= c.value.end)
                .map(|w| w.bytes)
                .collect();
            common = Some(if let Some(old) = common {
                old.intersection(&outside).cloned().collect()
            } else {
                outside
            });
        }
        evidence.extend(common.ok_or(Error::Shape)?);
    }
    let result: Result<Vec<_>> = evidence
        .difference(&positive)
        .map(|bytes| {
            Ok(Word {
                bytes: bytes.clone(),
                geometry: ordered::Query::encode(g, bytes, ordered::CANONICAL)?,
            })
        })
        .collect();
    let result = result?;
    validate(g, &result)?;
    Ok(result)
}
#[cfg(test)]
mod credit_tests {
    use super::super::{span, span_learning::Prepared};
    use super::*;
    use crate::native_geometric::language_relation::runtime::Candidate;
    #[test]
    fn unrelated_records_and_noncommon_latent_context_supply_no_credit() -> Result<()> {
        let g = BoundGeometry::canonical().map_err(|_| Error::Geometry)?;
        let e = Example {
            id: "credit".into(),
            family: "credit".into(),
            depth: 1,
            variant: "baseline".into(),
            records: [
                b"today amber.".to_vec(),
                b"today amber ruby.".to_vec(),
                b"oscar helen.".to_vec(),
                b"unknown.".to_vec(),
            ],
            prompt: b"who?".to_vec(),
            answer: b"amber".to_vec(),
            expected_path: vec![],
            expected_span: [0; 3],
            expected_bounds: [0; 2],
        };
        let c = |source| span::Candidate {
            value: Candidate {
                source,
                word: 1,
                start: 6,
                end: 11,
                bytes: b"amber".to_vec(),
                features: 0,
            },
            last_word: 2,
            features: 0,
        };
        let p = Prepared {
            candidates: vec![c(0), c(1)],
            compatible: vec![true, true],
        };
        let words = learn_from_compatible(&g, &[e], &[p])?;
        assert_eq!(
            words.iter().map(|w| w.bytes.as_slice()).collect::<Vec<_>>(),
            vec![b"today".as_slice()]
        );
        Ok(())
    }
}

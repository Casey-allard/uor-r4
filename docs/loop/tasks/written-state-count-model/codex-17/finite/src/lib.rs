#![forbid(unsafe_code)]
//! Finite-only adapter for original-offset, in-fence onset candidates.

use uor_r4_integer::{
    assertion_value::AssertionValue,
    discrete_d11::{ExactCopyState, ROLE_COPY_STOPS},
    paired_role::{ROLE_CONTEXT, ROLE_VOCAB},
    span_selector::{CONT, START},
};

const MAX_STARTS: usize = 7;
const LEGACY_DEPTHS: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidTape,
    InvalidBounds,
    FenceMismatch,
    InconsistentLegacyMask,
    InvalidEvidence,
    WriterRejected,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Candidate {
    pub source: u32,
    /// Half-open end of the first complete word.
    pub first_word_end: u32,
    pub connector_ids: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyProposal {
    pub source: u32,
    pub writer_reachable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub source: u32,
    /// Hand-supplied finite score. None means the fixture has no evidence.
    pub score: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Nomination {
    Source(u32),
    NoCandidate,
    Unseen,
    Tie,
    Nonpositive,
}

pub struct Candidates<'a> {
    original: &'a [u32],
    pub fence: Option<u32>,
    pub legacy_mask: u32,
    pub legacy: [Option<LegacyProposal>; LEGACY_DEPTHS],
    starts: [Candidate; MAX_STARTS],
    count: usize,
}

impl<'a> Candidates<'a> {
    /// Scope starts after at least one connector ID and stops at the original
    /// first delimiter before the requested word. The reported legacy mask is
    /// preserved independently of the later writer-reachability gate.
    pub fn capture(
        ids: &'a [u32],
        classes: &[u32],
        fact_end: u32,
        query_start: u32,
        frozen: AssertionValue,
    ) -> Result<Self> {
        if ids.first() != Some(&0)
            || ids.len() > ROLE_CONTEXT
            || classes.len() != ROLE_VOCAB
            || ids
                .iter()
                .any(|&id| classes.get(id as usize).is_none_or(|&class| class > CONT))
        {
            return Err(Error::InvalidTape);
        }
        let (fact, query) = (fact_end as usize, query_start as usize);
        if fact == 0 || fact >= query || query >= ids.len() {
            return Err(Error::InvalidBounds);
        }
        let fence = ids[fact + 1..query]
            .iter()
            .position(|id| ROLE_COPY_STOPS.contains(id))
            .map(|offset| (fact + 1 + offset) as u32);
        if frozen.fence != fence {
            return Err(Error::FenceMismatch);
        }
        let legacy_mask = frozen.value.eligible_mask & frozen.value.word_start_mask;
        if (frozen.value.eligible_mask | frozen.value.word_start_mask) >> LEGACY_DEPTHS != 0 {
            return Err(Error::InconsistentLegacyMask);
        }
        let mut output = Self {
            original: ids,
            fence,
            legacy_mask,
            legacy: [None; LEGACY_DEPTHS],
            starts: [Candidate::default(); MAX_STARTS],
            count: 0,
        };
        let Some(fence) = fence else {
            return if legacy_mask == 0 {
                Ok(output)
            } else {
                Err(Error::InconsistentLegacyMask)
            };
        };

        for source in fact_end + 2..fence {
            if classes[ids[source as usize] as usize] != START
                || !writer_reaches(ids, source, fence)?
            {
                continue;
            }
            let end = (source as usize + 1..fence as usize)
                .find(|&at| classes[ids[at] as usize] != CONT)
                .unwrap_or(fence as usize);
            let slot = output
                .starts
                .get_mut(output.count)
                .ok_or(Error::WriterRejected)?;
            *slot = Candidate {
                source,
                first_word_end: end as u32,
                connector_ids: source - fact_end - 1,
            };
            output.count += 1;
        }
        for index in 0..LEGACY_DEPTHS {
            if legacy_mask & (1 << index) == 0 {
                continue;
            }
            let source = fact_end + 2 + index as u32;
            if source >= fence || classes[ids[source as usize] as usize] != START {
                return Err(Error::InconsistentLegacyMask);
            }
            output.legacy[index] = Some(LegacyProposal {
                source,
                writer_reachable: output.reachable().iter().any(|item| item.source == source),
            });
        }
        Ok(output)
    }

    pub fn reachable(&self) -> &[Candidate] {
        &self.starts[..self.count]
    }

    /// The finite fixture policy requires one keyed score for every candidate.
    /// Unknown scores and equal best scores never manufacture a source.
    pub fn nominate(&self, evidence: &[Evidence]) -> Result<Nomination> {
        if evidence.len() != self.count
            || evidence
                .iter()
                .zip(self.reachable())
                .any(|(score, candidate)| score.source != candidate.source)
        {
            return Err(Error::InvalidEvidence);
        }
        let Some(best) = evidence.iter().filter_map(|item| item.score).max() else {
            return Ok(if evidence.is_empty() {
                Nomination::NoCandidate
            } else {
                Nomination::Unseen
            });
        };
        if evidence.iter().any(|item| item.score.is_none()) {
            return Ok(Nomination::Unseen);
        }
        let mut winners = evidence.iter().filter(|item| item.score == Some(best));
        if best <= 0 {
            return Ok(Nomination::Nonpositive);
        }
        let Some(winner) = winners.next() else {
            return Err(Error::InvalidEvidence);
        };
        Ok(if winners.next().is_some() {
            Nomination::Tie
        } else {
            Nomination::Source(winner.source)
        })
    }

    /// Finite cursor replay. Copies untouched original IDs and existing EOS.
    /// A request to replay an address absent from this capture fails closed.
    pub fn replay(&self, source: u32) -> Result<[Option<u32>; MAX_STARTS + 1]> {
        if !self.reachable().iter().any(|item| item.source == source) {
            return Err(Error::WriterRejected);
        }
        let fence = self.fence.ok_or(Error::WriterRejected)?;
        let mut cursor = ExactCopyState::new(Some(source), self.original.len() as u32);
        let mut output = [None; MAX_STARTS + 1];
        for slot in &mut output {
            // Ordinary next ID is irrelevant on accepted copy and delimiter steps.
            let step = cursor
                .write(self.original, u32::MAX, ROLE_COPY_STOPS)
                .map_err(|_| Error::WriterRejected)?
                .finish_with_eos(true, 1);
            *slot = Some(step.emitted);
            if step.release != 0 {
                return if step.completed_span() && step.state.cursor == fence {
                    Ok(output)
                } else {
                    Err(Error::WriterRejected)
                };
            }
            cursor = step.state;
        }
        Err(Error::WriterRejected)
    }
}

fn writer_reaches(original: &[u32], source: u32, fence: u32) -> Result<bool> {
    let mut cursor = ExactCopyState::new(Some(source), original.len() as u32);
    loop {
        let step = cursor
            .write(original, u32::MAX, ROLE_COPY_STOPS)
            .map_err(|_| Error::WriterRejected)?;
        if step.release != 0 {
            return Ok(step.completed_span() && step.state.cursor == fence);
        }
        cursor = step.state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::value_selector::Decisions;

    fn token_classes() -> [u32; ROLE_VOCAB] {
        let mut classes = [0; ROLE_VOCAB];
        for id in [42, 10, 11, 13, 15, 20, 21, 45, 46, 47, 48, 49] {
            classes[id] = START;
        }
        for id in [12, 14] {
            classes[id] = CONT;
        }
        classes
    }

    fn frozen(fence: Option<u32>, word_start_mask: u32) -> AssertionValue {
        AssertionValue {
            fence,
            value: Decisions {
                eligible_mask: if fence.is_some() { 0b111 } else { 0 },
                word_start_mask,
                ..Decisions::default()
            },
            ..AssertionValue::default()
        }
    }

    fn tape(cue: u32) -> [u32; 14] {
        [0, 55, 42, cue, 11, 12, 13, 14, 15, 16, 46, 47, 42, 201]
    }

    fn scores(scores: [Option<i32>; 3]) -> [Evidence; 3] {
        [4, 6, 8]
            .into_iter()
            .zip(scores)
            .map(|(source, score)| Evidence { source, score })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    fn emitted(record: [Option<u32>; 8]) -> Vec<u32> {
        record.into_iter().flatten().collect()
    }

    #[test]
    fn earlier_middle_and_later_original_candidates_each_reach_exact_eos() {
        let classes = token_classes();
        let cases = [
            (10, [9, 2, 3], 4, vec![11, 12, 13, 14, 15, 1]),
            (20, [2, 9, 3], 6, vec![13, 14, 15, 1]),
            (21, [2, 3, 9], 8, vec![15, 1]),
        ];
        for (cue, words, expected_source, expected_ids) in cases {
            let ids = tape(cue);
            let found = Candidates::capture(&ids, &classes, 2, 12, frozen(Some(9), 0b101)).unwrap();
            assert_eq!(
                found.reachable(),
                [
                    Candidate {
                        source: 4,
                        first_word_end: 6,
                        connector_ids: 1
                    },
                    Candidate {
                        source: 6,
                        first_word_end: 8,
                        connector_ids: 3
                    },
                    Candidate {
                        source: 8,
                        first_word_end: 9,
                        connector_ids: 5
                    },
                ]
            );
            assert_eq!(found.legacy_mask, 0b101);
            assert_eq!(
                found.legacy,
                [
                    Some(LegacyProposal {
                        source: 4,
                        writer_reachable: true
                    }),
                    None,
                    Some(LegacyProposal {
                        source: 6,
                        writer_reachable: true
                    }),
                ]
            );
            assert_eq!(
                found.nominate(&scores(words.map(Some))).unwrap(),
                Nomination::Source(expected_source)
            );
            assert_eq!(
                emitted(found.replay(expected_source).unwrap()),
                expected_ids
            );
        }
    }

    #[test]
    fn real_frozen_masks_retain_all_three_legacy_cuts_and_separate_extended_cut() {
        use uor_r4_integer::{
            assertion_value,
            content_selector::BOUNDARY,
            span_selector::{Bank, SpanChoice},
            value_selector::ExactBank,
        };
        let classes = token_classes();
        let bank = Bank {
            classes: classes.to_vec(),
            relations: vec![
                [1, 0, BOUNDARY, BOUNDARY, 0],
                [2, 0, 0, BOUNDARY, 0],
                [3, 0, 0, 0, 0],
            ],
        };
        let exact = ExactBank {
            connectors: vec![[1, 10, BOUNDARY, BOUNDARY]],
        };
        let ids = [0, 55, 42, 10, 11, 13, 15, 14, 45, 16, 46, 47, 42, 201];
        let spans = SpanChoice {
            query_start: Some(12),
            query_end: Some(12),
            fact_start: Some(2),
            fact_end: Some(2),
            ..Default::default()
        };
        let frozen = assertion_value::decide(
            &ids,
            &[0; ROLE_VOCAB],
            &bank,
            &exact,
            spans,
            ROLE_COPY_STOPS,
        )
        .unwrap();
        assert_eq!(frozen.fence, Some(9));
        assert_eq!(frozen.value.eligible_mask, 0b111);
        assert_eq!(frozen.value.word_start_mask, 0b111);
        assert_eq!(frozen.value.word_start, Some(6));
        let all = Candidates::capture(&ids, &classes, 2, 12, frozen).unwrap();
        assert_eq!(all.legacy_mask, 0b111);
        assert_eq!(
            all.legacy.map(|proposal| proposal.unwrap().source),
            [4, 5, 6]
        );
        assert_eq!(
            all.reachable().iter().map(|x| x.source).collect::<Vec<_>>(),
            [4, 5, 6, 8]
        );
        for (gold, expected) in [
            (4, vec![11, 13, 15, 14, 45, 1]),
            (5, vec![13, 15, 14, 45, 1]),
            (6, vec![15, 14, 45, 1]),
        ] {
            let observations = all
                .reachable()
                .iter()
                .map(|item| Evidence {
                    source: item.source,
                    score: Some(if item.source == gold { 10 } else { 1 }),
                })
                .collect::<Vec<_>>();
            assert_eq!(all.nominate(&observations), Ok(Nomination::Source(gold)));
            assert_eq!(emitted(all.replay(gold).unwrap()), expected);
        }
    }

    #[test]
    fn tied_unseen_and_misbound_scores_do_not_nominate() {
        let classes = token_classes();
        let ids = tape(10);
        let found = Candidates::capture(&ids, &classes, 2, 12, frozen(Some(9), 0b101)).unwrap();
        assert_eq!(
            found.nominate(&scores([Some(4), Some(4), Some(1)])),
            Ok(Nomination::Tie)
        );
        assert_eq!(
            found.nominate(&scores([Some(4), None, Some(1)])),
            Ok(Nomination::Unseen)
        );
        assert_eq!(found.nominate(&scores([None; 3])), Ok(Nomination::Unseen));
        assert_eq!(
            found.nominate(&scores([Some(0), Some(-1), Some(-2)])),
            Ok(Nomination::Nonpositive)
        );
        let mut misplaced = scores([Some(5), Some(0), Some(0)]);
        misplaced.swap(0, 1);
        assert_eq!(found.nominate(&misplaced), Err(Error::InvalidEvidence));
    }

    #[test]
    fn wrong_address_copies_wrong_literal_through_the_unchanged_cursor() {
        let classes = token_classes();
        let ids = tape(10);
        let found = Candidates::capture(&ids, &classes, 2, 12, frozen(Some(9), 0b101)).unwrap();
        let Nomination::Source(wrong) = found
            .nominate(&scores([Some(1), Some(4), Some(2)]))
            .unwrap()
        else {
            panic!("the deliberate wrong-score fixture must nominate");
        };
        let expected = [11, 12, 13, 14, 15, 1];
        let actual = emitted(found.replay(wrong).unwrap());
        assert_eq!(actual, [13, 14, 15, 1]);
        assert_ne!(actual[0], expected[0]);
        assert_eq!(actual.last(), Some(&1));
        assert_eq!(found.replay(5), Err(Error::WriterRejected)); // interior BPE piece
        assert_eq!(found.replay(10), Err(Error::WriterRejected)); // after delimiter
    }

    #[test]
    fn mask_hit_does_not_bypass_cap_or_first_protocol_stop() {
        let classes = token_classes();
        let ids = [
            0, 55, 42, 10, 11, 12, 13, 14, 15, 45, 46, 47, 201, 48, 16, 42,
        ];
        let found = Candidates::capture(&ids, &classes, 2, 15, frozen(Some(12), 0b101)).unwrap();
        assert_eq!(found.legacy_mask, 0b101);
        assert_eq!(
            found.legacy[0],
            Some(LegacyProposal {
                source: 4,
                writer_reachable: false
            })
        );
        assert_eq!(
            found.legacy[2],
            Some(LegacyProposal {
                source: 6,
                writer_reachable: true
            })
        );
        assert_eq!(
            found
                .reachable()
                .iter()
                .map(|c| c.source)
                .collect::<Vec<_>>(),
            [6, 8, 9, 10, 11]
        );
        assert_eq!(found.replay(4), Err(Error::WriterRejected));
        assert_eq!(found.replay(13), Err(Error::WriterRejected));
        assert!(matches!(
            Candidates::capture(&ids, &classes, 2, 15, frozen(Some(14), 0b101)),
            Err(Error::FenceMismatch)
        ));
        assert!(matches!(
            Candidates::capture(&ids, &classes, 2, 15, frozen(Some(12), 0b111)),
            Err(Error::InconsistentLegacyMask) // falsely marks continuation as START
        ));
        let no_fence = [0, 55, 42, 10, 11, 12, 13, 42, 201];
        let empty = Candidates::capture(&no_fence, &classes, 2, 7, frozen(None, 0)).unwrap();
        assert_eq!(empty.fence, None);
        assert_eq!(empty.nominate(&[]), Ok(Nomination::NoCandidate));
        assert_eq!(empty.replay(4), Err(Error::WriterRejected));
    }
}

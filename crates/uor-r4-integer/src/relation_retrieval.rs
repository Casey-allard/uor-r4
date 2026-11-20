//! Opt-in assertion selection on bounded D11 prompt words.
use std::{io::Write, ops::Range};

use serde::Serialize;

use crate::{
    discrete_d11::spiralcore_orbit_roots,
    invalid,
    paired_role::{validate_role_ids, RoleSpan, ROLE_CONTEXT, ROLE_VOCAB},
    span_selector::{CONT, START},
    Result,
};

const RADIUS_SQUARED_PER_PIECE: u32 = 16;
pub const NO_SCORE: u32 = u32::MAX;
pub const DEVICE_OUTPUT_ROW_BYTES: usize = (2 + 5 * ROLE_CONTEXT) * size_of::<u32>();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm {
    Exact,
    Geometry,
}

/// All spans are half-open positions in the original BOS-inclusive token tape.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Candidate {
    pub fact: RoleSpan,
    pub cue: RoleSpan,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Evidence {
    pub literal: bool,
    pub collision: bool,
    /// Minimum admitted nonzero squared distance. The denominator is cue length.
    pub distance: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[repr(u32)]
pub enum Outcome {
    Empty = 0,
    Selected = 1,
    Tie = 2,
    Collision = 3,
}

#[derive(Debug, Serialize)]
pub struct Decision {
    pub outcome: Outcome,
    pub selected_facts: Vec<RoleSpan>,
}

pub struct Plan<'a> {
    ids: &'a [u32],
    roots: &'a [u32],
    candidates: Vec<Candidate>,
    query_words: Vec<RoleSpan>,
    query_fence: Range<usize>,
}

impl<'a> Plan<'a> {
    /// Discover candidates using the incumbent's fact-predecessor and stop
    /// tokens. The relation cue is the first complete, contiguous word after
    /// the matched fact person. No value or source location is consulted.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ids: &'a [u32],
        roots: &'a [u32],
        classes: &[u32],
        content: Range<usize>,
        query_fence: Range<usize>,
        requested: RoleSpan,
        excluded: Option<RoleSpan>,
        fact_predecessors: &[u32],
        fact_stops: &[u32],
    ) -> Result<Self> {
        validate_role_ids(ids)?;
        if roots.len() != ROLE_VOCAB
            || classes.len() != ROLE_VOCAB
            || ids
                .iter()
                .any(|&id| roots[id as usize] > u16::MAX as u32 || classes[id as usize] > CONT)
            || content.start == 0
            || content.start > query_fence.start
            || query_fence.start > query_fence.end
            || query_fence.end > content.end
            || content.end > ids.len()
            || requested.start < query_fence.start
            || requested.start >= requested.end
            || requested.end > query_fence.end
        {
            return Err(invalid("invalid assertion retrieval tables or framing"));
        }
        let class = |position: usize| classes[ids[position] as usize];
        let whole_word = |start: usize| -> Option<RoleSpan> {
            if start >= content.end || class(start) != START {
                return None;
            }
            let mut end = start + 1;
            while end < content.end && class(end) == CONT {
                end += 1;
            }
            Some(RoleSpan { start, end })
        };
        if whole_word(requested.start) != Some(requested)
            || excluded.is_some_and(|span| {
                span.start < content.start || whole_word(span.start) != Some(span)
            })
        {
            return Err(invalid("invalid assertion retrieval person span"));
        }

        let person = &ids[requested.start..requested.end];
        let candidates = (content.start..query_fence.start)
            .filter(|&start| fact_predecessors.contains(&ids[start - 1]))
            .filter_map(|start| {
                let fact = whole_word(start)?;
                if fact.end >= query_fence.start || &ids[fact.start..fact.end] != person {
                    return None;
                }
                let fence = (fact.end..query_fence.start)
                    .find(|&position| fact_stops.contains(&ids[position]))?;
                let cue = whole_word(fact.end)?;
                (cue.end <= fence).then_some(Candidate { fact, cue })
            })
            .collect();

        let excluded_word = excluded.map(|span| &ids[span.start..span.end]);
        let query_words = query_fence
            .clone()
            .filter_map(whole_word)
            .filter(|&span| {
                let word = &ids[span.start..span.end];
                span.end <= query_fence.end
                    && word != person
                    && excluded_word != Some(word)
                    && !word.iter().any(|id| fact_stops.contains(id))
            })
            .collect();

        Ok(Self {
            ids,
            roots,
            candidates,
            query_words,
            query_fence,
        })
    }

    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    pub fn query_words(&self) -> &[RoleSpan] {
        &self.query_words
    }

    pub fn query_fence(&self) -> Range<usize> {
        self.query_fence.clone()
    }

    /// Diagnostic intervention using an already eligible original query word.
    pub fn restrict_to_relation(&mut self, word: RoleSpan) -> Result<()> {
        if !self.query_words.contains(&word) {
            return Err(invalid(
                "supplied relation must be an eligible complete query word",
            ));
        }
        self.query_words.retain(|&candidate| candidate == word);
        Ok(())
    }

    fn word(&self, span: RoleSpan) -> &[u32] {
        &self.ids[span.start..span.end]
    }

    fn score(&self, cue: RoleSpan, compare_roots: bool) -> Evidence {
        let cue = self.word(cue);
        let mut evidence = Evidence::default();
        for &query in &self.query_words {
            let query = self.word(query);
            if cue.len() != query.len() {
                continue;
            }
            if cue == query {
                evidence.literal = true;
                continue;
            }
            if !compare_roots {
                continue;
            }
            let distance = cue
                .iter()
                .zip(query)
                .map(|(&fact, &ask)| {
                    packed_distance(self.roots[fact as usize], self.roots[ask as usize])
                })
                .sum();
            if distance == 0 {
                evidence.collision = true;
            } else if distance <= RADIUS_SQUARED_PER_PIECE * cue.len() as u32
                && evidence.distance.is_none_or(|old| distance < old)
            {
                evidence.distance = Some(distance);
            }
        }
        evidence
    }

    pub fn evidence(&self) -> Vec<Evidence> {
        self.candidates
            .iter()
            .map(|candidate| self.score(candidate.cue, true))
            .collect()
    }

    pub fn select(&self, arm: Arm) -> Decision {
        let evidence: Vec<_> = self
            .candidates
            .iter()
            .map(|candidate| self.score(candidate.cue, arm == Arm::Geometry))
            .collect();
        self.decide(arm, &evidence)
    }

    fn decide(&self, arm: Arm, evidence: &[Evidence]) -> Decision {
        let literal = evidence.iter().any(|score| score.literal);
        let mut decision = Decision {
            outcome: Outcome::Empty,
            selected_facts: Vec::new(),
        };
        if !literal && arm == Arm::Geometry && evidence.iter().any(|score| score.collision) {
            decision.outcome = Outcome::Collision;
            return decision;
        }
        let mut winner: Option<(RoleSpan, u32)> = None;
        for (candidate, score) in self.candidates.iter().zip(evidence) {
            let distance = if literal {
                score.literal.then_some(0)
            } else if arm == Arm::Geometry {
                score.distance
            } else {
                None
            };
            let Some(distance) = distance else { continue };
            let cue = candidate.cue;
            match winner.map(|(best, best_distance)| {
                (distance * (best.end - best.start) as u32)
                    .cmp(&(best_distance * (cue.end - cue.start) as u32))
            }) {
                None | Some(std::cmp::Ordering::Less) => {
                    winner = Some((cue, distance));
                    decision.selected_facts.clear();
                    decision.selected_facts.push(candidate.fact);
                    decision.outcome = Outcome::Selected;
                }
                Some(std::cmp::Ordering::Equal) => {
                    if winner.is_some_and(|(best, _)| self.word(best) != self.word(cue)) {
                        decision.outcome = Outcome::Tie;
                    } else {
                        decision.selected_facts.push(candidate.fact);
                    }
                }
                Some(std::cmp::Ordering::Greater) => {}
            }
        }
        if decision.outcome != Outcome::Selected {
            decision.selected_facts.clear();
        }
        decision
    }

    /// Append one fixed-width device row containing original IDs, their root
    /// words and the discovered cue/query spans. Missing entries are zero.
    pub fn write_device_row(&self, output: &mut impl Write) -> std::io::Result<()> {
        for value in [
            self.ids.len(),
            self.candidates.len(),
            self.query_words.len(),
        ] {
            output.write_all(&(value as u32).to_le_bytes())?;
        }
        for position in 0..ROLE_CONTEXT {
            let id = self.ids.get(position).copied().unwrap_or_default();
            let root = self
                .ids
                .get(position)
                .map_or(0, |_| self.roots[id as usize]);
            output.write_all(&id.to_le_bytes())?;
            output.write_all(&root.to_le_bytes())?;
        }
        for position in 0..ROLE_CONTEXT {
            let cue = self.candidates.get(position).map(|candidate| candidate.cue);
            let query_word = self.query_words.get(position).copied();
            for span in [cue, query_word] {
                let [start, end] = span.map_or([0, 0], |span| [span.start as u32, span.end as u32]);
                output.write_all(&start.to_le_bytes())?;
                output.write_all(&end.to_le_bytes())?;
            }
        }
        Ok(())
    }

    /// Canonical host words for comparing both decision lanes and all scores.
    pub fn write_device_oracle(&self, output: &mut impl Write) -> std::io::Result<()> {
        let evidence = self.evidence();
        let exact = self.decide(Arm::Exact, &evidence);
        let geometry = self.decide(Arm::Geometry, &evidence);
        output.write_all(&(exact.outcome as u32).to_le_bytes())?;
        output.write_all(&(geometry.outcome as u32).to_le_bytes())?;
        for position in 0..ROLE_CONTEXT {
            let row = match self.candidates.get(position).zip(evidence.get(position)) {
                Some((candidate, score)) => [
                    u32::from(score.literal),
                    u32::from(score.collision),
                    score.distance.unwrap_or(NO_SCORE),
                    u32::from(exact.selected_facts.contains(&candidate.fact)),
                    u32::from(geometry.selected_facts.contains(&candidate.fact)),
                ],
                None => [0, 0, NO_SCORE, 0, 0],
            };
            for value in row {
                output.write_all(&value.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

/// D11 root coordinates are copied once, with no conversion beyond sign
/// extension into canonical little-endian integer words.
pub fn write_device_preamble(output: &mut impl Write) -> std::io::Result<()> {
    output.write_all(b"UORREL01")?;
    for root in spiralcore_orbit_roots() {
        for &coordinate in root {
            output.write_all(&i32::from(coordinate).to_le_bytes())?;
        }
    }
    Ok(())
}

fn packed_distance(fact: u32, query: u32) -> u32 {
    let roots = spiralcore_orbit_roots();
    (0..4)
        .map(|block| {
            let left = &roots[((fact >> (block * 4)) & 15) as usize];
            let right = &roots[((query >> (block * 4)) & 15) as usize];
            left.iter()
                .zip(right)
                .map(|(&a, &b)| {
                    let delta = i32::from(a) - i32::from(b);
                    (delta * delta) as u32
                })
                .sum::<u32>()
        })
        .sum()
}

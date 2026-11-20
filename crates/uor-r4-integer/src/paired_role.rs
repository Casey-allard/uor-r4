//! Sparse integer evidence at offsets from a candidate word in the D11 root lattice.
use crate::{invalid, Result};
use serde::Serialize;

pub const ROLE_VOCAB: usize = 4096;
pub const ROLE_CONTEXT: usize = 192;
pub const ROLE_OFFSETS: usize = 6;
pub const ROLE_ROOTS: usize = 65_537;
pub const ROLE_NIL: u32 = u32::MAX;
const MISSING_ROOT: u32 = (ROLE_ROOTS - 1) as u32;
pub const ROLE_STATE_MAGIC: &[u8; 8] = b"UORROLE1";
const MAX_CELL_COUNT: u32 = 8192 * ROLE_CONTEXT as u32;
const WEIGHTS: [i32; ROLE_OFFSETS] = [1, 2, 4, 4, 2, 1];
const MIN_EVIDENCE: i32 = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RoleSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairedRoleDecision {
    pub requested: Option<RoleSpan>,
    pub excluded: Option<RoleSpan>,
    pub requested_score: i32,
    pub excluded_score: i32,
    pub requested_runner_up: i32,
    pub excluded_runner_up: i32,
}

impl PairedRoleDecision {
    pub fn words(self) -> [u32; 8] {
        let range =
            |span: Option<RoleSpan>| span.map_or([ROLE_NIL; 2], |s| [s.start as u32, s.end as u32]);
        let [q0, q1] = range(self.requested);
        let [e0, e1] = range(self.excluded);
        [
            q0,
            q1,
            e0,
            e1,
            self.requested_score as u32,
            self.excluded_score as u32,
            self.requested_runner_up as u32,
            self.excluded_runner_up as u32,
        ]
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FrozenRoleCandidateTrace {
    pub span: [usize; 2],
    pub admitted: bool,
    pub feature_keys: [u32; ROLE_OFFSETS],
    pub scores: [i32; 2],
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestedOccurrenceCandidateTrace {
    pub span: [usize; 2],
    pub inherited: bool,
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    pub detached: bool,
    pub repeated_head: Option<[usize; 2]>,
    pub requested_admitted: bool,
    pub excluded_admitted: bool,
    pub scores: [i32; 2],
    pub feature_keys: [u32; ROLE_OFFSETS],
    /// Original full-prefix score pair when the requested rank used other keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_decision_scores: Option<[i32; 2]>,
    pub eligibility: RequestedEligibility,
    pub supporting_head: Option<[usize; 2]>,
    pub contrast: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clause_evidence: Option<ClauseEvidence>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClauseEvidence {
    pub span: [usize; 2],
    pub scores: [i32; 2],
    pub multiple_supported_mentions: bool,
}

/// Policy for carrying frozen role evidence to an assertion-supported occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestedContext {
    FullPrefix,
    UserTextOnly,
    ClauseToEntity([u32; 2]),
}

/// Bounded frozen-state role replay. Production uses the historical arm.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RequestedReplayArm {
    #[default]
    HistoricalInherited,
    HistoricalRepeated,
    FullInherited,
    FullRepeated,
}

impl RequestedReplayArm {
    fn full(self) -> bool {
        matches!(self, Self::FullInherited | Self::FullRepeated)
    }

    fn repeated(self) -> bool {
        matches!(self, Self::HistoricalRepeated | Self::FullRepeated)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedEligibility {
    NoBoundary,
    OutsideQuery,
    OutsideRegion,
    NoExactAssertionHead,
    Eligible,
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestedOccurrenceTrace {
    pub content: [usize; 2],
    pub query_boundary: Option<usize>,
    pub candidates: Vec<RequestedOccurrenceCandidateTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detached: Option<DetachedOccurrenceTrace>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DetachedOccurrenceTrace {
    pub query_candidates: usize,
    pub assertion_candidates: usize,
    pub raw_assertion_heads: Vec<[usize; 2]>,
    pub detached_assertion_heads: Vec<[usize; 2]>,
    pub matching_assertion_heads: Vec<[usize; 2]>,
    pub requested_is_detached: bool,
}

/// Tokenizer facts supplement the frozen word classes without altering their banks.
pub struct DetachedWordSites {
    classes: Vec<u32>,
}

impl DetachedWordSites {
    const BLANK: u32 = 1;
    const LEXICAL: u32 = 2;

    #[cfg(feature = "serving-wasm")]
    pub fn from_tokenizer(tokenizer: &uor_r4_tokenizer::ByteBpeTokenizer) -> Result<Self> {
        if tokenizer.vocab_size() != ROLE_VOCAB {
            return Err(invalid("detached role replay requires a V4096 tokenizer"));
        }
        let mut classes = vec![0; ROLE_VOCAB];
        for (id, class) in classes
            .iter_mut()
            .enumerate()
            .take(tokenizer.model_vocab_size())
        {
            let bytes = tokenizer.decode_bytes(&[id as u32]);
            *class = if bytes == b" " {
                Self::BLANK
            } else if !bytes.is_empty() && bytes.iter().all(u8::is_ascii_alphabetic) {
                Self::LEXICAL
            } else {
                0
            };
        }
        Ok(Self { classes })
    }

    pub fn classes(&self) -> &[u32] {
        &self.classes
    }

    fn span_at(&self, ids: &[u32], assertion_classes: &[u32], start: usize) -> Option<RoleSpan> {
        if self.classes[ids[start] as usize] != Self::BLANK {
            return None;
        }
        let lexical_pieces = ids[start + 1..]
            .iter()
            .take_while(|&&id| {
                self.classes[id as usize] == Self::LEXICAL
                    && assertion_classes[id as usize] == crate::span_selector::CONT
            })
            .count();
        (lexical_pieces != 0).then_some(RoleSpan {
            start,
            end: start + 1 + lexical_pieces,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PairedRoleState {
    /// Model, tokenizer, TRAIN tape and TRAIN labels, in that order.
    bindings: [[u8; 32]; 4],
    /// Root at case-folded token ID, paired with the original word-boundary class.
    sites: Vec<[u32; 2]>,
    /// Offset-cell key and counts in unspecified, requested, excluded order.
    cells: Vec<[u32; 4]>,
}

impl PairedRoleState {
    pub fn new(
        bindings: [[u8; 32]; 4],
        sites: Vec<[u32; 2]>,
        cells: Vec<[u32; 4]>,
    ) -> Result<Self> {
        if sites.len() != ROLE_VOCAB
            || sites.iter().any(|s| s[0] >= 65_536 || s[1] > 2)
            || cells.is_empty()
            || !cells.windows(2).all(|w| w[0][0] < w[1][0])
            || cells.iter().any(|c| {
                c[0] as usize >= ROLE_OFFSETS * ROLE_ROOTS
                    || c[1..].iter().any(|&n| n > MAX_CELL_COUNT)
                    || c[1..].iter().sum::<u32>() == 0
            })
        {
            return Err(invalid("invalid paired-role sites or count cells"));
        }
        Ok(Self {
            bindings,
            sites,
            cells,
        })
    }

    pub fn bindings(&self) -> &[[u8; 32]; 4] {
        &self.bindings
    }
    pub fn sites(&self) -> &[[u32; 2]] {
        &self.sites
    }
    pub fn cells(&self) -> &[[u32; 4]] {
        &self.cells
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        const HEADER: usize = 8 + 128 + ROLE_VOCAB * 8;
        if bytes.len() < HEADER
            || bytes.get(..8) != Some(ROLE_STATE_MAGIC)
            || (bytes.len() - HEADER) % 16 != 0
            || (bytes.len() - HEADER) / 16 > ROLE_OFFSETS * ROLE_ROOTS
        {
            return Err(invalid("invalid paired-role serialization"));
        }
        let mut bindings = [[0; 32]; 4];
        for (hash, src) in bindings.iter_mut().zip(bytes[8..136].chunks_exact(32)) {
            hash.copy_from_slice(src);
        }
        let word = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        let sites = bytes[136..HEADER]
            .chunks_exact(8)
            .map(|b| [word(b), word(&b[4..])])
            .collect();
        let cells = bytes[HEADER..]
            .chunks_exact(16)
            .map(|b| [word(b), word(&b[4..]), word(&b[8..]), word(&b[12..])])
            .collect();
        Self::new(bindings, sites, cells)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = ROLE_STATE_MAGIC.to_vec();
        bytes.extend(self.bindings.iter().flatten());
        for word in self
            .sites
            .iter()
            .flatten()
            .chain(self.cells.iter().flatten())
        {
            bytes.extend(word.to_le_bytes());
        }
        bytes
    }

    pub fn predict(&self, ids: &[u32]) -> Result<PairedRoleDecision> {
        self.predict_in_region(ids, 0..ids.len())
    }

    /// Diagnostic candidate filter over original token offsets. Feature reads
    /// and returned endpoints always address the full, unchanged prompt.
    pub fn predict_in_region(
        &self,
        ids: &[u32],
        region: core::ops::Range<usize>,
    ) -> Result<PairedRoleDecision> {
        self.predict_where(ids, region, |_| true)
    }

    pub fn predict_where(
        &self,
        ids: &[u32],
        region: core::ops::Range<usize>,
        mut admit: impl FnMut(RoleSpan) -> bool,
    ) -> Result<PairedRoleDecision> {
        if region.start > region.end || region.end > ids.len() {
            return Err(invalid("invalid paired-role candidate region"));
        }
        let mut ranks = [RoleRank::new(); 2];
        for (span, keys) in role_candidates(&self.sites, ids)? {
            if span.start < region.start || span.end > region.end || !admit(span) {
                continue;
            }
            let scores = self.scores(keys);
            for (role, rank) in ranks.iter_mut().enumerate() {
                if scores[role] > scores[1 - role] {
                    rank.update(span, scores[role]);
                }
            }
        }
        Ok(PairedRoleDecision {
            requested: ranks[0].winner(),
            excluded: ranks[1].winner(),
            requested_score: ranks[0].score,
            excluded_score: ranks[1].score,
            requested_runner_up: ranks[0].runner_up,
            excluded_runner_up: ranks[1].runner_up,
        })
    }

    /// Rank requested occurrences in the final clause with an earlier exact
    /// assertion head. The original excluded decision is retained.
    pub fn predict_assertion_agreement(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
    ) -> Result<PairedRoleDecision> {
        self.predict_requested_occurrence(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            RequestedContext::FullPrefix,
        )
    }

    pub fn predict_requested_occurrence(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
    ) -> Result<PairedRoleDecision> {
        self.replay_assertion_agreement(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            context,
            RequestedReplayArm::HistoricalInherited,
            None,
            |_| {},
        )
        .map(|(decision, _, _)| decision)
    }

    pub fn trace_assertion_agreement(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
    ) -> Result<RequestedOccurrenceTrace> {
        self.trace_requested_occurrence(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            RequestedContext::FullPrefix,
        )
    }

    pub fn trace_requested_occurrence(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
    ) -> Result<RequestedOccurrenceTrace> {
        self.requested_occurrence_replay(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            context,
            RequestedReplayArm::HistoricalInherited,
        )
        .map(|(_, trace)| trace)
    }

    pub fn requested_occurrence_replay(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
        arm: RequestedReplayArm,
    ) -> Result<(PairedRoleDecision, RequestedOccurrenceTrace)> {
        self.trace_occurrence_replay(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            context,
            arm,
            None,
        )
    }

    pub fn requested_detached_occurrence_replay(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
        sites: &DetachedWordSites,
    ) -> Result<(PairedRoleDecision, RequestedOccurrenceTrace)> {
        self.trace_occurrence_replay(
            ids,
            content,
            region,
            delimiters,
            assertion_classes,
            context,
            RequestedReplayArm::FullRepeated,
            Some(sites),
        )
    }

    fn trace_occurrence_replay(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
        arm: RequestedReplayArm,
        detached_sites: Option<&DetachedWordSites>,
    ) -> Result<(PairedRoleDecision, RequestedOccurrenceTrace)> {
        let mut candidates = Vec::new();
        let (decision, query_boundary, detached) = self.replay_assertion_agreement(
            ids,
            content.clone(),
            region,
            delimiters,
            assertion_classes,
            context,
            arm,
            detached_sites,
            |candidate| candidates.push(candidate),
        )?;
        Ok((
            decision,
            RequestedOccurrenceTrace {
                content: [content.start, content.end],
                query_boundary,
                candidates,
                detached,
            },
        ))
    }

    fn replay_assertion_agreement(
        &self,
        ids: &[u32],
        content: core::ops::Range<usize>,
        region: core::ops::Range<usize>,
        delimiters: &[u32],
        assertion_classes: &[u32],
        context: RequestedContext,
        arm: RequestedReplayArm,
        detached_sites: Option<&DetachedWordSites>,
        mut visit: impl FnMut(RequestedOccurrenceCandidateTrace),
    ) -> Result<(
        PairedRoleDecision,
        Option<usize>,
        Option<DetachedOccurrenceTrace>,
    )> {
        use RequestedEligibility::*;
        let candidates = role_candidates(&self.sites, ids)?;
        if content.start > content.end
            || content.end > ids.len()
            || region.start > region.end
            || region.end > ids.len()
            || assertion_classes.len() != ROLE_VOCAB
        {
            return Err(invalid("invalid requested occurrence state"));
        }

        let last_word = content
            .clone()
            .rfind(|&p| self.sites[ids[p] as usize][1] == 1);
        let historical_boundary = last_word
            .and_then(|last| (content.start..last).rfind(|&p| delimiters.contains(&ids[p])));
        let boundary = if arm.full() {
            None
        } else {
            historical_boundary
        };
        let detached_span = |start: usize| {
            detached_sites?
                .span_at(ids, assertion_classes, start)
                .filter(|span| span.end <= content.end)
        };
        let inherited_head = |start: usize| {
            use crate::span_selector::{bounds, START};
            if assertion_classes[ids[start] as usize] != START
                || !delimiters.contains(&ids[start - 1])
            {
                return None;
            }
            let end = bounds(ids, assertion_classes, start).1 + 1;
            (end <= content.end).then_some([start, end])
        };
        let detached_head = |start: usize| {
            if start > content.start && delimiters.contains(&ids[start - 1]) {
                let span = detached_span(start)?;
                Some([span.start, span.end])
            } else {
                None
            }
        };
        let assertion_head = |start| inherited_head(start).or_else(|| detached_head(start));
        let matching_heads = |span: RoleSpan, stop: usize| {
            (content.start.max(1)..stop)
                .filter_map(assertion_head)
                .filter(move |&[start, end]| {
                    end <= stop && ids[start..end] == ids[span.start..span.end]
                })
        };
        let supporting_head = |span: RoleSpan| {
            let stop = if arm.full() { span.start } else { boundary? };
            let query_start = boundary.map_or(content.start, |p| p + 1);
            (span.start >= query_start && span.end <= content.end)
                .then(|| matching_heads(span, stop).next())
                .flatten()
        };
        let mut repeated = std::collections::BTreeMap::new();
        if arm.repeated() {
            for [head_start, head_end] in
                (content.start.max(1)..content.end).filter_map(inherited_head)
            {
                let head = &ids[head_start..head_end];
                for (offset, occurrence) in
                    ids[head_end..content.end].windows(head.len()).enumerate()
                {
                    if occurrence == head {
                        let start = head_end + offset;
                        let end = start + head.len();
                        if end == ids.len()
                            || assertion_classes[ids[end] as usize] != crate::span_selector::CONT
                        {
                            repeated
                                .entry((start, end))
                                .or_insert(Some([head_start, head_end]));
                        }
                    }
                }
            }
        }
        let mut detached_candidates = 0;
        if detached_sites.is_some() {
            for span in content.clone().filter_map(detached_span) {
                detached_candidates += 1;
                repeated.insert((span.start, span.end), supporting_head(span));
            }
        }
        let mut supported_mentions = Vec::new();
        if matches!(context, RequestedContext::ClauseToEntity(_)) {
            supported_mentions.extend(
                role_spans(&self.sites, ids).filter(|&span| supporting_head(span).is_some()),
            );
            supported_mentions.extend(repeated.keys().filter_map(|&(start, end)| {
                let span = RoleSpan { start, end };
                (role_span_at(&self.sites, ids, start, ids[start]) != Some(span)
                    && supporting_head(span).is_some())
                .then_some(span)
            }));
        }
        let mut requested_rank = RoleRank::new();
        let mut excluded_rank = RoleRank::new();
        let inherited = candidates.map(|(span, keys)| {
            (
                span,
                keys,
                true,
                repeated.get(&(span.start, span.end)).copied().flatten(),
            )
        });
        let augmented = repeated.iter().filter_map(|(&(start, end), &head)| {
            let span = RoleSpan { start, end };
            (role_span_at(&self.sites, ids, start, ids[start]) != Some(span))
                .then(|| (span, role_keys(&self.sites, ids, span), false, head))
        });
        for (span, keys, inherited, repeated_head) in inherited.chain(augmented) {
            let full_scores = self.scores(keys);
            let [requested, excluded] = full_scores;
            let in_region = span.start >= region.start && span.end <= region.end;
            let excluded_admitted = in_region && excluded > requested;
            if excluded_admitted {
                excluded_rank.update(span, excluded);
            }
            let query_start = boundary.map_or(content.start, |p| p + 1);
            let (eligibility, head) = match () {
                _ if !arm.full() && boundary.is_none() => (NoBoundary, None),
                _ if span.start < query_start || span.end > content.end => (OutsideQuery, None),
                _ if !in_region => (OutsideRegion, None),
                _ => {
                    let head = supporting_head(span);
                    (
                        if head.is_some() {
                            Eligible
                        } else {
                            NoExactAssertionHead
                        },
                        head,
                    )
                }
            };
            let feature_keys = match context {
                RequestedContext::FullPrefix | RequestedContext::ClauseToEntity(_) => keys,
                RequestedContext::UserTextOnly => {
                    let offsets = role_offsets(span);
                    core::array::from_fn(|i| {
                        if content.contains(&offsets[i]) {
                            keys[i]
                        } else {
                            i as u32 * ROLE_ROOTS as u32 + MISSING_ROOT
                        }
                    })
                }
            };
            let scores = if feature_keys == keys {
                full_scores
            } else {
                self.scores(feature_keys)
            };
            let clause_evidence = match (context, head) {
                (RequestedContext::ClauseToEntity(separators), Some(_)) => {
                    Some(self.clause_evidence(
                        ids,
                        span,
                        query_start..content.end,
                        separators,
                        delimiters,
                        &supported_mentions,
                    ))
                }
                _ => None,
            };
            let contrast = match &clause_evidence {
                Some(clause) if clause.multiple_supported_mentions => None,
                Some(clause) => Some(clause.scores[0] - clause.scores[1]),
                None => head.map(|_| scores[0] - scores[1]),
            };
            if let Some(contrast) = contrast {
                requested_rank.update(span, contrast);
            }
            visit(RequestedOccurrenceCandidateTrace {
                span: [span.start, span.end],
                inherited,
                detached: content.contains(&span.start) && detached_span(span.start) == Some(span),
                repeated_head,
                requested_admitted: contrast.is_some(),
                excluded_admitted,
                scores,
                feature_keys,
                excluded_decision_scores: (feature_keys != keys).then_some(full_scores),
                eligibility,
                supporting_head: head,
                contrast,
                clause_evidence,
            });
        }
        let decision = PairedRoleDecision {
            requested: requested_rank
                .span
                .filter(|_| requested_rank.score > requested_rank.runner_up),
            excluded: excluded_rank.winner(),
            requested_score: requested_rank.score,
            excluded_score: excluded_rank.score,
            requested_runner_up: requested_rank.runner_up,
            excluded_runner_up: excluded_rank.runner_up,
        };
        let detached_trace = detached_sites.map(|_| {
            let raw_assertion_heads = (content.start.max(1)..content.end)
                .filter_map(assertion_head)
                .collect();
            let detached_assertion_heads: Vec<_> = (content.start.max(1)..content.end)
                .filter_map(detached_head)
                .collect();
            let matching_assertion_heads = decision
                .requested
                .map_or_else(Vec::new, |span| matching_heads(span, span.start).collect());
            DetachedOccurrenceTrace {
                query_candidates: detached_candidates,
                assertion_candidates: detached_assertion_heads.len(),
                raw_assertion_heads,
                detached_assertion_heads,
                matching_assertion_heads,
                requested_is_detached: decision
                    .requested
                    .is_some_and(|span| detached_span(span.start) == Some(span)),
            }
        });
        Ok((decision, boundary, detached_trace))
    }

    fn clause_evidence(
        &self,
        ids: &[u32],
        span: RoleSpan,
        query: core::ops::Range<usize>,
        separators: [u32; 2],
        sentence_delimiters: &[u32],
        supported_mentions: &[RoleSpan],
    ) -> ClauseEvidence {
        let start = (query.start..span.start)
            .rfind(|&position| separators.contains(&ids[position]))
            .map_or(query.start, |position| position + 1);
        let end = (span.end..query.end)
            .find(|&position| separators.contains(&ids[position]))
            .unwrap_or(query.end);
        let multiple_supported_mentions = supported_mentions
            .iter()
            .any(|other| *other != span && other.start >= start && other.end <= end);
        let mut scores = [0; 2];
        if !multiple_supported_mentions {
            for &id in ids[start..span.start]
                .iter()
                .chain(&ids[span.end..end])
                .filter(|id| !sentence_delimiters.contains(id))
            {
                let root = self.sites[id as usize][0];
                let keys = core::array::from_fn(|offset| offset as u32 * ROLE_ROOTS as u32 + root);
                let site = self.scores(keys);
                for (score, signed) in scores.iter_mut().zip(site) {
                    *score += signed;
                }
            }
        }
        ClauseEvidence {
            span: [start, end],
            scores,
            multiple_supported_mentions,
        }
    }

    /// Project a whole-name candidate onto the original six role offsets,
    /// replacing reads outside the clause with the missing root at each slot.
    /// Both ranges use absolute token offsets in the original tape. The caller
    /// supplies the clause boundaries. Scores are ordered requested, excluded.
    pub fn project_candidate_in_clause(
        &self,
        ids: &[u32],
        span: RoleSpan,
        clause: core::ops::Range<usize>,
    ) -> Result<([u32; ROLE_OFFSETS], [i32; 2])> {
        validate_role_ids(ids)?;
        if span.start < clause.start
            || span.start >= span.end
            || span.end > clause.end
            || clause.end > ids.len()
        {
            return Err(invalid("invalid paired-role candidate or clause"));
        }
        if role_span_at(&self.sites, ids, span.start, ids[span.start]) != Some(span) {
            return Err(invalid("invalid paired-role candidate span"));
        }
        // Rebase only the key reads. Slice bounds become the two missing-root fences.
        let local = RoleSpan {
            start: span.start - clause.start,
            end: span.end - clause.start,
        };
        let keys = role_keys(&self.sites, &ids[clause], local);
        Ok((keys, self.scores(keys)))
    }

    pub fn trace_where(
        &self,
        ids: &[u32],
        region: core::ops::Range<usize>,
        mut admit: impl FnMut(RoleSpan) -> bool,
    ) -> Result<Vec<FrozenRoleCandidateTrace>> {
        if region.start > region.end || region.end > ids.len() {
            return Err(invalid("invalid paired-role candidate region"));
        }
        Ok(role_candidates(&self.sites, ids)?
            .map(|(span, feature_keys)| FrozenRoleCandidateTrace {
                span: [span.start, span.end],
                admitted: span.start >= region.start && span.end <= region.end && admit(span),
                feature_keys,
                scores: self.scores(feature_keys),
            })
            .collect())
    }

    fn scores(&self, keys: [u32; ROLE_OFFSETS]) -> [i32; 2] {
        let mut scores = [0; 2];
        for (offset, key) in keys.into_iter().enumerate() {
            if let Ok(i) = self.cells.binary_search_by_key(&key, |c| c[0]) {
                let count = self.cells[i];
                let total = (count[1] + count[2] + count[3]) as i32;
                for (role, score) in scores.iter_mut().enumerate() {
                    let signed = 16 * count[role + 2] as i32 - 3 * total;
                    *score += WEIGHTS[offset] * (signed / (total + 2));
                }
            }
        }
        scores
    }
}

#[derive(Clone, Copy)]
struct RoleRank {
    span: Option<RoleSpan>,
    score: i32,
    runner_up: i32,
}
impl RoleRank {
    fn new() -> Self {
        Self {
            span: None,
            score: i32::MIN,
            runner_up: i32::MIN,
        }
    }
    fn update(&mut self, span: RoleSpan, score: i32) {
        if score > self.score {
            self.runner_up = self.score;
            self.score = score;
            self.span = Some(span);
        } else {
            self.runner_up = self.runner_up.max(score);
        }
    }
    fn winner(self) -> Option<RoleSpan> {
        if self.score >= MIN_EVIDENCE && self.score > self.runner_up {
            self.span
        } else {
            None
        }
    }
}

pub fn validate_role_ids(ids: &[u32]) -> Result<()> {
    if ids.first() != Some(&0)
        || ids.len() > ROLE_CONTEXT
        || ids[1..]
            .iter()
            .any(|&id| !(2..ROLE_VOCAB as u32).contains(&id))
    {
        return Err(invalid("invalid paired-role prompt tape"));
    }
    Ok(())
}

/// Validate the prompt and site table before exposing complete candidate spans
/// and their six feature keys. Only spans produced from this prompt reach the indexer.
pub fn role_candidates<'a>(
    sites: &'a [[u32; 2]],
    ids: &'a [u32],
) -> Result<impl Iterator<Item = (RoleSpan, [u32; ROLE_OFFSETS])> + 'a> {
    validate_role_ids(ids)?;
    if sites.len() != ROLE_VOCAB
        || ids.iter().any(|&id| {
            let [root, class] = sites[id as usize];
            root >= 65_536 || class > 2
        })
    {
        return Err(invalid("invalid candidate site geometry"));
    }
    Ok(role_spans(sites, ids).map(move |span| (span, role_keys(sites, ids, span))))
}

fn role_spans<'a>(sites: &'a [[u32; 2]], ids: &'a [u32]) -> impl Iterator<Item = RoleSpan> + 'a {
    ids.iter()
        .enumerate()
        .filter_map(move |(start, &id)| role_span_at(sites, ids, start, id))
}

fn role_span_at(sites: &[[u32; 2]], ids: &[u32], start: usize, id: u32) -> Option<RoleSpan> {
    if sites[id as usize][1] != 1 {
        return None;
    }
    let end = start
        + 1
        + ids[start + 1..]
            .iter()
            .take_while(|&&id| sites[id as usize][1] == 2)
            .count();
    Some(RoleSpan { start, end })
}

fn role_offsets(span: RoleSpan) -> [usize; ROLE_OFFSETS] {
    [
        span.start.wrapping_sub(3),
        span.start.wrapping_sub(2),
        span.start.wrapping_sub(1),
        span.end,
        span.end + 1,
        span.end + 2,
    ]
}

fn role_keys(sites: &[[u32; 2]], ids: &[u32], span: RoleSpan) -> [u32; ROLE_OFFSETS] {
    let at = role_offsets(span);
    core::array::from_fn(|i| {
        i as u32 * ROLE_ROOTS as u32
            + ids
                .get(at[i])
                .map_or(MISSING_ROOT, |&id| sites[id as usize][0])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_input_is_checked_before_iteration() {
        let mut sites = vec![[0, 0]; ROLE_VOCAB];
        assert!(role_candidates(&[], &[0]).is_err());
        assert!(role_candidates(&sites, &[0, ROLE_VOCAB as u32]).is_err());
        sites[7] = [9, 3];
        assert!(role_candidates(&sites, &[0, 7]).is_err());
        sites[7] = [9, 1];
        let candidates: Vec<_> = role_candidates(&sites, &[0, 7]).unwrap().collect();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, RoleSpan { start: 1, end: 2 });
    }

    #[test]
    fn full_repeated_combines_scope_and_whole_factual_head_recovery() {
        let mut sites = vec![[0, 0]; ROLE_VOCAB];
        sites[5][1] = 1;
        sites[8][0] = 8;
        sites[11][1] = 1;
        let excluded_key = 3 * ROLE_ROOTS as u32 + 8;
        let state =
            PairedRoleState::new([[0; 32]; 4], sites, vec![[excluded_key, 0, 0, 8]]).unwrap();
        let mut classes = vec![0; ROLE_VOCAB];
        classes[5] = crate::span_selector::START;
        classes[6] = crate::span_selector::CONT;
        classes[10] = crate::span_selector::CONT;
        let ids = [0, 2, 5, 6, 8, 2, 7, 5, 6, 8, 2, 6, 5, 6, 10, 2, 7, 11];
        for (arm, expected) in [
            (RequestedReplayArm::HistoricalInherited, None),
            (RequestedReplayArm::HistoricalRepeated, None),
            (RequestedReplayArm::FullInherited, None),
            (
                RequestedReplayArm::FullRepeated,
                Some(RoleSpan { start: 7, end: 9 }),
            ),
        ] {
            let (decision, trace) = state
                .requested_occurrence_replay(
                    &ids,
                    1..ids.len(),
                    0..ids.len(),
                    &[2],
                    &classes,
                    RequestedContext::FullPrefix,
                    arm,
                )
                .unwrap();
            assert_eq!(decision.requested, expected);
            assert_eq!(
                decision.excluded,
                arm.repeated().then_some(RoleSpan { start: 7, end: 9 }),
            );
            assert!(trace.candidates.iter().all(|row| row.span != [12, 14]));
            assert!(trace.candidates.iter().all(|row| row.span != [11, 12]));
            let recovered = trace.candidates.iter().find(|row| row.span == [7, 9]);
            assert_eq!(recovered.is_some(), arm.repeated());
            if let Some(row) = recovered {
                assert!(!row.inherited);
                assert_eq!(row.repeated_head, Some([2, 4]));
                assert_eq!(row.requested_admitted, arm.full());
                assert!(row.excluded_admitted);
            }
        }
        let (_, boundary_trace) = state
            .requested_occurrence_replay(
                &ids,
                1..14,
                0..ids.len(),
                &[2],
                &classes,
                RequestedContext::FullPrefix,
                RequestedReplayArm::FullRepeated,
            )
            .unwrap();
        assert!(
            boundary_trace
                .candidates
                .iter()
                .all(|row| row.span != [12, 14]),
            "a continuation outside user content still makes the match incomplete"
        );
    }

    #[test]
    fn detached_candidates_preserve_complete_original_occurrences_and_exact_witnesses() {
        const MARK: u32 = 2;
        const PUNCTUATION: u32 = 3;
        const BLANK: u32 = 4;
        const FIRST: u32 = 5;
        const LAST: u32 = 6;
        const LONGER: u32 = 7;
        const INCIDENTAL: u32 = 8;
        const REQUESTED: u32 = 9;
        const EXCLUDED: u32 = 10;
        let mut role_sites = vec![[0, 0]; ROLE_VOCAB];
        role_sites[REQUESTED as usize][0] = REQUESTED;
        role_sites[EXCLUDED as usize][0] = EXCLUDED;
        let before = 2 * ROLE_ROOTS as u32;
        let state = PairedRoleState::new(
            [[0; 32]; 4],
            role_sites,
            vec![[before + REQUESTED, 0, 8, 0], [before + EXCLUDED, 0, 0, 8]],
        )
        .unwrap();
        let mut assertions = vec![0; ROLE_VOCAB];
        let mut lexical = DetachedWordSites {
            classes: vec![0; ROLE_VOCAB],
        };
        lexical.classes[BLANK as usize] = DetachedWordSites::BLANK;
        for id in [FIRST, LAST, LONGER, PUNCTUATION] {
            assertions[id as usize] = crate::span_selector::CONT;
        }
        for id in [FIRST, LAST, LONGER] {
            lexical.classes[id as usize] = DetachedWordSites::LEXICAL;
        }
        let ids = [
            &[0, MARK, BLANK, FIRST, LAST, PUNCTUATION][..],
            &[MARK, BLANK, FIRST, LAST, PUNCTUATION],
            &[INCIDENTAL, BLANK, LAST, PUNCTUATION],
            &[INCIDENTAL, BLANK, FIRST, LAST, LONGER, PUNCTUATION],
            &[INCIDENTAL, BLANK, PUNCTUATION],
            &[INCIDENTAL, BLANK, BLANK, FIRST, LAST, PUNCTUATION],
            &[REQUESTED, BLANK, FIRST, LAST, PUNCTUATION],
            &[EXCLUDED, BLANK, FIRST, LAST, PUNCTUATION],
        ]
        .concat();
        let (decision, trace) = state
            .requested_detached_occurrence_replay(
                &ids,
                1..ids.len(),
                0..ids.len(),
                &[MARK],
                &assertions,
                RequestedContext::FullPrefix,
                &lexical,
            )
            .unwrap();
        assert_eq!(decision.requested, Some(RoleSpan { start: 31, end: 34 }));
        assert_eq!(decision.excluded, Some(RoleSpan { start: 36, end: 39 }));
        let candidate = |span| trace.candidates.iter().find(|c| c.span == span).unwrap();
        assert_eq!(candidate([12, 14]).supporting_head, None);
        assert_eq!(candidate([16, 20]).supporting_head, None);
        assert_eq!(candidate([26, 29]).supporting_head, Some([2, 5]));
        assert_eq!(candidate([31, 34]).repeated_head, Some([2, 5]));
        assert!(candidate([36, 39]).excluded_admitted);
        assert!(trace.candidates.iter().all(|c| c.detached));
        assert_eq!(trace.candidates.len(), 7);
        let audit = trace.detached.unwrap();
        assert_eq!(audit.query_candidates, 7);
        assert_eq!(audit.assertion_candidates, 2);
        assert_eq!(audit.detached_assertion_heads, [[2, 5], [7, 10]]);
        assert_eq!(audit.matching_assertion_heads, [[2, 5], [7, 10]]);
        assert!(audit.requested_is_detached);

        let (_, cropped) = state
            .requested_detached_occurrence_replay(
                &ids,
                2..19,
                0..ids.len(),
                &[MARK],
                &assertions,
                RequestedContext::FullPrefix,
                &lexical,
            )
            .unwrap();
        assert_eq!(
            cropped.detached.unwrap().detached_assertion_heads,
            [[7, 10]]
        );
        assert!(cropped.candidates.iter().all(|c| c.span[0] != 16));

        let (baseline, baseline_trace) = state
            .requested_occurrence_replay(
                &ids,
                1..ids.len(),
                0..ids.len(),
                &[MARK],
                &assertions,
                RequestedContext::FullPrefix,
                RequestedReplayArm::FullRepeated,
            )
            .unwrap();
        assert_eq!(baseline.words()[..4], [ROLE_NIL; 4]);
        assert!(baseline_trace.candidates.is_empty());
        assert!(baseline_trace.detached.is_none());
    }

    #[test]
    fn clause_projection_keeps_original_whole_name_and_masks_both_fences() {
        let mut sites = vec![[0, 0]; ROLE_VOCAB];
        for (id, site) in sites.iter_mut().enumerate().take(10) {
            site[0] = id as u32;
        }
        sites[5][1] = 1;
        sites[6][1] = 2;
        let slot = ROLE_ROOTS as u32;
        let state = PairedRoleState::new(
            [[0; 32]; 4],
            sites,
            vec![[2, 0, 8, 0], [MISSING_ROOT, 0, 0, 8]],
        )
        .unwrap();
        let ids = [0, 2, 3, 4, 5, 6, 7, 8, 9];
        let span = RoleSpan { start: 4, end: 6 };
        let original = state.trace_where(&ids, 0..ids.len(), |_| true).unwrap();
        assert_eq!(original.len(), 1);
        assert_eq!(original[0].span, [4, 6]);
        assert_eq!(original[0].scores, [10, -2]);
        assert_eq!(
            state.project_candidate_in_clause(&ids, span, 1..9).unwrap(),
            (original[0].feature_keys, original[0].scores),
        );
        assert_eq!(
            state.project_candidate_in_clause(&ids, span, 2..8).unwrap(),
            (
                [
                    MISSING_ROOT,
                    slot + 3,
                    2 * slot + 4,
                    3 * slot + 7,
                    4 * slot + 8,
                    5 * slot + MISSING_ROOT
                ],
                [-2, 10],
            ),
        );
        assert!(state
            .project_candidate_in_clause(&ids, RoleSpan { start: 4, end: 5 }, 2..5)
            .is_err());
        assert!(state
            .project_candidate_in_clause(&ids, RoleSpan { start: 5, end: 6 }, 2..8)
            .is_err());
        assert!(state.project_candidate_in_clause(&ids, span, 5..8).is_err());
        assert!(state
            .project_candidate_in_clause(&ids, span, 2..10)
            .is_err());
        assert!(state
            .project_candidate_in_clause(&ids[1..], span, 2..8)
            .is_err());
    }

    #[test]
    fn clause_projection_distinguishes_a_local_cue_and_collides_beyond_the_shared_tail() {
        let mut sites = vec![[0, 0]; ROLE_VOCAB];
        for (id, site) in sites.iter_mut().enumerate().take(12) {
            site[0] = id as u32;
        }
        sites[7][1] = 1;
        sites[8][1] = 2;
        let state =
            PairedRoleState::new([[0; 32]; 4], sites, vec![[2, 0, 8, 0], [3, 0, 0, 8]]).unwrap();
        let span = RoleSpan { start: 5, end: 7 };
        let project = |ids: &[u32]| state.project_candidate_in_clause(ids, span, 1..10).unwrap();
        let requested = project(&[0, 4, 2, 5, 6, 7, 8, 9, 10, 11]);
        let excluded = project(&[0, 4, 3, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!((requested.0[0], requested.1), (2, [10, -2]));
        assert_eq!((excluded.0[0], excluded.1), (3, [-2, 10]));
        assert_eq!(requested.0[1..], excluded.0[1..]);

        let requested = project(&[0, 2, 4, 5, 6, 7, 8, 9, 10, 11]);
        let excluded = project(&[0, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(requested.1, [0, 0]);
        assert_eq!(requested, excluded);
    }
}

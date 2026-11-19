//! Native D11 role classification over positional Cl(0,6) actions.
use core::ops::Range;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    discrete_d11::{
        cl06_action, pack_4block_roots, spiralcore_orbit_roots, spiralcore_signed_axes,
    },
    invalid,
    paired_role::{validate_role_ids, PairedRoleDecision, RoleSpan, ROLE_CONTEXT, ROLE_VOCAB},
    span_selector::{CONT, START},
    Result,
};

pub const ROLE_RECORD_BYTES: usize = 400;
pub const ROLE_DIMENSIONS: usize = 32;
pub const ROLE_RADIUS: usize = 12;
pub const ROLE_LIMIT: i16 = 48;
pub const ROLE_ORDERED_MAGIC: &[u8; 8] = b"UORORL02";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum RoleMode {
    Identity = 0,
    Positional = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[repr(u8)]
pub enum RoleClass {
    Requested = 0,
    Excluded = 1,
    Neither = 2,
}

const CLASSES: [RoleClass; 3] = [
    RoleClass::Requested,
    RoleClass::Excluded,
    RoleClass::Neither,
];

#[derive(Clone, Copy)]
enum Sites<'a> {
    Packed(&'a [u32], &'a [u32]),
    D11(&'a [[u8; 4]], &'a [u8]),
    Frozen(&'a [[u8; 4]], &'a [[u32; 2]]),
}

#[derive(Clone, Copy)]
pub struct RoleSites<'a>(Sites<'a>);

impl<'a> RoleSites<'a> {
    pub fn new(roots: &'a [u32], classes: &'a [u32]) -> Result<Self> {
        if roots.len() != ROLE_VOCAB
            || classes.len() != ROLE_VOCAB
            || roots.iter().any(|&root| root > u16::MAX as u32)
            || classes.iter().any(|&class| class > CONT)
        {
            return Err(invalid("invalid ordered-role site tables"));
        }
        Ok(Self(Sites::Packed(roots, classes)))
    }

    pub(crate) fn from_d11(roots: &'a [[u8; 4]], classes: &'a [u8]) -> Self {
        Self(Sites::D11(roots, classes))
    }

    pub(crate) fn from_frozen(roots: &'a [[u8; 4]], sites: &'a [[u32; 2]]) -> Self {
        Self(Sites::Frozen(roots, sites))
    }

    fn class(self, id: u32) -> u32 {
        match self.0 {
            Sites::Packed(_, classes) => classes[id as usize],
            Sites::D11(_, classes) => u32::from(classes[id as usize]),
            Sites::Frozen(_, sites) => sites[id as usize][1],
        }
    }

    fn root(self, id: u32) -> u16 {
        match self.0 {
            Sites::Packed(roots, _) => roots[id as usize] as u16,
            Sites::D11(roots, _) | Sites::Frozen(roots, _) => pack_4block_roots(roots[id as usize]),
        }
    }

    pub fn word_classes(self) -> impl Iterator<Item = u8> + 'a {
        (0..ROLE_VOCAB as u32).map(move |id| self.class(id) as u8)
    }

    pub fn packed_roots(self) -> impl Iterator<Item = u32> + 'a {
        (0..ROLE_VOCAB as u32).map(move |id| u32::from(self.root(id)))
    }

    pub fn prompt(self, ids: &'a [u32], content: Range<usize>) -> Result<RolePrompt<'a>> {
        validate_role_ids(ids)?;
        if content.start == 0 || content.start > content.end || content.end > ids.len() {
            return Err(invalid("invalid ordered-role prompt framing"));
        }
        Ok(RolePrompt {
            sites: self,
            ids,
            content,
        })
    }
}

/// A validated prompt with structural slots identified by their absolute
/// positions, preserving the original BOS-inclusive tape.
pub struct RolePrompt<'a> {
    sites: RoleSites<'a>,
    ids: &'a [u32],
    content: Range<usize>,
}

impl RolePrompt<'_> {
    pub fn candidates(&self) -> impl Iterator<Item = RoleSpan> + '_ {
        self.content.clone().filter_map(move |start| {
            (self.sites.class(self.ids[start]) == START).then(|| RoleSpan {
                start,
                end: self.candidate_end(start),
            })
        })
    }

    fn candidate_end(&self, start: usize) -> usize {
        start
            + 1
            + self.ids[start + 1..self.content.end]
                .iter()
                .take_while(|&&id| self.sites.class(id) == CONT)
                .count()
    }

    fn validate_region(&self, region: &Range<usize>) -> Result<()> {
        if region.start > region.end || region.end > self.ids.len() {
            return Err(invalid("invalid ordered-role eligibility region"));
        }
        Ok(())
    }

    fn admit(&self, span: RoleSpan, region: &Range<usize>) -> bool {
        span.start >= region.start && span.end <= region.end
    }

    fn positions(&self, span: RoleSpan, region: &Range<usize>) -> [Option<usize>; 2 * ROLE_RADIUS] {
        core::array::from_fn(|slot| {
            let position = if slot < ROLE_RADIUS {
                span.start.checked_sub(1 + slot)?
            } else {
                span.end + slot - ROLE_RADIUS
            };
            (region.contains(&position) && self.content.contains(&position)).then_some(position)
        })
    }

    /// Nearest to farthest left slots, followed by nearest to farthest right.
    /// The optional locations index the unmodified input tokens.
    pub fn feature_positions(
        &self,
        span: RoleSpan,
        region: Range<usize>,
    ) -> Result<[Option<usize>; 2 * ROLE_RADIUS]> {
        self.validate_region(&region)?;
        if !self.content.contains(&span.start)
            || self.sites.class(self.ids[span.start]) != START
            || self.candidate_end(span.start) != span.end
            || !self.admit(span, &region)
        {
            return Err(invalid(
                "ordered-role feature requires a complete eligible span",
            ));
        }
        Ok(self.positions(span, &region))
    }

    fn extract(
        &self,
        mode: RoleMode,
        positions: [Option<usize>; 2 * ROLE_RADIUS],
    ) -> [i16; ROLE_DIMENSIONS] {
        let mut feature = [0; ROLE_DIMENSIONS];
        for (slot, position) in positions.into_iter().enumerate() {
            let Some(position) = position else { continue };
            let roots = self.sites.root(self.ids[position]);
            let state = match mode {
                RoleMode::Identity => 0,
                RoleMode::Positional if slot < ROLE_RADIUS => 1 + slot,
                RoleMode::Positional => 33 + slot - ROLE_RADIUS,
            };
            for (block, coordinates) in feature.chunks_exact_mut(8).enumerate() {
                let root = (roots >> (block * 4)) as usize & 15;
                let anchor = spiralcore_orbit_roots()[root].map(i32::from);
                for (total, value) in coordinates.iter_mut().zip(cl06_action(state, &anchor)) {
                    *total += value as i16;
                }
            }
        }
        feature
    }

    pub fn feature(
        &self,
        mode: RoleMode,
        span: RoleSpan,
        region: Range<usize>,
    ) -> Result<[i16; ROLE_DIMENSIONS]> {
        Ok(self.extract(mode, self.feature_positions(span, region)?))
    }

    /// Score eligible spans with a constant amount of scratch independent of the prefix length.
    pub fn predict(
        &self,
        mode: RoleMode,
        prototypes: &RolePrototypes,
        region: Range<usize>,
    ) -> Result<PairedRoleDecision> {
        self.validate_region(&region)?;
        let scored = self
            .candidates()
            .filter(|&span| self.admit(span, &region))
            .map(|span| {
                let feature = self.extract(mode, self.positions(span, &region));
                (span, prototypes.score_feature(&feature))
            });
        Ok(rank_roles(scored))
    }

    /// Diagnostic view. Excluded-by-region spans retain their original endpoints.
    pub fn trace(
        &self,
        mode: RoleMode,
        prototypes: &RolePrototypes,
        region: Range<usize>,
    ) -> Result<Vec<RoleCandidateTrace>> {
        self.validate_region(&region)?;
        Ok(self
            .candidates()
            .map(|span| {
                let admitted = self.admit(span, &region);
                let positions = admitted.then(|| self.positions(span, &region));
                let feature = positions.map(|slots| self.extract(mode, slots));
                let score = feature.as_ref().map(|x| prototypes.score_feature(x));
                RoleCandidateTrace {
                    span: [span.start, span.end],
                    admitted,
                    positions,
                    feature,
                    score,
                }
            })
            .collect())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RoleCandidateTrace {
    pub span: [usize; 2],
    pub admitted: bool,
    pub positions: Option<[Option<usize>; 2 * ROLE_RADIUS]>,
    pub feature: Option<[i16; ROLE_DIMENSIONS]>,
    pub score: Option<RoleScore>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RoleScore {
    pub distances: [u16; 3],
    pub winner: Option<RoleClass>,
    pub evidence: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RoleUpdate {
    pub distances: [u16; 3],
    pub rival: RoleClass,
    pub applied: bool,
    pub clamp_low: u16,
    pub clamp_high: u16,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RolePrototypes([[i16; ROLE_DIMENSIONS]; 3]);

impl RolePrototypes {
    pub fn new(coordinates: [[i16; ROLE_DIMENSIONS]; 3]) -> Result<Self> {
        for point in &coordinates {
            validate_point(point)?;
        }
        Ok(Self(coordinates))
    }

    pub fn coordinates(&self) -> &[[i16; ROLE_DIMENSIONS]; 3] {
        &self.0
    }

    fn distances(&self, feature: &[i16; ROLE_DIMENSIONS]) -> [u16; 3] {
        self.0.each_ref().map(|point| {
            point
                .iter()
                .zip(feature)
                .map(|(&a, &b)| a.abs_diff(b))
                .sum()
        })
    }

    fn score_feature(&self, feature: &[i16; ROLE_DIMENSIONS]) -> RoleScore {
        let distances = self.distances(feature);
        let mut ranked = CLASSES;
        ranked.sort_unstable_by_key(|&class| distances[class as usize]);
        let evidence = distances[ranked[1] as usize] - distances[ranked[0] as usize];
        RoleScore {
            distances,
            winner: (evidence > 0).then_some(ranked[0]),
            evidence,
        }
    }

    pub fn score(&self, feature: &[i16; ROLE_DIMENSIONS]) -> Result<RoleScore> {
        validate_point(feature)?;
        Ok(self.score_feature(feature))
    }

    pub fn update(
        &mut self,
        feature: &[i16; ROLE_DIMENSIONS],
        gold: RoleClass,
    ) -> Result<RoleUpdate> {
        validate_point(feature)?;
        let distances = self.distances(feature);
        let rival = match gold {
            RoleClass::Requested if distances[1] <= distances[2] => RoleClass::Excluded,
            RoleClass::Requested => RoleClass::Neither,
            RoleClass::Excluded if distances[0] <= distances[2] => RoleClass::Requested,
            RoleClass::Excluded => RoleClass::Neither,
            RoleClass::Neither if distances[0] <= distances[1] => RoleClass::Requested,
            RoleClass::Neither => RoleClass::Excluded,
        };
        let applied = distances[rival as usize] <= distances[gold as usize];
        let mut result = RoleUpdate {
            distances,
            rival,
            applied,
            clamp_low: 0,
            clamp_high: 0,
        };
        if applied {
            for (class, direction) in [(gold, 1), (rival, -1)] {
                for (at, &target) in self.0[class as usize].iter_mut().zip(feature) {
                    let stepped = *at + direction * (target - *at).signum();
                    result.clamp_low += u16::from(stepped < -ROLE_LIMIT);
                    result.clamp_high += u16::from(stepped > ROLE_LIMIT);
                    *at = stepped.clamp(-ROLE_LIMIT, ROLE_LIMIT);
                }
            }
        }
        Ok(result)
    }
}

fn validate_point(point: &[i16; ROLE_DIMENSIONS]) -> Result<()> {
    if point
        .iter()
        .any(|coordinate| !(-ROLE_LIMIT..=ROLE_LIMIT).contains(coordinate))
    {
        return Err(invalid("ordered-role lattice coordinate out of bounds"));
    }
    Ok(())
}

/// Use the same tie and abstention rule for public inference and diagnostic reranking.
pub fn rank_roles(scored: impl IntoIterator<Item = (RoleSpan, RoleScore)>) -> PairedRoleDecision {
    let mut ranks = [(None, i32::MIN, i32::MIN); 2];
    for (span, score) in scored {
        let Some(class @ (RoleClass::Requested | RoleClass::Excluded)) = score.winner else {
            continue;
        };
        let (best_span, best, runner_up) = &mut ranks[class as usize];
        let evidence = i32::from(score.evidence);
        if evidence > *best {
            (*best_span, *best, *runner_up) = (Some(span), evidence, *best);
        } else {
            *runner_up = (*runner_up).max(evidence);
        }
    }
    let [(request, requested_score, requested_runner_up), (exclude, excluded_score, excluded_runner_up)] =
        ranks;
    PairedRoleDecision {
        requested: request.filter(|_| requested_score > 0 && requested_score > requested_runner_up),
        excluded: exclude.filter(|_| excluded_score > 0 && excluded_score > excluded_runner_up),
        requested_score,
        excluded_score,
        requested_runner_up,
        excluded_runner_up,
    }
}

/// Serialized bank provenance order is base D11, tokenizer, word classes,
/// orbit/action tables, ordered TRAIN receipt and operator contract.
pub type RoleBindings = [[u8; 32]; 6];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleBank {
    pub mode: RoleMode,
    pub prototypes: RolePrototypes,
    bindings: RoleBindings,
}

impl RoleBank {
    pub fn new(mode: RoleMode, prototypes: RolePrototypes, bindings: RoleBindings) -> Self {
        Self {
            mode,
            prototypes,
            bindings,
        }
    }

    pub fn bindings(&self) -> &RoleBindings {
        &self.bindings
    }

    pub fn verify_bindings(&self, expected: &RoleBindings) -> Result<()> {
        if &self.bindings != expected {
            return Err(invalid("ordered-role bank binding mismatch"));
        }
        Ok(())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != ROLE_RECORD_BYTES
            || bytes.get(..8) != Some(ROLE_ORDERED_MAGIC)
            || bytes[9..16].iter().any(|&value| value != 0)
        {
            return Err(invalid("invalid ordered-role bank record"));
        }
        let mode = match bytes[8] {
            0 => RoleMode::Identity,
            1 => RoleMode::Positional,
            _ => return Err(invalid("unknown ordered-role bank mode")),
        };
        let mut bindings = [[0; 32]; 6];
        for (binding, source) in bindings.iter_mut().zip(bytes[16..208].chunks_exact(32)) {
            binding.copy_from_slice(source);
        }
        let mut coordinates = [[0; ROLE_DIMENSIONS]; 3];
        for (word, source) in coordinates
            .iter_mut()
            .flatten()
            .zip(bytes[208..].chunks_exact(2))
        {
            *word = i16::from_le_bytes([source[0], source[1]]);
        }
        Ok(Self::new(mode, RolePrototypes::new(coordinates)?, bindings))
    }

    pub fn to_bytes(&self) -> [u8; ROLE_RECORD_BYTES] {
        let mut bytes = [0; ROLE_RECORD_BYTES];
        bytes[..8].copy_from_slice(ROLE_ORDERED_MAGIC);
        bytes[8] = self.mode as u8;
        for (dest, binding) in bytes[16..208].chunks_exact_mut(32).zip(&self.bindings) {
            dest.copy_from_slice(binding);
        }
        for (dest, word) in bytes[208..]
            .chunks_exact_mut(2)
            .zip(self.prototypes.0.iter().flatten())
        {
            dest.copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

pub fn orbit_action_digest() -> [u8; 32] {
    let mut digest = Sha256::new();
    for root in spiralcore_orbit_roots() {
        digest.update(root.map(|coordinate| coordinate as u8));
    }
    digest.update(spiralcore_signed_axes());
    digest.finalize().into()
}

/// Framing comes from literal-role protocol 2 and is bound to the operator.
/// The separate empty-prompt shape is the encoder's all-skipped path.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleFraming {
    prefix: Vec<u32>,
    suffix: Vec<u32>,
    empty: Vec<u32>,
}

impl RoleFraming {
    #[cfg(feature = "serving-wasm")]
    pub fn from_tokenizer(tokenizer: &uor_r4_tokenizer::ByteBpeTokenizer) -> Result<Self> {
        use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};

        let protocol = DialogueProtocol::literal_roles_v2(tokenizer)
            .map_err(|error| invalid(error.to_string()))?;
        let encoder = protocol
            .bind(tokenizer)
            .map_err(|error| invalid(error.to_string()))?;
        let probe = [Message {
            role: "user",
            content: "x",
        }];
        let full = encoder.encode_assistant_prefix(&probe).tokens;
        let history = encoder.encode_open_history(&probe).tokens;
        let encoded_content = tokenizer.encode(" x");
        let prefix = history
            .strip_suffix(encoded_content.as_slice())
            .ok_or_else(|| invalid("literal role framing content boundary mismatch"))?;
        let suffix = full
            .strip_prefix(history.as_slice())
            .ok_or_else(|| invalid("literal role framing assistant boundary mismatch"))?;
        let empty = encoder.encode_assistant_prefix(&[]).tokens;
        Self::new(prefix.to_vec(), suffix.to_vec(), empty)
    }

    pub fn new(prefix: Vec<u32>, suffix: Vec<u32>, empty: Vec<u32>) -> Result<Self> {
        validate_role_ids(&prefix)?;
        validate_role_ids(&empty)?;
        if prefix.len() < 2
            || empty.len() < 2
            || suffix.is_empty()
            || suffix.iter().any(|id| !(2..ROLE_VOCAB as u32).contains(id))
            || prefix.len().saturating_add(suffix.len()) >= ROLE_CONTEXT
        {
            return Err(invalid("invalid ordered-role literal framing"));
        }
        Ok(Self {
            prefix,
            suffix,
            empty,
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        Self::new(self.prefix.clone(), self.suffix.clone(), self.empty.clone()).map(|_| ())
    }

    pub fn content(&self, ids: &[u32]) -> Result<Range<usize>> {
        if ids == self.empty {
            return Ok(1..1);
        }
        let content = ids
            .strip_prefix(self.prefix.as_slice())
            .and_then(|ids| ids.strip_suffix(self.suffix.as_slice()))
            .filter(|ids| !ids.is_empty())
            .ok_or_else(|| {
                invalid("ordered-role prompt does not match literal protocol 2 framing")
            })?;
        Ok(self.prefix.len()..self.prefix.len() + content.len())
    }

    pub fn operator_digest(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"uor-r4.ordered-role/2;d11-low-nibble-first;4x8;radius=12;L=1+d,R=33+d,I=0;original-position/content-mask/clipped-range;L1-i16;unique-class-and-role;unit-step/-48:48;literal-protocol-2");
        for segment in [&self.prefix, &self.suffix, &self.empty] {
            digest.update((segment.len() as u32).to_le_bytes());
            for id in segment {
                digest.update(id.to_le_bytes());
            }
        }
        digest.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_keep_absolute_offsets_and_skip_structural_slots() {
        let mut roots = [0u32; ROLE_VOCAB];
        roots[6] = 0x1234;
        roots[7] = 0x5678;
        let mut classes = [0u32; ROLE_VOCAB];
        classes[3] = START;
        classes[4] = CONT;
        classes[5] = START;
        let ids = [0, 5, 6, 3, 4, 7, 5];
        let prompt = RoleSites::new(&roots, &classes)
            .unwrap()
            .prompt(&ids, 2..6)
            .unwrap();
        let span = RoleSpan { start: 3, end: 5 };
        assert_eq!(prompt.candidates().collect::<Vec<_>>(), vec![span]);
        let slots = prompt.feature_positions(span, 0..7).unwrap();
        let mut expected = [None; 24];
        expected[0] = Some(2);
        expected[12] = Some(5);
        assert_eq!(slots, expected);
        assert!(prompt.feature_positions(span, 4..6).is_err());
        let clipped = prompt.feature(RoleMode::Identity, span, 3..6).unwrap();
        assert_eq!(&clipped[..8], &spiralcore_orbit_roots()[8].map(i16::from));
        let full = prompt.feature(RoleMode::Identity, span, 0..7).unwrap();
        let transformed = prompt.feature(RoleMode::Positional, span, 0..7).unwrap();
        for (block, (full, transformed)) in full
            .chunks_exact(8)
            .zip(transformed.chunks_exact(8))
            .enumerate()
        {
            let left = spiralcore_orbit_roots()[4 - block];
            let right = spiralcore_orbit_roots()[8 - block];
            for axis in 0..8 {
                assert_eq!(full[axis], i16::from(left[axis]) + i16::from(right[axis]));
                let signed = |state: usize, root: [i8; 8]| {
                    let code = spiralcore_signed_axes()[state * 8 + axis];
                    i16::from(root[usize::from(code & 7)]) * if code & 8 == 0 { 1 } else { -1 }
                };
                assert_eq!(transformed[axis], signed(1, left) + signed(33, right));
            }
        }
    }

    #[test]
    fn distance_and_update_preserve_lattice_limits() {
        let mut roots = [0; ROLE_VOCAB];
        let mut classes = [0; ROLE_VOCAB];
        classes[5] = START;
        // Reach +48 on two axes with 24 identical root-0 neighbours.
        let mut ids = [6; 26];
        ids[0] = 0;
        ids[13] = 5;
        let span = RoleSpan { start: 13, end: 14 };
        let feature = RoleSites::new(&roots, &classes)
            .unwrap()
            .prompt(&ids, 1..26)
            .unwrap()
            .feature(RoleMode::Identity, span, 0..26)
            .unwrap();
        assert_eq!(&feature[..8], &[48, 48, 0, 0, 0, 0, 0, 0]);
        let mut prototypes = RolePrototypes::new([[-48; 32], [48; 32], [48; 32]]).unwrap();
        let score = prototypes.score(&[48; 32]).unwrap();
        assert_eq!(score.distances, [3_072, 0, 0]);
        assert_eq!(score.winner, None);
        let step = prototypes.update(&[-48; 32], RoleClass::Excluded).unwrap();
        assert_eq!(step.distances, [0, 3_072, 3_072]);
        assert_eq!(step.rival, RoleClass::Requested);
        assert!(step.applied);
        assert_eq!(step.clamp_low, 0);
        assert_eq!(step.clamp_high, 0);
        assert_eq!(prototypes.coordinates()[0], [-48; 32]);
        assert_eq!(prototypes.coordinates()[1], [47; 32]);

        let mut positive = RolePrototypes::new([[48; 32]; 3]).unwrap();
        let step = positive.update(&[0; 32], RoleClass::Requested).unwrap();
        assert_eq!(step.rival, RoleClass::Excluded);
        assert_eq!((step.clamp_low, step.clamp_high), (0, 32));
        let mut negative = RolePrototypes::new([[-48; 32]; 3]).unwrap();
        let step = negative.update(&[0; 32], RoleClass::Requested).unwrap();
        assert_eq!((step.clamp_low, step.clamp_high), (32, 0));
        let frozen = negative.clone();
        assert!(
            !negative
                .update(&[0; 32], RoleClass::Requested)
                .unwrap()
                .applied
        );
        assert_eq!(negative, frozen);
        assert!(negative.update(&[49; 32], RoleClass::Requested).is_err());
        roots[5] = 65_536;
        assert!(RoleSites::new(&roots, &classes).is_err());
    }

    #[test]
    fn class_and_competitor_ties_abstain_independently() {
        let zero = RolePrototypes::default().score(&[0; 32]).unwrap();
        assert_eq!((zero.winner, zero.evidence), (None, 0));
        let score = |winner, evidence| RoleScore {
            distances: [0; 3],
            winner: Some(winner),
            evidence,
        };
        let sites: Vec<_> = (1..=3)
            .map(|start| RoleSpan {
                start,
                end: start + 1,
            })
            .collect();
        let ranked = rank_roles([
            (sites[0], score(RoleClass::Requested, 3)),
            (sites[1], score(RoleClass::Requested, 3)),
            (sites[2], score(RoleClass::Excluded, 2)),
        ]);
        assert_eq!(ranked.requested, None);
        assert_eq!((ranked.requested_score, ranked.requested_runner_up), (3, 3));
        assert_eq!(ranked.excluded, Some(sites[2]));
        assert_eq!(rank_roles([]).requested_score, i32::MIN);
    }

    #[test]
    fn ordered_record_is_fixed_and_rejects_malformed_or_wrong_bindings() {
        let mut coordinates = [[0; 32]; 3];
        coordinates[0][0] = -48;
        coordinates[2][31] = 47;
        let frozen = [[1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32]];
        let bank = RoleBank::new(
            RoleMode::Positional,
            RolePrototypes::new(coordinates).unwrap(),
            frozen,
        );
        let record = bank.to_bytes();
        assert_eq!(record.len(), 400);
        assert_eq!(&record[..8], b"UORORL02");
        assert_eq!(&record[208..210], &(-48_i16).to_le_bytes());
        assert_eq!(&record[398..], &47_i16.to_le_bytes());
        let reopened = RoleBank::from_bytes(&record).unwrap();
        assert_eq!(reopened, bank);
        assert_eq!(reopened.to_bytes(), record);
        assert!(reopened.verify_bindings(&frozen).is_ok());
        for index in 0..6 {
            let mut wrong = frozen;
            wrong[index][0] ^= 1;
            assert!(reopened.verify_bindings(&wrong).is_err());
        }
        assert!(RoleBank::from_bytes(&record[..399]).is_err());
        assert!(RoleBank::from_bytes(&[record.as_slice(), &[0]].concat()).is_err());
        for (offset, value) in [(0, b'x'), (8, 2), (15, 1), (398, 49)] {
            let mut corrupt = record;
            corrupt[offset] = value;
            assert!(RoleBank::from_bytes(&corrupt).is_err());
        }
    }
}

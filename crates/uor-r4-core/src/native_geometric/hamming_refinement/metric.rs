//! Hamming relevance over artifact-bound hemisphere predicates, not hash identity.
//! Construction and the exhaustive census are host work. Distance evaluation uses
//! two fixed root lookups per lane, XOR, popcount and integer addition only.
use crate::native_geometric::addressed_attention::artifact::BoundGeometry;
use serde::{Deserialize, Serialize};

pub const ROOTS: usize = 120;
pub const SIGNATURE_BITS: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetricError {
    Geometry,
    RootOutOfRange,
    SignaturePadding,
}
impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MetricError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metric {
    signatures: [[u64; 2]; ROOTS],
    geometry_digest: [u8; 32],
}
impl Metric {
    pub fn new(geometry: &BoundGeometry) -> Result<Self, MetricError> {
        let mut signatures = [[0u64; 2]; ROOTS];
        for (root, signature) in signatures.iter_mut().enumerate() {
            *signature = geometry
                .signature(root as u16)
                .map_err(|_| MetricError::Geometry)?;
        }
        Self::from_signatures(signatures, geometry.identity_digest())
    }

    fn from_signatures(
        signatures: [[u64; 2]; ROOTS],
        geometry_digest: [u8; 32],
    ) -> Result<Self, MetricError> {
        if signatures.iter().any(|s| s[1] >> 56 != 0) {
            return Err(MetricError::SignaturePadding);
        }
        Ok(Self {
            signatures,
            geometry_digest,
        })
    }

    pub fn geometry_digest(&self) -> [u8; 32] {
        self.geometry_digest
    }

    /// Number of unequal hemisphere predicates, in 0..=120.
    pub fn root_distance(&self, a: u16, b: u16) -> Result<u16, MetricError> {
        let a = self
            .signatures
            .get(usize::from(a))
            .ok_or(MetricError::RootOutOfRange)?;
        let b = self
            .signatures
            .get(usize::from(b))
            .ok_or(MetricError::RootOutOfRange)?;
        Ok(((a[0] ^ b[0]).count_ones() + (a[1] ^ b[1]).count_ones()) as u16)
    }

    /// Ordered concatenation of two root signatures; range 0..=240.
    /// The lanes are distinct contextual roots, not a claimed paired-H4 lift.
    pub fn distance(&self, q: [u16; 2], k: [u16; 2]) -> Result<u16, MetricError> {
        Ok(self.root_distance(q[0], k[0])? + self.root_distance(q[1], k[1])?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankingCensus {
    /// For each query, all unordered pairs of distinct candidate roots.
    pub query_candidate_pair_comparisons: usize,
    pub same_strict_preference: usize,
    pub opposite_strict_preference: usize,
    pub both_tied: usize,
    pub angular_tied_hamming_strict: usize,
    pub hamming_tied_angular_strict: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricCensus {
    pub geometry_digest: [u8; 32],
    pub roots: usize,
    pub signature_bits: usize,
    pub distinct_signatures: usize,
    /// Unequal root IDs with equal signatures; (a,b) and (b,a) count separately.
    pub colliding_ordered_distinct_root_pairs: usize,
    pub ordered_root_pairs: usize,
    pub reflexive: bool,
    pub symmetric: bool,
    pub distance_histogram: Vec<usize>,
    pub signature_population_histogram: Vec<usize>,
    /// Minimum signed angular-rank partners, enumerated per query. The separate
    /// checks establish whether these are unique and negate the self score.
    pub minimum_angular_partner_pairs: usize,
    pub unique_minimum_angular_partner_for_every_root: bool,
    pub minimum_angular_score_negates_self_for_every_root: bool,
    pub minimum_angular_partner_hamming_histogram: Vec<usize>,
    pub ranking: RankingCensus,
}

/// Geometry-only census. Counts 14,400 ordered root pairs and 856,800 ranking
/// comparisons. This measures the existing codebook, not linguistic relevance.
pub fn census(geometry: &BoundGeometry) -> Result<MetricCensus, MetricError> {
    let metric = Metric::new(geometry)?;
    let mut unique = metric.signatures.to_vec();
    unique.sort_unstable();
    unique.dedup();
    let mut result = MetricCensus {
        geometry_digest: metric.geometry_digest(),
        roots: ROOTS,
        signature_bits: SIGNATURE_BITS,
        distinct_signatures: unique.len(),
        colliding_ordered_distinct_root_pairs: 0,
        ordered_root_pairs: 0,
        reflexive: true,
        symmetric: true,
        distance_histogram: vec![0; SIGNATURE_BITS + 1],
        signature_population_histogram: vec![0; SIGNATURE_BITS + 1],
        minimum_angular_partner_pairs: 0,
        unique_minimum_angular_partner_for_every_root: true,
        minimum_angular_score_negates_self_for_every_root: true,
        minimum_angular_partner_hamming_histogram: vec![0; SIGNATURE_BITS + 1],
        ranking: RankingCensus {
            query_candidate_pair_comparisons: 0,
            same_strict_preference: 0,
            opposite_strict_preference: 0,
            both_tied: 0,
            angular_tied_hamming_strict: 0,
            hamming_tied_angular_strict: 0,
        },
    };
    for query in 0..ROOTS as u16 {
        let signature = metric.signatures[usize::from(query)];
        result.signature_population_histogram
            [(signature[0].count_ones() + signature[1].count_ones()) as usize] += 1;
        let mut scores = [0i16; ROOTS];
        let mut distances = [0u16; ROOTS];
        for key in 0..ROOTS as u16 {
            let d = metric.root_distance(query, key)?;
            distances[usize::from(key)] = d;
            scores[usize::from(key)] = geometry
                .score([query; 2], [key; 2])
                .map_err(|_| MetricError::Geometry)?;
            result.ordered_root_pairs += 1;
            result.distance_histogram[usize::from(d)] += 1;
            result.symmetric &= d == metric.root_distance(key, query)?;
            if query == key {
                result.reflexive &= d == 0;
            } else if d == 0 {
                result.colliding_ordered_distinct_root_pairs += 1;
            }
        }
        let minimum = scores.iter().copied().min().ok_or(MetricError::Geometry)?;
        let mut partners = 0;
        for key in 0..ROOTS {
            if scores[key] == minimum {
                partners += 1;
                result.minimum_angular_partner_pairs += 1;
                result.minimum_angular_partner_hamming_histogram[usize::from(distances[key])] += 1;
            }
        }
        result.unique_minimum_angular_partner_for_every_root &= partners == 1;
        result.minimum_angular_score_negates_self_for_every_root &=
            i32::from(minimum) == -i32::from(scores[usize::from(query)]);
        for a in 0..ROOTS {
            for b in a + 1..ROOTS {
                use std::cmp::Ordering::Equal;
                // Higher angular score and LOWER Hamming distance are preferred.
                let angular = scores[a].cmp(&scores[b]);
                let hamming = distances[b].cmp(&distances[a]);
                let ranking = &mut result.ranking;
                ranking.query_candidate_pair_comparisons += 1;
                match (angular, hamming) {
                    (Equal, Equal) => ranking.both_tied += 1,
                    (Equal, _) => ranking.angular_tied_hamming_strict += 1,
                    (_, Equal) => ranking.hamming_tied_angular_strict += 1,
                    (a, b) if a == b => ranking.same_strict_preference += 1,
                    _ => ranking.opposite_strict_preference += 1,
                }
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
#[path = "metric_tests.rs"]
mod tests;

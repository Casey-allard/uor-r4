//! Source→compiled geometry projections (#600): the typed, versioned
//! record of how a teacher's source embedding width is reduced to the
//! compiled geometry, plus the named implementations themselves.
//!
//! Before #600 the reduction was hidden inside
//! `HuggingFaceLlamaOracle::embedding` — the pinned 576-wide SmolLM2 rows
//! were bucket-averaged down to the legacy compiled width D=288 with no
//! record of the algorithm, its parameters, or its version anywhere in the
//! produced provenance. A change to that code could silently alter
//! observations and downstream artifact κs. This module makes the
//! projection explicit:
//!
//! - [`GeometryProjection`] is the serializable record `{id, version,
//!   source_width, compiled_width, params, implementation_digest}` carried
//!   by the compile report and the observation manifest.
//! - [`bucket_average_project`] is the free, deterministic implementation
//!   of `bucket-average/1` — the exact arithmetic (iteration order,
//!   sequential f32 sum, one divide per bucket) the oracle has always
//!   used, factored out unchanged.
//! - [`projection_implementation`] is the versioned registry mapping
//!   `(id, version)` to an implementation; an unknown pair is refused by
//!   name on the sanctioned [`SourceUnavailable`](crate::SourceUnavailable)
//!   surface rather than guessed.
//!
//! **Implementation digest.** `implementation_digest` is the blake3 of the
//! [canonical serialization](GeometryProjection::canonical_bytes) of the
//! algorithm's *declared parameters* plus a stable algorithm tag — NOT a
//! hash of the source code text. Source text is a fragile identity: a
//! rename, comment, or formatting pass would change it while the computed
//! function stays bit-identical. The declared parameters + versioned tag
//! are the algorithm's contract; the unit tests in this module pin the
//! implementation to that declaration, so a behavioral change must bump
//! the version (a new registry entry) instead of silently drifting.

use serde::{Deserialize, Serialize};

/// The legacy compiled geometry width (the transformerless compiler's
/// `D = 288`). The Hugging Face teacher adapter reports this as its
/// [`TeacherOracle::dim`](crate::TeacherOracle::dim) and projects source
/// embedding rows down to it.
pub const COMPILED_WIDTH: u32 = 288;

/// Declared parameters of a projection algorithm — the bucket layout,
/// including the remainder policy for non-divisible widths, and the
/// reduction order. These strings are stable machine tokens (they enter
/// the canonical digest serialization byte-for-byte), documented on
/// [`GeometryProjectionParams::bucket_average`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryProjectionParams {
    /// How source columns are grouped into buckets.
    #[serde(default)]
    pub bucket_layout: String,
    /// What happens when `compiled_width` does not divide `source_width`.
    #[serde(default)]
    pub remainder_policy: String,
    /// The reduction applied to each bucket, including its order.
    #[serde(default)]
    pub accumulation: String,
}

impl GeometryProjectionParams {
    /// The declared parameters of `bucket-average/1`:
    ///
    /// - `bucket_layout = "contiguous-floor-boundaries"` — output index
    ///   `i` averages the contiguous source slice
    ///   `[floor(i·S/C), floor((i+1)·S/C))` for source width `S` and
    ///   compiled width `C`.
    /// - `remainder_policy = "floor-spread"` — when `C` does not divide
    ///   `S`, the floor boundaries spread the remainder `S mod C` across
    ///   the buckets whose scaled boundary crosses an extra integer, so
    ///   bucket sizes differ by at most one and every source column
    ///   belongs to exactly one bucket. `S < C` (which would make some
    ///   buckets empty) is refused by [`bucket_average_project`].
    /// - `accumulation = "sequential-f32-sum-then-mean"` — each bucket is
    ///   summed left-to-right in f32 and divided once by the bucket
    ///   length (the exact legacy arithmetic order).
    pub fn bucket_average() -> Self {
        Self {
            bucket_layout: "contiguous-floor-boundaries".to_owned(),
            remainder_policy: "floor-spread".to_owned(),
            accumulation: "sequential-f32-sum-then-mean".to_owned(),
        }
    }
}

/// The typed, versioned record of one source→compiled geometry projection
/// (#600). Serialized (all fields serde-defaulted) into the cover/compile
/// report and the observation manifest wherever the projection is known,
/// so a geometry change is visible in provenance instead of silent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryProjection {
    /// Registry id of the projection algorithm (e.g. `"bucket-average"`).
    #[serde(default)]
    pub id: String,
    /// Registry version of the algorithm. A behavioral change is a new
    /// version, never an in-place edit.
    #[serde(default)]
    pub version: u32,
    /// Source embedding width the projection reads (e.g. 576 for the
    /// pinned SmolLM2-135M).
    #[serde(default)]
    pub source_width: u32,
    /// Compiled width the projection writes (the legacy geometry
    /// [`COMPILED_WIDTH`] = 288).
    #[serde(default)]
    pub compiled_width: u32,
    /// The algorithm's declared parameters (bucket layout, remainder
    /// policy, accumulation order).
    #[serde(default)]
    pub params: GeometryProjectionParams,
    /// `blake3:<hex>` of [`GeometryProjection::canonical_bytes`] — the
    /// declared parameters + stable algorithm tag, not source code text
    /// (see the module docs for why).
    #[serde(default)]
    pub implementation_digest: String,
}

impl GeometryProjection {
    /// Registry id of the bucket-average projection.
    pub const BUCKET_AVERAGE_ID: &'static str = "bucket-average";
    /// Registry version of the bucket-average projection currently
    /// implemented by [`bucket_average_project`].
    pub const BUCKET_AVERAGE_VERSION: u32 = 1;

    /// The `bucket-average/1` record for a concrete width pair — the
    /// projection [`bucket_average_project`] implements and the Hugging
    /// Face teacher adapter has always applied (576→288 for the pinned
    /// SmolLM2-135M).
    pub fn bucket_average(source_width: u32, compiled_width: u32) -> Self {
        let mut record = Self {
            id: Self::BUCKET_AVERAGE_ID.to_owned(),
            version: Self::BUCKET_AVERAGE_VERSION,
            source_width,
            compiled_width,
            params: GeometryProjectionParams::bucket_average(),
            implementation_digest: String::new(),
        };
        record.implementation_digest = record.declared_digest();
        record
    }

    /// Canonical serialization of the record's declared identity: a fixed
    /// line format (format tag, id, version, widths, parameters, each
    /// `key=value\n`). Byte-stable by construction — field order and
    /// separators are fixed here, not derived from any serializer — so
    /// the digest over these bytes is reproducible everywhere.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        format!(
            "uor-r4-geometry-projection/1\n\
             id={}\n\
             version={}\n\
             source_width={}\n\
             compiled_width={}\n\
             param.bucket_layout={}\n\
             param.remainder_policy={}\n\
             param.accumulation={}\n",
            self.id,
            self.version,
            self.source_width,
            self.compiled_width,
            self.params.bucket_layout,
            self.params.remainder_policy,
            self.params.accumulation,
        )
        .into_bytes()
    }

    /// The implementation digest this record's declared parameters imply:
    /// `blake3:<hex>` over [`GeometryProjection::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// A projection implementation: reads one source-width row, writes one
/// compiled-width row. Deterministic and free of hidden state.
pub type ProjectionFn = fn(&[f32], &mut [f32]);

/// The `bucket-average/1` implementation, factored verbatim out of the
/// Hugging Face oracle's `embedding` (#600): output index `i` is the mean
/// of the contiguous source slice `[floor(i·S/C), floor((i+1)·S/C))`,
/// summed left-to-right in f32 and divided once by the bucket length. The
/// iteration order and arithmetic are bit-identical to the pre-#600 code.
///
/// `row.len() < out.len()` would make some buckets empty (a mean over
/// nothing); it is refused with the legacy panic message rather than
/// producing NaNs. `out.len() == 0` writes nothing.
pub fn bucket_average_project(row: &[f32], out: &mut [f32]) {
    let source_width = row.len();
    assert!(
        source_width >= out.len(),
        "source dimension is smaller than runtime geometry"
    );
    let compiled_width = out.len();
    for (index, value) in out.iter_mut().enumerate() {
        let start = index * source_width / compiled_width;
        let end = (index + 1) * source_width / compiled_width;
        let bucket = &row[start..end];
        *value = bucket.iter().sum::<f32>() / bucket.len() as f32;
    }
}

/// The versioned projection registry (#600): map `(id, version)` to the
/// implementation that computes it. Every pair outside the registry is
/// refused by name on the sanctioned [`SourceUnavailable`] surface
/// ([`SourceIngestKind::UnknownGeometryProjection`]) — never guessed,
/// never approximated by a "closest" version.
///
/// [`SourceUnavailable`]: crate::SourceUnavailable
/// [`SourceIngestKind::UnknownGeometryProjection`]: crate::SourceIngestKind::UnknownGeometryProjection
#[cfg(not(target_arch = "wasm32"))]
pub fn projection_implementation(
    id: &str,
    version: u32,
) -> Result<ProjectionFn, crate::SourceUnavailable> {
    match (id, version) {
        (GeometryProjection::BUCKET_AVERAGE_ID, GeometryProjection::BUCKET_AVERAGE_VERSION) => {
            Ok(bucket_average_project)
        }
        _ => Err(crate::SourceIngestKind::UnknownGeometryProjection {
            id: id.to_owned(),
            version,
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference bucket layout, written independently of the projection:
    /// bucket sizes under the floor rule.
    fn bucket_sizes(source_width: usize, compiled_width: usize) -> Vec<usize> {
        (0..compiled_width)
            .map(|index| {
                (index + 1) * source_width / compiled_width - index * source_width / compiled_width
            })
            .collect()
    }

    fn ramp(len: usize) -> Vec<f32> {
        (0..len)
            .map(|index| ((index * 37 % 101) as f32 - 50.0) / 8.0)
            .collect()
    }

    #[test]
    fn divisible_projection_is_the_exact_pairwise_mean() {
        // 576 → 288: every bucket is exactly two adjacent columns.
        let row = ramp(576);
        let mut out = vec![0f32; 288];
        bucket_average_project(&row, &mut out);
        for (index, &value) in out.iter().enumerate() {
            let expected = (row[2 * index] + row[2 * index + 1]) / 2.0;
            assert_eq!(value.to_bits(), expected.to_bits(), "bucket {index}");
        }
    }

    #[test]
    fn non_divisible_projection_spreads_the_remainder_by_floor_boundaries() {
        // 577 → 288: remainder 1; the floor rule gives 287 buckets of two
        // columns and one bucket (the last) of three.
        let sizes = bucket_sizes(577, 288);
        assert_eq!(sizes.iter().sum::<usize>(), 577, "every column is used");
        assert!(sizes.iter().all(|&s| s == 2 || s == 3));
        assert_eq!(sizes.iter().filter(|&&s| s == 3).count(), 1);
        assert_eq!(sizes[287], 3, "floor boundaries put the wide bucket last");

        let row = ramp(577);
        let mut out = vec![0f32; 288];
        bucket_average_project(&row, &mut out);
        let mut start = 0usize;
        for (index, (&value, &size)) in out.iter().zip(&sizes).enumerate() {
            let bucket = &row[start..start + size];
            let expected = bucket.iter().sum::<f32>() / bucket.len() as f32;
            assert_eq!(value.to_bits(), expected.to_bits(), "bucket {index}");
            start += size;
        }

        // A wider remainder: 700 → 288 spreads 124 three-column buckets
        // among 164 two-column buckets, positions fixed by the floor rule.
        let sizes = bucket_sizes(700, 288);
        assert_eq!(sizes.iter().sum::<usize>(), 700);
        assert_eq!(sizes.iter().filter(|&&s| s == 3).count(), 700 - 2 * 288);
    }

    #[test]
    fn equal_widths_project_to_identity_singleton_buckets() {
        // Short (single-column) buckets: S == C is the identity mean.
        let row = ramp(288);
        let mut out = vec![0f32; 288];
        bucket_average_project(&row, &mut out);
        for (index, &value) in out.iter().enumerate() {
            let expected = row[index] / 1.0;
            assert_eq!(value.to_bits(), expected.to_bits(), "bucket {index}");
        }
    }

    #[test]
    fn empty_compiled_row_writes_nothing() {
        let row = ramp(16);
        let mut out: Vec<f32> = Vec::new();
        bucket_average_project(&row, &mut out);
        assert!(out.is_empty());
    }

    #[test]
    #[should_panic(expected = "source dimension is smaller than runtime geometry")]
    fn a_source_narrower_than_the_compiled_width_is_refused() {
        // 100 → 288 would make buckets empty (NaN means); the legacy
        // guard refuses it instead.
        let row = ramp(100);
        let mut out = vec![0f32; 288];
        bucket_average_project(&row, &mut out);
    }

    #[test]
    fn double_projection_is_bit_deterministic() {
        let row = ramp(577);
        let mut first = vec![0f32; 288];
        let mut second = vec![0f32; 288];
        bucket_average_project(&row, &mut first);
        bucket_average_project(&row, &mut second);
        let first_bits: Vec<u32> = first.iter().map(|v| v.to_bits()).collect();
        let second_bits: Vec<u32> = second.iter().map(|v| v.to_bits()).collect();
        assert_eq!(first_bits, second_bits);
    }

    #[test]
    fn canonical_serialization_is_byte_stable() {
        // The canonical form is pinned byte-for-byte: any drift in field
        // order, separators, or parameter tokens fails here, which is the
        // point — the digest identity must not move silently.
        let record = GeometryProjection::bucket_average(576, COMPILED_WIDTH);
        let pinned = "uor-r4-geometry-projection/1\n\
                      id=bucket-average\n\
                      version=1\n\
                      source_width=576\n\
                      compiled_width=288\n\
                      param.bucket_layout=contiguous-floor-boundaries\n\
                      param.remainder_policy=floor-spread\n\
                      param.accumulation=sequential-f32-sum-then-mean\n";
        assert_eq!(record.canonical_bytes(), pinned.as_bytes());
        // The digest is the blake3 of exactly those pinned bytes (declared
        // parameters + algorithm tag, not source code text).
        let expected = format!("blake3:{}", blake3::hash(pinned.as_bytes()).to_hex());
        assert_eq!(record.implementation_digest, expected);
        assert_eq!(record.declared_digest(), expected);
        // Rebuilding the record reproduces the digest bit-for-bit.
        let again = GeometryProjection::bucket_average(576, COMPILED_WIDTH);
        assert_eq!(record, again);
    }

    #[test]
    fn record_round_trips_through_serde_json() {
        let record = GeometryProjection::bucket_average(576, 288);
        let json = serde_json::to_string(&record).expect("serializes");
        let back: GeometryProjection = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(record, back);
        // Serde-defaulted fields: a legacy/partial document still parses.
        let partial: GeometryProjection =
            serde_json::from_str("{\"id\":\"bucket-average\"}").expect("defaults fill in");
        assert_eq!(partial.id, "bucket-average");
        assert_eq!(partial.version, 0);
        assert_eq!(partial.params, GeometryProjectionParams::default());
    }

    #[test]
    fn registry_resolves_bucket_average_1() {
        let projection = projection_implementation(
            GeometryProjection::BUCKET_AVERAGE_ID,
            GeometryProjection::BUCKET_AVERAGE_VERSION,
        )
        .expect("registered implementation");
        let row = ramp(576);
        let mut via_registry = vec![0f32; 288];
        let mut direct = vec![0f32; 288];
        projection(&row, &mut via_registry);
        bucket_average_project(&row, &mut direct);
        assert_eq!(via_registry, direct);
    }

    #[test]
    fn registry_refuses_unknown_id_and_version_by_name() {
        for (id, version) in [("bucket-average", 2u32), ("mystery-projection", 1)] {
            let error = projection_implementation(id, version)
                .expect_err("unknown (id, version) is not a product");
            match &error.kind {
                crate::SourceIngestKind::UnknownGeometryProjection {
                    id: got_id,
                    version: got_version,
                } => {
                    assert_eq!(got_id, id);
                    assert_eq!(*got_version, version);
                }
                other => panic!("wrong failure class: {other:?}"),
            }
            assert!(error.reason.contains(id), "reason names the id: {error}");
        }
    }
}

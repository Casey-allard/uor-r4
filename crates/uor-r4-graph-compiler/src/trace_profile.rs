//! Deterministic versioned teacher-trace profiles (#603): the typed
//! `uor-r4-teacher-trace/1` record naming exactly which observation lanes
//! a pass captures and their bounds, so trace richness, boundedness,
//! profile identity, absence semantics, and source/provenance
//! dependencies travel as ONE stable bundle on the observation manifest
//! (see [`crate::observation::ObservationManifest::identity_bundle_digest`]).
//!
//! Four profiles are registered, every one an extension of the existing
//! observation pipeline — no second pipeline exists:
//!
//! - `minimal/1` — exactly today's surface: the v4 88-byte records
//!   (bounded token / top-8 / probability rows) plus the aligned
//!   probability sidecar. No richer lane; a minimal pass writes bytes
//!   and manifests byte-identical to a pre-#603 pass (the manifest
//!   carries NO `trace_profile` field for it — absence marks the
//!   implicit legacy era, exactly like the #597/#600/#601/#602 fields).
//! - `layer/1` — minimal plus the per-layer residual lane at declared
//!   layer indices and the final-hidden lane (the post-final-rmsnorm
//!   hidden state, `TeacherOracle::hidden_state`). **Measurement record
//!   (#95):** the final-hidden lane was measured NEGATIVE for the cover
//!   compiler — 2.8% vs 31.7% Gate C top-1 when used as the cover
//!   observation vector (merged #95). The lane is preserved here as a
//!   recorded capture for measurement, NOT adopted for fitting; adopting
//!   any richer profile for fitting requires a separate measured issue.
//! - `attention-support/1` — minimal plus the attention-support lane:
//!   per-head top-S attention weights, captured ONLY for declared layer
//!   indices and within the declared support cap (tapped from the #602
//!   factored per-head weight functions through the exact executor).
//! - `full/1` — minimal plus all richer lanes: per-layer residuals,
//!   final hidden, current-position q/k/v rows, and attention support.
//!
//! Richer lanes live in a per-shard SIDE-CAR file (`shard-NN.bin.trace`,
//! same deterministic partitioning, content addressing, and manifest
//! registration as the `.prob` sidecar), so the primary 88-byte shard
//! record format stays era-stable: the v4 record bytes never change for
//! any profile. Absent lanes are absent — never zero-filled: a profile
//! that does not declare a lane produces no bytes for it, and a padding
//! slot inside the bounded attention-support rows is written as the
//! explicit absence marker [`SUPPORT_ABSENT_MARKER`], never as a
//! zero-valued entry.
//!
//! Identity follows the #600/#601/#602 discipline: the profile's
//! declared identity has a canonical pinned-line serialization
//! ([`TraceProfile::canonical_bytes`]) and a blake3 declared-identity
//! digest ([`TraceProfile::declared_digest`]) — deliberately not a hash
//! of source code text; a behavioral change must arrive as a new
//! registry version, never an in-place edit. The versioned registry
//! ([`profile_spec`]) maps `(profile, version)` to the record and
//! refuses every unknown pair by name on the sanctioned
//! [`SourceUnavailable`](uor_r4_model_source::SourceUnavailable) surface
//! — the same type this crate's manifest/parse failures already use —
//! rather than guessing.

use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use uor_r4_model_source::SourceUnavailable;

/// Format tag of the canonical serialization (and the version prefix of
/// every registered profile identity).
pub const TRACE_PROFILE_FORMAT: &str = "uor-r4-teacher-trace/1";

/// Registry id of the profile capturing exactly today's surface.
pub const MINIMAL_PROFILE: &str = "minimal";
/// Registry id of the per-layer residual + final-hidden profile.
pub const LAYER_PROFILE: &str = "layer";
/// Registry id of the bounded attention-support profile.
pub const ATTENTION_SUPPORT_PROFILE: &str = "attention-support";
/// Registry id of the all-lanes profile.
pub const FULL_PROFILE: &str = "full";
/// Registry version shared by the four registered profiles.
pub const PROFILE_VERSION: u32 = 1;

/// Bound of the primary lane's top-k row — the v4 record's top-8, fixed
/// for every registered profile (the primary record format never
/// changes).
pub const PRIMARY_TOP_K: u32 = 8;

/// Maximum number of declarable capture layer indices per lane.
pub const MAX_TRACE_LAYERS: usize = 64;

/// Maximum declarable per-head attention-support cap.
pub const MAX_SUPPORT_SIZE: u32 = 64;

/// Explicit absence marker for an unfilled slot inside a bounded
/// attention-support row (fewer prefix positions than the declared cap):
/// both the u32 position field and the u32 weight-bits field carry this
/// marker. Absence is marked, never zero-filled — a real entry at
/// position 0 with weight 0.0 is distinguishable from an absent slot.
pub const SUPPORT_ABSENT_MARKER: u32 = u32::MAX;

/// The per-layer residual lane declaration: which post-layer residual
/// streams are captured, and whether the final-hidden (#95) lane rides
/// along.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerLane {
    /// Declared post-layer capture indices, ascending and deduplicated.
    #[serde(default)]
    pub layer_indices: Vec<u32>,
    /// Whether the post-final-rmsnorm hidden state
    /// (`TeacherOracle::hidden_state`) is captured. This is the #95
    /// lane, measured NEGATIVE for the cover compiler (2.8% vs 31.7%
    /// Gate C top-1, merged #95) — preserved as a recorded measurement
    /// capture, not adopted for fitting.
    #[serde(default)]
    pub final_hidden: bool,
}

/// The current-position q/k/v lane declaration (rotated query plus the
/// key/value cache rows the #602 operators consumed), `full/1` only.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QkvLane {
    /// Declared capture indices, ascending and deduplicated.
    #[serde(default)]
    pub layer_indices: Vec<u32>,
}

/// The attention-support lane declaration: per-head top-S attention
/// weights for declared layer indices, within the declared cap.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttentionSupportLane {
    /// Declared capture indices, ascending and deduplicated.
    #[serde(default)]
    pub layer_indices: Vec<u32>,
    /// Per-head support cap S: at most S `(position, weight)` entries
    /// per head per declared layer per step, ordered by descending
    /// weight (ties to the lower position). `1..=MAX_SUPPORT_SIZE`.
    #[serde(default)]
    pub support_size: u32,
}

/// The typed, versioned record of one teacher-trace profile (#603):
/// which lanes a pass captures and their bounds. Serialized (all fields
/// serde-defaulted, lanes as `Option` — absence is absence, never a
/// zero-filled default) into the observation manifest whenever a
/// non-minimal profile is active; a minimal pass carries NO record, so
/// legacy manifest bytes are unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceProfile {
    /// Registry id (`minimal`, `layer`, `attention-support`, `full`).
    #[serde(default)]
    pub id: String,
    /// Registry version. A behavioral change is a new version, never an
    /// in-place edit.
    #[serde(default)]
    pub version: u32,
    /// Bound of the primary lane's top-k row (always
    /// [`PRIMARY_TOP_K`]; the v4 record format is era-stable).
    #[serde(default)]
    pub top_k: u32,
    /// Per-layer residual + final-hidden lane; `None` = not captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_lane: Option<LayerLane>,
    /// Current-position q/k/v lane; `None` = not captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qkv_lane: Option<QkvLane>,
    /// Bounded attention-support lane; `None` = not captured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention_support_lane: Option<AttentionSupportLane>,
    /// `blake3:<hex>` of [`TraceProfile::canonical_bytes`] — the
    /// declared identity, not source code text (#600/#601/#602
    /// discipline).
    #[serde(default)]
    pub declared_digest: String,
}

fn normalized(indices: &[u32]) -> Vec<u32> {
    let mut indices = indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn indices_line(indices: &[u32]) -> String {
    indices
        .iter()
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

impl TraceProfile {
    /// The `minimal/1` record: exactly today's surface, no richer lane.
    pub fn minimal() -> Self {
        let mut record = Self {
            id: MINIMAL_PROFILE.to_owned(),
            version: PROFILE_VERSION,
            top_k: PRIMARY_TOP_K,
            layer_lane: None,
            qkv_lane: None,
            attention_support_lane: None,
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// The `layer/1` record over `layer_indices` (sorted, deduplicated;
    /// the final-hidden #95 lane is always declared with it).
    pub fn layer(layer_indices: &[u32]) -> Self {
        let mut record = Self {
            id: LAYER_PROFILE.to_owned(),
            version: PROFILE_VERSION,
            top_k: PRIMARY_TOP_K,
            layer_lane: Some(LayerLane {
                layer_indices: normalized(layer_indices),
                final_hidden: true,
            }),
            qkv_lane: None,
            attention_support_lane: None,
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// The `attention-support/1` record over `layer_indices` with the
    /// per-head support cap `support_size`.
    pub fn attention_support(layer_indices: &[u32], support_size: u32) -> Self {
        let mut record = Self {
            id: ATTENTION_SUPPORT_PROFILE.to_owned(),
            version: PROFILE_VERSION,
            top_k: PRIMARY_TOP_K,
            layer_lane: None,
            qkv_lane: None,
            attention_support_lane: Some(AttentionSupportLane {
                layer_indices: normalized(layer_indices),
                support_size,
            }),
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// The `full/1` record: every richer lane over the same declared
    /// `layer_indices`, attention support capped at `support_size`.
    pub fn full(layer_indices: &[u32], support_size: u32) -> Self {
        let indices = normalized(layer_indices);
        let mut record = Self {
            id: FULL_PROFILE.to_owned(),
            version: PROFILE_VERSION,
            top_k: PRIMARY_TOP_K,
            layer_lane: Some(LayerLane {
                layer_indices: indices.clone(),
                final_hidden: true,
            }),
            qkv_lane: Some(QkvLane {
                layer_indices: indices.clone(),
            }),
            attention_support_lane: Some(AttentionSupportLane {
                layer_indices: indices,
                support_size,
            }),
            declared_digest: String::new(),
        };
        record.declared_digest = record.declared_digest();
        record
    }

    /// Whether this is the minimal profile: no richer lane declared —
    /// the pass writes exactly today's bytes and records nothing in the
    /// manifest.
    pub fn is_minimal(&self) -> bool {
        self.layer_lane.is_none()
            && self.qkv_lane.is_none()
            && self.attention_support_lane.is_none()
    }

    /// Canonical serialization of the record's declared identity: a
    /// fixed line format (format tag, id, version, bounds, one
    /// `key=value\n` per field) with EXPLICIT absence markers — an
    /// undeclared lane serializes as `<lane>=absent`, a declared lane as
    /// `<lane>=present` followed by its bound lines, so absence is part
    /// of the digest input and distinct from any empty declaration.
    /// Byte-stable by construction: field order and separators are fixed
    /// here, not derived from any serializer.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut text = format!(
            "{TRACE_PROFILE_FORMAT}\nid={}\nversion={}\ntop_k={}\n",
            self.id, self.version, self.top_k
        );
        match &self.layer_lane {
            None => text.push_str("layer_lane=absent\n"),
            Some(lane) => {
                text.push_str("layer_lane=present\n");
                text.push_str(&format!(
                    "layer_lane.layer_indices={}\nlayer_lane.final_hidden={}\n",
                    indices_line(&lane.layer_indices),
                    lane.final_hidden
                ));
            }
        }
        match &self.qkv_lane {
            None => text.push_str("qkv_lane=absent\n"),
            Some(lane) => {
                text.push_str("qkv_lane=present\n");
                text.push_str(&format!(
                    "qkv_lane.layer_indices={}\n",
                    indices_line(&lane.layer_indices)
                ));
            }
        }
        match &self.attention_support_lane {
            None => text.push_str("attention_support_lane=absent\n"),
            Some(lane) => {
                text.push_str("attention_support_lane=present\n");
                text.push_str(&format!(
                    "attention_support_lane.layer_indices={}\nattention_support_lane.support_size={}\n",
                    indices_line(&lane.layer_indices),
                    lane.support_size
                ));
            }
        }
        text.into_bytes()
    }

    /// The declared-identity digest: `blake3:<hex>` over
    /// [`TraceProfile::canonical_bytes`].
    pub fn declared_digest(&self) -> String {
        format!("blake3:{}", blake3::hash(&self.canonical_bytes()).to_hex())
    }
}

/// Capture bounds a caller declares when resolving a profile through the
/// registry: the layer indices the richer lanes capture at and the
/// per-head attention-support cap. Ignored by profiles whose lanes do
/// not use them (`minimal/1` uses neither).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceCaptureBounds {
    /// Declared capture layer indices (at most [`MAX_TRACE_LAYERS`]).
    pub layer_indices: Vec<u32>,
    /// Per-head attention-support cap (`1..=MAX_SUPPORT_SIZE` where the
    /// profile declares the lane).
    pub support_size: u32,
}

impl Default for TraceCaptureBounds {
    fn default() -> Self {
        Self {
            layer_indices: Vec::new(),
            support_size: PRIMARY_TOP_K,
        }
    }
}

/// The versioned profile registry (#603): map `(profile, version)` plus
/// the caller-declared bounds to the typed record. Every pair outside
/// the registry — and every unbounded declaration — is refused by name
/// on the sanctioned [`SourceUnavailable`] surface (the same type this
/// crate reports manifest/parse failures on), never guessed and never
/// approximated by a "closest" profile or version.
#[cfg(not(target_arch = "wasm32"))]
pub fn profile_spec(
    id: &str,
    version: u32,
    bounds: &TraceCaptureBounds,
) -> Result<TraceProfile, SourceUnavailable> {
    if bounds.layer_indices.len() > MAX_TRACE_LAYERS {
        return Err(SourceUnavailable::new(format!(
            "trace profile {id}/{version} declares {} capture layers, more than the bound {MAX_TRACE_LAYERS}",
            bounds.layer_indices.len()
        )));
    }
    let bounded_support = |lane: &str| -> Result<u32, SourceUnavailable> {
        if bounds.support_size == 0 || bounds.support_size > MAX_SUPPORT_SIZE {
            return Err(SourceUnavailable::new(format!(
                "trace profile {id}/{version} {lane} support cap {} is outside 1..={MAX_SUPPORT_SIZE}",
                bounds.support_size
            )));
        }
        Ok(bounds.support_size)
    };
    let declared_layers = |lane: &str| -> Result<&[u32], SourceUnavailable> {
        if bounds.layer_indices.is_empty() {
            return Err(SourceUnavailable::new(format!(
                "trace profile {id}/{version} {lane} declares no capture layer indices; \
                 declare the bounded layer list explicitly"
            )));
        }
        Ok(&bounds.layer_indices)
    };
    match (id, version) {
        (MINIMAL_PROFILE, PROFILE_VERSION) => Ok(TraceProfile::minimal()),
        (LAYER_PROFILE, PROFILE_VERSION) => Ok(TraceProfile::layer(&bounds.layer_indices)),
        (ATTENTION_SUPPORT_PROFILE, PROFILE_VERSION) => Ok(TraceProfile::attention_support(
            declared_layers("attention-support lane")?,
            bounded_support("attention-support lane")?,
        )),
        (FULL_PROFILE, PROFILE_VERSION) => Ok(TraceProfile::full(
            declared_layers("richer lanes")?,
            bounded_support("attention-support lane")?,
        )),
        _ => Err(SourceUnavailable::new(format!(
            "unknown teacher-trace profile ({id}, {version}); registered: \
             {MINIMAL_PROFILE}/{PROFILE_VERSION}, {LAYER_PROFILE}/{PROFILE_VERSION}, \
             {ATTENTION_SUPPORT_PROFILE}/{PROFILE_VERSION}, {FULL_PROFILE}/{PROFILE_VERSION}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_serialization_is_byte_stable() {
        // Pinned byte-for-byte: any drift in field order, separators, or
        // absence markers fails here — profile identity must not move
        // silently.
        let minimal = TraceProfile::minimal();
        let pinned_minimal = "uor-r4-teacher-trace/1\n\
             id=minimal\n\
             version=1\n\
             top_k=8\n\
             layer_lane=absent\n\
             qkv_lane=absent\n\
             attention_support_lane=absent\n";
        assert_eq!(minimal.canonical_bytes(), pinned_minimal.as_bytes());
        let expected = format!(
            "blake3:{}",
            blake3::hash(pinned_minimal.as_bytes()).to_hex()
        );
        assert_eq!(minimal.declared_digest, expected);
        assert_eq!(minimal.declared_digest(), expected);

        let full = TraceProfile::full(&[2, 0, 2], 4);
        let pinned_full = "uor-r4-teacher-trace/1\n\
             id=full\n\
             version=1\n\
             top_k=8\n\
             layer_lane=present\n\
             layer_lane.layer_indices=0,2\n\
             layer_lane.final_hidden=true\n\
             qkv_lane=present\n\
             qkv_lane.layer_indices=0,2\n\
             attention_support_lane=present\n\
             attention_support_lane.layer_indices=0,2\n\
             attention_support_lane.support_size=4\n";
        assert_eq!(full.canonical_bytes(), pinned_full.as_bytes());
        assert_eq!(full.declared_digest, full.declared_digest());

        // The four registered profiles are four distinct identities.
        let digests = [
            TraceProfile::minimal().declared_digest,
            TraceProfile::layer(&[0]).declared_digest,
            TraceProfile::attention_support(&[0], 4).declared_digest,
            TraceProfile::full(&[0], 4).declared_digest,
        ];
        for (i, a) in digests.iter().enumerate() {
            for b in digests.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        // Bounds are identity: a different layer list or support cap is
        // a different declared digest.
        assert_ne!(
            TraceProfile::layer(&[0]).declared_digest,
            TraceProfile::layer(&[1]).declared_digest
        );
        assert_ne!(
            TraceProfile::attention_support(&[0], 4).declared_digest,
            TraceProfile::attention_support(&[0], 8).declared_digest
        );
    }

    #[test]
    fn absent_lane_is_not_an_empty_declaration() {
        // Absence semantics in the digest input: an undeclared lane and
        // a declared-but-empty lane are different identities.
        let absent = TraceProfile::minimal();
        let mut empty = TraceProfile::minimal();
        empty.layer_lane = Some(LayerLane {
            layer_indices: Vec::new(),
            final_hidden: false,
        });
        assert_ne!(absent.declared_digest(), empty.declared_digest());
    }

    #[test]
    fn record_round_trips_through_serde_json_and_absent_lanes_serialize_no_key() {
        for record in [
            TraceProfile::minimal(),
            TraceProfile::layer(&[0, 3]),
            TraceProfile::attention_support(&[1], 4),
            TraceProfile::full(&[0, 1], 2),
        ] {
            let json = serde_json::to_string(&record).expect("serializes");
            let back: TraceProfile = serde_json::from_str(&json).expect("deserializes");
            assert_eq!(record, back);
        }
        // Absence stays absence on the wire: no lane keys at all.
        let minimal_json = serde_json::to_string(&TraceProfile::minimal()).expect("serializes");
        assert!(!minimal_json.contains("layer_lane"));
        assert!(!minimal_json.contains("qkv_lane"));
        assert!(!minimal_json.contains("attention_support_lane"));
        // Serde-defaulted fields: a partial document still parses.
        let partial: TraceProfile =
            serde_json::from_str("{\"id\":\"minimal\"}").expect("defaults fill");
        assert_eq!(partial.id, "minimal");
        assert_eq!(partial.version, 0);
        assert_eq!(partial.layer_lane, None);
    }

    #[test]
    fn registry_resolves_the_four_profiles() {
        let bounds = TraceCaptureBounds {
            layer_indices: vec![0, 2],
            support_size: 4,
        };
        assert_eq!(
            profile_spec(MINIMAL_PROFILE, 1, &bounds).expect("minimal"),
            TraceProfile::minimal()
        );
        assert_eq!(
            profile_spec(LAYER_PROFILE, 1, &bounds).expect("layer"),
            TraceProfile::layer(&[0, 2])
        );
        assert_eq!(
            profile_spec(ATTENTION_SUPPORT_PROFILE, 1, &bounds).expect("attention-support"),
            TraceProfile::attention_support(&[0, 2], 4)
        );
        assert_eq!(
            profile_spec(FULL_PROFILE, 1, &bounds).expect("full"),
            TraceProfile::full(&[0, 2], 4)
        );
    }

    #[test]
    fn registry_refuses_unknown_and_unbounded_by_name() {
        let bounds = TraceCaptureBounds {
            layer_indices: vec![0],
            support_size: 4,
        };
        for (id, version) in [("minimal", 2u32), ("layer", 9), ("mystery-trace", 1)] {
            let error = profile_spec(id, version, &bounds)
                .expect_err("unknown (profile, version) is not a product");
            assert!(error.reason.contains(id), "reason names the id: {error}");
        }
        // Unbounded declarations are refused, not clamped.
        let unbounded_support = TraceCaptureBounds {
            layer_indices: vec![0],
            support_size: MAX_SUPPORT_SIZE + 1,
        };
        assert!(profile_spec(FULL_PROFILE, 1, &unbounded_support).is_err());
        let zero_support = TraceCaptureBounds {
            layer_indices: vec![0],
            support_size: 0,
        };
        assert!(profile_spec(ATTENTION_SUPPORT_PROFILE, 1, &zero_support).is_err());
        let no_layers = TraceCaptureBounds {
            layer_indices: Vec::new(),
            support_size: 4,
        };
        assert!(profile_spec(ATTENTION_SUPPORT_PROFILE, 1, &no_layers).is_err());
        assert!(profile_spec(FULL_PROFILE, 1, &no_layers).is_err());
        let too_many = TraceCaptureBounds {
            layer_indices: (0..=MAX_TRACE_LAYERS as u32).collect(),
            support_size: 4,
        };
        assert!(profile_spec(LAYER_PROFILE, 1, &too_many).is_err());
    }
}

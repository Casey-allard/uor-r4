//! The N-tier serving cascade (issue #248).
//!
//! [`run_cascade`] walks an ordered list of named tier closures, serves
//! the first [`EngineStatus::Success`] with non-empty text, and records
//! every attempted tier's typed outcome in a [`CascadeOutcome::trail`].
//! A declared abstention is a recorded event the cascade continues past
//! (PR #223 semantics: recording, not refusing), and a run where no tier
//! serves ends with an honest empty outcome the caller types as
//! `declined_by_all`.
//!
//! #790 item 4 (2026-08-18 audit finding): the original
//! `FallbackRouter` two-engine pipeline that founded this module — plus
//! its `EngineResponse`/`FallbackResult` types, the never-selected
//! `AbstainPolicy::Terminal` parameterization, the never-constructed
//! `UnmappedRegion` status, and the `r4g1-graph`/`transformerless-tla5`
//! engine names — had zero callers outside its own tests while the docs
//! presented it as the live serving path. It was removed rather than
//! kept as dead weight; `run_cascade` and its typed outcome records are
//! the one serving surface.

use serde::{Deserialize, Serialize};

/// Error classification of engine execution outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineStatus {
    /// Successful inference generation.
    Success,
    /// Primary engine encountered a pathological loop or cycle.
    Pathological,
    /// Unrecoverable engine failure.
    Failed,
    /// The engine declined to serve by declared policy (R4G1 D4
    /// abstention, geometric sparse-resonance). A declared outcome, not
    /// a fault: no text exists, none is guessed.
    Abstained,
}

impl std::fmt::Display for EngineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineStatus::Success => write!(f, "success"),
            EngineStatus::Pathological => write!(f, "pathological"),
            EngineStatus::Failed => write!(f, "failed"),
            EngineStatus::Abstained => write!(f, "abstained"),
        }
    }
}

/// One tier's declared result inside a serving cascade: the typed status,
/// the generated text (only meaningful on `Success`), and an optional
/// human-readable detail (abstention status, pathology reason, error).
#[derive(Debug, Clone)]
pub struct TierResult {
    pub status: EngineStatus,
    pub text: Option<String>,
    pub detail: Option<String>,
}

impl TierResult {
    /// A tier that served usable text.
    pub fn success(text: String) -> Self {
        Self {
            status: EngineStatus::Success,
            text: Some(text),
            detail: None,
        }
    }

    /// A tier that abstained by declared policy.
    pub fn abstained(detail: impl Into<String>) -> Self {
        Self {
            status: EngineStatus::Abstained,
            text: None,
            detail: Some(detail.into()),
        }
    }

    /// A tier whose output was rejected as pathological or unreadable.
    pub fn pathological(detail: impl Into<String>) -> Self {
        Self {
            status: EngineStatus::Pathological,
            text: None,
            detail: Some(detail.into()),
        }
    }

    /// A tier that failed outright (runtime unavailable, generation error).
    pub fn failed(detail: impl Into<String>) -> Self {
        Self {
            status: EngineStatus::Failed,
            text: None,
            detail: Some(detail.into()),
        }
    }
}

/// Per-tier record of an attempted cascade step, in attempt order.
#[derive(Debug, Clone, Serialize)]
pub struct TierOutcome {
    pub tier: &'static str,
    pub status: EngineStatus,
    pub detail: Option<String>,
}

/// Consolidated result of a serving-cascade run: the served text (or
/// `None` when every attempted tier declined), the tier that served it,
/// and the full per-tier trail of attempted outcomes.
#[derive(Debug, Clone, Serialize)]
pub struct CascadeOutcome {
    pub text: Option<String>,
    pub served_by: Option<&'static str>,
    pub trail: Vec<TierOutcome>,
}

/// A named cascade tier's generation closure.
pub type TierFn<'a> = Box<dyn FnMut() -> TierResult + 'a>;

/// Run an ordered serving cascade.
///
/// The first tier returning [`EngineStatus::Success`] with non-empty text
/// serves; every attempted tier's outcome is recorded in the trail, and a
/// declared abstention is recorded while later tiers still attempt (the
/// PR #223 record-and-continue semantics — previously a
/// policy parameter whose alternative was never selected, inlined by
/// #790 item 4). A `Success` without text is downgraded to a recorded
/// `Failed` so a contradictory tier cannot serve emptiness.
pub fn run_cascade(tiers: Vec<(&'static str, TierFn<'_>)>) -> CascadeOutcome {
    let mut trail = Vec::with_capacity(tiers.len());
    for (tier, mut run) in tiers {
        let mut result = run();
        if result.status == EngineStatus::Success {
            match result.text.take().filter(|text| !text.trim().is_empty()) {
                Some(text) => {
                    trail.push(TierOutcome {
                        tier,
                        status: EngineStatus::Success,
                        detail: result.detail,
                    });
                    return CascadeOutcome {
                        text: Some(text),
                        served_by: Some(tier),
                        trail,
                    };
                }
                None => {
                    trail.push(TierOutcome {
                        tier,
                        status: EngineStatus::Failed,
                        detail: Some("tier reported success without text".to_owned()),
                    });
                    continue;
                }
            }
        }
        tracing::info!(
            target: "uor_r4_router::fallback",
            tier,
            status = %result.status,
            detail = result.detail.as_deref().unwrap_or(""),
            "Cascade tier declined"
        );
        trail.push(TierOutcome {
            tier,
            status: result.status,
            detail: result.detail,
        });
    }
    CascadeOutcome {
        text: None,
        served_by: None,
        trail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_cascade_first_success_wins() {
        let mut later_tier_called = false;
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            (
                "r4g1",
                Box::new(|| TierResult::success("first tier text".to_owned())),
            ),
            (
                "transformerless",
                Box::new(|| {
                    later_tier_called = true;
                    TierResult::success("never served".to_owned())
                }),
            ),
        ];
        let outcome = run_cascade(tiers);
        assert_eq!(outcome.text.as_deref(), Some("first tier text"));
        assert_eq!(outcome.served_by, Some("r4g1"));
        assert_eq!(outcome.trail.len(), 1);
        assert_eq!(outcome.trail[0].tier, "r4g1");
        assert_eq!(outcome.trail[0].status, EngineStatus::Success);
        assert!(!later_tier_called);
    }

    #[test]
    fn test_run_cascade_abstain_continues_and_records() {
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            (
                "r4g1",
                Box::new(|| TierResult::abstained("R4G1 policy abstained (status: novel)")),
            ),
            (
                "transformerless",
                Box::new(|| TierResult::success("served after abstention".to_owned())),
            ),
        ];
        let outcome = run_cascade(tiers);
        assert_eq!(outcome.text.as_deref(), Some("served after abstention"));
        assert_eq!(outcome.served_by, Some("transformerless"));
        assert_eq!(outcome.trail.len(), 2);
        assert_eq!(outcome.trail[0].status, EngineStatus::Abstained);
        assert_eq!(
            outcome.trail[0].detail.as_deref(),
            Some("R4G1 policy abstained (status: novel)")
        );
        assert_eq!(outcome.trail[1].status, EngineStatus::Success);
    }

    #[test]
    fn test_run_cascade_pathological_records_detail() {
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            (
                "r4g1",
                Box::new(|| TierResult::pathological("repeated word loop detected")),
            ),
            (
                "transformerless",
                Box::new(|| TierResult::success("clean recovery".to_owned())),
            ),
        ];
        let outcome = run_cascade(tiers);
        assert_eq!(outcome.served_by, Some("transformerless"));
        assert_eq!(outcome.trail[0].status, EngineStatus::Pathological);
        assert_eq!(
            outcome.trail[0].detail.as_deref(),
            Some("repeated word loop detected")
        );
    }

    #[test]
    fn test_run_cascade_declined_by_all_trail_shape() {
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            ("r4g1", Box::new(|| TierResult::abstained("novel input"))),
            (
                "transformerless",
                Box::new(|| TierResult::failed("runtime unavailable")),
            ),
            (
                "teacher-oracle",
                Box::new(|| TierResult::pathological("gibberish output")),
            ),
            (
                "geometric",
                Box::new(|| TierResult::abstained("manifold resonance too sparse for synthesis")),
            ),
        ];
        let outcome = run_cascade(tiers);
        assert!(outcome.text.is_none());
        assert!(outcome.served_by.is_none());
        assert_eq!(outcome.trail.len(), 4);
        let statuses: Vec<EngineStatus> = outcome.trail.iter().map(|step| step.status).collect();
        assert_eq!(
            statuses,
            vec![
                EngineStatus::Abstained,
                EngineStatus::Failed,
                EngineStatus::Pathological,
                EngineStatus::Abstained,
            ]
        );
        let tiers_in_trail: Vec<&str> = outcome.trail.iter().map(|step| step.tier).collect();
        assert_eq!(
            tiers_in_trail,
            vec!["r4g1", "transformerless", "teacher-oracle", "geometric"]
        );
        assert!(outcome.trail.iter().all(|step| step.detail.is_some()));
    }

    #[test]
    fn test_run_cascade_pinned_single_tier() {
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![(
            "geometric",
            Box::new(|| TierResult::abstained("manifold resonance too sparse for synthesis")),
        )];
        let outcome = run_cascade(tiers);
        assert!(outcome.text.is_none());
        assert!(outcome.served_by.is_none());
        assert_eq!(outcome.trail.len(), 1);
        assert_eq!(outcome.trail[0].tier, "geometric");
        assert_eq!(outcome.trail[0].status, EngineStatus::Abstained);
    }

    #[test]
    fn test_run_cascade_success_without_text_cannot_serve() {
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            (
                "r4g1",
                Box::new(|| TierResult {
                    status: EngineStatus::Success,
                    text: Some("   ".to_owned()),
                    detail: None,
                }),
            ),
            (
                "transformerless",
                Box::new(|| TierResult::success("real text".to_owned())),
            ),
        ];
        let outcome = run_cascade(tiers);
        assert_eq!(outcome.text.as_deref(), Some("real text"));
        assert_eq!(outcome.trail[0].status, EngineStatus::Failed);
    }
}

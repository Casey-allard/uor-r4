//! Dynamic Fallback Engine Pipeline (`FallbackRouter`).
//!
//! Formalizes dynamic engine fallback: when primary `R4G1` graph inference
//! encounters an unmapped region, pathological loop, or `Novel`/`Contradictory`
//! state status, `FallbackRouter` seamlessly cascades to secondary `transformerless`
//! (TLA5/TLS1) engine generation, returning a valid response without dropping HTTP/WS payloads.
//!
//! Issue #248 adds the generalized N-tier serving cascade: [`run_cascade`]
//! walks an ordered list of named tier closures, serves the first
//! [`EngineStatus::Success`], records every attempted tier's typed outcome
//! in a [`CascadeOutcome::trail`], and — under the central
//! [`SERVING_ABSTAIN_POLICY`] — treats a declared abstention as a recorded
//! event the cascade continues past, not a refusal to try later tiers.

use serde::{Deserialize, Serialize};

/// Error classification of engine execution outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineStatus {
    /// Successful inference generation.
    Success,
    /// Primary engine encountered an unmapped region or novel context.
    UnmappedRegion,
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
            EngineStatus::UnmappedRegion => write!(f, "unmapped_region"),
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

/// Whether a declared tier abstention ends the cascade or is recorded
/// while later tiers still get to attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbstainPolicy {
    /// Record the abstention in the trail and continue to the next tier
    /// (PR #223 semantics: recording, not refusing).
    Cascade,
    /// Record the abstention and end the cascade with no text.
    Terminal,
}

/// The centralized serving policy for abstentions (issue #248). Both HTTP
/// serving endpoints route through this one constant, so flipping the
/// abstention semantics later is a one-line change.
pub const SERVING_ABSTAIN_POLICY: AbstainPolicy = AbstainPolicy::Cascade;

/// A named cascade tier's generation closure.
pub type TierFn<'a> = Box<dyn FnMut() -> TierResult + 'a>;

/// Run an ordered serving cascade under [`SERVING_ABSTAIN_POLICY`].
///
/// The first tier returning [`EngineStatus::Success`] with non-empty text
/// serves; every attempted tier's outcome is recorded in the trail. A
/// `Success` without text is downgraded to a recorded `Failed` so a
/// contradictory tier cannot serve emptiness.
pub fn run_cascade(tiers: Vec<(&'static str, TierFn<'_>)>) -> CascadeOutcome {
    run_cascade_with_policy(tiers, SERVING_ABSTAIN_POLICY)
}

/// [`run_cascade`] with an explicit abstention policy.
pub fn run_cascade_with_policy(
    tiers: Vec<(&'static str, TierFn<'_>)>,
    policy: AbstainPolicy,
) -> CascadeOutcome {
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
        let terminal =
            result.status == EngineStatus::Abstained && policy == AbstainPolicy::Terminal;
        trail.push(TierOutcome {
            tier,
            status: result.status,
            detail: result.detail,
        });
        if terminal {
            break;
        }
    }
    CascadeOutcome {
        text: None,
        served_by: None,
        trail,
    }
}

/// Response returned by an individual engine step/generation call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineResponse {
    pub text: String,
    pub status: EngineStatus,
    pub engine: String,
    pub tokens_generated: usize,
}

/// Consolidated result from `FallbackRouter::execute`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResult {
    pub text: String,
    pub primary_status: EngineStatus,
    pub fallback_triggered: bool,
    pub active_engine: String,
    pub tokens_generated: usize,
}

/// A dynamic fallback router managing primary (R4G1) and secondary (TLA5) inference engines.
#[derive(Debug, Clone)]
pub struct FallbackRouter {
    primary_name: String,
    secondary_name: String,
}

impl Default for FallbackRouter {
    fn default() -> Self {
        Self {
            primary_name: "r4g1-graph".to_string(),
            secondary_name: "transformerless-tla5".to_string(),
        }
    }
}

impl FallbackRouter {
    /// Create a new `FallbackRouter` with custom engine identifiers.
    pub fn new(primary_name: impl Into<String>, secondary_name: impl Into<String>) -> Self {
        Self {
            primary_name: primary_name.into(),
            secondary_name: secondary_name.into(),
        }
    }

    /// Primary engine name.
    pub fn primary_name(&self) -> &str {
        &self.primary_name
    }

    /// Secondary engine name.
    pub fn secondary_name(&self) -> &str {
        &self.secondary_name
    }

    /// Execute inference generation with automatic fallback cascading.
    ///
    /// Evaluates `primary_fn`. If it returns [`EngineStatus::Success`], returns
    /// that result directly. For any other status (`UnmappedRegion`,
    /// `Pathological`, `Failed`, `Abstained`) it logs a `tracing::info!`
    /// fallback event and invokes `secondary_fn`, returning a clean
    /// [`FallbackResult`].
    ///
    /// Each engine closure returns an [`EngineResponse`] directly: the outcome
    /// — success or otherwise — is carried in the response's own
    /// [`EngineStatus`], not a separate error channel. A failed engine reports
    /// `EngineStatus::Failed` with its explanatory text, so there is no bound
    /// on how an engine may report (R5 — the closures are total).
    pub fn execute<FPrimary, FSecondary>(
        &self,
        mut primary_fn: FPrimary,
        mut secondary_fn: FSecondary,
    ) -> FallbackResult
    where
        FPrimary: FnMut() -> EngineResponse,
        FSecondary: FnMut() -> EngineResponse,
    {
        let primary_res = primary_fn();
        if primary_res.status == EngineStatus::Success {
            return FallbackResult {
                text: primary_res.text,
                primary_status: EngineStatus::Success,
                fallback_triggered: false,
                active_engine: primary_res.engine,
                tokens_generated: primary_res.tokens_generated,
            };
        }
        tracing::info!(
            target: "uor_r4_router::fallback",
            primary = %self.primary_name,
            secondary = %self.secondary_name,
            primary_status = %primary_res.status,
            "Primary engine status requiring fallback; cascading to secondary engine"
        );
        let sec_res = secondary_fn();
        FallbackResult {
            text: sec_res.text,
            primary_status: primary_res.status,
            fallback_triggered: true,
            active_engine: sec_res.engine,
            tokens_generated: sec_res.tokens_generated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_router_primary_success() {
        let router = FallbackRouter::default();
        let res = router.execute(
            || EngineResponse {
                text: "Primary clean output".to_string(),
                status: EngineStatus::Success,
                engine: "r4g1-graph".to_string(),
                tokens_generated: 10,
            },
            || panic!("secondary should not be called"),
        );

        assert!(!res.fallback_triggered);
        assert_eq!(res.primary_status, EngineStatus::Success);
        assert_eq!(res.active_engine, "r4g1-graph");
        assert_eq!(res.text, "Primary clean output");
    }

    #[test]
    fn test_fallback_router_unmapped_region_triggers_fallback() {
        let router = FallbackRouter::default();
        let res = router.execute(
            || EngineResponse {
                text: "".to_string(),
                status: EngineStatus::UnmappedRegion,
                engine: "r4g1-graph".to_string(),
                tokens_generated: 0,
            },
            || EngineResponse {
                text: "Secondary fallback output".to_string(),
                status: EngineStatus::Success,
                engine: "transformerless-tla5".to_string(),
                tokens_generated: 8,
            },
        );

        assert!(res.fallback_triggered);
        assert_eq!(res.primary_status, EngineStatus::UnmappedRegion);
        assert_eq!(res.active_engine, "transformerless-tla5");
        assert_eq!(res.text, "Secondary fallback output");
    }

    #[test]
    fn test_fallback_router_pathological_loop_triggers_fallback() {
        let router = FallbackRouter::default();
        let res = router.execute(
            || EngineResponse {
                text: "Loop detected".to_string(),
                status: EngineStatus::Pathological,
                engine: "r4g1-graph".to_string(),
                tokens_generated: 2,
            },
            || EngineResponse {
                text: "Secondary fallback recovery".to_string(),
                status: EngineStatus::Success,
                engine: "transformerless-tla5".to_string(),
                tokens_generated: 12,
            },
        );

        assert!(res.fallback_triggered);
        assert_eq!(res.primary_status, EngineStatus::Pathological);
        assert_eq!(res.active_engine, "transformerless-tla5");
        assert_eq!(res.text, "Secondary fallback recovery");
    }

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
        let outcome = run_cascade_with_policy(tiers, AbstainPolicy::Cascade);
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
    fn test_run_cascade_terminal_policy_stops_at_abstention() {
        let mut later_tier_called = false;
        let tiers: Vec<(&'static str, TierFn<'_>)> = vec![
            ("r4g1", Box::new(|| TierResult::abstained("novel input"))),
            (
                "transformerless",
                Box::new(|| {
                    later_tier_called = true;
                    TierResult::success("never attempted".to_owned())
                }),
            ),
        ];
        let outcome = run_cascade_with_policy(tiers, AbstainPolicy::Terminal);
        assert!(outcome.text.is_none());
        assert!(outcome.served_by.is_none());
        assert_eq!(outcome.trail.len(), 1);
        assert_eq!(outcome.trail[0].status, EngineStatus::Abstained);
        assert!(!later_tier_called);
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

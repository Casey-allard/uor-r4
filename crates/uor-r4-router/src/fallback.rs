//! Dynamic Fallback Engine Pipeline (`FallbackRouter`).
//!
//! Formalizes dynamic engine fallback: when primary `R4G1` graph inference
//! encounters an unmapped region, pathological loop, or `Novel`/`Contradictory`
//! state status, `FallbackRouter` seamlessly cascades to secondary `transformerless`
//! (TLA5/TLS1) engine generation, returning a valid response without dropping HTTP/WS payloads.

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
}

impl std::fmt::Display for EngineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineStatus::Success => write!(f, "success"),
            EngineStatus::UnmappedRegion => write!(f, "unmapped_region"),
            EngineStatus::Pathological => write!(f, "pathological"),
            EngineStatus::Failed => write!(f, "failed"),
        }
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
    /// Evaluates `primary_fn`. If `primary_fn` returns `EngineStatus::Success`, returns the result directly.
    /// If `primary_fn` returns `UnmappedRegion` or `Pathological`, logs a `tracing::info!` fallback event
    /// and invokes `secondary_fn`, returning a clean `FallbackResult`.
    pub fn execute<FPrimary, FSecondary>(
        &self,
        mut primary_fn: FPrimary,
        mut secondary_fn: FSecondary,
    ) -> FallbackResult
    where
        FPrimary: FnMut() -> Result<EngineResponse, String>,
        FSecondary: FnMut() -> Result<EngineResponse, String>,
    {
        match primary_fn() {
            Ok(res) if res.status == EngineStatus::Success => FallbackResult {
                text: res.text,
                primary_status: EngineStatus::Success,
                fallback_triggered: false,
                active_engine: res.engine,
                tokens_generated: res.tokens_generated,
            },
            Ok(primary_res) => {
                tracing::info!(
                    target: "uor_r4_router::fallback",
                    primary = %self.primary_name,
                    secondary = %self.secondary_name,
                    primary_status = %primary_res.status,
                    "Primary engine status requiring fallback; cascading to secondary engine"
                );
                match secondary_fn() {
                    Ok(sec_res) => FallbackResult {
                        text: sec_res.text,
                        primary_status: primary_res.status,
                        fallback_triggered: true,
                        active_engine: sec_res.engine,
                        tokens_generated: sec_res.tokens_generated,
                    },
                    Err(err) => FallbackResult {
                        text: format!("Fallback engine error: {err}"),
                        primary_status: primary_res.status,
                        fallback_triggered: true,
                        active_engine: self.secondary_name.clone(),
                        tokens_generated: 0,
                    },
                }
            }
            Err(err) => {
                tracing::info!(
                    target: "uor_r4_router::fallback",
                    primary = %self.primary_name,
                    secondary = %self.secondary_name,
                    error = %err,
                    "Primary engine returned error; cascading to secondary fallback engine"
                );
                match secondary_fn() {
                    Ok(sec_res) => FallbackResult {
                        text: sec_res.text,
                        primary_status: EngineStatus::Failed,
                        fallback_triggered: true,
                        active_engine: sec_res.engine,
                        tokens_generated: sec_res.tokens_generated,
                    },
                    Err(sec_err) => FallbackResult {
                        text: format!("Primary error: {err}; Secondary error: {sec_err}"),
                        primary_status: EngineStatus::Failed,
                        fallback_triggered: true,
                        active_engine: self.secondary_name.clone(),
                        tokens_generated: 0,
                    },
                }
            }
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
            || {
                Ok(EngineResponse {
                    text: "Primary clean output".to_string(),
                    status: EngineStatus::Success,
                    engine: "r4g1-graph".to_string(),
                    tokens_generated: 10,
                })
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
            || {
                Ok(EngineResponse {
                    text: "".to_string(),
                    status: EngineStatus::UnmappedRegion,
                    engine: "r4g1-graph".to_string(),
                    tokens_generated: 0,
                })
            },
            || {
                Ok(EngineResponse {
                    text: "Secondary fallback output".to_string(),
                    status: EngineStatus::Success,
                    engine: "transformerless-tla5".to_string(),
                    tokens_generated: 8,
                })
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
            || {
                Ok(EngineResponse {
                    text: "Loop detected".to_string(),
                    status: EngineStatus::Pathological,
                    engine: "r4g1-graph".to_string(),
                    tokens_generated: 2,
                })
            },
            || {
                Ok(EngineResponse {
                    text: "Secondary fallback recovery".to_string(),
                    status: EngineStatus::Success,
                    engine: "transformerless-tla5".to_string(),
                    tokens_generated: 12,
                })
            },
        );

        assert!(res.fallback_triggered);
        assert_eq!(res.primary_status, EngineStatus::Pathological);
        assert_eq!(res.active_engine, "transformerless-tla5");
        assert_eq!(res.text, "Secondary fallback recovery");
    }
}

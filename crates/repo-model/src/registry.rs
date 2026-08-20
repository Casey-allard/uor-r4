//! The typed shape of `model/*.toml`.
//!
//! Nothing here interprets the model; [`crate::Model::check`] does that. These
//! types exist so that a malformed model is a parse error rather than a
//! silently wrong constant.

use serde::Deserialize;

use crate::ModelError;

/// One of the three honesty levels (R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// A fact reproduced from an authority. Not established here.
    SomeTrue,
    /// Constructed here and validated against its oracle.
    Build,
    /// Measured and reported, never asserted.
    Open,
}

impl Level {
    /// The token used in `model/*.toml` and in generated documentation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SomeTrue => "some-true",
            Self::Build => "build",
            Self::Open => "open",
        }
    }
}

/// Execution scope: the evidence class a capability row sits in (#830, item B
/// of #821).
///
/// The register previously could not distinguish a reference/oracle model from
/// an offline compiler stage, a certifier instrument, a dormant portable-runtime
/// mechanism, the normative runtime contract, or the deployed production call
/// graph. Conflating them permits over-reading a structural harness as a
/// production result, which is exactly what this scope makes machine-visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    /// A reference / oracle model, not the deployed integer path (often f32).
    ReferenceOnly,
    /// Offline compiler behaviour: runs at compile time, never at serving time.
    OfflineCompiler,
    /// A certifier, instrument, or measurement harness.
    CertifierInstrument,
    /// A portable-runtime mechanism retained dormant behind an activation gate.
    DormantPortableRuntime,
    /// The normative runtime contract/semantics (the one normative scorer's
    /// rules). The single-normative-scorer designation is #831 (item C of #821).
    NormativeRuntime,
    /// The actual reachable production call graph / released binary. Requires a
    /// `deployed-serving` reachability assertion (enforced by [`crate::Model::check`]).
    DeployedProduction,
}

impl Scope {
    /// The token used in `model/*.toml` and in generated documentation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReferenceOnly => "reference-only",
            Self::OfflineCompiler => "offline-compiler",
            Self::CertifierInstrument => "certifier-instrument",
            Self::DormantPortableRuntime => "dormant-portable-runtime",
            Self::NormativeRuntime => "normative-runtime",
            Self::DeployedProduction => "deployed-production",
        }
    }
}

/// Serving reachability: whether a capability is reached on the deployed `r4`
/// serving/chat call graph (#830).
///
/// Kept as its own axis, separate from [`Scope`], so that a non-production test
/// cannot be cited as production evidence without an explicit, dedicated
/// reachability assertion. The default is the non-crediting
/// [`Reachability::OffServingPath`]: a row is not read as deployed-serving
/// evidence unless it says so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reachability {
    /// Exercised on the deployed serving/chat path (the `R4G1Runtime` /
    /// `R4Engine` call graph reached by `src/tless_uor.rs` and `src/chat.rs`).
    DeployedServing,
    /// Built or measured, but not reached by the serving path.
    OffServingPath,
    /// Retained behind an activation gate registered in `model/ledger.toml`.
    DormantGated,
}

impl Reachability {
    /// The token used in `model/*.toml` and in generated documentation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeployedServing => "deployed-serving",
            Self::OffServingPath => "off-serving-path",
            Self::DormantGated => "dormant-gated",
        }
    }
}

/// The least-crediting execution scope, used as the migration default so a row
/// that omits `scope` understates rather than over-reads (#830).
fn default_scope() -> Scope {
    Scope::ReferenceOnly
}

/// The non-crediting serving reachability, used as the migration default so a
/// row is never read as deployed-serving evidence without an explicit
/// assertion (#830).
fn default_reachability() -> Reachability {
    Reachability::OffServingPath
}

/// `model/ledger.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Ledger {
    /// The schema tag.
    pub spec: String,
    /// One row per claim.
    pub claim: Vec<Claim>,
}

/// One claim, at exactly one honesty level.
#[derive(Debug, Clone, Deserialize)]
pub struct Claim {
    /// The conformance ID, or an `AUTH-`/`OPEN-` prefixed identifier.
    pub id: String,
    /// The honesty level. Untagged claims do not ship (R2).
    pub level: Level,
    /// What is claimed.
    pub statement: String,
    /// The Gherkin file carrying the scenario (R3).
    #[serde(default)]
    pub feature: Option<String>,
    /// The authority a `some-true` claim is reproduced from.
    #[serde(default)]
    pub authority: Option<String>,
    /// Recorded sample size, for a claim that is a statistic.
    #[serde(default)]
    pub sample_size: Option<u64>,
    /// Recorded seed, for a claim that is a statistic.
    #[serde(default)]
    pub seed: Option<u64>,
}

impl Ledger {
    /// The meta-gate's structural half: every claim is well formed for its
    /// level (R2).
    ///
    /// The behavioural half --- that no test asserts an `open` claim as
    /// established --- lives in `repo-conformance`, because it needs the
    /// test names, not the model.
    pub fn check(&self) -> Result<(), ModelError> {
        for c in &self.claim {
            match c.level {
                Level::SomeTrue => {
                    if c.authority.is_none() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: a some-true claim must name the authority it is \
                             reproduced from",
                            c.id
                        )));
                    }
                }
                Level::Build => {
                    if c.feature.is_none() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: a build claim must name the Gherkin scenario that \
                             validates it (R3)",
                            c.id
                        )));
                    }
                    if c.authority.is_some() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: a build claim is evidence, not a reproduction of an \
                             authority; it must not name one",
                            c.id
                        )));
                    }
                }
                Level::Open => {
                    if c.authority.is_some() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: an open claim is a measurement and cannot cite an \
                             authority for its value",
                            c.id
                        )));
                    }
                }
            }
            // No rules about a *class* of ID here. `CP-` recording a sample size,
            // `CG-` being measured rather than asserted, `CN-` not existing at
            // all --- each was a fact about a repository that had that class, and
            // a rule enforcing a taxonomy the register does not have is a
            // restriction on the first person to want one. A repository adding a
            // class adds its rule here, in the commit that adds the first ID in
            // it. The level rules above apply to every claim and stay.
        }
        Ok(())
    }

    /// Look up a claim by conformance ID.
    pub fn get(&self, id: &str) -> Option<&Claim> {
        self.claim.iter().find(|c| c.id == id)
    }
}

/// `model/ids.toml` --- the conformance ID register.
#[derive(Debug, Clone, Deserialize)]
pub struct Ids {
    /// The schema tag.
    pub spec: String,
    /// One row per conformance ID.
    pub id: Vec<IdRow>,
}

/// One registered conformance ID.
#[derive(Debug, Clone, Deserialize)]
pub struct IdRow {
    /// The ID, e.g. `CS-04`.
    pub id: String,
    /// The honesty level of the claim (R2).
    pub level: Level,
    /// The Gherkin suite the scenario belongs to.
    pub suite: String,
    /// What the ID claims.
    pub statement: String,
    /// Execution scope: the evidence class this row sits in (#830). Defaults to
    /// the least-crediting `reference-only`, so a row that omits it understates.
    #[serde(default = "default_scope")]
    pub scope: Scope,
    /// Serving reachability (#830). Defaults to `off-serving-path`: a row is not
    /// credited as deployed-serving production evidence without an explicit
    /// assertion.
    #[serde(default = "default_reachability")]
    pub reachability: Reachability,
    /// Evidence pointer: the harness, suite, or source that validates a `build`
    /// claim (#830). Required non-empty for a `build` row (enforced by
    /// [`crate::Model::check`]). This is harness-built (structural) evidence —
    /// it is never, on its own, an empirical PASS (see [`crate::empirical`]).
    #[serde(default)]
    pub evidence: String,
}

impl Ids {
    /// Look up a row.
    pub fn get(&self, id: &str) -> Option<&IdRow> {
        self.id.iter().find(|r| r.id == id)
    }
}

/// `model/authorities.toml` --- what this repository cites (`CM-03`).
#[derive(Debug, Clone, Deserialize)]
pub struct Authorities {
    /// The schema tag.
    pub spec: String,
    /// One row per cited authority.
    pub authority: Vec<AuthorityRow>,
}

/// A cited authority. Never re-derived, vendored, or gated on.
#[derive(Debug, Clone, Deserialize)]
pub struct AuthorityRow {
    /// Stable identifier, e.g. `CL-MM01`.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// What a third party needs to find the source.
    pub citation: String,
    /// A checksum over the committed artifact, or `none`.
    pub checksum: String,
    /// Why there is no checksum, when there is none.
    #[serde(default)]
    pub checksum_reason: String,
    /// What the authority says.
    pub statement: String,
    /// The conformance IDs that are evidence this library realizes it.
    #[serde(default)]
    pub realized_by: Vec<String>,
}

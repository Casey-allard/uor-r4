//! Typed registries parsed from `model/*.toml`.
//!
//! The model is authored once and has exactly one source (R1): the conformance
//! ID register, the claim ledger, and the authorities this repository cites.
//! `CONFORMANCE.md` is generated from it by [`codegen`], so a claim cannot exist
//! in the documentation without a ledger row, or in the ledger without appearing
//! in the documentation.
//!
//! This crate is build-time and CI infrastructure. It is not a dependency of
//! any shipped crate, and it may use `std`.

#![deny(missing_docs)]

pub mod codegen;
pub mod empirical;
pub mod registry;

pub use empirical::EmpiricalStatus;
pub use registry::{
    Authorities, AuthorityRow, Claim, IdRow, Ids, Ledger, Level, Reachability, Scope,
};

use std::path::{Path, PathBuf};

/// Everything `model/*.toml` says, parsed and cross-checked.
#[derive(Debug, Clone)]
pub struct Model {
    /// `model/ledger.toml`: one row per claim, at exactly one honesty level.
    pub ledger: Ledger,
    /// `model/ids.toml`: the conformance ID register.
    pub ids: Ids,
    /// `model/authorities.toml`: what this repository cites rather than proves.
    pub authorities: Authorities,
}

/// A failure to load or to cross-check the model.
#[derive(Debug)]
pub enum ModelError {
    /// A model file could not be read.
    Io(PathBuf, std::io::Error),
    /// A model file could not be parsed.
    Parse(PathBuf, toml::de::Error),
    /// The model disagrees with itself, or with a derivation (CM-01).
    Inconsistent(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "reading {}: {e}", p.display()),
            Self::Parse(p, e) => write!(f, "parsing {}: {e}", p.display()),
            Self::Inconsistent(m) => write!(f, "model is inconsistent: {m}"),
        }
    }
}

impl std::error::Error for ModelError {}

impl Model {
    /// Load every model file from a `model/` directory.
    pub fn load(dir: &Path) -> Result<Self, ModelError> {
        Ok(Self {
            ledger: read(dir, "ledger.toml")?,
            ids: read(dir, "ids.toml")?,
            authorities: read(dir, "authorities.toml")?,
        })
    }

    /// Load the model from the repository root, resolved from this crate's
    /// manifest directory so that it works from any working directory.
    pub fn load_from_repo_root() -> Result<Self, ModelError> {
        Self::load(&repo_root().join("model"))
    }

    /// Cross-check the model against itself: every ID well formed, every claim
    /// well formed for its level, and every `some-true` claim bound to an
    /// authority that exists (`CM-01` .. `CM-03`, R2).
    pub fn check(&self) -> Result<(), ModelError> {
        self.ledger.check()?;
        self.check_ids()?;
        self.check_authorities()?;
        Ok(())
    }

    /// `CM-02`: every registered ID is well formed.
    ///
    /// The structural rules only. Rules about a *class* of ID --- that a fitted
    /// exponent is always `open`, that a cross-library result is never
    /// `some-true` --- belong to the repository that has those classes, and they
    /// belong to the repository that has those classes. A repository adding one adds
    /// its rule here, in the same commit that adds the first ID in it.
    fn check_ids(&self) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);
        let mut seen: Vec<&str> = Vec::new();
        for row in &self.ids.id {
            if seen.contains(&row.id.as_str()) {
                return Err(bad(format!("{}: registered twice", row.id)));
            }
            seen.push(&row.id);

            if row.statement.trim().is_empty() {
                return Err(bad(format!(
                    "{}: an untagged claim does not ship (R2)",
                    row.id
                )));
            }
            if row.suite.trim().is_empty() {
                return Err(bad(format!(
                    "{}: every ID names the Gherkin suite its scenario lives in (R3)",
                    row.id
                )));
            }
            // #830: a build claim must point at the evidence that validates it,
            // so "constructed here" is never an unbacked assertion. This is the
            // harness-built (structural) pointer; it is not, on its own, an
            // empirical PASS.
            if row.level == Level::Build && row.evidence.trim().is_empty() {
                return Err(bad(format!(
                    "{}: a build claim must carry an evidence pointer (#830). A \
                     constructed capability names the harness, suite, or source \
                     that validates it.",
                    row.id
                )));
            }
            // #830: a non-production test cannot be cited as production evidence
            // without a dedicated reachability assertion. Claiming the
            // deployed-production scope requires stating deployed-serving
            // reachability explicitly.
            if row.scope == Scope::DeployedProduction
                && row.reachability != Reachability::DeployedServing
            {
                return Err(bad(format!(
                    "{}: scope `deployed-production` requires `reachability = \
                     deployed-serving` (#830): a non-production test cannot be \
                     cited as production evidence without a dedicated \
                     reachability assertion.",
                    row.id
                )));
            }
        }
        Ok(())
    }

    /// `CM-03`: every `some-true` claim has a row in `model/authorities.toml`
    /// with a citation, and every authority names IDs that exist.
    fn check_authorities(&self) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);
        for a in &self.authorities.authority {
            if a.citation.trim().is_empty() {
                return Err(bad(format!("{}: an authority with no citation", a.id)));
            }
            if a.checksum == "none" && a.checksum_reason.trim().is_empty() {
                return Err(bad(format!(
                    "{}: no checksum and no reason. A missing checksum must be a stated \
                     fact, not an omission (R6)",
                    a.id
                )));
            }
            for id in &a.realized_by {
                if self.ids.get(id).is_none() {
                    return Err(bad(format!("{}: realized_by names unknown ID {id}", a.id)));
                }
            }
        }
        // Every some-true claim in the ledger names a known authority.
        for c in &self.ledger.claim {
            if c.level != Level::SomeTrue {
                continue;
            }
            let Some(name) = &c.authority else {
                return Err(bad(format!(
                    "{}: a some-true claim must name an authority",
                    c.id
                )));
            };
            if !self.authorities.authority.iter().any(|a| &a.id == name) {
                return Err(bad(format!(
                    "{}: cites {name}, which has no row in model/authorities.toml (CM-03)",
                    c.id
                )));
            }
        }
        Ok(())
    }
}

fn read<T: serde::de::DeserializeOwned>(dir: &Path, name: &str) -> Result<T, ModelError> {
    let path = dir.join(name);
    let text = std::fs::read_to_string(&path).map_err(|e| ModelError::Io(path.clone(), e))?;
    toml::from_str(&text).map_err(|e| ModelError::Parse(path, e))
}

/// The repository root, resolved at **runtime**.
///
/// #788: this must not use compile-time `env!("CARGO_MANIFEST_DIR")` —
/// a cached rlib carries the path of whatever checkout (or since-deleted
/// worktree) built it, which poisoned the R1/R2/R3/R4 register gates and
/// let R5 pass vacuously against a nonexistent root (AUD-VER-001).
/// Runtime resolution order:
/// 1. `CARGO_MANIFEST_DIR` from the process environment (cargo sets it
///    for `cargo run`/`cargo test`), walking up to the workspace root;
/// 2. otherwise walk up from the current directory to the first ancestor
///    containing `model/ledger.toml` (direct binary invocation).
pub fn repo_root() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        for ancestor in Path::new(&manifest_dir).ancestors() {
            if ancestor.join("model/ledger.toml").is_file() {
                return ancestor.to_path_buf();
            }
        }
    }
    let cwd = std::env::current_dir().expect("current directory is readable");
    for ancestor in cwd.ancestors() {
        if ancestor.join("model/ledger.toml").is_file() {
            return ancestor.to_path_buf();
        }
    }
    panic!(
        "repository root not found: no ancestor of CARGO_MANIFEST_DIR or the \
         current directory contains model/ledger.toml"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CM-01: the model is self-consistent and every numeral in it derives.
    #[test]
    fn model_is_consistent_cm_01() {
        let model = Model::load_from_repo_root().expect("model loads");
        model.check().expect("model checks");
    }

    /// CM-02: every registered ID is unique and well formed.
    #[test]
    fn the_id_register_is_well_formed_cm_02() {
        let model = Model::load_from_repo_root().expect("model loads");
        model.check().expect("model checks");
        // No lower bound on the count. "More than fifty IDs" was a fact about
        // the repository this template was cut from, not a property of a
        // well-formed register, and a threshold copied forward would fail here
        // for the whole time the register is being rebuilt --- teaching whoever
        // is rebuilding it to delete the assertion. What `check` above enforces
        // is the part that is true at every size: no duplicate, no untagged
        // claim, no ID without a suite.
        let ids = model.ids.id.len();
        eprintln!("CM-02: {ids} registered IDs, each unique and tagged");
    }

    /// CM-03: every `some-true` claim cites an authority that exists.
    #[test]
    fn every_some_true_claim_cites_an_authority_cm_03() {
        let model = Model::load_from_repo_root().expect("model loads");
        for c in &model.ledger.claim {
            if c.level == Level::SomeTrue {
                let name = c
                    .authority
                    .as_ref()
                    .expect("a some-true claim names its authority");
                assert!(
                    model.authorities.authority.iter().any(|a| &a.id == name),
                    "{name}"
                );
            }
        }
    }

    /// A one-row model helper for the #830 schema guards.
    fn one_row_model(scope: Scope, reachability: Reachability, evidence: &str) -> Model {
        Model {
            ledger: Ledger {
                spec: "test".into(),
                claim: vec![],
            },
            authorities: Authorities {
                spec: "test".into(),
                authority: vec![],
            },
            ids: Ids {
                spec: "test".into(),
                id: vec![IdRow {
                    id: "RF-99".into(),
                    level: Level::Build,
                    suite: "s".into(),
                    statement: "a claim".into(),
                    scope,
                    reachability,
                    evidence: evidence.into(),
                }],
            },
        }
    }

    /// #830 planted negative: a row claiming `deployed-production` scope without
    /// a `deployed-serving` reachability assertion must be rejected. A
    /// production claim cannot ride in on a non-production test.
    #[test]
    fn deployed_production_without_serving_reachability_is_rejected() {
        let model = one_row_model(
            Scope::DeployedProduction,
            Reachability::OffServingPath,
            "features/suites/s.feature",
        );
        let err = model
            .check()
            .expect_err("deployed-production without deployed-serving must be rejected");
        assert!(
            format!("{err}").contains("deployed-serving"),
            "rejection must name the missing reachability assertion, got: {err}"
        );
    }

    /// #830 positive control: the same row with the `deployed-serving`
    /// assertion (and a build evidence pointer) is accepted, so the guard above
    /// is not vacuous.
    #[test]
    fn deployed_production_with_serving_reachability_is_accepted() {
        let model = one_row_model(
            Scope::DeployedProduction,
            Reachability::DeployedServing,
            "features/suites/s.feature",
        );
        model
            .check()
            .expect("a deployed-production row with a serving assertion and evidence is valid");
    }

    /// #830: a `build` claim with no evidence pointer is rejected — "constructed
    /// here" must name what validates it.
    #[test]
    fn a_build_claim_without_an_evidence_pointer_is_rejected() {
        let model = one_row_model(
            Scope::CertifierInstrument,
            Reachability::OffServingPath,
            "  ",
        );
        let err = model
            .check()
            .expect_err("a build claim without evidence must be rejected");
        assert!(
            format!("{err}").contains("evidence pointer"),
            "rejection must name the missing evidence pointer, got: {err}"
        );
    }
}

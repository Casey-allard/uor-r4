//! Deterministic generator, 3-family oracles (`Compositions`, `AlteredConstraints`, `CodeTasks`),
//! overlap-exclusion verifier, and lawful count-table baseline noise evaluator for the frozen
//! `m6-reasoning` acceptance panel (`m6-reasoning-panel-freeze`, #2866 / #2034).
//!
//! # Contract (CHARTER §3.10, §4.4, §4.5, D22, D28)
//!
//! 1. **Seeded byte-exact reproducibility:** [`generate_m6_panel`] and [`generate_frozen_m6_panel`]
//!    deterministically synthesize `240` fresh rows (`80` `Compositions`, `80` `AlteredConstraints`,
//!    `80` `CodeTasks`) from [`M6_PANEL_FROZEN_SEED_V1`] (`0x004D_365F_5245_4153`).
//! 2. **Zero external teacher / zero neural judge (D28):** Every row is graded by a pure
//!    deterministic integer oracle ([`grade_m6_row`], [`grade_m6_panel`]):
//!    - `Compositions`: exact integer equality on both the intermediate addressed-memory / register
//!      state and the final output (`INT=<intermediate>;OUT=<final>`).
//!    - `AlteredConstraints`: exact value and altered-invariant satisfaction on satisfiable rows
//!      (explicitly distinguishing habit-capture errors that output `default_habit_answer`), and
//!      exact [`ABSTAIN_UNSATISFIABLE_TOKEN`] (`ABSTAIN:UNSATISFIABLE`) matching on unsatisfiable
//!      rows.
//!    - `CodeTasks`: total, zero-I/O, zero-allocation register-machine execution
//!      ([`SandboxProgram`]) with a hard step ceiling ([`MAX_SANDBOX_STEPS`] = `256`) and checked
//!      integer arithmetic across `4` held-out `(input, expected_output)` test vectors, plus a
//!      standalone Rust `#[test]` transpiler ([`SandboxProgram::to_rust_source`]) for offline
//!      `rustc --test` / `cargo test` verification.
//! 3. **Corpus & control overlap exclusion:** [`M6ReasoningPanel::verify_zero_overlap`] enforces
//!    zero collision with the finite CPU control literals in `m6_exact_arithmetic.rs` and
//!    `m6_ablation.rs`, zero duplicate row ids and prompts inside the panel (targets may repeat:
//!    altered-constraint answers recur), and zero overlap across independent seeds.
//! 4. **Lawful count-table baseline & noise measurement (CHARTER §4.4):**
//!    [`evaluate_count_table_baseline_noise`] trains a deterministic integer count table across
//!    independent seeded draws and measures the unconditioned numeric count-table score (`1..=3 / 240`
//!    from small-modulus residue collisions on `ModularStepCap`), the context-keyed count-table
//!    scores (`10..=12 / 240`, empirical half-band `+-1 rows`), and the family-conditioned modal
//!    count-table ceiling (`10 / 240` from the `10` `ABSTAIN:UNSATISFIABLE` rows).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::attention_reference::sha256_hex;

/// Schema tag for the frozen M6 reasoning and coding panel.
pub const M6_PANEL_SCHEMA: &str = "uor-r4.panel.m6-reasoning/1";

/// Canonical 64-bit seed (`"M6_REAS"` in ASCII) for the frozen 240-row `m6-reasoning` panel.
pub const M6_PANEL_FROZEN_SEED_V1: u64 = 0x004D_365F_5245_4153;

/// Number of fresh rows generated per task family (`80 * 3 = 240 >= 200` D22 §3 floor).
pub const M6_PANEL_ROWS_PER_FAMILY: usize = 80;

/// Total number of fresh rows in the frozen `m6-reasoning` panel.
pub const M6_PANEL_TOTAL_ROWS: usize = M6_PANEL_ROWS_PER_FAMILY * 3;

/// Hard instruction fuel ceiling for sandboxed `CodeTasks` execution.
pub const MAX_SANDBOX_STEPS: usize = 256;

/// Canonical honest-abstention token required on unsatisfiable `AlteredConstraints` rows.
pub const ABSTAIN_UNSATISFIABLE_TOKEN: &str = "ABSTAIN:UNSATISFIABLE";

/// Finite CPU control operand pairs from `crates/uor-r4-integer/src/m6_exact_arithmetic.rs`
/// (`control_ripple_carry_add_and_sub` and `control_refusal_and_causal_knockouts`) excluded from
/// the frozen panel by [`M6ReasoningPanel::verify_zero_overlap`].
pub const FORBIDDEN_CONTROL_PAIRS: &[(u64, u64)] = &[
    (0, 0),
    (7, 8),
    (9, 1),
    (99, 1),
    (999, 1),
    (4789, 5643),
    (12345, 67890),
    (50000, 50000),
    (999_999, 1),
    (314_159, 271_828),
    (808_080, 191_920),
    (987_654, 12_346),
    (0x00FF, 0x0001),
    (0xFFFF, 0x0001),
    (0x1A2B, 0x3C4D),
    (0xDEAD, 0xBEEF),
    (42, 99),
    (99, 77),
];

/// Finite CPU control operand triples from `crates/uor-r4-integer/src/m6_exact_arithmetic.rs`
/// (`control_addressed_memory_composition_and_multiply` and `control_refusal_and_causal_knockouts`)
/// excluded from the frozen panel by [`M6ReasoningPanel::verify_zero_overlap`].
pub const FORBIDDEN_CONTROL_TRIPLES: &[(u64, u64, u64)] = &[
    (999, 1, 500),
    (4789, 5643, 1432),
    (12000, 8500, 20500),
    (765, 432, 197),
    (99999, 1, 88888),
    (54321, 12345, 66660),
    (1024, 2048, 512),
    (3333, 7777, 1110),
    (12, 34, 56),
    (99, 99, 1),
    (125, 8, 1000),
    (37, 27, 1),
    (256, 16, 4096),
    (405, 19, 305),
    (700, 13, 900),
    (999, 9, 9),
    (99, 1, 10),
    (199, 2, 50),
    (299, 3, 100),
    (399, 4, 150),
    (499, 5, 200),
    (599, 6, 250),
    (699, 7, 300),
    (799, 8, 350),
    (899, 9, 400),
    (5555, 6666, 2221),
    (7777, 8888, 6665),
    (1234, 9876, 1110),
    (9009, 1091, 100),
    (9999, 1, 9000),
];

/// Deterministic SplitMix64 stream used to synthesize panel rows from a fixed 64-bit seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Construct a new deterministic SplitMix64 generator from `seed`.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the state and return the next 64-bit word.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Draw an integer deterministically in `lo..=hi_inclusive`.
    pub fn next_range(&mut self, lo: u64, hi_inclusive: u64) -> u64 {
        assert!(lo <= hi_inclusive, "invalid range {lo}..={hi_inclusive}");
        let span = (hi_inclusive - lo).saturating_add(1);
        lo + (self.next_u64() % span)
    }
}

/// The three M6 acceptance task families required by issue #2034 and `#2866`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum M6RowFamily {
    Compositions,
    AlteredConstraints,
    CodeTasks,
}

impl M6RowFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compositions => "compositions",
            Self::AlteredConstraints => "altered_constraints",
            Self::CodeTasks => "code_tasks",
        }
    }
}

/// Sub-family for `Compositions` (`80` rows: `40` 3-step register chains, `40` versioned memory chains).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionKind {
    ThreeStepRegisterAddSubAdd,
    VersionedMemoryMulSub,
}

/// Version selector queried in `VersionedMemoryMulSub` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueriedMemoryVersion {
    Initial,
    Previous,
}

impl QueriedMemoryVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Previous => "previous",
        }
    }
}

/// Specification and deterministic oracle target for one `Compositions` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositionSpec {
    pub kind: CompositionKind,
    pub operand_a: u64,
    pub operand_b: u64,
    pub operand_c: u64,
    pub operand_d_or_scale: u64,
    pub queried_version: Option<QueriedMemoryVersion>,
    pub decimal_carries_ab: u32,
    pub decimal_borrows_sub: u32,
    pub expected_intermediate: i64,
    pub expected_final: i64,
}

impl CompositionSpec {
    /// Canonical deterministic target string (`INT=<intermediate>;OUT=<final>`).
    pub fn canonical_answer(&self) -> String {
        format!(
            "INT={};OUT={}",
            self.expected_intermediate, self.expected_final
        )
    }
}

/// Sub-family for `AlteredConstraints` (`80` rows across 4 sub-families of `20` rows each).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum AlteredConstraintSpec {
    /// Addition or subtraction in non-decimal radix `B in {7, 8, 9, 11, 12, 13, 16}` where
    /// interpreting the digit string in base 10 yields `default_habit_answer != expected_answer`.
    NonDecimalRadix {
        radix: u8,
        left_repr: String,
        right_repr: String,
        op_is_add: bool,
        expected_answer: String,
        default_habit_answer: String,
    },
    /// Expression `a [+] b [*] c` where the prompt rebinds `[+]` to have higher precedence than
    /// `[*]`: `expected = (a + b) * c` vs `default_habit = a + (b * c)`.
    ReboundPrecedence {
        a: i64,
        b: i64,
        c: i64,
        expected_answer: String,
        default_habit_answer: String,
    },
    /// Two-step modular accumulation where hitting `s1_raw % modulus == forbidden_rem` adds
    /// `fallback_delta` before adding `c` modulo `modulus` (balanced across `branch_taken == true`
    /// and `branch_taken == false`).
    ModularStepCap {
        a: i64,
        b: i64,
        c: i64,
        modulus: i64,
        forbidden_rem: i64,
        fallback_delta: i64,
        branch_taken: bool,
        expected_answer: String,
        default_habit_answer: String,
    },
    /// Feasibility constraint pair (balanced: 10 satisfiable rows with a unique integer solution
    /// and 10 unsatisfiable rows where the sole lawful output is `ABSTAIN:UNSATISFIABLE`, sharing
    /// identical prompt wording so surface keyword rules cannot leak abstention).
    FeasibilityConstraint {
        a: i64,
        b: i64,
        satisfiable: bool,
        constraint_summary: String,
        expected_answer: String,
        default_habit_answer: String,
    },
}

impl AlteredConstraintSpec {
    pub fn expected_answer(&self) -> &str {
        match self {
            Self::NonDecimalRadix {
                expected_answer, ..
            }
            | Self::ReboundPrecedence {
                expected_answer, ..
            }
            | Self::ModularStepCap {
                expected_answer, ..
            }
            | Self::FeasibilityConstraint {
                expected_answer, ..
            } => expected_answer,
        }
    }

    pub fn default_habit_answer(&self) -> &str {
        match self {
            Self::NonDecimalRadix {
                default_habit_answer,
                ..
            }
            | Self::ReboundPrecedence {
                default_habit_answer,
                ..
            }
            | Self::ModularStepCap {
                default_habit_answer,
                ..
            }
            | Self::FeasibilityConstraint {
                default_habit_answer,
                ..
            } => default_habit_answer,
        }
    }

    pub const fn requires_abstention(&self) -> bool {
        matches!(
            self,
            Self::FeasibilityConstraint {
                satisfiable: false,
                ..
            }
        )
    }
}

/// Total, zero-I/O register machine instruction for sandboxed `CodeTasks` execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SandboxOp {
    LoadImm {
        dst: usize,
        val: i64,
    },
    Copy {
        dst: usize,
        src: usize,
    },
    Add {
        dst: usize,
        lhs: usize,
        rhs: usize,
    },
    Sub {
        dst: usize,
        lhs: usize,
        rhs: usize,
    },
    Mul {
        dst: usize,
        lhs: usize,
        rhs: usize,
    },
    Div {
        dst: usize,
        lhs: usize,
        rhs: usize,
    },
    Rem {
        dst: usize,
        lhs: usize,
        rhs: usize,
    },
    Clamp {
        dst: usize,
        src: usize,
        lo: i64,
        hi: i64,
    },
    SelectParity {
        dst: usize,
        cond_reg: usize,
        if_even: usize,
        if_odd: usize,
    },
    SelectGeImm {
        dst: usize,
        cond_reg: usize,
        threshold: i64,
        if_ge: usize,
        if_lt: usize,
    },
    Return {
        src: usize,
    },
}

/// Sandboxed integer register program with 8 registers (`r[0..8]`, `r[0] = input x`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxProgram {
    pub ops: Vec<SandboxOp>,
}

/// Deterministic error returned when a [`SandboxProgram`] traps or exceeds its step ceiling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxError {
    StepLimitExceeded { max_steps: usize },
    InvalidRegister { reg: usize },
    ArithmeticOverflow,
    DivisionByZero,
    MissingReturn,
}

impl SandboxProgram {
    /// Execute the program on `input_x` with a hard instruction cap `max_steps`.
    /// Performs zero heap allocations beyond the 8-register stack array and zero I/O.
    pub fn execute(&self, input_x: i64, max_steps: usize) -> Result<i64, SandboxError> {
        let mut regs = [0i64; 8];
        regs[0] = input_x;

        let check_reg = |r: usize| -> Result<usize, SandboxError> {
            if r < 8 {
                Ok(r)
            } else {
                Err(SandboxError::InvalidRegister { reg: r })
            }
        };

        for (step, op) in self.ops.iter().enumerate() {
            if step >= max_steps {
                return Err(SandboxError::StepLimitExceeded { max_steps });
            }
            match *op {
                SandboxOp::LoadImm { dst, val } => {
                    regs[check_reg(dst)?] = val;
                }
                SandboxOp::Copy { dst, src } => {
                    let v = regs[check_reg(src)?];
                    regs[check_reg(dst)?] = v;
                }
                SandboxOp::Add { dst, lhs, rhs } => {
                    let a = regs[check_reg(lhs)?];
                    let b = regs[check_reg(rhs)?];
                    regs[check_reg(dst)?] =
                        a.checked_add(b).ok_or(SandboxError::ArithmeticOverflow)?;
                }
                SandboxOp::Sub { dst, lhs, rhs } => {
                    let a = regs[check_reg(lhs)?];
                    let b = regs[check_reg(rhs)?];
                    regs[check_reg(dst)?] =
                        a.checked_sub(b).ok_or(SandboxError::ArithmeticOverflow)?;
                }
                SandboxOp::Mul { dst, lhs, rhs } => {
                    let a = regs[check_reg(lhs)?];
                    let b = regs[check_reg(rhs)?];
                    regs[check_reg(dst)?] =
                        a.checked_mul(b).ok_or(SandboxError::ArithmeticOverflow)?;
                }
                SandboxOp::Div { dst, lhs, rhs } => {
                    let a = regs[check_reg(lhs)?];
                    let b = regs[check_reg(rhs)?];
                    if b == 0 {
                        return Err(SandboxError::DivisionByZero);
                    }
                    regs[check_reg(dst)?] =
                        a.checked_div(b).ok_or(SandboxError::ArithmeticOverflow)?;
                }
                SandboxOp::Rem { dst, lhs, rhs } => {
                    let a = regs[check_reg(lhs)?];
                    let b = regs[check_reg(rhs)?];
                    if b == 0 {
                        return Err(SandboxError::DivisionByZero);
                    }
                    regs[check_reg(dst)?] =
                        a.checked_rem(b).ok_or(SandboxError::ArithmeticOverflow)?;
                }
                SandboxOp::Clamp { dst, src, lo, hi } => {
                    let v = regs[check_reg(src)?];
                    regs[check_reg(dst)?] = v.max(lo).min(hi);
                }
                SandboxOp::SelectParity {
                    dst,
                    cond_reg,
                    if_even,
                    if_odd,
                } => {
                    let c = regs[check_reg(cond_reg)?];
                    let chosen = if c.rem_euclid(2) == 0 {
                        regs[check_reg(if_even)?]
                    } else {
                        regs[check_reg(if_odd)?]
                    };
                    regs[check_reg(dst)?] = chosen;
                }
                SandboxOp::SelectGeImm {
                    dst,
                    cond_reg,
                    threshold,
                    if_ge,
                    if_lt,
                } => {
                    let c = regs[check_reg(cond_reg)?];
                    let chosen = if c >= threshold {
                        regs[check_reg(if_ge)?]
                    } else {
                        regs[check_reg(if_lt)?]
                    };
                    regs[check_reg(dst)?] = chosen;
                }
                SandboxOp::Return { src } => {
                    return Ok(regs[check_reg(src)?]);
                }
            }
        }
        Err(SandboxError::MissingReturn)
    }

    /// Render a standalone, deterministic Rust source module (`pub fn solve(x: i64) -> i64` and a
    /// `#[test]` harness verifying all `test_vectors`) suitable for `rustc --test` or `cargo test`.
    pub fn to_rust_source(&self, fn_ident: &str, test_vectors: &[(i64, i64)]) -> String {
        let mut out = String::new();
        out.push_str(&format!("pub fn {fn_ident}(x: i64) -> i64 {{\n"));
        out.push_str("    let mut r = [0i64; 8];\n");
        out.push_str("    r[0] = x;\n");
        for op in &self.ops {
            match *op {
                SandboxOp::LoadImm { dst, val } => {
                    out.push_str(&format!("    r[{dst}] = {val}i64;\n"));
                }
                SandboxOp::Copy { dst, src } => {
                    out.push_str(&format!("    r[{dst}] = r[{src}];\n"));
                }
                SandboxOp::Add { dst, lhs, rhs } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{lhs}].checked_add(r[{rhs}]).expect(\"add overflow\");\n"
                    ));
                }
                SandboxOp::Sub { dst, lhs, rhs } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{lhs}].checked_sub(r[{rhs}]).expect(\"sub overflow\");\n"
                    ));
                }
                SandboxOp::Mul { dst, lhs, rhs } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{lhs}].checked_mul(r[{rhs}]).expect(\"mul overflow\");\n"
                    ));
                }
                SandboxOp::Div { dst, lhs, rhs } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{lhs}].checked_div(r[{rhs}]).expect(\"div by zero\");\n"
                    ));
                }
                SandboxOp::Rem { dst, lhs, rhs } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{lhs}].checked_rem(r[{rhs}]).expect(\"rem by zero\");\n"
                    ));
                }
                SandboxOp::Clamp { dst, src, lo, hi } => {
                    out.push_str(&format!(
                        "    r[{dst}] = r[{src}].max({lo}i64).min({hi}i64);\n"
                    ));
                }
                SandboxOp::SelectParity {
                    dst,
                    cond_reg,
                    if_even,
                    if_odd,
                } => {
                    out.push_str(&format!(
                        "    r[{dst}] = if r[{cond_reg}].rem_euclid(2) == 0 {{ r[{if_even}] }} else {{ r[{if_odd}] }};\n"
                    ));
                }
                SandboxOp::SelectGeImm {
                    dst,
                    cond_reg,
                    threshold,
                    if_ge,
                    if_lt,
                } => {
                    out.push_str(&format!(
                        "    r[{dst}] = if r[{cond_reg}] >= {threshold}i64 {{ r[{if_ge}] }} else {{ r[{if_lt}] }};\n"
                    ));
                }
                SandboxOp::Return { src } => {
                    out.push_str(&format!("    return r[{src}];\n"));
                }
            }
        }
        out.push_str("    r[0]\n}\n\n");
        out.push_str("#[test]\nfn verify_vectors() {\n");
        for &(inp, exp) in test_vectors {
            out.push_str(&format!(
                "    assert_eq!({fn_ident}({inp}i64), {exp}i64);\n"
            ));
        }
        out.push_str("}\n");
        out
    }
}

/// Sub-family for `CodeTasks` (`80` rows across 4 algorithmic templates of `20` rows each).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeTaskAlgorithm {
    AffinelyScaledClampLoop,
    RadixDigitWeightedSum,
    BoundedRingStep,
    PiecewiseParityRecurrence,
}

impl CodeTaskAlgorithm {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AffinelyScaledClampLoop => "affinely_scaled_clamp_loop",
            Self::RadixDigitWeightedSum => "radix_digit_weighted_sum",
            Self::BoundedRingStep => "bounded_ring_step",
            Self::PiecewiseParityRecurrence => "piecewise_parity_recurrence",
        }
    }
}

/// Specification and deterministic sandbox/test-vector oracle for one `CodeTasks` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeTaskSpec {
    pub algorithm: CodeTaskAlgorithm,
    pub fn_signature: String,
    pub test_vectors: Vec<(i64, i64)>,
    pub canonical_program: SandboxProgram,
}

impl CodeTaskSpec {
    /// Canonical deterministic output vector string (`V0=<y0>;V1=<y1>;V2=<y2>;V3=<y3>`).
    pub fn canonical_vector_answer(&self) -> String {
        self.test_vectors
            .iter()
            .enumerate()
            .map(|(i, &(_, y))| format!("V{i}={y}"))
            .collect::<Vec<_>>()
            .join(";")
    }
}

/// Family-specific specification payload carried by an [`M6FrozenRow`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "family_spec", rename_all = "snake_case")]
pub enum M6RowSpec {
    Compositions(CompositionSpec),
    AlteredConstraints(AlteredConstraintSpec),
    CodeTasks(CodeTaskSpec),
}

/// One frozen row in the `m6-reasoning` acceptance panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6FrozenRow {
    pub row_id: String,
    pub family: M6RowFamily,
    pub seed_index: usize,
    pub prompt: String,
    pub spec: M6RowSpec,
}

impl M6FrozenRow {
    /// Return the canonical expected answer string used by the deterministic oracle.
    pub fn canonical_answer(&self) -> String {
        match &self.spec {
            M6RowSpec::Compositions(c) => c.canonical_answer(),
            M6RowSpec::AlteredConstraints(a) => a.expected_answer().to_string(),
            M6RowSpec::CodeTasks(k) => k.canonical_vector_answer(),
        }
    }

    /// Project this row to its unscored prompt view (stripping the oracle `spec`).
    pub fn to_unscored(&self) -> M6UnscoredPromptRow {
        M6UnscoredPromptRow {
            row_id: self.row_id.clone(),
            family: self.family,
            seed_index: self.seed_index,
            prompt: self.prompt.clone(),
        }
    }
}

/// Unscored view of a frozen row (emitted when exporting the unscored evaluation prompt stream).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6UnscoredPromptRow {
    pub row_id: String,
    pub family: M6RowFamily,
    pub seed_index: usize,
    pub prompt: String,
}

/// The complete 240-row frozen `m6-reasoning` panel generated from a fixed 64-bit seed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6ReasoningPanel {
    pub schema: String,
    pub seed: u64,
    pub rows: Vec<M6FrozenRow>,
}

impl M6ReasoningPanel {
    /// Serialize the panel to deterministic JSONL bytes.
    /// When `include_oracles` is `false`, only `row_id`, `family`, `seed_index`, and `prompt` are
    /// emitted on each line (the unscored evaluation stream).
    pub fn to_canonical_jsonl_bytes(&self, include_oracles: bool) -> Vec<u8> {
        let mut out = String::new();
        for row in &self.rows {
            if include_oracles {
                let line = serde_json::to_string(row).expect("serialize M6FrozenRow");
                out.push_str(&line);
            } else {
                let unscored = row.to_unscored();
                let line = serde_json::to_string(&unscored).expect("serialize M6UnscoredPromptRow");
                out.push_str(&line);
            }
            out.push('\n');
        }
        out.into_bytes()
    }

    /// 64-hex SHA-256 content hash of the unscored prompt JSONL stream.
    pub fn unscored_sha256(&self) -> String {
        sha256_hex(&self.to_canonical_jsonl_bytes(false))
    }

    /// 64-hex SHA-256 content hash of the full sealed oracle JSONL stream.
    pub fn sealed_oracle_sha256(&self) -> String {
        sha256_hex(&self.to_canonical_jsonl_bytes(true))
    }

    /// Verify that:
    /// 1. The panel has exactly `240` rows (`80` per family);
    /// 2. Every `row_id` and `prompt` is unique;
    /// 3. Every `Compositions` row has ZERO overlap with [`FORBIDDEN_CONTROL_PAIRS`] (`18` pairs)
    ///    and [`FORBIDDEN_CONTROL_TRIPLES`] (`30` triples) from `m6_exact_arithmetic.rs`, and
    ///    neither `expected_intermediate` nor `expected_final` appears as a substring of `prompt`;
    /// 4. Every `Prompt` carries the `[M6-PANEL-V1::...]` domain tag;
    /// 5. Every `AlteredConstraints` row has `expected_answer != default_habit_answer`,
    ///    `ModularStepCap` is balanced `10` taken / `10` not-taken, and `FeasibilityConstraint` is
    ///    balanced `10` satisfiable / `10` unsatisfiable;
    /// 6. Every `CodeTasks` row's canonical program halts within [`MAX_SANDBOX_STEPS`] and produces
    ///    non-constant, non-identity outputs across all 4 test vectors that do not appear in the
    ///    prompt.
    pub fn verify_zero_overlap(&self) -> Result<(), String> {
        if self.rows.len() != M6_PANEL_TOTAL_ROWS {
            return Err(format!(
                "expected {} total rows, got {}",
                M6_PANEL_TOTAL_ROWS,
                self.rows.len()
            ));
        }

        let mut comp_count = 0usize;
        let mut alt_count = 0usize;
        let mut code_count = 0usize;
        let mut mod_taken_count = 0usize;
        let mut mod_not_taken_count = 0usize;
        let mut feas_sat_count = 0usize;
        let mut feas_unsat_count = 0usize;
        let mut ids = BTreeSet::new();
        let mut prompts = BTreeSet::new();

        for row in &self.rows {
            if !ids.insert(row.row_id.clone()) {
                return Err(format!("duplicate row_id: {}", row.row_id));
            }
            if !prompts.insert(row.prompt.clone()) {
                return Err(format!("duplicate prompt on row {}", row.row_id));
            }
            if !row.prompt.starts_with("[M6-PANEL-V1::") {
                return Err(format!(
                    "row {} missing domain-separation prefix [M6-PANEL-V1::",
                    row.row_id
                ));
            }

            match &row.spec {
                M6RowSpec::Compositions(c) => {
                    comp_count += 1;
                    if FORBIDDEN_CONTROL_PAIRS.contains(&(c.operand_a, c.operand_b))
                        || FORBIDDEN_CONTROL_PAIRS.contains(&(c.operand_b, c.operand_c))
                        || FORBIDDEN_CONTROL_TRIPLES.contains(&(
                            c.operand_a,
                            c.operand_b,
                            c.operand_c,
                        ))
                    {
                        return Err(format!(
                            "row {} collides with forbidden CPU control literal ({}, {}, {})",
                            row.row_id, c.operand_a, c.operand_b, c.operand_c
                        ));
                    }
                    if c.decimal_carries_ab < 2 {
                        return Err(format!(
                            "row {} has insufficient decimal carries ({} < 2)",
                            row.row_id, c.decimal_carries_ab
                        ));
                    }
                    if c.decimal_borrows_sub < 1 {
                        return Err(format!(
                            "row {} has insufficient decimal borrows ({} < 1)",
                            row.row_id,
                            c.decimal_borries_check_msg()
                        ));
                    }
                    let int_str = c.expected_intermediate.to_string();
                    let out_str = c.expected_final.to_string();
                    if row.prompt.contains(&int_str) || row.prompt.contains(&out_str) {
                        return Err(format!(
                            "row {} leaks target numeral ({int_str} or {out_str}) in prompt",
                            row.row_id
                        ));
                    }
                }
                M6RowSpec::AlteredConstraints(a) => {
                    alt_count += 1;
                    if a.expected_answer() == a.default_habit_answer() {
                        return Err(format!(
                            "row {} has expected_answer == default_habit_answer ({})",
                            row.row_id,
                            a.expected_answer()
                        ));
                    }
                    match a {
                        AlteredConstraintSpec::ModularStepCap {
                            a: op_a,
                            b: op_b,
                            modulus,
                            forbidden_rem,
                            branch_taken,
                            ..
                        } => {
                            let actual_taken = (op_a + op_b).rem_euclid(*modulus) == *forbidden_rem;
                            if actual_taken != *branch_taken {
                                return Err(format!(
                                    "row {} ModularStepCap branch_taken mismatch",
                                    row.row_id
                                ));
                            }
                            if *branch_taken {
                                mod_taken_count += 1;
                            } else {
                                mod_not_taken_count += 1;
                            }
                        }
                        AlteredConstraintSpec::FeasibilityConstraint {
                            satisfiable,
                            expected_answer,
                            ..
                        } => {
                            if *satisfiable {
                                if expected_answer == ABSTAIN_UNSATISFIABLE_TOKEN {
                                    return Err(format!(
                                        "row {} is satisfiable but expected_answer is ABSTAIN",
                                        row.row_id
                                    ));
                                }
                                feas_sat_count += 1;
                            } else {
                                if expected_answer != ABSTAIN_UNSATISFIABLE_TOKEN {
                                    return Err(format!(
                                        "row {} is unsatisfiable but expected_answer is not ABSTAIN",
                                        row.row_id
                                    ));
                                }
                                feas_unsat_count += 1;
                            }
                        }
                        _ => {}
                    }
                }
                M6RowSpec::CodeTasks(k) => {
                    code_count += 1;
                    if k.test_vectors.len() != 4 {
                        return Err(format!(
                            "row {} must have 4 test vectors, got {}",
                            row.row_id,
                            k.test_vectors.len()
                        ));
                    }
                    for &(inp, exp) in &k.test_vectors {
                        let actual = k
                            .canonical_program
                            .execute(inp, MAX_SANDBOX_STEPS)
                            .map_err(|e| {
                                format!("row {} canonical program failed: {e:?}", row.row_id)
                            })?;
                        if actual != exp {
                            return Err(format!(
                                "row {} canonical program mismatch on {inp}: got {actual}, expected {exp}",
                                row.row_id
                            ));
                        }
                        let exp_token = format!("={exp}");
                        let exp_space = format!(" {exp} ");
                        if row.prompt.contains(&exp_token) || row.prompt.contains(&exp_space) {
                            return Err(format!(
                                "row {} leaks expected output {exp} in prompt",
                                row.row_id
                            ));
                        }
                    }
                    let all_same_output = k.test_vectors.windows(2).all(|w| w[0].1 == w[1].1);
                    let all_identity = k.test_vectors.iter().all(|&(x, y)| x == y);
                    if all_same_output || all_identity {
                        return Err(format!(
                            "row {} test vectors are degenerate (constant or identity)",
                            row.row_id
                        ));
                    }
                }
            }
        }

        if comp_count != M6_PANEL_ROWS_PER_FAMILY
            || alt_count != M6_PANEL_ROWS_PER_FAMILY
            || code_count != M6_PANEL_ROWS_PER_FAMILY
        {
            return Err(format!(
                "family count mismatch: comp={comp_count}, alt={alt_count}, code={code_count}"
            ));
        }
        if mod_taken_count != 10 || mod_not_taken_count != 10 {
            return Err(format!(
                "ModularStepCap branch balance mismatch: taken={mod_taken_count}, not_taken={mod_not_taken_count}"
            ));
        }
        if feas_sat_count != 10 || feas_unsat_count != 10 {
            return Err(format!(
                "FeasibilityConstraint balance mismatch: sat={feas_sat_count}, unsat={feas_unsat_count}"
            ));
        }

        Ok(())
    }
}

impl CompositionSpec {
    fn decimal_borries_check_msg(&self) -> u32 {
        self.decimal_borrows_sub
    }
}

/// Count decimal carry transitions when adding `a + b` in base 10.
pub fn count_decimal_carries(mut a: u64, mut b: u64) -> u32 {
    let mut carry = 0u64;
    let mut count = 0u32;
    while a > 0 || b > 0 {
        let sum = (a % 10) + (b % 10) + carry;
        if sum >= 10 {
            count += 1;
            carry = 1;
        } else {
            carry = 0;
        }
        a /= 10;
        b /= 10;
    }
    count
}

/// Count decimal borrow transitions when subtracting `left - right` (`left >= right`) in base 10.
pub fn count_decimal_borrows(mut left: u64, mut right: u64) -> u32 {
    let mut borrow = 0i64;
    let mut count = 0u32;
    while left > 0 || right > 0 {
        let l = (left % 10) as i64;
        let r = (right % 10) as i64;
        if l - r - borrow < 0 {
            count += 1;
            borrow = 1;
        } else {
            borrow = 0;
        }
        left /= 10;
        right /= 10;
    }
    count
}

fn format_in_radix(mut val: u64, radix: u8) -> String {
    assert!((2..=16).contains(&radix));
    if val == 0 {
        return "0".to_string();
    }
    let mut chars = Vec::new();
    let r = u64::from(radix);
    while val > 0 {
        let d = (val % r) as u8;
        let ch = if d < 10 {
            (b'0' + d) as char
        } else {
            (b'A' + (d - 10)) as char
        };
        chars.push(ch);
        val /= r;
    }
    chars.reverse();
    chars.into_iter().collect()
}

/// Generate the frozen 240-row `m6-reasoning` panel from [`M6_PANEL_FROZEN_SEED_V1`].
pub fn generate_frozen_m6_panel() -> M6ReasoningPanel {
    generate_m6_panel(M6_PANEL_FROZEN_SEED_V1)
}

/// Generate a 240-row `m6-reasoning` panel deterministically from any 64-bit `seed`.
pub fn generate_m6_panel(seed: u64) -> M6ReasoningPanel {
    let mut rows = Vec::with_capacity(M6_PANEL_TOTAL_ROWS);

    // 1. Family 1: Compositions (80 rows)
    let mut comp_rng = SplitMix64::new(seed ^ 0x434F_4D50_5F56_3100);
    let mut seen_comp_tuples = BTreeSet::new();
    for idx in 0..M6_PANEL_ROWS_PER_FAMILY {
        let row_id = format!("m6-comp-{idx:03}");
        let is_register_chain = idx < (M6_PANEL_ROWS_PER_FAMILY / 2);
        loop {
            let a = comp_rng.next_range(1_200, 78_999);
            let b = comp_rng.next_range(1_200, 78_999);
            let carries = count_decimal_carries(a, b);
            if carries < 2 {
                continue;
            }
            if FORBIDDEN_CONTROL_PAIRS.contains(&(a, b)) {
                continue;
            }
            let sum_ab = a + b;
            let c = comp_rng.next_range(1_100, sum_ab.saturating_sub(500).min(65_000));
            if FORBIDDEN_CONTROL_PAIRS.contains(&(b, c))
                || FORBIDDEN_CONTROL_TRIPLES.contains(&(a, b, c))
            {
                continue;
            }

            if is_register_chain {
                let borrows = count_decimal_borrows(sum_ab, c);
                if borrows < 1 {
                    continue;
                }
                let d = comp_rng.next_range(350, 29_999);
                let t1 = sum_ab as i64;
                let t2 = t1 - (c as i64);
                let out = t2 + (d as i64);
                let prompt = format!(
                    "[M6-PANEL-V1::compositions::{row_id}] Register chain: write A={a} and B={b} to addressed memory; compute T1 = A + B into R0; read R0 and C={c} to compute T2 = R0 - C into R1; read R1 and D={d} to compute OUT = R1 + D. Emit INT=<T1>;OUT=<OUT>."
                );
                if prompt.contains(&t1.to_string()) || prompt.contains(&out.to_string()) {
                    continue;
                }
                if !seen_comp_tuples.insert((a, b, c)) {
                    continue;
                }
                rows.push(M6FrozenRow {
                    row_id,
                    family: M6RowFamily::Compositions,
                    seed_index: idx,
                    prompt,
                    spec: M6RowSpec::Compositions(CompositionSpec {
                        kind: CompositionKind::ThreeStepRegisterAddSubAdd,
                        operand_a: a,
                        operand_b: b,
                        operand_c: c,
                        operand_d_or_scale: d,
                        queried_version: None,
                        decimal_carries_ab: carries,
                        decimal_borrows_sub: borrows,
                        expected_intermediate: t1,
                        expected_final: out,
                    }),
                });
                break;
            } else {
                let scale = comp_rng.next_range(2, 7);
                let step_delta = u64::from((idx as u32 % 7) + 2) * 137;
                let init_val = sum_ab;
                let prev = init_val + step_delta;
                let scaled = prev * scale;
                if scaled <= c + 500 {
                    continue;
                }
                let borrows = count_decimal_borrows(scaled, c);
                if borrows < 1 {
                    continue;
                }
                let curr = (scaled - c) as i64;
                let q_ver = if idx.is_multiple_of(2) {
                    QueriedMemoryVersion::Previous
                } else {
                    QueriedMemoryVersion::Initial
                };
                let expected_int = match q_ver {
                    QueriedMemoryVersion::Initial => init_val as i64,
                    QueriedMemoryVersion::Previous => prev as i64,
                };
                let prompt = format!(
                    "[M6-PANEL-V1::compositions::{row_id}] Versioned slot R0: write R0@initial = {a} + {b}; update R0@previous = R0@initial + {step_delta}; update R0@current = (R0@previous * {scale}) - {c}. Query INT=R0@{} and OUT=R0@current. Emit INT=<INT>;OUT=<OUT>.",
                    q_ver.as_str()
                );
                if prompt.contains(&expected_int.to_string()) || prompt.contains(&curr.to_string())
                {
                    continue;
                }
                if !seen_comp_tuples.insert((a, b, c)) {
                    continue;
                }
                rows.push(M6FrozenRow {
                    row_id,
                    family: M6RowFamily::Compositions,
                    seed_index: idx,
                    prompt,
                    spec: M6RowSpec::Compositions(CompositionSpec {
                        kind: CompositionKind::VersionedMemoryMulSub,
                        operand_a: a,
                        operand_b: b,
                        operand_c: c,
                        operand_d_or_scale: scale,
                        queried_version: Some(q_ver),
                        decimal_carries_ab: carries,
                        decimal_borrows_sub: borrows,
                        expected_intermediate: expected_int,
                        expected_final: curr,
                    }),
                });
                break;
            }
        }
    }

    // 2. Family 2: AlteredConstraints (80 rows: 4 sub-families x 20 rows)
    let mut alt_rng = SplitMix64::new(seed ^ 0x414C_5443_5F56_3100);
    let mut seen_alt_prompts = BTreeSet::new();
    for idx in 0..M6_PANEL_ROWS_PER_FAMILY {
        let row_id = format!("m6-alt-{idx:03}");
        let sub_family = idx / 20;
        loop {
            match sub_family {
                0 => {
                    // Non-decimal radix in {7, 8, 9, 11, 12, 13, 16} using digits 1..radix.min(10)
                    // so every input numeral is also a valid base-10 numeral whose base-10 sum
                    // differs from the base-radix sum.
                    let radices: [u8; 7] = [7, 8, 9, 11, 12, 13, 16];
                    let radix = radices[(alt_rng.next_u64() as usize) % radices.len()];
                    let r_u64 = u64::from(radix);
                    let max_dec_digit = r_u64.min(10) - 1;
                    let d2_a = alt_rng.next_range(1, max_dec_digit);
                    let d1_a = alt_rng.next_range(2, max_dec_digit);
                    let min_d0_a = (r_u64.saturating_sub(max_dec_digit)).max(2);
                    if min_d0_a > max_dec_digit {
                        continue;
                    }
                    let d0_a = alt_rng.next_range(min_d0_a, max_dec_digit);
                    let d2_b = alt_rng.next_range(1, max_dec_digit);
                    let d1_b = alt_rng.next_range(2, max_dec_digit);
                    let min_d0_b = (r_u64.saturating_sub(d0_a)).max(1);
                    if min_d0_b > max_dec_digit {
                        continue;
                    }
                    let d0_b = alt_rng.next_range(min_d0_b, max_dec_digit);

                    let val_a_base_b = (d2_a * r_u64 + d1_a) * r_u64 + d0_a;
                    let val_b_base_b = (d2_b * r_u64 + d1_b) * r_u64 + d0_b;
                    let left_repr = format!("{d2_a}{d1_a}{d0_a}");
                    let right_repr = format!("{d2_b}{d1_b}{d0_b}");
                    let expected_answer = format_in_radix(val_a_base_b + val_b_base_b, radix);
                    let dec_a: u64 = left_repr.parse().unwrap();
                    let dec_b: u64 = right_repr.parse().unwrap();
                    let default_habit_answer = (dec_a + dec_b).to_string();
                    if expected_answer == default_habit_answer {
                        continue;
                    }
                    let max_symbol = if radix <= 10 {
                        (b'0' + (radix - 1)) as char
                    } else {
                        (b'A' + (radix - 11)) as char
                    };
                    let prompt = format!(
                        "[M6-PANEL-V1::altered_constraints::{row_id}] Altered radix constraint: all numerals are in base-{radix} (digits 0..{max_symbol}). Compute {left_repr}_{radix} + {right_repr}_{radix} and emit the result in base-{radix} as VAL=<base-{radix} numeral>.",
                    );
                    if !seen_alt_prompts.insert(prompt.clone()) {
                        continue;
                    }
                    rows.push(M6FrozenRow {
                        row_id,
                        family: M6RowFamily::AlteredConstraints,
                        seed_index: idx,
                        prompt,
                        spec: M6RowSpec::AlteredConstraints(
                            AlteredConstraintSpec::NonDecimalRadix {
                                radix,
                                left_repr,
                                right_repr,
                                op_is_add: true,
                                expected_answer,
                                default_habit_answer,
                            },
                        ),
                    });
                    break;
                }
                1 => {
                    // Rebound precedence: `[+]` binds tighter than `[*]`
                    let a = alt_rng.next_range(11, 149) as i64;
                    let b = alt_rng.next_range(12, 149) as i64;
                    let c = alt_rng.next_range(3, 19) as i64;
                    let expected = (a + b) * c;
                    let habit = a + (b * c);
                    if expected == habit {
                        continue;
                    }
                    let prompt = format!(
                        "[M6-PANEL-V1::altered_constraints::{row_id}] Altered precedence constraint: in this grammar, operator [+] has HIGHER precedence than operator [*], so X [+] Y [*] Z means (X + Y) * Z. Evaluate {a} [+] {b} [*] {c} and emit VAL=<integer>."
                    );
                    if !seen_alt_prompts.insert(prompt.clone()) {
                        continue;
                    }
                    rows.push(M6FrozenRow {
                        row_id,
                        family: M6RowFamily::AlteredConstraints,
                        seed_index: idx,
                        prompt,
                        spec: M6RowSpec::AlteredConstraints(
                            AlteredConstraintSpec::ReboundPrecedence {
                                a,
                                b,
                                c,
                                expected_answer: expected.to_string(),
                                default_habit_answer: habit.to_string(),
                            },
                        ),
                    });
                    break;
                }
                2 => {
                    // ModularStepCap: balanced 10 branch-taken rows (`(a+b) mod m == forbidden_rem`)
                    // and 10 branch-not-taken rows (`(a+b) mod m != forbidden_rem`).
                    let modulus = alt_rng.next_range(17, 53) as i64;
                    let a = alt_rng.next_range(25, 180) as i64;
                    let b = alt_rng.next_range(25, 180) as i64;
                    let c = alt_rng.next_range(15, 95) as i64;
                    let fallback_delta = alt_rng.next_range(5, (modulus - 2) as u64) as i64;
                    let raw_rem = (a + b).rem_euclid(modulus);
                    let branch_taken = idx.is_multiple_of(2);
                    let (forbidden_rem, expected, habit) = if branch_taken {
                        let frem = raw_rem;
                        let exp = (a + b + fallback_delta + c).rem_euclid(modulus);
                        // Habit on taken rows: ignoring the triggered fallback_delta
                        let hab = (a + b + c).rem_euclid(modulus);
                        (frem, exp, hab)
                    } else {
                        let offset = alt_rng.next_range(1, (modulus - 1) as u64) as i64;
                        let frem = (raw_rem + offset).rem_euclid(modulus);
                        let exp = (a + b + c).rem_euclid(modulus);
                        // Habit on not-taken rows: blindly adding fallback_delta without checking condition
                        let hab = (a + b + fallback_delta + c).rem_euclid(modulus);
                        (frem, exp, hab)
                    };
                    if expected == habit {
                        continue;
                    }
                    let prompt = format!(
                        "[M6-PANEL-V1::altered_constraints::{row_id}] Altered transition constraint: compute S1_raw = {a} + {b}; if S1_raw mod {modulus} == {forbidden_rem}, set S1 = S1_raw + {fallback_delta}, else S1 = S1_raw; then compute OUT = (S1 + {c}) mod {modulus}. Emit VAL=<OUT>."
                    );
                    if !seen_alt_prompts.insert(prompt.clone()) {
                        continue;
                    }
                    rows.push(M6FrozenRow {
                        row_id,
                        family: M6RowFamily::AlteredConstraints,
                        seed_index: idx,
                        prompt,
                        spec: M6RowSpec::AlteredConstraints(
                            AlteredConstraintSpec::ModularStepCap {
                                a,
                                b,
                                c,
                                modulus,
                                forbidden_rem,
                                fallback_delta,
                                branch_taken,
                                expected_answer: expected.to_string(),
                                default_habit_answer: habit.to_string(),
                            },
                        ),
                    });
                    break;
                }
                _ => {
                    // FeasibilityConstraint: 10 unsatisfiable rows (even idx) requiring
                    // ABSTAIN:UNSATISFIABLE and 10 satisfiable rows (odd idx) with a unique integer
                    // solution, using identical wording across both sub-templates so keyword rules
                    // ("contains 'feasibility' -> ABSTAIN") score only 10/20.
                    let satisfiable = !idx.is_multiple_of(2);
                    let use_unsigned_sub = (idx / 2).is_multiple_of(2);
                    let a = alt_rng.next_range(40, 380) as i64;
                    let b = alt_rng.next_range(a as u64 + 25, 850) as i64;
                    let (summary, expected_answer, habit) = if use_unsigned_sub {
                        if count_decimal_borrows(b as u64, a as u64) < 1 {
                            continue;
                        }
                        let (lhs, rhs, exp, hab) = if satisfiable {
                            (
                                b,
                                a,
                                (b - a).to_string(),
                                ABSTAIN_UNSATISFIABLE_TOKEN.to_string(),
                            )
                        } else {
                            (
                                a,
                                b,
                                ABSTAIN_UNSATISFIABLE_TOKEN.to_string(),
                                (a - b).to_string(),
                            )
                        };
                        let s = format!(
                            "strict unsigned register R0 initialized to {lhs} must subtract {rhs} without borrow wraparound and remain >= 0"
                        );
                        (s, exp, hab)
                    } else {
                        let k = alt_rng.next_range(3, 9) as i64;
                        let even_a = a * 2;
                        let even_b = b * 2;
                        let (target, exp, hab) = if satisfiable {
                            (
                                (even_a + even_b) * k,
                                k.to_string(),
                                ABSTAIN_UNSATISFIABLE_TOKEN.to_string(),
                            )
                        } else {
                            (
                                (even_a + even_b) * k + 1,
                                ABSTAIN_UNSATISFIABLE_TOKEN.to_string(),
                                k.to_string(),
                            )
                        };
                        let s = format!(
                            "find integer step count k >= 1 such that ({even_a} * k + {even_b} * k) == {target} under even-lattice closure"
                        );
                        (s, exp, hab)
                    };
                    let prompt = format!(
                        "[M6-PANEL-V1::altered_constraints::{row_id}] Altered feasibility constraint: {summary}. If the constraint system has no valid integer solution, emit {ABSTAIN_UNSATISFIABLE_TOKEN}; otherwise emit VAL=<integer>."
                    );
                    if !seen_alt_prompts.insert(prompt.clone()) {
                        continue;
                    }
                    rows.push(M6FrozenRow {
                        row_id,
                        family: M6RowFamily::AlteredConstraints,
                        seed_index: idx,
                        prompt,
                        spec: M6RowSpec::AlteredConstraints(
                            AlteredConstraintSpec::FeasibilityConstraint {
                                a,
                                b,
                                satisfiable,
                                constraint_summary: summary,
                                expected_answer,
                                default_habit_answer: habit,
                            },
                        ),
                    });
                    break;
                }
            }
        }
    }

    // 3. Family 3: CodeTasks (80 rows: 4 algorithm families x 20 rows)
    let mut code_rng = SplitMix64::new(seed ^ 0x434F_4445_5F56_3100);
    let mut seen_code_prompts = BTreeSet::new();
    for idx in 0..M6_PANEL_ROWS_PER_FAMILY {
        let row_id = format!("m6-code-{idx:03}");
        let alg_idx = idx / 20;
        loop {
            let (algorithm, spec_desc, program, inputs): (
                CodeTaskAlgorithm,
                String,
                SandboxProgram,
                [i64; 4],
            ) = match alg_idx {
                0 => {
                    let scale = code_rng.next_range(2, 5) as i64;
                    let bias = code_rng.next_range(3, 29) as i64;
                    let lo = -(code_rng.next_range(20, 90) as i64);
                    let hi = code_rng.next_range(120, 480) as i64;
                    let steps = code_rng.next_range(3, 5) as usize;
                    let mut ops = vec![
                        SandboxOp::Copy { dst: 1, src: 0 },
                        SandboxOp::LoadImm { dst: 2, val: scale },
                        SandboxOp::LoadImm { dst: 3, val: bias },
                    ];
                    for _ in 0..steps {
                        ops.push(SandboxOp::Mul {
                            dst: 1,
                            lhs: 1,
                            rhs: 2,
                        });
                        ops.push(SandboxOp::Sub {
                            dst: 1,
                            lhs: 1,
                            rhs: 3,
                        });
                        ops.push(SandboxOp::Clamp {
                            dst: 1,
                            src: 1,
                            lo,
                            hi,
                        });
                    }
                    ops.push(SandboxOp::Return { src: 1 });
                    let x0 = code_rng.next_range(2, 9) as i64;
                    let inputs = [x0, x0 + 3, x0 + 7, -(x0 + 2)];
                    let desc = format!(
                        "starting from acc = x, repeat {steps} steps of acc = (acc * {scale} - {bias}).clamp({lo}, {hi}) and return acc"
                    );
                    (
                        CodeTaskAlgorithm::AffinelyScaledClampLoop,
                        desc,
                        SandboxProgram { ops },
                        inputs,
                    )
                }
                1 => {
                    let radix = code_rng.next_range(5, 13) as i64;
                    let w_even = code_rng.next_range(2, 9) as i64;
                    let w_odd = code_rng.next_range(3, 11) as i64;
                    if w_even == w_odd {
                        continue;
                    }
                    let ops = vec![
                        SandboxOp::Copy { dst: 1, src: 0 },
                        SandboxOp::LoadImm { dst: 2, val: radix },
                        SandboxOp::LoadImm {
                            dst: 3,
                            val: w_even,
                        },
                        SandboxOp::LoadImm { dst: 4, val: w_odd },
                        SandboxOp::LoadImm { dst: 5, val: 0 },
                        // d0 (even place 0)
                        SandboxOp::Rem {
                            dst: 6,
                            lhs: 1,
                            rhs: 2,
                        },
                        SandboxOp::Mul {
                            dst: 6,
                            lhs: 6,
                            rhs: 3,
                        },
                        SandboxOp::Add {
                            dst: 5,
                            lhs: 5,
                            rhs: 6,
                        },
                        SandboxOp::Div {
                            dst: 1,
                            lhs: 1,
                            rhs: 2,
                        },
                        // d1 (odd place 1)
                        SandboxOp::Rem {
                            dst: 6,
                            lhs: 1,
                            rhs: 2,
                        },
                        SandboxOp::Mul {
                            dst: 6,
                            lhs: 6,
                            rhs: 4,
                        },
                        SandboxOp::Add {
                            dst: 5,
                            lhs: 5,
                            rhs: 6,
                        },
                        SandboxOp::Div {
                            dst: 1,
                            lhs: 1,
                            rhs: 2,
                        },
                        // d2 (even place 2)
                        SandboxOp::Rem {
                            dst: 6,
                            lhs: 1,
                            rhs: 2,
                        },
                        SandboxOp::Mul {
                            dst: 6,
                            lhs: 6,
                            rhs: 3,
                        },
                        SandboxOp::Add {
                            dst: 5,
                            lhs: 5,
                            rhs: 6,
                        },
                        SandboxOp::Div {
                            dst: 1,
                            lhs: 1,
                            rhs: 2,
                        },
                        // d3 (odd place 3)
                        SandboxOp::Rem {
                            dst: 6,
                            lhs: 1,
                            rhs: 2,
                        },
                        SandboxOp::Mul {
                            dst: 6,
                            lhs: 6,
                            rhs: 4,
                        },
                        SandboxOp::Add {
                            dst: 5,
                            lhs: 5,
                            rhs: 6,
                        },
                        SandboxOp::Return { src: 5 },
                    ];
                    let base_in = code_rng.next_range(35, 220) as i64;
                    let inputs = [base_in, base_in + 19, base_in + 47, base_in + 103];
                    let desc = format!(
                        "extract the lowest 4 base-{radix} digits d0..d3 of non-negative x and return d0*{w_even} + d1*{w_odd} + d2*{w_even} + d3*{w_odd}"
                    );
                    (
                        CodeTaskAlgorithm::RadixDigitWeightedSum,
                        desc,
                        SandboxProgram { ops },
                        inputs,
                    )
                }
                2 => {
                    let cap_mod = code_rng.next_range(31, 97) as i64;
                    let mult_a = code_rng.next_range(3, 8) as i64;
                    let add_b = code_rng.next_range(5, 27) as i64;
                    let threshold = cap_mod / 2;
                    let penalty = code_rng.next_range(4, threshold.max(5) as u64) as i64;
                    let mut ops = vec![
                        SandboxOp::Copy { dst: 1, src: 0 },
                        SandboxOp::LoadImm {
                            dst: 2,
                            val: mult_a,
                        },
                        SandboxOp::LoadImm { dst: 3, val: add_b },
                        SandboxOp::LoadImm {
                            dst: 4,
                            val: cap_mod,
                        },
                        SandboxOp::LoadImm {
                            dst: 5,
                            val: penalty,
                        },
                    ];
                    for _ in 0..3 {
                        ops.push(SandboxOp::Mul {
                            dst: 1,
                            lhs: 1,
                            rhs: 2,
                        });
                        ops.push(SandboxOp::Add {
                            dst: 1,
                            lhs: 1,
                            rhs: 3,
                        });
                        ops.push(SandboxOp::Rem {
                            dst: 1,
                            lhs: 1,
                            rhs: 4,
                        });
                        ops.push(SandboxOp::Sub {
                            dst: 6,
                            lhs: 1,
                            rhs: 5,
                        });
                        ops.push(SandboxOp::SelectGeImm {
                            dst: 1,
                            cond_reg: 1,
                            threshold,
                            if_ge: 6,
                            if_lt: 1,
                        });
                    }
                    ops.push(SandboxOp::Return { src: 1 });
                    let x0 = code_rng.next_range(3, 25) as i64;
                    let inputs = [x0, x0 + 4, x0 + 9, x0 + 16];
                    let desc = format!(
                        "starting from s = x, run 3 ring updates: s = (s * {mult_a} + {add_b}) % {cap_mod}; if s >= {threshold} then s = s - {penalty}; return s"
                    );
                    (
                        CodeTaskAlgorithm::BoundedRingStep,
                        desc,
                        SandboxProgram { ops },
                        inputs,
                    )
                }
                _ => {
                    let steps = code_rng.next_range(3, 5) as usize;
                    let even_add = code_rng.next_range(5, 31) as i64;
                    let odd_mul = 3i64;
                    let odd_add = code_rng.next_range(1, 19) as i64;
                    let modulus = code_rng.next_range(53, 199) as i64;
                    let mut ops = vec![
                        SandboxOp::Copy { dst: 1, src: 0 },
                        SandboxOp::LoadImm { dst: 2, val: 2 },
                        SandboxOp::LoadImm {
                            dst: 3,
                            val: even_add,
                        },
                        SandboxOp::LoadImm {
                            dst: 4,
                            val: odd_mul,
                        },
                        SandboxOp::LoadImm {
                            dst: 5,
                            val: odd_add,
                        },
                        SandboxOp::LoadImm {
                            dst: 6,
                            val: modulus,
                        },
                    ];
                    for _ in 0..steps {
                        // r[7] = even branch: (r[1] / 2 + even_add) % modulus
                        ops.push(SandboxOp::Div {
                            dst: 7,
                            lhs: 1,
                            rhs: 2,
                        });
                        ops.push(SandboxOp::Add {
                            dst: 7,
                            lhs: 7,
                            rhs: 3,
                        });
                        ops.push(SandboxOp::Rem {
                            dst: 7,
                            lhs: 7,
                            rhs: 6,
                        });
                        // r[0] reused as scratch for odd branch: (r[1] * odd_mul + odd_add) % modulus
                        ops.push(SandboxOp::Mul {
                            dst: 0,
                            lhs: 1,
                            rhs: 4,
                        });
                        ops.push(SandboxOp::Add {
                            dst: 0,
                            lhs: 0,
                            rhs: 5,
                        });
                        ops.push(SandboxOp::Rem {
                            dst: 0,
                            lhs: 0,
                            rhs: 6,
                        });
                        ops.push(SandboxOp::SelectParity {
                            dst: 1,
                            cond_reg: 1,
                            if_even: 7,
                            if_odd: 0,
                        });
                    }
                    ops.push(SandboxOp::Return { src: 1 });
                    let x0 = code_rng.next_range(6, 40) as i64;
                    let inputs = [x0, x0 + 1, x0 + 4, x0 + 7];
                    let desc = format!(
                        "starting from s = x, run {steps} steps: if s is even, s = (s / 2 + {even_add}) % {modulus}, else s = (s * {odd_mul} + {odd_add}) % {modulus}; return s"
                    );
                    (
                        CodeTaskAlgorithm::PiecewiseParityRecurrence,
                        desc,
                        SandboxProgram { ops },
                        inputs,
                    )
                }
            };

            let mut test_vectors = Vec::with_capacity(4);
            let mut ok = true;
            for &inp in &inputs {
                match program.execute(inp, MAX_SANDBOX_STEPS) {
                    Ok(out) => test_vectors.push((inp, out)),
                    Err(_) => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            let distinct_outputs: BTreeSet<i64> = test_vectors.iter().map(|&(_, y)| y).collect();
            let any_non_identity = test_vectors.iter().any(|&(x, y)| x != y);
            if distinct_outputs.len() < 3 || !any_non_identity {
                continue;
            }

            let fn_signature = "pub fn solve(x: i64) -> i64".to_string();
            let vec_inputs_str = inputs
                .iter()
                .enumerate()
                .map(|(i, x)| format!("x{i}={x}"))
                .collect::<Vec<_>>()
                .join(", ");
            let prompt = format!(
                "[M6-PANEL-V1::code_tasks::{row_id}] Implement `{fn_signature}`: {spec_desc}. Evaluate on test inputs ({vec_inputs_str}) and emit V0=<y0>;V1=<y1>;V2=<y2>;V3=<y3>."
            );
            if test_vectors.iter().any(|&(_, exp)| {
                prompt.contains(&format!("={exp}")) || prompt.contains(&format!(" {exp} "))
            }) {
                continue;
            }
            if !seen_code_prompts.insert(prompt.clone()) {
                continue;
            }

            rows.push(M6FrozenRow {
                row_id,
                family: M6RowFamily::CodeTasks,
                seed_index: idx,
                prompt,
                spec: M6RowSpec::CodeTasks(CodeTaskSpec {
                    algorithm,
                    fn_signature,
                    test_vectors,
                    canonical_program: program,
                }),
            });
            break;
        }
    }

    M6ReasoningPanel {
        schema: M6_PANEL_SCHEMA.to_string(),
        seed,
        rows,
    }
}

/// Diagnostic classification returned by the deterministic row oracle [`grade_m6_row`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum M6OracleDiagnostic {
    ExactPass,
    IntermediateStateMismatch {
        expected: i64,
        actual: i64,
    },
    FinalValueMismatch {
        expected: String,
        actual: String,
    },
    HabitCaptureError {
        default_habit_answer: String,
    },
    FalseAbstentionOnSatisfiableRow,
    FailedToAbstainOnUnsatisfiableRow {
        emitted: String,
    },
    CodeVectorMismatch {
        vector_index: usize,
        input: i64,
        expected: i64,
        actual: i64,
    },
    CodeSandboxError(SandboxError),
    MalformedOutput {
        detail: String,
    },
}

/// Result of grading one candidate response against an [`M6FrozenRow`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6RowGrade {
    pub row_id: String,
    pub family: M6RowFamily,
    pub passed: bool,
    pub diagnostic: M6OracleDiagnostic,
}

/// Grade a candidate response string against one [`M6FrozenRow`] using only deterministic checks
/// (D28: zero neural grader).
pub fn grade_m6_row(row: &M6FrozenRow, candidate: &str) -> M6RowGrade {
    let trimmed = candidate.trim();
    match &row.spec {
        M6RowSpec::Compositions(spec) => grade_composition_row(&row.row_id, spec, trimmed),
        M6RowSpec::AlteredConstraints(spec) => {
            grade_altered_constraint_row(&row.row_id, spec, trimmed)
        }
        M6RowSpec::CodeTasks(spec) => grade_code_task_row(&row.row_id, spec, trimmed),
    }
}

/// Grade a candidate [`SandboxProgram`] directly against a `CodeTasks` row by running the
/// 256-step sandboxed register machine across all 4 held-out test vectors.
pub fn grade_code_task_program(
    row_id: &str,
    spec: &CodeTaskSpec,
    program: &SandboxProgram,
) -> M6RowGrade {
    for (idx, &(inp, expected)) in spec.test_vectors.iter().enumerate() {
        match program.execute(inp, MAX_SANDBOX_STEPS) {
            Ok(actual) => {
                if actual != expected {
                    return M6RowGrade {
                        row_id: row_id.to_string(),
                        family: M6RowFamily::CodeTasks,
                        passed: false,
                        diagnostic: M6OracleDiagnostic::CodeVectorMismatch {
                            vector_index: idx,
                            input: inp,
                            expected,
                            actual,
                        },
                    };
                }
            }
            Err(err) => {
                return M6RowGrade {
                    row_id: row_id.to_string(),
                    family: M6RowFamily::CodeTasks,
                    passed: false,
                    diagnostic: M6OracleDiagnostic::CodeSandboxError(err),
                };
            }
        }
    }
    M6RowGrade {
        row_id: row_id.to_string(),
        family: M6RowFamily::CodeTasks,
        passed: true,
        diagnostic: M6OracleDiagnostic::ExactPass,
    }
}

fn parse_canonical_i64(s: &str) -> Option<i64> {
    if s.is_empty() || s.trim() != s {
        return None;
    }
    let val = s.parse::<i64>().ok()?;
    if val.to_string() == s {
        Some(val)
    } else {
        None
    }
}

fn grade_composition_row(row_id: &str, spec: &CompositionSpec, trimmed: &str) -> M6RowGrade {
    let parts: Vec<&str> = trimmed.split(';').collect();
    if parts.len() != 2 {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::Compositions,
            passed: false,
            diagnostic: M6OracleDiagnostic::MalformedOutput {
                detail: format!(
                    "expected exactly 2 fields INT=<canonical_i64>;OUT=<canonical_i64>, got {trimmed:?}"
                ),
            },
        };
    }
    let parsed_int = parts[0].strip_prefix("INT=").and_then(parse_canonical_i64);
    let parsed_out = parts[1].strip_prefix("OUT=").and_then(parse_canonical_i64);
    let (Some(actual_int), Some(actual_out)) = (parsed_int, parsed_out) else {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::Compositions,
            passed: false,
            diagnostic: M6OracleDiagnostic::MalformedOutput {
                detail: format!(
                    "expected canonical INT=<i64>;OUT=<i64> with no leading '+' or zeroes, got {trimmed:?}"
                ),
            },
        };
    };
    if actual_int != spec.expected_intermediate {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::Compositions,
            passed: false,
            diagnostic: M6OracleDiagnostic::IntermediateStateMismatch {
                expected: spec.expected_intermediate,
                actual: actual_int,
            },
        };
    }
    if actual_out != spec.expected_final {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::Compositions,
            passed: false,
            diagnostic: M6OracleDiagnostic::FinalValueMismatch {
                expected: spec.expected_final.to_string(),
                actual: actual_out.to_string(),
            },
        };
    }
    M6RowGrade {
        row_id: row_id.to_string(),
        family: M6RowFamily::Compositions,
        passed: true,
        diagnostic: M6OracleDiagnostic::ExactPass,
    }
}

fn grade_altered_constraint_row(
    row_id: &str,
    spec: &AlteredConstraintSpec,
    trimmed: &str,
) -> M6RowGrade {
    let normalized = match trimmed.strip_prefix("VAL=") {
        Some(rest) if !rest.is_empty() && rest.trim() == rest => rest,
        Some(_) => {
            return M6RowGrade {
                row_id: row_id.to_string(),
                family: M6RowFamily::AlteredConstraints,
                passed: false,
                diagnostic: M6OracleDiagnostic::MalformedOutput {
                    detail: format!("non-canonical VAL= payload in {trimmed:?}"),
                },
            };
        }
        None => trimmed,
    };
    if spec.requires_abstention() {
        if normalized == ABSTAIN_UNSATISFIABLE_TOKEN {
            return M6RowGrade {
                row_id: row_id.to_string(),
                family: M6RowFamily::AlteredConstraints,
                passed: true,
                diagnostic: M6OracleDiagnostic::ExactPass,
            };
        }
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::AlteredConstraints,
            passed: false,
            diagnostic: M6OracleDiagnostic::FailedToAbstainOnUnsatisfiableRow {
                emitted: normalized.to_string(),
            },
        };
    }

    if normalized == ABSTAIN_UNSATISFIABLE_TOKEN {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::AlteredConstraints,
            passed: false,
            diagnostic: M6OracleDiagnostic::FalseAbstentionOnSatisfiableRow,
        };
    }
    if normalized == spec.expected_answer() {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::AlteredConstraints,
            passed: true,
            diagnostic: M6OracleDiagnostic::ExactPass,
        };
    }
    if normalized == spec.default_habit_answer() {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::AlteredConstraints,
            passed: false,
            diagnostic: M6OracleDiagnostic::HabitCaptureError {
                default_habit_answer: spec.default_habit_answer().to_string(),
            },
        };
    }
    M6RowGrade {
        row_id: row_id.to_string(),
        family: M6RowFamily::AlteredConstraints,
        passed: false,
        diagnostic: M6OracleDiagnostic::FinalValueMismatch {
            expected: spec.expected_answer().to_string(),
            actual: normalized.to_string(),
        },
    }
}

fn grade_code_task_row(row_id: &str, spec: &CodeTaskSpec, trimmed: &str) -> M6RowGrade {
    let parts: Vec<&str> = trimmed.split(';').collect();
    if parts.len() != 4 {
        return M6RowGrade {
            row_id: row_id.to_string(),
            family: M6RowFamily::CodeTasks,
            passed: false,
            diagnostic: M6OracleDiagnostic::MalformedOutput {
                detail: format!(
                    "expected exactly 4 semicolon-separated fields V0=..;V1=..;V2=..;V3=.., got {trimmed:?}"
                ),
            },
        };
    }
    let mut values = [0i64; 4];
    for (idx, slot) in values.iter_mut().enumerate() {
        let prefix = format!("V{idx}=");
        let Some(val) = parts[idx]
            .strip_prefix(&prefix)
            .and_then(parse_canonical_i64)
        else {
            return M6RowGrade {
                row_id: row_id.to_string(),
                family: M6RowFamily::CodeTasks,
                passed: false,
                diagnostic: M6OracleDiagnostic::MalformedOutput {
                    detail: format!(
                        "missing or non-canonical V{idx} in candidate output {trimmed:?}"
                    ),
                },
            };
        };
        *slot = val;
    }
    for (idx, &(inp, expected)) in spec.test_vectors.iter().enumerate() {
        let actual = values[idx];
        if actual != expected {
            return M6RowGrade {
                row_id: row_id.to_string(),
                family: M6RowFamily::CodeTasks,
                passed: false,
                diagnostic: M6OracleDiagnostic::CodeVectorMismatch {
                    vector_index: idx,
                    input: inp,
                    expected,
                    actual,
                },
            };
        }
    }
    M6RowGrade {
        row_id: row_id.to_string(),
        family: M6RowFamily::CodeTasks,
        passed: true,
        diagnostic: M6OracleDiagnostic::ExactPass,
    }
}

/// Summary of grading all 240 rows of an [`M6ReasoningPanel`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6PanelScorecard {
    pub total_rows: usize,
    pub overall_passed: usize,
    pub compositions_passed: usize,
    pub altered_constraints_passed: usize,
    pub altered_constraints_satisfiable_passed: usize,
    pub altered_constraints_abstention_passed: usize,
    pub habit_capture_errors: usize,
    pub code_tasks_passed: usize,
    pub row_grades: Vec<M6RowGrade>,
}

/// Grade a slice of candidate answers (aligned 1-to-1 with `panel.rows`) against `panel`.
pub fn grade_m6_panel(panel: &M6ReasoningPanel, answers: &[String]) -> M6PanelScorecard {
    assert_eq!(
        panel.rows.len(),
        answers.len(),
        "candidate answers length ({}) must equal panel rows length ({})",
        answers.len(),
        panel.rows.len()
    );
    let mut overall_passed = 0usize;
    let mut compositions_passed = 0usize;
    let mut altered_constraints_passed = 0usize;
    let mut altered_constraints_satisfiable_passed = 0usize;
    let mut altered_constraints_abstention_passed = 0usize;
    let mut habit_capture_errors = 0usize;
    let mut code_tasks_passed = 0usize;
    let mut row_grades = Vec::with_capacity(panel.rows.len());

    for (row, ans) in panel.rows.iter().zip(answers.iter()) {
        let g = grade_m6_row(row, ans);
        if matches!(g.diagnostic, M6OracleDiagnostic::HabitCaptureError { .. }) {
            habit_capture_errors += 1;
        }
        if g.passed {
            overall_passed += 1;
            match row.family {
                M6RowFamily::Compositions => compositions_passed += 1,
                M6RowFamily::AlteredConstraints => {
                    altered_constraints_passed += 1;
                    if let M6RowSpec::AlteredConstraints(spec) = &row.spec {
                        if spec.requires_abstention() {
                            altered_constraints_abstention_passed += 1;
                        } else {
                            altered_constraints_satisfiable_passed += 1;
                        }
                    }
                }
                M6RowFamily::CodeTasks => code_tasks_passed += 1,
            }
        }
        row_grades.push(g);
    }

    M6PanelScorecard {
        total_rows: panel.rows.len(),
        overall_passed,
        compositions_passed,
        altered_constraints_passed,
        altered_constraints_satisfiable_passed,
        altered_constraints_abstention_passed,
        habit_capture_errors,
        code_tasks_passed,
        row_grades,
    }
}

/// Extract the first unsigned integer numeral occurring in `text` (used by [`M6CountTableBaseline`]
/// to form prompt-only surface buckets without inspecting `M6RowSpec`).
fn first_prompt_u64(text: &str) -> u64 {
    let mut acc: Option<u64> = None;
    for b in text.bytes() {
        if b.is_ascii_digit() {
            let d = u64::from(b - b'0');
            acc = Some(acc.unwrap_or(0).saturating_mul(10).saturating_add(d));
        } else if let Some(v) = acc {
            return v;
        }
    }
    acc.unwrap_or(0)
}

/// Lawful non-neural count-table baseline (CHARTER §4.4) for measuring baseline score and noise on
/// `m6-reasoning`, keyed strictly on the unscored [`M6UnscoredPromptRow`] surface text.
#[derive(Debug, Clone, Default)]
pub struct M6CountTableBaseline {
    /// Prompt-surface feature key -> (answer -> count)
    context_counts: BTreeMap<String, BTreeMap<String, u32>>,
    /// Family -> (answer -> count)
    family_counts: BTreeMap<M6RowFamily, BTreeMap<String, u32>>,
    /// Global answer -> count (excluding abstention when evaluating numeric-only baseline)
    numeric_counts: BTreeMap<String, u32>,
}

impl M6CountTableBaseline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract a surface context key from an unscored [`M6UnscoredPromptRow`] using only the
    /// prompt text and family tag (cannot peek at [`M6RowSpec`] or oracle fields).
    pub fn context_key(prompt_row: &M6UnscoredPromptRow) -> String {
        let body = prompt_row
            .prompt
            .split_once("] ")
            .map(|(_, rest)| rest)
            .unwrap_or(&prompt_row.prompt);
        let first_num = first_prompt_u64(body);
        match prompt_row.family {
            M6RowFamily::Compositions => {
                let kind_tag = if body.starts_with("Register chain:") {
                    "reg_chain"
                } else {
                    "versioned"
                };
                format!("comp::{kind_tag}::{}", first_num % 16)
            }
            M6RowFamily::AlteredConstraints => {
                let sub_tag = if body.starts_with("Altered radix constraint:") {
                    "radix"
                } else if body.starts_with("Altered precedence constraint:") {
                    "prec"
                } else if body.starts_with("Altered transition constraint:") {
                    "mod"
                } else {
                    "feasibility"
                };
                format!("alt::{sub_tag}::{}", first_num % 8)
            }
            M6RowFamily::CodeTasks => {
                let alg_tag = if body.contains("clamp(") {
                    "clamp"
                } else if body.contains("Horner") || body.contains("(((") {
                    "poly"
                } else if body.contains("ring updates") {
                    "ring"
                } else {
                    "parity"
                };
                format!("code::{alg_tag}::{}", first_num % 8)
            }
        }
    }

    /// Observe one training row into the count tables using only its unscored prompt view and
    /// canonical target string.
    pub fn observe(&mut self, prompt_row: &M6UnscoredPromptRow, target: &str) {
        let key = Self::context_key(prompt_row);
        let target_owned = target.to_string();
        *self
            .context_counts
            .entry(key)
            .or_default()
            .entry(target_owned.clone())
            .or_insert(0) += 1;
        *self
            .family_counts
            .entry(prompt_row.family)
            .or_default()
            .entry(target_owned.clone())
            .or_insert(0) += 1;
        if target != ABSTAIN_UNSATISFIABLE_TOKEN {
            *self.numeric_counts.entry(target_owned).or_insert(0) += 1;
        }
    }

    fn top_answer(map: &BTreeMap<String, u32>) -> Option<String> {
        map.iter()
            .max_by(|(k1, c1), (k2, c2)| c1.cmp(c2).then_with(|| k2.cmp(k1)))
            .map(|(k, _)| k.clone())
    }

    /// Predict using the prompt-surface context-key count table first (falling back to global
    /// numeric majority). Accepts `&M6UnscoredPromptRow` so it cannot access [`M6RowSpec`].
    pub fn predict_context_table(&self, prompt_row: &M6UnscoredPromptRow) -> String {
        let key = Self::context_key(prompt_row);
        if let Some(bucket) = self.context_counts.get(&key) {
            if let Some(ans) = Self::top_answer(bucket) {
                return ans;
            }
        }
        Self::top_answer(&self.numeric_counts).unwrap_or_else(|| "INT=0;OUT=0".to_string())
    }

    /// Predict using the family-conditioned modal answer (the best-constant per family).
    pub fn predict_family_modal(&self, prompt_row: &M6UnscoredPromptRow) -> String {
        if let Some(bucket) = self.family_counts.get(&prompt_row.family) {
            if let Some(ans) = Self::top_answer(bucket) {
                return ans;
            }
        }
        "INT=0;OUT=0".to_string()
    }
}

/// Measured noise and baseline summary from evaluating [`M6CountTableBaseline`] across `num_seeds`
/// full 240-row training draws on the frozen 240-row panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6BaselineNoiseReport {
    pub num_seeds: usize,
    pub numeric_only_scores: Vec<usize>,
    pub context_table_scores: Vec<usize>,
    pub family_modal_ceiling: usize,
    pub context_min_score: usize,
    pub context_max_score: usize,
    /// Empirical half-range noise band (`ceil((max - min) / 2)`) measured across the 8 full
    /// seeded count-table draws.
    pub measured_noise_half_band_rows: usize,
}

/// Evaluate the lawful non-neural [`M6CountTableBaseline`] across `8` independent full 240-row
/// seeded training draws on `frozen_panel` (CHARTER §4.4), using only [`M6UnscoredPromptRow`]
/// surface features at prediction time.
pub fn evaluate_count_table_baseline_noise(
    frozen_panel: &M6ReasoningPanel,
) -> M6BaselineNoiseReport {
    let num_seeds = 8usize;
    let mut numeric_only_scores = Vec::with_capacity(num_seeds);
    let mut context_table_scores = Vec::with_capacity(num_seeds);
    let mut family_modal_ceiling = 0usize;
    let eval_prompts: Vec<M6UnscoredPromptRow> = frozen_panel
        .rows
        .iter()
        .map(M6FrozenRow::to_unscored)
        .collect();

    for seed_offset in 1..=num_seeds {
        let train_seed = frozen_panel
            .seed
            .wrapping_add((seed_offset as u64).wrapping_mul(0x517C_C1B7_2722_0A95));
        let train_panel = generate_m6_panel(train_seed);
        let mut baseline = M6CountTableBaseline::new();
        for row in &train_panel.rows {
            baseline.observe(&row.to_unscored(), &row.canonical_answer());
        }

        let numeric_ans =
            M6CountTableBaseline::top_answer(&baseline.numeric_counts).unwrap_or_default();
        let numeric_preds: Vec<String> = vec![numeric_ans; frozen_panel.rows.len()];
        let num_score = grade_m6_panel(frozen_panel, &numeric_preds).overall_passed;
        numeric_only_scores.push(num_score);

        let ctx_preds: Vec<String> = eval_prompts
            .iter()
            .map(|p| baseline.predict_context_table(p))
            .collect();
        let ctx_score = grade_m6_panel(frozen_panel, &ctx_preds).overall_passed;
        context_table_scores.push(ctx_score);

        let modal_preds: Vec<String> = eval_prompts
            .iter()
            .map(|p| baseline.predict_family_modal(p))
            .collect();
        let modal_score = grade_m6_panel(frozen_panel, &modal_preds).overall_passed;
        family_modal_ceiling = family_modal_ceiling.max(modal_score);
    }

    let context_min_score = context_table_scores.iter().copied().min().unwrap_or(0);
    let context_max_score = context_table_scores.iter().copied().max().unwrap_or(0);
    let spread = context_max_score.saturating_sub(context_min_score);
    let measured_noise_half_band_rows = spread.div_ceil(2);

    M6BaselineNoiseReport {
        num_seeds,
        numeric_only_scores,
        context_table_scores,
        family_modal_ceiling,
        context_min_score,
        context_max_score,
        measured_noise_half_band_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_m6_carry_borrow_and_radix_helpers() {
        assert_eq!(count_decimal_carries(0, 0), 0);
        assert_eq!(count_decimal_carries(999, 1), 3);
        assert_eq!(count_decimal_borrows(1000, 1), 3);
        assert_eq!(format_in_radix(0, 16), "0");
        assert_eq!(format_in_radix(255, 16), "FF");
    }

    #[test]
    fn panel_m6_strict_canonical_grading_rejects_non_canonical_numerals_and_junk() {
        let panel = generate_frozen_m6_panel();
        panel.verify_zero_overlap().expect("zero overlap");
        let comp_row = &panel.rows[0];
        let M6RowSpec::Compositions(c) = &comp_row.spec else {
            panic!("row 0 must be Compositions");
        };
        let i = c.expected_intermediate;
        let o = c.expected_final;
        assert!(grade_m6_row(comp_row, &format!("INT={i};OUT={o}")).passed);
        assert!(!grade_m6_row(comp_row, &format!("INT=+{i};OUT={o}")).passed);
        assert!(!grade_m6_row(comp_row, &format!("INT=0{i};OUT={o}")).passed);
        assert!(!grade_m6_row(comp_row, &format!("INT={i};OUT={o};JUNK=1")).passed);
        assert!(!grade_m6_row(comp_row, &format!("INT=0;OUT={o};INT={i}")).passed);

        let code_row = &panel.rows[160];
        let M6RowSpec::CodeTasks(k) = &code_row.spec else {
            panic!("row 160 must be CodeTasks");
        };
        let [(_, v0), (_, v1), (_, v2), (_, v3)] = k.test_vectors.as_slice() else {
            panic!("4 test vectors");
        };
        assert!(grade_m6_row(code_row, &format!("V0={v0};V1={v1};V2={v2};V3={v3}")).passed);
        assert!(!grade_m6_row(code_row, &format!("V0=0{v0};V1={v1};V2={v2};V3={v3}")).passed);
        assert!(
            !grade_m6_row(
                code_row,
                &format!("V0={v0};V1={v1};V2={v2};V3={v3};EXTRA=9")
            )
            .passed
        );
    }

    #[test]
    fn panel_m6_prompt_only_count_table_key_cannot_distinguish_feasibility_polarity() {
        let panel = generate_frozen_m6_panel();
        let mut sat_prefixes = BTreeSet::new();
        let mut unsat_prefixes = BTreeSet::new();
        for row in panel.rows.iter().skip(80 + 60).take(20) {
            let unscored = row.to_unscored();
            let key = M6CountTableBaseline::context_key(&unscored);
            let prefix = key.rsplit_once("::").map(|(p, _)| p).unwrap_or(&key);
            if let M6RowSpec::AlteredConstraints(AlteredConstraintSpec::FeasibilityConstraint {
                satisfiable,
                ..
            }) = &row.spec
            {
                if *satisfiable {
                    sat_prefixes.insert(prefix.to_string());
                } else {
                    unsat_prefixes.insert(prefix.to_string());
                }
            }
        }
        assert_eq!(sat_prefixes, unsat_prefixes);
        assert!(sat_prefixes.contains("alt::feasibility"));
    }
}

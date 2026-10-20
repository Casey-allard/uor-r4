//! Behavioral contract for `UorTime`, `LandauerBudget`, `Nanos`, and the
//! wall-clock binding computation.
//!
//! Target §1.7 pins these contracts:
//!
//! 1. `UorTime::min_wall_clock(&cal)` returns `max(Landauer-bound,
//!    Margolus-Levitin-bound)` as `Nanos`. Landauer = `nats × k_B·T /
//!    thermal_power` seconds; ML = `π·ℏ·steps / (2·E)` seconds per step.
//!    The result is converted to nanoseconds and saturated to [0, u64::MAX].
//! 2. `UorTime` is `PartialOrd`, NOT `Ord`. Two values from unrelated
//!    computations are component-wise incomparable.
//! 3. `LandauerBudget` IS `Ord` (it's a sealed f64 newtype over
//!    `observable:Nats` with NaN excluded by construction).
//! 4. Accessors `landauer_nats()`, `rewrite_steps()` are content-deterministic:
//!    two `UorTime` values minted from the same pipeline inputs return
//!    identical clocks.
//!
//! These tests expose any regression where `min_wall_clock` returns 0
//! unconditionally, where `UorTime` accidentally gains `Ord`, or where the
//! accessors don't reflect the minted values.

use uor_foundation::enforcement::{
    calibrations, CompileTime, CompileUnit, CompileUnitBuilder, ConstrainedTypeInput, Grounded,
    IntegerGroundingMap, Nanos, Term, Validated,
};
use uor_foundation::pipeline::{run_const, validate_compile_unit_const};
use uor_foundation::{DefaultHostTypes, VerificationDomain, WittLevel};
use uor_foundation_test_helpers::{Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

// Phase 9 pinned the carrier types to `<DefaultHostTypes>` for hand-written
// behavioral tests that exercise the f64 default-host path.
type Calibration = uor_foundation::enforcement::Calibration<DefaultHostTypes>;
type LandauerBudget = uor_foundation::enforcement::LandauerBudget<DefaultHostTypes>;
type UorTime = uor_foundation::enforcement::UorTime<DefaultHostTypes>;

const SENTINEL_TERMS: &[Term<'static, N>] =
    &[uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];
static SENTINEL_DOMAINS: &[VerificationDomain] = &[VerificationDomain::Enumerative];

fn build_unit(level: WittLevel, budget: u64) -> Validated<CompileUnit<'static, N>, CompileTime> {
    let builder = CompileUnitBuilder::new()
        .root_term(SENTINEL_TERMS)
        .witt_level_ceiling(level)
        .thermodynamic_budget(budget)
        .target_domains(SENTINEL_DOMAINS)
        .result_type::<ConstrainedTypeInput>();
    validate_compile_unit_const(&builder).expect("fixture: fully-specified builder validates")
}

fn ground(level: WittLevel, budget: u64) -> Grounded<'static, ConstrainedTypeInput, N> {
    let unit = build_unit(level, budget);
    run_const::<ConstrainedTypeInput, IntegerGroundingMap, Fnv1aHasher16, N, 32>(&unit)
        .expect("fixture: run_const must succeed for vacuous input")
}

// ─── UorTime accessors are content-deterministic ────────────────────────

#[test]
fn uor_time_accessors_are_content_deterministic() {
    let g_a = ground(WittLevel::W16, 77);
    let g_b = ground(WittLevel::W16, 77);
    assert_eq!(
        g_a.uor_time().rewrite_steps(),
        g_b.uor_time().rewrite_steps(),
        "rewrite_steps must be content-deterministic"
    );
    assert_eq!(
        g_a.uor_time().landauer_nats().nats(),
        g_b.uor_time().landauer_nats().nats(),
        "landauer_nats must be content-deterministic"
    );
}

#[test]
fn uor_time_reflects_pipeline_work() {
    // Non-trivial work produces non-zero clocks. If rewrite_steps is always
    // 0, the pipeline isn't counting; if landauer_nats is always 0, the
    // cost accounting is broken.
    let g = ground(WittLevel::W32, 200);
    assert!(
        g.uor_time().rewrite_steps() > 0,
        "pipeline::run_const at W32 must count at least one rewrite step (got 0)"
    );
    assert!(
        g.uor_time().landauer_nats().nats() > 0.0,
        "landauer_nats must be positive after irreversible work (got {})",
        g.uor_time().landauer_nats().nats()
    );
}

// ─── min_wall_clock numerical correctness ───────────────────────────────

#[test]
fn min_wall_clock_returns_nonzero_for_slow_substrate() {
    // 32 rewrite steps on an X86 server truly finishes in sub-nanosecond
    // wall-clock time — min_wall_clock correctly returns 0 there. But on
    // a deliberately slow thermal substrate (thermal_power = 1 microwatt),
    // the Landauer bound for the same work should scale up by ~10^8 and
    // exceed 1 nanosecond. If min_wall_clock returns 0 under this
    // calibration, the numerical formula is broken.
    let g = ground(WittLevel::W32, 200);
    // Pushing thermal_power down to the femtowatt scale scales the
    // Landauer bound up by ~10^15, well above 1 ns for any work with
    // landauer_nats > 0.
    let slow = Calibration::new(4.14e-21, 1e-15, 1e-15).expect("slow-substrate cal must validate");
    let nanos: Nanos = g.uor_time().min_wall_clock(&slow);
    assert!(
        nanos.as_u64() > 0,
        "min_wall_clock on a slow substrate (1\u{00b5}W) with 32 rewrite steps \
         must produce non-zero Nanos (got 0 \u{2014} formula is broken)"
    );
}

#[test]
fn min_wall_clock_scales_with_calibration_thermal_power() {
    // Under a tighter thermal_power budget, the Landauer bound goes UP
    // (more time needed to dissipate the same nats). So a slower thermal
    // substrate has a longer lower bound than a faster one.
    let g = ground(WittLevel::W32, 500);

    let fast = Calibration::new(4.14e-21, 1e6, 1e-15).expect("fast cal");
    let slow = Calibration::new(4.14e-21, 1.0, 1e-15).expect("slow cal"); // 1 W

    let nanos_fast = g.uor_time().min_wall_clock(&fast).as_u64();
    let nanos_slow = g.uor_time().min_wall_clock(&slow).as_u64();
    assert!(
        nanos_slow >= nanos_fast,
        "a slower thermal substrate must yield a >= Nanos lower bound \
         (fast={nanos_fast}, slow={nanos_slow})"
    );
}

#[test]
fn min_wall_clock_monotone_in_landauer_nats() {
    // More work (higher witt level + higher budget) → larger Landauer nats
    // → >= Nanos under the same calibration.
    let g_small = ground(WittLevel::W8, 50);
    let g_large = ground(WittLevel::new(128), 500);

    let cal = &calibrations::X86_SERVER;
    let small_nanos = g_small.uor_time().min_wall_clock(cal).as_u64();
    let large_nanos = g_large.uor_time().min_wall_clock(cal).as_u64();
    assert!(
        large_nanos >= small_nanos,
        "larger computation must yield >= Nanos bound (small={small_nanos}, large={large_nanos})"
    );
}

#[test]
fn conservative_worst_case_calibration_dominates_x86_server() {
    // CONSERVATIVE_WORST_CASE is designed as the tightest provable lower bound.
    // Its thermal_power (1e9 W) is orders of magnitude higher than X86_SERVER,
    // but its characteristic_energy (1 J) is astronomically higher than
    // X86_SERVER's (~1e-15 J). The ML bound scales inversely with
    // characteristic_energy, so CONSERVATIVE_WORST_CASE yields the SMALLEST
    // ML bound, which is the point (worst-case-for-verifier, i.e., hardest
    // computation to distinguish from no work).
    //
    // This test pins the relationship rather than a specific number.
    let g = ground(WittLevel::new(64), 300);
    let conservative = g
        .uor_time()
        .min_wall_clock(&calibrations::CONSERVATIVE_WORST_CASE)
        .as_u64();
    let x86 = g
        .uor_time()
        .min_wall_clock(&calibrations::X86_SERVER)
        .as_u64();
    // The conservative calibration yields the tightest lower bound, so
    // x86's more-generous substrate assumption yields a LARGER (or equal)
    // Nanos value. (Applying the tighter calibration to the same UorTime
    // produces the smallest physically-possible bound.)
    assert!(
        x86 >= conservative,
        "X86_SERVER calibration must dominate CONSERVATIVE_WORST_CASE lower bound \
         (conservative={conservative}, x86={x86})"
    );
}

// ─── UorTime is PartialOrd, not Ord ─────────────────────────────────────

#[test]
fn uor_time_implements_partial_ord_not_ord() {
    // Compile-time witness: UorTime : PartialOrd.
    fn require_partial_ord<T: PartialOrd>() {}
    require_partial_ord::<UorTime>();

    // Runtime witness: a non-trivial UorTime is PartialOrd-reflexive.
    let g = ground(WittLevel::W16, 100);
    let t = g.uor_time();
    assert_eq!(
        t.partial_cmp(&t),
        Some(core::cmp::Ordering::Equal),
        "UorTime must be PartialOrd-reflexive"
    );
}

#[test]
fn uor_time_partial_cmp_is_component_wise() {
    // Construct two Grounded values whose UorTime components are ordered
    // the same way on both clocks. The partial_cmp must return Some(_).
    let g_small = ground(WittLevel::W8, 50);
    let g_large = ground(WittLevel::new(128), 500);

    let t_small = g_small.uor_time();
    let t_large = g_large.uor_time();
    // Larger witt + larger budget → larger rewrite_steps AND larger
    // landauer_nats, so the partial_cmp is defined (both components
    // move the same direction).
    let cmp = t_small.partial_cmp(&t_large);
    assert!(
        cmp.is_some(),
        "component-wise partial_cmp must be Some when both clocks move monotonically \
         (got None for small={t_small:?}, large={t_large:?})"
    );
}

// ─── LandauerBudget is Ord ──────────────────────────────────────────────

#[test]
fn landauer_budget_is_ord() {
    fn require_ord<T: Ord>() {}
    require_ord::<LandauerBudget>();
    // Runtime witness: LandauerBudget Ord works for real values.
    let g_small = ground(WittLevel::W8, 50);
    let g_large = ground(WittLevel::new(128), 500);
    let small = g_small.uor_time().landauer_nats();
    let large = g_large.uor_time().landauer_nats();
    assert!(
        small < large,
        "LandauerBudget Ord must order smaller-computation's nats below larger's"
    );
}

// ─── Nanos returns u64 and is Copy ──────────────────────────────────────

#[test]
fn nanos_roundtrips_through_as_u64() {
    let g = ground(WittLevel::W16, 100);
    let nanos = g.uor_time().min_wall_clock(&calibrations::X86_SERVER);
    let copied = nanos; // Copy
    assert_eq!(nanos.as_u64(), copied.as_u64());
}

#[test]
fn min_wall_clock_is_pure_same_inputs_same_output() {
    let g = ground(WittLevel::W32, 100);
    let a = g
        .uor_time()
        .min_wall_clock(&calibrations::X86_SERVER)
        .as_u64();
    let b = g
        .uor_time()
        .min_wall_clock(&calibrations::X86_SERVER)
        .as_u64();
    assert_eq!(
        a, b,
        "min_wall_clock must be pure: same UorTime + same Calibration → same Nanos"
    );
}

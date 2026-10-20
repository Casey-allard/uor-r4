# Phase 1: Prerequisite corrections

## Scope

Three sub-tasks, executed in a single Phase-1 commit:

- **1a** — capacity-guard fail-fast in `primitive_simplicial_nerve_betti`.
- **1b** — hashable-entropy drift gate (audit test only; no current violations).
- **1c** — `HostTypes::Decimal` threading. Infrastructure audit only
  (see "Scope of 1c" below); substantive `f64 → H::Decimal` rewrites
  are deferred to Phase 4, which is where Path-3 blanket impls
  actually need arithmetic-bounded `H::Decimal`.

Phase 1 closes no orphans; it just prevents codegen rules in Phases 2–4
from emitting code that silently degrades or fails to type-check.

## 1a — Capacity-guard fail-fast

**Contract.** `primitive_simplicial_nerve_betti::<T>()` MUST return
`Err(GenericImpossibilityWitness::for_identity("NERVE_CAPACITY_EXCEEDED"))`
when `T::CONSTRAINTS.len() > NERVE_CONSTRAINTS_CAP` or
`T::SITE_COUNT > NERVE_SITES_CAP`. Previously the primitive silently
truncated inputs to the caps, which could produce Betti numbers for a
differently-shaped complex than the caller asked about.

**Scope.**
- Change `primitive_simplicial_nerve_betti` from `const fn` returning
  `[u32; MAX_BETTI_DIMENSION]` to `fn` returning
  `Result<[u32; MAX_BETTI_DIMENSION], GenericImpossibilityWitness>`.
- Update the 5 call-sites in `certify_at<...>` methods in
  `enforcement.rs` + the 1 call-site in `pipeline.rs` to propagate
  via `.map_err(...)?` where the caller wraps the witness.
- `primitive_cartesian_nerve_betti` (in `pipeline.rs`, `const fn`,
  Künneth over the component primitives) also changes to return
  `Result<...>`; likewise drops `const fn`. No in-tree callers exist,
  but the signature must stay consistent.
- The const-ness drop is acceptable: no in-tree caller evaluates either
  primitive in a `const` context (verified by grep for `const [A-Z_]*:`
  bindings of the return type).

**Sites touched** (all in codegen source; foundation regenerated):
- [codegen/src/enforcement.rs](../../codegen/src/enforcement.rs) — primitive definition (~line 6385), 5 caller sites (~lines 8613, 8707, 8799, 8895, 9581).
- [codegen/src/pipeline.rs](../../codegen/src/pipeline.rs) — `primitive_cartesian_nerve_betti` signature and the pipeline.rs:1396 caller site.

**Test.** `foundation/tests/orphan_closure/capacity_fail_fast.rs` — a
test `ConstrainedTypeShape` impl with `CONSTRAINTS.len() == 9` triggers
the fail-fast; a second impl with `SITE_COUNT == 9` triggers it; a
third impl with ≤8 of each returns `Ok(_)` (regression guard).

## 1b — Hashable-entropy drift gate

**Current audit.** No witness or certificate struct in
`foundation/src/enforcement.rs` carries an entropy field (`bits`,
`bits_dissipated`, `landauer_cost`, `landauer_nats`, `entropy`,
`cross_entropy`, `free_energy`) inline. Every certificate stores
identity via `content_fingerprint: ContentFingerprint`; entropy lives
in sibling structs or the `LandauerBudget` carrier. So the rule is
already satisfied.

`UorTime { landauer_nats: LandauerBudget, ... }` (enforcement.rs:1134)
*does* carry entropy and derives `Hash`. This is intentional — `UorTime`
is a structural time carrier, not a content-addressed witness, and its
`PartialEq` is bit-exact on `LandauerBudget`'s underlying `f64::to_bits()`.
Its `Hash` is consistent with its `PartialEq`; content-addressing goes
through fingerprints on types that carry them.

**Gate.** `codegen/tests/no_entropy_hash.rs` — greps
`foundation/src/enforcement.rs` for struct declarations ending in
`Witness` or `Certificate` (the content-addressed-types criterion)
that derive `Hash` AND carry any R7 entropy field inline. Current
matches: zero. The test asserts zero matches with no allow-list.

Rationale for narrowing to `*Witness`/`*Certificate`: the R14 concern is
that content-addressed witnesses should key on fingerprint, not entropy.
`UorTime` is a structural aggregate, not a witness — its Hash is fine.

## 1c — `HostTypes::Decimal` arithmetic bounds

**Current state.** `HostTypes::Decimal` is an unbounded associated type
([foundation/src/lib.rs:180](../../foundation/src/lib.rs#L180)). `DefaultHostTypes` selects `f64`.
Primitives that produce observable values (`LandauerBudget`,
transcendental functions in `math` module, `primitive_descent_metrics`)
all return `f64` directly rather than `H::Decimal`.

**Scope of 1c at Phase 1.** Defer the substantive `f64 → H::Decimal`
rewrite to Phase 4. Rationale:

1. Phase 4's Path-3 blanket impls are the first code that actually needs
   `H::Decimal` arithmetic on the return path. Phase 0's classification
   report shows `CLASSIFICATION_PATH3 = 0` — no Path-3 classes exist yet
   to drive the conversion.
2. The conversion ripples through `LandauerBudget` (backed by `f64`),
   `UorTime` (composed of `LandauerBudget`), `Calibration` (physics
   constants in `f64`), and every `Certificate` carrying these — a
   300–500 line refactor unrelated to Phase 1's bounded scope.
3. Phase 4 will introduce `DecimalArith` (either as a supertrait or as
   direct bounds on `HostTypes::Decimal`) at the moment the first
   blanket impl needs it, with a concrete type-check driving the
   minimal-surface change.

**Gate.** No `no_hardcoded_f64` test at Phase 1. The Phase 4 implementation
will introduce it against a cleaned surface. Documenting this deferral
here satisfies the plan's TDD contract — the Phase 1 boundary is
clear about what it does and doesn't ship.

## Overall verification (Phase 1 close)

- `foundation/tests/orphan_closure/capacity_fail_fast.rs` passes
  (1a work green).
- `codegen/tests/no_entropy_hash.rs` passes with zero matches
  (1b drift gate green, no current violations).
- All Phase 0 tests + conformance still green.
- `cargo run --bin uor-crate` regenerates `foundation/src/enforcement.rs`
  and `foundation/src/pipeline.rs` with the new signatures, and
  `git diff --exit-code foundation/src/` stays clean after the commit.

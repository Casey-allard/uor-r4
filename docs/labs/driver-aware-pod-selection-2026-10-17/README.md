# Exact-driver pod selection — 17 October 2026

## Problem and bounded change

The [stopped timing attempt](../counterfactual-baseline-reuse-timing-startup-2026-10-17/README.md)
allocated a host advertising a driver outside the qualified timing environment,
then correctly refused that driver before instrument execution. Its preserved
[provider investigation](../counterfactual-baseline-reuse-timing-startup-2026-10-17/provider-selection/findings.md)
shows that Vast exposes driver identity before allocation, while the existing
Rust adapter omitted it. The 01:25:10Z exact580.95.05 query found no offers; that
observation does not assert present availability.

[Commission6228059206](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6228059206)
assigns the smallest opt-in repair in the existing provider and pod runner.
The qualified baseline-reuse binary, native geometry, kernel and scientific
criteria are unchanged. Prior evidence #2566 is protected at
`6d3175dabaca055785ac701736296188c83cff94`, with full reviewed-tree equality;
owned transport is removed and required hygiene completed with exit0, preserving peer work.

## Scope and admission

`UOR_POD_VAST_DRIVER_VERSION` selects one exact three-part decimal version.
The Rust parser retains it as a string and admits equality only. The runner
queries that version and enforces equality locally before creation. Missing,
nonstring or mismatched advertised identity cannot become a candidate. The
mutation boundary requires exactly one matching object and successful receipt
construction/append; ambiguous duplicates or write failures stop creation.
An unavailable qualified driver means zero allocation. A constrained request
cannot silently fall back to Runpod, which cannot enforce this Vast predicate.
The selected offer, advertised driver, machine/host identity, raw and admission
quotes, query and selection times are recorded before creation and linked into
the successful up receipt/cache. These are advertised fields, not remote
qualification results. The default remains unset.

Existing unselected behavior, GPU/placement/reliability/storage constraints,
fleet and hourly caps remain. The remote exact-driver gate remains necessary
because advertised metadata can drift before startup. This feature does not
qualify a driver, install a driver, expand the timing allowlist, or grant an
execution release. It performs host orchestration only.

## Verification and delivery

The [author verification receipt](checks/verification.md) names the exact source,
commands, dependency locks and limits. [Selected check files](checks/SHA256SUMS)
retain the initial failures as well as the final successful results:

- Eight Rust unit tests pass; strict Clippy across all targets, rustfmt,
  shell syntax and diff checks pass. The built CLI refuses unsupported driver
  predicates before credentials or transport.
- The complete local fake-provider runner suite passes **365/365**. It covers
  wrong/missing/numeric driver exclusion, cheaper mismatches, exact queries,
  price caps, explicit-provider and wait/fallback refusals, precreation receipt
  ordering, duplicate IDs, failed ledger writes, and successful up/cache linkage.
  Existing unselected placement, caps, uncertain-create reconciliation, secret
  sentinel and archive tests remain covered. No real provider, SSH or GitHub
  command is called by this test harness.
- The sparse workspace crate was copied byte-identically for an isolated
  offline build. Its resolved dependencies match the root lock; the initial
  `--locked` refusal from the reduced package graph is retained. This is scoped
  crate verification, not a whole-workspace build. Temporary build output was
  removed and disk stayed above the 30GiB floor.

The initial runner run had 351 passes and ten failures from stale fixture/message
expectations, including one new `--no-bootstrap` expectation. The corrected
fixture setup pins the fake adapter, tests token setup on the mocked bootstrap
path, and matches existing build-completion/missing-Rust-adapter wording.
Production bootstrap code is unchanged. The intermediate 361/0 run preceded the
four additional duplicate/ledger guard assertions; final 365/0 is the result
for this source. The receipt explains these repairs without treating setup
failures as model evidence.

Independent exact-head review and protected delivery precede use. No real
provider mutation, rental, GPU/model compute or new spending class occurred.

## Next dependency and unchanged holds

After source review, protected delivery and owned cleanup, install/pin the
reviewed helper and runner, and check exact-driver availability without
allocation. No matching offer is an availability blocker, not a model result.
A replacement timing packet needs independent readiness, fresh disk/fleet/price/
ownership admission and its own distinct execution release. Prior timing
release6227893756 remains consumed. No automatic retry or qualification rerun.

Training/M2/Stage B **HELD**; Q4 **NOT_EXECUTABLE**. No apply, speedup, language,
training-rate or energy result. M1 qualification/cost tooling count0/3 and
suspended headline37/232 are unchanged. New GPU cost: **$0**; prior36-lease
Codex lifecycle estimate **$3.418265556**, plus separate PM; storage/transfer
USD **UNMEASURED**.

# Contributing to R⁴

[AGENTS.md](AGENTS.md) is the full operating manual — gates, normative
invariants, the κ re-pin procedure, long-run discipline. Read it before your
first change. The current architecture and issue order live in the
[Geometric Intelligence Programme](docs/geometric_intelligence_programme.md).
The active #973 dense geometric-attention reference and recurrent-factorization
sequence are frozen in
[ADR-0005](docs/adr/0005-predictive-geometric-connection-memory.md).
This file is the short version.

## The loop

1. **Assign yourself the issue.** In this repo assignment means
   *actively-being-worked-right-now*, not "someday" — queued work stays
   unassigned so anyone can pick it up. Never start an unassigned issue without
   assigning it first.
2. Branch `issue-<n>-<slug>` off `main`. **No direct pushes to `main`.**
3. Work and produce the declared product/release evidence. Run a check only if
   the issue explicitly activates it.
4. Open a PR. Automatic QA remains dormant. The immutable repository ruleset
   forces a merge queue, so five instantaneous no-QA acknowledgements carry the
   reviewed commit through it; they are transport metadata, not test results.
5. **Close the issue with the evidence** — the numbers, the verdict against the
   pre-declared exit rule, and the merge commit. Then unassign yourself.
6. Follow-up work discovered mid-stream gets **filed as an issue immediately**,
   not left in a PR body.

## Decision-bearing checks (dormant by default)

Testing and QA do not run automatically. A product or release contract must
name the check, the decision it can change, fixture identity, outcome actions,
and resource budget. These are reference commands, not a pre-commit list:

```bash
cargo fmt --check
cargo check -p <touched-package> --all-targets --offline
cargo test -p <touched-package> --lib --offline
python3 scripts/check_claim_wording.py  # claims/docs only
```

Activate cross-target or certification work only when the product/release
decision requires it. For example, a release decision that names the WASM
boundary may activate:

```bash
cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib
```

Automatic QA is disabled. Legacy ruleset `19597522` still requires five status
names and a merge queue, so pull-request and merge-group events emit transparent
no-QA acknowledgements with those names. They perform no checkout, build, test,
audit, fuzz, WASM, or model work and must never be cited as PASS evidence. A
future product-ready release issue may activate only the QA required for that
frozen release decision. #940 tracks eventual administrator cleanup.
BDD, doctests, no_std, deterministic rebuild, kappa, Gate C, all-features,
WASM, fuzz, Kani, conformance, audit, and corpus-scale work remain dormant
unless the active product/release decision activates them. Source-free
attention probes, anti-recall controls, product transcripts, and serving
censuses are product evidence only when their programme stage calls for them.
Do not create a new test framework for routine confidence.

**Check the check.** Some ways these have silently lied before:

- `cargo … | grep …; echo $?` reports *grep's* exit status. Read
  `${PIPESTATUS[0]}` or run bare.
- After changing a public signature, `cargo clean -p <crate>` before the
  verifying run — stale test targets have "passed" pre-edit code four times.
- A κ test that finishes suspiciously fast has skipped. Confirm
  `/tmp/ref/out/model.bin` exists before trusting green.

## Execution-lane invariants

- **Active geometric intelligence path:** compiler-side floating point and
  allocation are allowed for witnessed chart construction. Source weights,
  `uor-matmul` intelligence projections, transformers, dense matrix
  intelligence, MoE, and sparse learned routing are not serving dependencies.
  Bind lexical codec, prime registry, recursive hierarchy, charts, payload,
  geometry, and decode identities; fixed inputs remain deterministic.
- **Frozen TLA/R4G1 runtime:** its multiplication-free, allocation-free,
  `no_std`, packed-format, and witness contracts remain in force. Do not weaken
  them when touching that lane.
- **Geometry is the route.** Kappa is canonical identity, not a tokenizer or
  semantic distance. Preserve the project shorthand `E8 = H4 x H4`; its exact
  code/serialization contract is the golden/Galois-coupled icosian pair
  `H4 ⊕ phi H4` with fixed basis, glue, and inverse witness.
- **Errors**: library boundaries return `Result` with focused error enums. No
  `unwrap`/`expect`/panic on recoverable paths.
- **No `unsafe`** in the portable runtime or the format crate.
- **Claim language**: `docs/formal_vocabulary.md` is normative, and
  `scripts/check_claim_wording.py` is the dormant automated wording check.
  Exact-equivalence wording
  needs a linked proof artifact.

## If your change is product research

The experiment must be able to change the next programme decision:

- **Preserve the established B0 reference.** #989 reached
  `ESTABLISH_TABLE_NATIVE_LEXICAL_BASELINE`: 99,362/446,342 (22.261404%)
  held-out table top-1 versus 24,163/446,342 (5.413561%) unigram, a
  +16.847843-point uplift, with bounded exact decoding and byte-identical full
  replay. This is statistical lexical prediction, not semantics, attention,
  geometry, correctness, reasoning, chat, or release evidence. The one
  permitted #953 intervention has since been accepted over this unchanged
  reference. #973 retained bounded prior-prefix, paragraph, and conversation
  mechanisms, retained a repaired bounded-global exact-spin witness, and then
  rejected the first natural componentwise-Frechet document placement: real
  8.367592% versus unchanged #953 12.221651%, with shuffled/permuted controls
  also stronger. The bounded gated-delta core later trailed plain delta.
  Direct-attention V2 is non-promotable because its comparators had fewer
  raw-manifold degrees of freedom; equal-manifold-budget V3 returned full H4 3/12 versus plain
  12/12 and isolated the connection/gauge seam. Do not tune any revealed
  fixture. Run fresh separately trained `ConnectionGaugeCovarianceV4`, explicit
  paired-E8/fiber binding, the corpus-bound softmax oracle, normalized resonance
  replacement, and bounded recurrence in that order before any protected
  held-out join. See the
  [#989 record](docs/source_free_table_baseline_989.md).
- **Preserve the GI evidence lineage.** GI-1 makes lexical/address state
  reversible;
  #952 found the reusable ordered-state defect; A1R/#967 repaired the fold but
  terminated `RETAIN_STATE_ONLY` after its scalar readout tied distinct
  candidate-relative states on 6/6 queries; A1P/#970's target-free paired-H4
  exact R4-heatmap gate exercised 14 classes across 36 decisions, had pure
  12/12 construction coverage, covered 10/12 validation decisions with a 10/12
  oracle ceiling, transferred 0/6 queries, and found eight incompatible heatmap
  classes. Downstream readout, selection-control, and placement work is
  `NOT_RUN_IDENTIFIABILITY_HARD_STOP`, not PASS.
  That negative is bounded to the heatmap readout; fixed-zeta phases, ordered
  n-lets, exact `phi` radial transport, and typed geometry adapters remain
  structural. A1Q-L/#969 subsequently qualified one local causal R4/S3 path
  selector. GI-3/#953 implemented bounded provider-free library/CLI loop
  plumbing, but its rank-preserving relabel smoke terminated
  `REVISE_I1_GENERATOR_IN_PLACE`. `PrimaryThenAdjacentSpinFallbackV1` repaired
  the later natural agreement admission: I1/I2/ordered-sentence plus divisor
  recovered exact `{still}` then `{run,runs}` support under equal work, while
  adjacent-spin stayed consulted, truthfully reported physical presence, and
  remained non-admitting until the primary tier was empty. The one permitted
  four-arm run produced `still run` for both full-path prompts
  and `still runs` for both state-disabled prompts, with deterministic replay.
  The first frozen local same-object, order-sensitive candidate-placement
  preflight then reproduced 7/7 construction prototypes with zero class
  collisions, but real placement selected 0/2 intended candidates while the
  same-artifact placement-permuted control selected 2/2. Generation and replay
  were `NOT_RUN`; the terminal remains `REVISE_I1_GENERATOR_IN_PLACE`. A1Q-L2/
  #983 then formed pure construction classes on an independent population but
  transferred on 0/6 held-out decisions and closed
  `UNAVAILABLE_ZERO_CONSTRUCTION_TRANSFER`. A1Q-L3/#986 then closed
  `UNAVAILABLE_FRAME_OR_POPULATION`: its raw corpus reproduced, but the exact
  corpus-scale codec/pair commitment and complete same-frame lexical
  Cl(0,6)/SpiralCore operator map were unavailable. Gate 0, labels, selection,
  and #953 were `NOT_RUN`. That pre-reset handoff left #953 parked pending a
  fresh successor; B0/#989 and the later accepted matched #953 intervention
  supersede that action. A1Q-H/#973 is active: bounded Gate 0, paragraph,
  conversation, and repaired noncommuting-global witnesses are retained; the
  first natural document placement is negative at
  `RETAIN_BOUNDED_GLOBAL_ONLY_REDESIGN_CORPUS_SPIN_PLACEMENT`. Its first bounded
  transported gated-delta core was also weaker than plain delta on its sealed
  construction fixture. The literal causal Q/K/V/O scaffold now exists, but
  equal-manifold-budget V3 rejected its current H4 parameterization while matched plain
  attention worked. #973 owns `ConnectionGaugeCovarianceV4`; recurrent
  factorization follows only after a geometric reference and its resonance
  replacement qualify. GI-4/#954 remains blocked, with GI-5/#955 downstream.
- **Exercise the accepted route path.** #953 geometry runs before token choice
  and emits admitted support and its fixed-point radius trace. The current #973
  reference must hold support fixed, freeze train/validation before outcomes,
  and compare separately trained geometric connections with equal-raw-manifold-budget ordinary
  dense attention and geometry-destroying controls. Transport overhead is
  measured explicitly; it is not hidden under an equal-work claim.
- **Use the smallest falsifier.** `DirectCausalGeometricAttentionR4V1` is one
  head over a bounded causal prefix: learned Q/K/V/O, H4-frame transport,
  stable softmax, and transported value aggregation. If neither it nor the
  matched plain arm beats #953, repair representation/objective before
  recurrence. If plain works and geometry falls outside a predeclared
  non-inferiority margin, repair only the geometric input/connection seam.
  Strict improvement over plain is an advantage claim, not the parity gate.
  The current code scaffold is H4-only and its mixed-gauge H4 representation/
  optimizer combination is negative on V3; the exact group action remains
  algebraically valid, and actual paired-E8 hierarchy/fiber/torsion binding remains
  `NOT_IMPLEMENTED`. Factor into resonance and gated-delta retention only after
  a repaired dense geometric reference works.
- **Parallelize deterministic corpus work.** Partition by content identity,
  use all available local workers with canonical ordered reductions, and
  compare independent multithreaded rebuilds. Do not launch a long experiment
  on one worker.
- **Use non-degenerate anti-recall controls.** Run the active issue's
  predeclared matched controls under equal information and work budgets. The
  historical current-only, additive-summary, factor/count-only,
  ordered-state-permutation, hierarchy-disabled, and exact-recall-only arms
  remain examples only when that issue explicitly activates them.
- **Preserve unseen global context.** Exact route identity remains separate
  from transported trajectory, hypersphere/window summaries, shared-factor
  retrieval, resonance, and accumulated Hopf phase. Those fields are storage,
  diagnostics, or controls until A1Q qualifies them for semantic scoring; an
  exact kappa miss still cannot collapse to a suffix-only default.
- **Keep teachers offline.** Teacher labels/comparisons begin only after a
  source-free report freezes and are never substituted for product output.
- **Include free-running output only when GI-3 or later is in scope.** Attention
  evidence at GI-2 is anti-recall candidate selection, not readable text.
- **Pre-declare the exit rule, the null baseline and the falsifier** before you
  run anything. Write them in the issue.
- **Compute the reachability ceiling first.** If the ceiling is below the effect
  you are hoping for, do not launch. This is a five-minute calculation that has
  invalidated four-hour runs.
- **Activate only a decision-bearing instrument.** Testing/QA stays dormant
  until a product or release contract names the instrument and its action.
- **Pre-declare what each outcome causes.** If the positive and negative branches
  lead to the same next action, the run has no decision value — drop it or
  redesign it.
- **Record negatives.** They are kept, not discarded, and several have redirected
  the whole programme.
- **Make sure your instrument can fail.** Assert the control arm is
  non-degenerate before comparing. An all-zero result across every arm is a
  harness bug until proven otherwise — seven have been found here.
- **Do not turn a negative into infrastructure by default.** A failed operator
  stops or returns to representation design; it does not automatically justify
  a larger corpus, graph format, proof lane, or benchmark suite.

For any run measured in hours, paste the run contract into the issue before
launching:

```
metric to move:       <name, current value>
reachability ceiling: <arithmetic, with the numbers it came from>
instrument + verdict: <which cheap test, what it must report to proceed>
exit rule:            <threshold, pre-declared>
if positive:          <the next action>
if negative:          <the next action, and it must differ>
cost estimate:        <wall clock, and what else it blocks>
```

## Documentation

Records live in `docs/` as `<topic>_<issue>.md` and are **appended to, not
rewritten**, when a later result revises them — the history of what was believed
and when is part of the record. If your work changes a claim in `README.md`,
`docs/RESEARCH.md` or another record, correct it in place and say what revised
it.

## License

By contributing you agree your contributions are licensed under the MIT License.

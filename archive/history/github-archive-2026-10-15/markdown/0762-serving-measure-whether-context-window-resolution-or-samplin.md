# #762 — serving: measure whether context-window resolution or sampling decoding reduce the #759 attractor-basin collapse

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T08:44:22Z
- **Updated:** 2026-08-17T13:45:40Z
- **Closed:** 2026-08-17T13:45:40Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/762
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

#759 (open, corrected diagnosis, no fix shipped) found that #758's
residual prompt-insensitivity -- distinct, unrelated prompts producing
byte-identical `r4 ask` output even after #755's corpus-ordering fix -- is
not a discrete fallback-tier bug. Direct instrumentation of the real
`predict_witness` function on a rebuilt bundle showed every colliding
token resolving at depth 3-4 (the deepest, most context-specific n-gram
level) with large, genuine evidence counts -- never at the empty-context
fallback. The actual mechanism: `assign_window`'s codebook quantization
maps different prompts' short trailing-token windows onto the same or a
sufficiently similar compiled code, and greedy argmax decoding then locks
onto the single highest-count continuation at each step -- the same
"attractor" trajectory -- because it is genuinely the highest-count option
in the table for that code, for every prompt that lands there.

This plausibly connects to the already-closed #460 ("Cover split
criterion and codebook fit"), which measured that **exact-context
match already dominates prediction resolution at 0.9882** and that
neither of the two capacity levers it tested (cover-split criterion,
codebook fit/resolution) moved that dominance by more than marginal
amounts (+5.0pp region-path top-1, capped near +0.15pp actual serving
impact; +0.44pp for codebook fit, saturating near `N/10`). #460's own
closing note: "The bottleneck is now located, and it is not the cover --
it is exact-context dominance."

Read together: #460 already established that *when* a prompt's context
resolves to a match, that match is real and dominant (not a coverage
problem). #759 shows that the actual failure mode isn't about whether a
match is found -- it's that *different* prompts are being quantized into
the *same* match, and once there, deterministic greedy decoding gives
every one of them the identical, single most-frequent continuation. This
is a resolution/decoding-strategy question, not a capacity question, and
#460 did not test it.

# Problem

Two candidate levers, neither yet measured:

1. **Context-window resolution.** `assign_window`'s quantization collapses
   distinct short trailing-token contexts onto the same compiled code.
   Unknown: how much distinct-prompt collision is actually attributable to
   this quantization step specifically (as opposed to genuinely identical
   short contexts, which would be expected and correct to collide), and
   whether a higher-resolution windowing/hashing scheme would meaningfully
   reduce collision without blowing up table size or violating the P-4
   zero-multiply/divide/float contract.
2. **Decoding strategy.** Even with zero quantization collision, greedy
   argmax decoding is deterministic per-context: the same code always
   produces the same next token. Any prompt that reaches that code will
   always get that exact continuation. Sampling (temperature, top-k/top-p
   over the stored count distribution) would break that determinism, at
   the cost of introducing run-to-run nondeterminism into `r4 ask`, which
   this repo's other testing/certification infrastructure may or may not
   tolerate -- unmeasured.

# Goal

Determine, with direct measurement (not speculation), whether either lever
meaningfully reduces the #758-style collapse rate, and if so, at what
cost (table size, P-4 compliance, determinism/testability, compute).

# Scope

- Build or reuse a controlled multi-prompt harness (the #758 15-prompt
  comparison, or similar) that can measure collapse rate --
  what fraction of a fixed prompt set produce byte-identical output -- as
  a single repeatable metric, so both levers can be evaluated against the
  same baseline.
- Measure how much of #758's observed collision is actually attributable
  to `assign_window` quantizing genuinely-different short contexts onto
  the same code (lever 1's target) versus how much is inherent to
  legitimately-identical short contexts colliding correctly (not a bug).
  This requires inspecting the actual codes being produced for the
  colliding prompts, not just the output tokens.
- If lever 1 looks promising: prototype a higher-resolution windowing/
  hashing change, measure its effect on collapse rate, table size, and
  the existing accuracy/parity test suite (`codebook_fit.rs` and similar),
  and confirm it stays within the P-4 shift/add/xor/compare-only
  constraint for any code touching the deployed kernel path.
- If lever 2 looks promising: prototype sampling over the stored count
  distribution (temperature or top-k/top-p) as an alternative to pure
  argmax, measure its effect on collapse rate, and separately assess the
  determinism-loss cost against this repo's existing witness-replay/
  reproducibility guarantees -- those may require an explicit seeded-RNG
  design or a documented "sampling breaks determinism, opt-in only" story
  rather than a silent default change.
- Document findings in `docs/RESEARCH.md` regardless of outcome (positive
  or negative), continuing this repo's existing falsifier-and-record
  discipline (see #460's own closing pattern).

# Non-goals

- Do not implement either lever as a default-on production change without
  a decisive, documented measurement first -- this issue is about finding
  out whether either lever is worth pursuing, not committing to build it.
- Do not re-litigate #460's capacity-lever conclusions; this issue is
  scoped to the resolution/decoding-strategy question #460 did not test,
  not a re-run of #460 itself.
- Do not treat this as blocking #655's broader serving-readiness work;
  this is a quality-research thread that can proceed independently.

# Dependencies and blockers

- Builds directly on #759's evidence and #460's closing measurement.
  Read both in full before starting -- #759 in particular documents the
  exact instrumentation approach (env-var-gated, reverted after
  measurement) that produced its evidence, which is a reusable pattern
  for this issue's own measurement work.

# Proposed implementation shape

Start with the codebook-collision measurement (bullet 2 above) before
prototyping either lever -- it's the cheapest way to find out whether
lever 1 is even addressing a real fraction of the problem, and the
answer changes how much lever 1 is worth building. Only prototype a
lever once its target fraction is measured and non-trivial.

# Acceptance criteria

- [ ] The fraction of #758-style collisions attributable to genuine
      context-window quantization collapse (as opposed to legitimately
      identical short contexts) is measured and documented.
- [ ] At least one of the two levers is prototyped and measured against
      the controlled collapse-rate harness, with a clear before/after
      number.
- [ ] If a lever is adopted, it passes the full verification ladder
      (fmt/clippy/tests, P-4 audit where applicable, allocation census
      where applicable) and does not regress `codebook_fit.rs`/parity
      tests without a documented, accepted trade-off.
- [ ] Findings are recorded in `docs/RESEARCH.md`, including a negative
      result if neither lever moves the needle -- consistent with #460's
      own precedent of documenting a negative/marginal result rather than
      only positive ones.

# Verification

- Re-run the #758 15-prompt (or expanded) comparison harness before and
  after any adopted change, on the same bundle(s), and report the exact
  collapse-rate delta.
- Re-run `codebook_fit.rs` and any other accuracy/parity regression tests
  potentially affected by a context-window resolution change.
- If sampling is adopted for any code path: add an explicit test proving
  the existing deterministic/argmax behavior remains available and
  unaffected when sampling is not opted into.


## Comments (4)

### comment 5313811853 — 2026-08-17T08:44:50Z — Casey-allard

Starting on this, continuing directly from tonight's #759 correction.

Working in `.worktrees/i762-collision-measurement` off `main@611b767b`.

First step per this issue's own proposed shape: measure how much of
#758's observed collision is attributable to `assign_window` quantizing
genuinely-different short contexts onto the same code, versus legitimately
identical short contexts colliding correctly (not a bug). This means
inspecting the actual codes produced for colliding prompts, not just their
output tokens -- reusing the env-var-gated instrumentation pattern from
#759's own investigation (added temporarily, reverted after measurement,
never shipped) rather than guessing from token output alone.

Will report the measured fraction before deciding whether either lever
(context-window resolution or sampling decoding) is worth prototyping, per
this issue's own "measure before building" scope.


### comment 5313862384 — 2026-08-17T08:50:13Z — Casey-allard

# Measurement: quantization collision is real and directly demonstrated

Added temporary instrumentation (env-var-gated `eprintln!` in
`generate_greedy_into`, printing the exact `window` tokens and resulting
`code` passed to `assign_window`/`predict_witness` at every step; not
shipped, reverted after this measurement -- same discipline as #759's own
investigation) and ran 13 distinct prompts against a plain-path-only
rebuild of `smollm2-135m-instruct` (current corpus-derived store, `.r4g1`
graph excluded from a scratch copy of the bundle directory so the legacy
`predict_witness` path is guaranteed to be what's exercised, not R4G1's
separate beam search).

## The direct hit

The two prompts #759 already flagged as colliding, captured at the very
first generation step (using only the prompt's own tokenized seed window,
before any generated tokens could possibly make the two contexts
converge):

```
"Tell me about art."          window=[31530, 549, 563, 984, 30]  code=[213, 84, 76, 140]
"Describe the water cycle."   window=[37964, 260, 913, 4834, 30] code=[213, 84, 76, 140]
```

Two completely different, unrelated 5-token windows (different first
token: 31530 vs 37964; different content tokens throughout) produce the
**exact same 4-byte code**. This is a genuine `assign_window` quantization
collision, not two prompts that happen to share literal context -- it's
measured at step 1, before generation has any chance to make the windows
actually equal. This directly confirms lever 1's hypothesis from this
issue's filing.

## Breadth check across 11 more prompts

Ran the fixed 8-question probe set from `model::LIVE_QUALITY_PROBES` plus
3 more ordinary questions, capturing each one's step-1 `(window, code)`
pair. Across all 13 prompts (78 possible pairs), exactly the one pair
above collided on the full 4-byte code at step 1; every other prompt
produced a distinct code. Several prompts sharing the same opening phrase
("What is...") shared a partial-depth prefix (e.g. bytes 0-1 matching) but
diverged by the full code, which is expected/correct -- shared surface
phrasing legitimately shares shallow context.

Sample size caveat, stated plainly: 1 confirmed collision out of 13
prompts is not enough to fit a reliable rate on its own -- it demonstrates
the mechanism is real and reproducible, not what fraction of arbitrary
prompts hit it. A proper rate estimate needs a larger, systematically
varied prompt sample (worth doing before any prototyping investment
decision, not before establishing the mechanism is real, which is what
this pass was for).

## A refinement to #758/#759's characterization

Diffing the two prompts' full generated output shows they are **not**
byte-identical, contrary to how #758's headline "byte-identical output"
framing might read for this specific pair:

```
< , the the the the, the, the, the,. is
---
> the the the, the, the, the, the,. is
...
<  the the the the, the,
---
>  the the the the, the, the
```

Both collapse into the same degenerate "the the the ..." attractor
trajectory, but diverge by a token or two at the edges. Root cause:
`predict_witness`'s scoring consults `self.state.token_occurrences(t)` --
a per-generation-call mutable repetition counter that starts fresh each
`ask` invocation. Even when two runs' codes match at every step, their
accumulated occurrence counts can differ by the time a near-tied
candidate is scored, occasionally tipping the argmax choice a different
way. So the right framing going forward: quantization collision produces
**attractor convergence** (the two trajectories collapse onto the same
degenerate basin), not a strict byte-identical-output guarantee -- #758's
measured "9 distinct outputs / largest group 5 identical" underestimates
how many prompts are actually in the same basin, since near-miss pairs
like this one would count as 2 distinct outputs under exact-match hashing
despite sharing the same root cause.

## Where this leaves the two levers

- **Lever 1 (context-window resolution)**: confirmed real and directly
  measured on at least one pair. Worth prototyping, but the fix design
  needs care -- `assign_window`'s quantization is presumably deliberately
  lossy/low-resolution as part of how the codebook generalizes across
  similar-but-not-identical contexts (that's the whole point of a
  compiled table-native codebook rather than exact-match memorization);
  a naive "make resolution finer" change risks trading generalization
  for collision-avoidance in a way that could hurt in-distribution
  accuracy elsewhere. Any prototype needs to be measured against both
  the collapse rate *and* `codebook_fit.rs`/parity tests, per this
  issue's own acceptance criteria.
- **Lever 2 (sampling decoding)**: not measured this pass. Would address
  the attractor-convergence symptom (breaks the deterministic "same code
  always same top pick" chain) without touching the quantization/codebook
  question at all, and is a much smaller, more contained change --
  probably the better next slice to prototype first given lever 1's
  generalization trade-off risk above.

## Not done this pass (leaving for the next slice)

Given the depth of tonight's work already, not prototyping either lever
in this pass -- this issue's own scope explicitly separates "measure" from
"prototype the promising lever," and the measurement above is a clean
stopping point: mechanism confirmed real, rate not yet estimated at scale,
and a concrete recommendation (try lever 2 first) with reasoning. Cleaned
up the scratch bundle and worktree; nothing shipped as code from this
pass.


### comment 5314086034 — 2026-08-17T09:12:53Z — Casey-allard

# Design read for lever 2 (sampling decoding) -- more architecturally invasive than it first looked

Read `predict_witness`, its scoring loop, and the P-4 contract test in full
before writing any code, per this repo's own established discipline. Two
things changed my assessment from the previous comment's "probably the
smaller, more contained change" read.

## Finding 1: P-4 is a whole-file source scan, not a reachability scan

`p4_contract_owned_graph_runtime_source_scan` scans entire files
(`runtime.rs`, `engine.rs`, etc.) for forbidden arithmetic tokens (`*`,
`/`, `%`, `wrapping_mul`, ...) -- it does not check whether the code is
actually reachable from the default serving path. That means an opt-in
`predict_witness_sampled` sibling function, if it lives in `runtime.rs`
alongside `predict_witness`, is held to exactly the same zero-multiply/
divide/remainder/float bar as the audited kernel, even though nothing
would call it by default. There is no existing "reachable from default
path" carve-out to lean on.

## Finding 2: weighted sampling needs a primitive this codebase doesn't have yet

Sampling proportional to `(token, score)` pairs needs, at minimum: a PRNG
(some deterministic bit-mixing state, fine under P-4 -- xorshift-style
shift/xor is cheap), and a way to map a random draw into
`[0, total_weight)` to pick among cumulative-weight buckets. The second
part is the hard one under P-4: the standard approach is `draw % total`,
which is forbidden outright (`%` is on the forbidden list). #759's
(abandoned) design comment sketched a shift/compare/subtract binary
long-division loop for an analogous reduction, but that was never actually
implemented anywhere in this codebase -- I looked for an existing
division-free reduction helper to reuse (checked
`signature_affinity_bonus` and its neighbors in `engine.rs`, since #759
pointed at that file's idiom) and found only shift-based *constant*
multiplication (`x * 10` as `(x << 3) + (x << 1)`), not a general
value-into-range reduction. Building one correctly (rejection sampling
via repeated compare-and-subtract, or an explicit binary long-division
loop) is a real, novel primitive that needs its own correctness tests, not
a small tweak.

## The actual design fork

**Option A: keep sampling inside the audited boundary.** Build the
division-free reduction primitive in `runtime.rs`, add a sibling
`predict_witness_sampled` (or a `SamplingMode` parameter threaded through)
that stays P-4-clean. Highest integration cost, but the new capability
lives next to what it modifies and inherits the existing kernel-counted
op-census discipline directly.

**Option B: keep sampling outside the audited boundary.** Split
`predict_witness`'s current "score every candidate, argmax internally, one
`Prediction` out" contract into two pieces: a `runtime.rs`-side function
that returns the full scored candidate distribution (already computed
internally today, just not exposed), and a new selection step -- argmax
(today's behavior, unchanged) or sampling (new) -- that lives in ordinary
`std` application code (`src/chat.rs` or similar), outside P-4's scanned
file set entirely. Lower arithmetic-primitive risk (normal Rust division
is fine there), but changes `predict_witness`'s public return contract,
which `generate_greedy_into` and every existing call site/test currently
depends on -- a wider, more invasive refactor with real risk of
regressing the many things already built on the current shape.

Neither option is a small, contained change. Both have real correctness
and blast-radius risk: A needs a new from-scratch arithmetic primitive
proven correct under the exact constraint that makes bugs easy to hide
(no division to sanity-check against); B changes a widely-depended-on
function's contract across the codebase for a still-experimental,
not-yet-measured-to-help capability.

## Not implementing this pass

Given both paths are genuinely more invasive than "prototype a small
opt-in flag," and this issue's own scope explicitly separates measurement
from prototyping (not "prototype no matter the cost"), not picking one
unilaterally and building around it here -- this reads like a real
design-direction call (which boundary a new probabilistic primitive
should live inside, and whether `predict_witness`'s contract is worth
changing for it) rather than a routine engineering judgment, similar in
kind to how #655-D's packaging-mechanism question got flagged rather than
picked. Recording the fork here so it doesn't need re-deriving next time
this issue is picked up, and leaving the decision open rather than
guessing.

**Lever 1 status unchanged**: still confirmed-real per the prior
measurement comment, not prototyped either, and carries its own
generalization-tradeoff risk noted there. Between the two, lever 1 at
least reuses `assign_window`'s existing arithmetic shape rather than
needing a wholly new primitive -- worth weighing that against lever 2's
P-4-boundary fork above when this is next picked up.


### comment 5316676890 — 2026-08-17T13:25:53Z — Casey-allard

PR incoming: #763.

Implemented lever 2 from this issue's own scoping (weighted sampling over the stored count distribution, opt-in via `--sample <SEED>`), left lever 1 (context-window/codebook-collision resolution) unimplemented -- that's the deeper root cause and belongs with #460's already-open codebook-fit work rather than being redone here.

Measured the actual effect using #758's own methodology (15 prompts, 12 real + 3 edge cases, exact-match dedup) against `smollm2-135m-instruct-tla6` (no `compiled.r4g1`, so this exercises the legacy path this lever targets): deterministic baseline collapses 15 prompts to 13 distinct outputs (one group of 3 byte-identical); `--sample 42` produces 15 distinct outputs, zero collisions. Full numbers and methodology in the PR description.

Auto-merge enabled on #763; this issue will close automatically on merge.



---
_Comment/review ids in this file: 5313811853, 5313862384, 5314086034, 5316676890_


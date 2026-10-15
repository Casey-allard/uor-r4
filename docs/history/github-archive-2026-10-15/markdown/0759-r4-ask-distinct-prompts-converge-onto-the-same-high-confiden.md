# #759 — r4 ask: distinct prompts converge onto the same high-confidence n-gram trajectory (greedy decoding + codebook quantization, not a fallback-tier bug)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-17T05:20:22Z
- **Updated:** 2026-08-17T13:26:13Z
- **Closed:** 2026-08-17T13:26:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/759
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Direct follow-up to #758 (itself a follow-up to #745/#755). #758 quantified and traced the mechanism behind observed prompt-insensitive output on compiled transformerless/R4G1 bundles: `r4 ask` genuinely uses the prompt's trailing tokens as a lookup key, but when that lookup fails to find manifold coverage (the common case for arbitrary, out-of-corpus prompts), both the R4G1 graph path and the legacy plain-TLA path fall through to a small, fixed, context-independent default rather than something that still varies with the prompt.

# The problem

Two independent code paths collapse to context-independent defaults on lookup miss:

**R4G1 graph path** (`crates/uor-r4-graph-runtime/src/engine.rs`, used by `r4 ask` for bundles with a compiled `compiled.r4g1` graph, which is the common case): `predict_candidates_with_signature_lanes` walks a real suffix DFA over the prompt/context's trailing tokens first, and does fall back to a **geometric routing fallback** (nearest region by continuous VSA signature) before finally defaulting. But the final tier is a hardcoded default:

```rust
if active_len == 0 {
    active_nodes[0] = 0;
    active_len = 1;
}
...
if best_token == 0 {
    // emit the first non-zero token from Node 0's emission list
}
```

Node 0's emission list is the same for every prompt, so every prompt that reaches this tier emits from the same fixed source.

**Legacy plain-TLA path** (`crates/uor-r4-core/src/transformerless/runtime.rs`, `predict_witness_plain`): degrades by depth, and at `d=0` the lookup key is the *empty slice* — literally one global table row shared by every prompt with zero deeper coverage:

```rust
pub fn predict_witness_plain(store: &Store, code: &[u8; STAGES]) -> Prediction {
    for d in (0..=STAGES).rev() {
        if let Some(dist) = store[d].get(&code[..d]) { ... return Prediction{token, depth: d as u8, count}; }
    }
    fallback_prediction(store)
}
```

# Evidence

#758's controlled 15-prompt comparison (12 varied real questions + 3 edge cases) against two bundles, hashed for exact-match clustering:

| Bundle | distinct outputs / prompts | largest collapse group |
|---|---|---|
| `smollm2-135m-instruct` (post-#755 fix) | 9 / 15 | 5 different prompts → byte-identical output |
| `smollm2-360m-broad` (#509's clean, larger-corpus bundle) | 11 / 15 | 4 different prompts → byte-identical output |

Larger corpus coverage reduces how often the fallback fires (11/15 vs 9/15 distinct) but doesn't remove the underlying design issue — once coverage runs out, the fallback still collapses to one fixed choice regardless of what was asked.

# Scope

- Change the final-tier fallback(s) so they don't collapse to a single fixed default across all prompts with equal footing. The geometric/VSA signature is *already computed* for region routing in the R4G1 path — investigate using it (or a similar signal already available at generation time) to also differentiate among candidate continuations at the final tier, rather than only using it to pick a region and then defaulting to that region's first non-zero emission.
- For the legacy plain-TLA `d=0` case: consider whether a lower-but-nonzero depth's distribution (rather than the fully context-independent empty-key row) can be preferred when available, or whether the fallback should draw from a wider evidence-weighted set instead of the same single global row every time.
- Whatever the fix, it must stay within the P-4 contract (no float/matmul in the deployed kernel) — this is exactly the kind of design decision that needs to happen carefully, not as a quick patch, given both paths are used by the actual serving surface.
- Surface the existing depth/fallback-tier witness (`Prediction.depth` in the legacy path; the R4G1 path's equivalent tier information) through `r4 ask`/`r4 chat` in some form (flag or verbose mode), so future investigations don't have to re-derive "did this response come from the fallback" via code reading and hashing — #758 found this witness already exists in `uor-r4-api::engine::R4Engine`'s serving path but `chat.rs` never surfaces it.
- After a fix, re-run #758's exact 15-prompt comparison against both bundles as the before/after decisive test, and report the new distinct-output counts.

# Non-goals

- Not attempting to eliminate all output repetition — some convergence across genuinely similar prompts, or short/generic outputs coinciding, is expected and not itself a bug. The target is specifically the *structural* collapse to a fixed, prompt-independent default at the fallback tier.
- Not re-opening #745/#755 — those findings (record-ordering bug, and its fix) stand independently of this issue.
- Not blocking on retraining or recompiling every existing bundle — the fix should be validated against the two bundles already used in #758's measurement.

# Acceptance criteria

- [ ] The final-tier fallback(s) in both the R4G1 and legacy plain-TLA paths no longer collapse dissimilar prompts to byte-identical output at anywhere near #758's measured rate.
- [ ] #758's 15-prompt comparison is re-run post-fix against both `smollm2-135m-instruct` and `smollm2-360m-broad`, and the before/after distinct-output counts are reported (positive or negative result, recorded either way).
- [ ] Existing tests covering `predict_witness_plain`/`predict_candidates_with_signature_lanes` and their fallback tiers still pass; new regression coverage added for the specific fallback-collapse behavior being changed.
- [ ] `docs/RESEARCH.md` updated with the before/after result.

# Verification

Full ladder: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`, `cargo run -p xtask -- audit-limits`, `cargo test --workspace --offline`, plus the #758 15-prompt before/after comparison described above.

References #745, #755, #758


## Comments (5)

### comment 5312386782 — 2026-08-17T05:56:39Z — Casey-allard

Starting on this one, continuing directly from tonight's #758 work.

Working in `.worktrees/i759-fallback-collapse` off `main@25fd9d6e` (post-#758).

Following this issue's own proposed shape: reading `predict_candidates_with_signature_lanes` and its fallback tiers in full first (not just the excerpt quoted in the issue) to understand what signal is actually available at the point where it currently defaults, before deciding how to use it — this is a serving-path change, so I want the design grounded in what's really there rather than a guess from a partial read. Will post a design summary before writing any code, given the issue's own note that this needs care rather than a quick patch.


### comment 5312424291 — 2026-08-17T06:02:38Z — Casey-allard

# Design: what's actually happening, and the planned fix

Read both fallback paths in full (not just the excerpts quoted when this issue was filed) before writing any code, since this is a serving-path change.

## Correction to the original filing

**R4G1 path** (`crates/uor-r4-graph-runtime/src/engine.rs`, `predict_distribution_with_signature_lanes`, called by `predict_candidates_with_signature_lanes`): there are actually **two** fallback tiers, not one. Tier A ("Geometric Routing Fallback", ~line 589) *does* route by the prompt's actual VSA/geometric signature via `rout_probe` — it's genuinely prompt-dependent and not itself the bug. The real fixed point is **Tier B** (~line 661/784): when the per-node emission-affinity scoring loop finds no non-zero-score candidate, it scans raw EMIT bytes from a fixed byte offset 0, **independent of which node Tier A routed to** — so two prompts that geometrically routed to two *different*, real, prompt-specific nodes can still both land on the same byte-offset-0 token if both nodes' emission lists are empty/uncovered. `context_signature`/`session_signature` are still live, unused parameters at that call site — nothing is structurally discarded, they're just not consulted at the final step.

**Legacy path** (`crates/uor-r4-core/src/transformerless/runtime.rs`): `fallback_prediction` (the function literally named "fallback") is almost dead code for any real trained artifact — it's only reached for empty/malformed stores. The actual collapse happens one level up, inside `predict_witness_plain`'s own `d == 0` loop iteration: the lookup key `code[..0]` is *always* the empty slice regardless of the real `code`, so `store[0].get(&[])` is one global table row shared by every prompt whose deeper n-gram codes miss. This exact pattern (`code[..0]` as the depth-0 key) recurs identically in five sibling functions (`predict_witness`, `predict_witness_beam`, `predict_witness_with_priors_beam`, `predict_witness_plain_with_priors`, `predict_witness_plain_beam`) — out of this issue's originally-named scope, noting here rather than silently expanding it.

## Planned fix

**R4G1 Tier B**: instead of taking the first valid EMIT candidate at byte offset 0, collect up to N valid candidates from the remainder into a stack array, and when `context_signature.or(session_signature)` is available, fold its bytes into an index via XOR/rotate bit-mixing (matching the file's existing `signature_affinity_bonus` idiom) reduced into `[0, count)` via a shift/compare/subtract binary long-division loop — zero `*`/`/`/`%` tokens, so it passes the project's machine-checked `p4_contract_owned_graph_runtime_source_scan` (no float/matmul/div in the deployed kernel) without needing an exception. No signature available → identical behavior to today (byte-for-byte compatible with existing tests that pass `signature: None`).

**Legacy path, `predict_witness_plain`'s `d == 0` branch only** (not touching `fallback_prediction`, and not the five siblings — noted below): reweight candidates using the real `code: &[u8; STAGES]` (not the truncated empty lookup key) via a small integer bias — count dominates for any real evidence gap, the code only breaks *ties* (including the common case of a tied root bucket currently silently favoring the smallest token id). This is a deliberately conservative choice: it will not, on its own, eliminate collapse when one token's global count is genuinely dominant (e.g. "the") — that's correct unigram-backoff behavior, not a bug, and consistent with what #758 already measured (larger corpus coverage reduces collapse rate but doesn't eliminate it). A tie-break is what's actually broken here, not the ranking itself.

## Scope decision

Filing the five sibling `d==0`-collapse functions as their own follow-up rather than folding them into this PR — #759 named `predict_witness_plain` specifically, and the beam/priors variants are a larger diff surface with their own test fixtures; keeping this PR's diff reviewable and scoped to what was actually measured in #758 (`r4 ask`'s real code path, which goes through the two functions above, not the beam variants).

## Verification plan

New regression tests: (1) R4G1 Tier B with two different signatures and populated-but-tied emission lists resolves to two different tokens; existing `signature_affinity_bonus`/`rout_probe` tests continue to pass unchanged. (2) `predict_witness_plain` with two different `code` arrays hitting the same `d==0` tied-root-bucket case resolves to two different tokens; `allocation_census.rs` (must stay allocation-free) and `codebook_fit.rs` (accuracy measurement, may shift slightly on tie-break changes — re-run, not just re-read) both re-verified. Then re-run #758's exact 15-prompt comparison against both bundles as the decisive before/after test, per this issue's own acceptance criteria.

PR incoming.


### comment 5312655381 — 2026-08-17T06:32:52Z — Casey-allard

# Correction: empirically verified mechanism is different from the planned fix

Before implementing the fallback-tie-break design from my last comment, I added temporary instrumentation (env-var-gated, not shipped — reverted after this check) to `predict_witness` and to the R4G1 `predict_distribution_with_signature_lanes` fallback tiers, rebuilt `smollm2-135m-instruct` from its corpus via `r4 transformerless compile-recorded`, and traced actual token-by-token resolution for two of #758's colliding prompts. The result contradicts my planned fix's premise, so I'm not implementing it as designed — recording the correction here rather than shipping a fix for the wrong mechanism.

## What I expected vs. what's actually happening

**First, an important scope correction:** `smollm2-135m-instruct` as rebuilt via `compile-recorded` has **no `compiled.r4g1` graph** (that's a separate build step `compile-recorded` doesn't run) — so #758's entire 15-prompt measurement on this bundle went through the **legacy plain-TLA path** (`predict_witness`, called from `runtime::generate_greedy_into`), never through `R4G1Runtime`/`predict_candidates_with_signature_lanes` at all. My design comment's R4G1 Tier B analysis is still an accurate reading of that code, but it's untested against any of #758's actual evidence — it may matter for other bundles that do ship a `compiled.r4g1`, but it isn't what explains the measured collapse.

**Second, and more importantly:** I expected `predict_witness`'s collapse to be happening at `d == 0` (the empty-key, one-global-row case my original design targeted). Direct instrumentation shows this is **not what's happening**. Tracing "Tell me about art." and "Describe the water cycle." — two of #758's five-prompt collapse group — token by token:

```
Tell me about art.:  d=4 t=198 (n=130, dist_len=16) -> d=4 t=198 (n=11869) -> d=4 t=504 (n=20952) -> d=4 t=2115 -> d=4 t=476 -> d=4 t=19146 -> d=4 t=49103 -> d=4 t=331 -> d=4 t=28 -> d=3 t=284 -> ...
Water cycle:          d=4 t=198 (n=136, dist_len=14) -> d=4 t=198 (n=4442)  -> d=4 t=504 (n=5062)  -> d=4 t=2115 -> d=4 t=476 -> d=4 t=19146 -> d=4 t=49103 -> d=4 t=331 -> d=4 t=28 -> d=3 t=284 -> ...
```

Every single token for both prompts resolves at **depth 3-4** (the deepest, most-specific n-gram levels), with real, substantial evidence counts (`dist_len` up to 1000+, counts up to tens of thousands) — never depth 0, never the fallback. These are genuine, well-evidenced predictions, not defaults. For contrast, "What is April?" (a uniquely-outputting prompt) resolves at depth 2 on its first token and follows a completely different token sequence throughout.

## What this actually means

Two different, unrelated prompts are producing byte-identical output through entirely genuine, high-confidence, deep n-gram matches — not through hitting a shared fallback default. The real mechanism looks like: `assign_window`'s codebook quantization maps both prompts' short trailing-token windows onto the same (or a sufficiently similar) compiled code, and once that happens, greedy argmax decoding locks onto the single strongest continuation at each step — the same "attractor" trajectory — because it's genuinely the highest-count option in the table for that code, for both prompts. This is closer to a well-known n-gram/greedy-decoding degeneracy (a small number of extremely frequent short-context -> next-token associations act as basins of attraction that many different starting contexts fall into) than to a "fallback collapses to a fixed choice" bug. It also plausibly connects to this repo's existing open research thread on codebook/cover resolution (**#460**, "Cover split criterion and codebook fit") rather than being a self-contained serving-path defect.

## Why I'm not shipping the originally-planned fix

The tie-break-only fix I designed (use the prompt's geometric signature to break ties at the R4G1/legacy fallback tier) would not touch this mechanism at all — these aren't ties at a fallback tier, they're outright wins at real depth with real evidence. Shipping it would add code and review burden without measurably changing #758's collapse rate, since it targets a tier that isn't firing for the evidence in hand. Reverted all the temporary instrumentation (env-var-gated `eprintln!`s in `predict_witness` and the R4G1 fallback tiers) rather than merge it as a shortcut fix — worktree is clean, nothing to land from this investigation as code.

## Where this leaves things

- The R4G1-path analysis from my previous comment (two real fallback tiers, Tier B collapsing to a fixed byte-offset-0 default) is still an accurate code reading and may be worth fixing on its own merits for bundles that do carry a `compiled.r4g1` graph, but it's unverified against any actual collapse evidence — I don't have a controlled multi-prompt run against an R4G1-bearing bundle showing that tier actually firing. That would need its own before/after measurement, not an assumption carried over from this investigation.
- The legacy-path mechanism now looks like a genuine short-context codebook-quantization/greedy-decoding degeneracy, not a discrete bug with an obvious localized fix — it's a research question about codebook resolution and/or decoding strategy (sampling vs. greedy), with a plausible connection to #460's already-open codebook-fit work.
- Recommending this issue stay open with this corrected diagnosis rather than closing it against a fix that wouldn't address the real mechanism, and treating "does the R4G1 Tier B default actually fire in practice, and does it matter" as a distinct, smaller, still-open sub-question if anyone wants to pick it up — the code-level observation stands, its real-world relevance doesn't (yet).

Recorded here rather than in `docs/RESEARCH.md` for now since this doesn't reach a resolution — happy to fold it into the research log once there's a decision on how (or whether) to pursue the codebook-quantization angle.


### comment 5313808775 — 2026-08-17T08:44:32Z — Casey-allard

Filed the deeper research question as its own scoped issue: #762 --
measures whether context-window resolution (fixing genuine quantization
collisions) or sampling-based decoding (breaking greedy determinism) can
reduce this issue's attractor-basin collapse, building on this issue's own
evidence and #460's closing measurement that exact-context match already
dominates resolution at 0.9882. Leaving this issue open with its corrected
diagnosis as the record of *why* the collapse happens; #762 is where the
"does anything fix it" follow-through lives.


### comment 5316680678 — 2026-08-17T13:26:12Z — Casey-allard

Closing this out with a corrected resolution path, rather than the originally-scoped fix.

Recap: this issue's original scope ("fix the fallback-tier collapse to a fixed, prompt-independent default") turned out not to match the real mechanism -- the correction comment above traced the actual collapse to genuine, well-evidenced argmax resolution at real depth, not a fallback tier. Two prompts land on byte-identical output because greedy decoding always takes the single highest-count continuation once their trailing-token windows quantize to the same (or a sufficiently similar) compiled code -- an "attractor basin" effect connected to #460's already-open codebook/cover-resolution work, not a discrete bug in this issue's originally-named function.

That corrected mechanism was tracked and acted on in #762, now shipped (#763): opt-in weighted sampling (`r4 ask --sample <SEED>`) breaks the exact-repeat collapse this issue measured -- the same 15-prompt methodology used in #758 went from 13/15 distinct (baseline) to 15/15 distinct (sampled), zero collisions, on a bundle exercising the legacy path this issue is scoped to.

This doesn't fix the underlying codebook-quantization root cause (two different prompts still *can* land on the same code), and #762 explicitly left that lever (context-window/collision resolution) to #460 rather than duplicating it here. But the specific, measured symptom this issue tracked -- prompts colliding on identical output -- now has a working, measured mitigation, so I'm closing this rather than leaving it open against a fix that no longer matches the diagnosis. Follow-up root-cause work continues under #460.



---
_Comment/review ids in this file: 5312386782, 5312424291, 5312655381, 5313808775, 5316680678_


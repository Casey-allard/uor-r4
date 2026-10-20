# #280 — R4G1 graph readout predicts 0.0% on held-out while the same store predicts 28.2% — conversion/readout defect

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-29T14:48:19Z
- **Updated:** 2026-07-29T21:28:10Z
- **Closed:** 2026-07-29T21:28:10Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/280
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Measured (2026-07-29, #244 amendment run)

- C R4G1 (graph path, deterministic subsample n=1002): **top1 0.0% | agreement 0.0%** | probe: 64/64 scored
- Same artifacts + store through the store path: top1 28.2% | agreement 31.9% (reproduced bit-for-bit ×3)
- Readiness probe (#232) passed: scored emission on 64/64 probe positions, non-constant — so the graph emits varied scored predictions; they are just systematically wrong. 0/1002 on BOTH metrics is below any trivial baseline (a most-frequent-token constant scores several %), i.e. structural, not statistical.
- Full-eval cost context: first attempt >13.5h in-phase without completing (#277 has the profile).

## Implication

The evidence is intact (store path proves it); the defect is in `convert_r4g1::convert` or the engine readout (`predict_distribution`). The serving cascade (#248) currently tries r4g1 first — until this is fixed, that tier contributes nothing and everything falls through on D4 abstention (verify it actually abstains rather than emitting garbage: a 0%-accurate non-abstaining tier would poison serving).

## Hypotheses, in test order

1. **Token-id domain mismatch**: `check_node_emits` matching against a different id space than corpus token ids (special-token offset; note the `context_tokens[0] <= 1` BOS strip in `predict_distribution` — off-by-one shifts every comparison).
2. **Signature=None fallback path**: certify calls `predict_token(&hist, None, …)`; with no signature the ROUT Hamming fallback may score nodes on a degenerate criterion. Re-run the subsample passing the real per-position signature — if accuracy appears, the readout requires signatures and the eval (and cascade) must supply them.
3. **Emission readout selects wrong field**: final-node emission decode reading a store key/count rather than a token id (would produce varied, scored, never-matching predictions — exactly the observed shape).
4. **Conversion drops/We rewires transition edges** (edge.kind filter, tombstones): DFA walks succeed on short suffixes only and land on unrelated nodes.

## Definition of done

A subsample C row ≥ the B-row floor (22.1% top1) OR a documented decision to demote the graph tier out of the serving cascade until redesign. Either way, add a readiness-probe extension (#232 follow-up): probe should include a small ACCURACY spot-check (e.g. ≥1 hit on 64 probe positions), since scored+non-constant demonstrably does not imply functional.

Refs: #232, #237, #244 (amendment comment), #248, #277.


## Comments (3)

### comment 5120227853 — 2026-07-29T15:50:48Z — Casey-allard

## Root cause found (code-reading diagnosis, line evidence; hypotheses H3+H4 confirmed, H1 ruled out, H2 secondary)

**The converted artifact contains no readable emission structure; the engine compensates by decoding raw TLS1 container bytes as an emission table.**

The chain:

1. **Converter writes no per-node emissions** — every NODE gets `emission_start=0, emission_len=0` (`convert_r4g1.rs:346-347`; declared by design at 81-83: "No PackedNode emission ranges are wired in v0"). The root prior goes into EMIT (282-294) but no node references it.
2. **Converter fills EXCT with the verbatim TLS1 store container** (`convert_r4g1.rs:370-373`: `exct = [2,0,0,0] ++ store_bytes(store)`).
3. **Engine decodes EXCT as (token u32, score i32) at stride 8** (`engine.rs:690-706`), with EXCT taking precedence over the legitimate EMIT root prior (`engine.rs:673-676`). TLS1 is a variable-stride format (magic + u32 counts + 1-byte key lengths + key bytes + 8-byte pairs, `runtime.rs:786-801`): even genuine (token,count) pairs sit at offsets ≡5 (mod 8), so a stride-8 scan **never** reads one on its boundary. Every accepted "token" is a byte-shifted composite of length/key/count fields; the `<49152` filter (`engine.rs:698`) even admits ids ≥ V=32000.
4. **This readout runs on every call**: `check_node_emits` is node-independent on this artifact (per-node branch needs `emission_len>0`, never true; falls through to the same global EXCT scan, `engine.rs:250-276`), and the root is forced into the active set when nothing matches (`engine.rs:652-655`).

**H4 confirmed in stronger form:** the converter emits only `kind=0` refinement edges (`convert_r4g1.rs:120,362`; transition set E_f intentionally empty, 64-73) while the suffix-DFA walk requires `kind==2` (`engine.rs:530-532`) — every suffix of length ≥2 fails unconditionally. There was never a token-walk semantics over the (stage,class) nodes to begin with.

**Why the #232 probe passed:** garbage records carry positive i32 words → always scored (`engine.rs:708-712` clamps raw≤0 to 1); predictions vary because the argmax over the fixed garbage set is modulated by the context-keyed repetition penalty (740-755) and `syntactic_morphism_score` (757-767). Scored + non-constant ⇒ `Ready` — exactly the failure mode the probe cannot see, confirming the accuracy-spot-check extension proposed above.

**Why the store path is unaffected:** `parse_store` decodes the same bytes with the correct grammar (`runtime.rs:805-849`); `predict_witness` argmaxes real token→count distributions in corpus id space (610-641). Same bytes, correct decoder: 28.2%.

**H1 (BOS strip/id offset): ruled out** — one shared token-id space end to end; the break is the byte-misaligned EXCT decode, not the call boundary. **H2 (signature=None): secondary** — with None the ROUT/sig_bonus paths are simply skipped; no degenerate None-scorer exists.

## Falsifying experiments (in cost order)

1. **Fingerprint (minutes, reading-only):** every C-stage prediction must be a member of the fixed set S = {u32 at stride-8 offsets of the TLS1 container, value in (0,49152)}. Enumerate S from the store file; all 1,002 predictions from the #244 run should be ⊆ S. Any prediction outside S falsifies this diagnosis.
2. **Node-independence unit test (<1 min):** `check_node_emits(g,n,t,…)` returns the same boolean for all nodes on the converted artifact, for any t — impossible for a correct emission readout.
3. **One-line ablation (<30 min bounded run):** `engine.rs:675` `exct_remainder.or(emit_remainder)` → `emit_remainder` only; top1 should move 0.0% → ~unigram-argmax baseline, isolating the EXCT decode as the mechanism.

## Implication for scope

This is not a bug fix; **R4G1 v0 was never a functional prediction path** — it is a container format whose readout was scaffolding. The real work is designing the emission/transition wiring (per-node emission ranges from store evidence + kind=2 transitions with defined semantics over (stage,class) nodes), which should be specified against the #243/#248 architecture rather than patched. Until then: the serving cascade's r4g1 tier must be verified to abstain (D4) rather than emit EXCT noise — that check is now the highest-priority item on this issue.

Diagnosis produced by structured code reading against main; full file:line citations inline above.

### comment 5120243233 — 2026-07-29T15:52:18Z — Casey-allard

## Scope refinement: what the 0.0% did and did not measure

Reading the serving path (src/r4g1.rs) against the certify path clarifies the blast radius:

- **Serving's r4g1 tier does NOT use the broken readout.** It loads a *scored* graph (`graph/score.r4g1` + `score_report.json`) through `R4Engine::load` with the D4 status policy (`exact`/`graph`/`novel` → WidenOnce → Abstain), and EXCT is explicitly not enabled there (src/r4g1.rs:105-107). The EXCT-soup decode diagnosed above lives in the certify C-stage pipeline: `convert_r4g1` scaffold → `R4G1Runtime::predict_distribution`.
- **Consequence 1 (good):** the production cascade is not serving EXCT noise; it has its own policy + readability gate, and the #248 abstention semantics stand.
- **Consequence 2 (bad, and the real finding):** the certify C row has never measured the serving graph path. The 0.0% measured a conversion scaffold that (per the diagnosis above) was never a functional prediction path — while the artifact that actually serves (`score.r4g1`, D4 policy) has **no held-out accuracy measurement at all**.

## Reframed definition of done

1. Point the certify C row at the serving surface: `R4Engine` + `score.r4g1` + D4 policy, measuring served-vs-abstained rates and top1/agreement on served positions (the stride-subsample + budget machinery from #282 applies unchanged). That is the honest C row.
2. Decide the fate of the `convert_r4g1`/`R4G1Runtime` scaffold: either specify real emission/transition wiring against the #243/#248 architecture, or mark it explicitly as a container-format demo and remove it from the measurement matrix (with the 0.0% row retained as the recorded reason).
3. The #232 probe accuracy spot-check still applies to whichever path certify measures.

### comment 5123548356 — 2026-07-29T21:28:09Z — Casey-allard

## Closing — reframed DoD delivered in PR #288 (merge commit 89d74d9)

1. **Honest C row shipped**: `r4 certify` now measures the serving surface — `R4Engine` + `score.r4g1` + D4 policy, token windows like the server — on the bundle's own held-out partition, with the #282 stride subsample and #279 budgets (same env vars). First measurement (SmolLM2-135M-Instruct-7e27bd9f9532, n=1021, ~1s vs 13.5h+ in-phase on the scaffold): **served 1021/1021 — every position ExactContext — top1 5.6% | agreement 12.6% | probe 14/64 hits**. Per-status breakdowns print with every row (#234 item 3).
2. **Scaffold fate decided (Casey, 2026-07-29): demoted.** `convert_r4g1` is documented as a container-format demo and `R4G1Runtime` carries the accuracy-status note; the 0.0% row recorded on this issue stands as the reason the scaffold left the measurement matrix. Real emission/transition wiring remains future work specified against the #243/#248 architecture.
3. **#232 probe extension shipped**: the readiness probe includes an accuracy spot-check — serving predictions with zero hits across 64 probe positions is now a recorded skip, not a green light. Scored + non-constant demonstrably did not imply functional; now it can't pass as such.

Follow-through: #278 closes as overtaken (emit index targeted the demoted scaffold); #277 item 2 rescopes to the serving engine; the all-ExactContext + 12.6%-vs-31.7% findings are posted to #234 and #268 respectively.


---
_Comment/review ids in this file: 5120227853, 5120243233, 5123548356_


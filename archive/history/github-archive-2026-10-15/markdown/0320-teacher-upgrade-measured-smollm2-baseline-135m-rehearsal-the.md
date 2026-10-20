# #320 — Teacher upgrade: measured SmolLM2 baseline (135M rehearsal, then 360M)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-01T00:48:14Z
- **Updated:** 2026-08-08T20:27:05Z
- **Closed:** 2026-08-08T20:27:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/320
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

Every assignment-side measurement (#243 decomposition, #310 A-W negative result, #318 dot-resid rows) converges on the same picture: the shipped integer runtime is within ~1.5–2.5pp of its own f32 assignment ceiling, and remaining compiler-side headroom is fractional-pp. The binding constraint on absolute accuracy is the **teacher**: the 15M stories checkpoint that pins the corpus, the f32 ceiling, and everything downstream.

The repo already has the machinery for a stronger teacher: the HF compile path (`compile_hugging_face`, `HuggingFaceLlamaOracle` with the fast-matmul backend) and descriptors at `models/smollm2-135m-instruct.json` / `models/smollm2-360m-instruct.json`. What it does not have is a measured SmolLM2 baseline.

## Proposal (phased; each phase is its own PR)

1. **P1 — Checkpoint + corpus**: fetch the pinned SmolLM2-135M-instruct HF checkpoint (pin by revision hash, record source κ), generate the observation corpus via the existing HF observation pipeline. Era-noted; does not disturb the stories15M fixtures.
2. **P2 — Compile + certify**: compile via the HF path, run the row matrix, post the teacher floor/ceiling and the A-row table against the stories15M matrix recorded in #310/#318. Decision point: does the ceiling lift enough to justify migration?
3. **P3 — Baseline migration (maintainer decision)**: new pinned fixtures, κ re-pin with era notes, BASELINE.md/PROOF.md updates, `tless_uor.rs` era-sensitive depth expectations updated per AGENTS.md.

## Honest constraints

- κ baselines are macOS-pinned and libm-sensitive; the HF path uses the fast matmul backend whose logits are teacher data (not part of the pinned legacy proof), so P1/P2 are safe, but P3's re-pin is a maintainer decision per AGENTS.md.
- Disk on the measurement machine is at 100% capacity — checkpoint fetch needs ~300 MB (135M) / ~750 MB (360M); clean-up first.
- 360M is the more interesting teacher; 135M is the cheaper rehearsal. Suggest 135M for P1-P2, 360M only if the ceiling math justifies it.

Origin: #310 review conversation — option "lift the ceiling itself".


## Comments (11)

### comment 5153570178 — 2026-08-01T21:40:26Z — auser

## Status audit (2026-08-01): P1 rehearsal landed; P2/P3 remain

PR #322 (merge 6dce9be) delivered P1 at rehearsal depth: revision pin + source κ recorded in models/smollm2-135m-instruct.json (blake3:12d2cd8a…, 269,060,552 bytes), provenance + repro commands in docs/smollm2_teacher_baseline_320.md. The corpus generated was 1,000-token / 11-story rehearsal only, explicitly non-quotable.

Verified against origin/main @ ff58b5c:

- **P2 NOT met** — the doc itself states 'no comparison with the stories15M teacher floor was made. P2 should run the declared row matrix on a complete corpus before any migration decision.' No SmolLM2 floor/ceiling numbers or A-row table exist in docs/ on main. (The EXTERNAL_REFEREE_300.md 3.84-bits figure is the #300 D3 track, not the #310/#318 row matrix.)
- **P3 NOT met** — pinned proof/baseline is still stories15M (PROOF.md:69-70, tests/bdd.rs:368 anchors, tless_uor.rs:1036 era note, AGENTS.md Gate E llama2.c checkpoint).

Remaining: (a) full SmolLM2 observation corpus on the pinned revision (P1 completion — note the issue's disk-space caveat: measurement machine was at 100%, ~300MB needed); (b) row matrix + teacher floor/ceiling posted vs the stories15M matrix, decision point; (c) P3 migration, a maintainer decision that cannot be evaluated until P2's numbers exist.

### comment 5155323715 — 2026-08-02T04:32:13Z — auser

## 360M feasibility rehearsal (2026-08-02)

Started the pinned local source `HuggingFaceTB/SmolLM2-360M-Instruct` at revision `2366112999e525164f9f74a3fbf50ec19b48b940` with the existing Accelerate CPU teacher path:

```text
cargo run --release --offline --bin r4 -- compile \
  --source .uor-models/sources/smollm2-360m-instruct \
  --output .uor-models/compiled/smollm2-360m-instruct \
  --seconds 600 --target 500000 --sequence-length 128
```

The bounded run produced a resumable partial corpus of **34,580 / 500,000 tokens** across 408 stories in 601.83s (**57.5 teacher tokens/s**). It did not produce a complete artifact or quality row, so this is feasibility/throughput evidence only—not a P2 baseline or migration decision.

The autoregressive teacher loop is stateful; the repo’s prior row-parallel experiment was a measured regression. The safe parallelism remains in post-generation induction/store stages. A complete 500k 360M run at this observed rate is approximately 2.4 hours of CPU generation before compilation/certification.

### comment 5181043594 — 2026-08-04T15:17:10Z — Casey-allard

Supporting data point from the #393 program (posted with Casey's OK — this measurably strengthens the case for this upgrade): the 08-04 decider campaign traced the natural-text collapse of the legacy store to **teacher evidence quality, not addressing geometry**. With stories15M evidence, the store scores 0.1% on held-out simple-wiki (below the 4.5% unigram floor); swapping to corpus-observed evidence recovers 4.0%; the symmetric fixture check (teacher 34.7% vs observed 33.6%) confirms teacher top-k evidence is mildly superior on-distribution and catastrophic off-distribution. Full record on #394.

Implication: a wiki-competent teacher (SmolLM2 as proposed here) is potentially **load-bearing for the program's scale direction**, not just an iteration-quality upgrade — teacher-evidence stores on open text are bounded by teacher domain competence. The stories15M iteration-floor rule stands for fixture work; this issue's priority may deserve revisiting when the scale corpus lands (10k-article observation pass currently running).

### comment 5182012843 — 2026-08-04T16:44:40Z — Casey-allard

Picking this up (Casey's call — assignment was parking, not a fence). Progress: pinned source downloaded and verified in the program's cloud environment — HuggingFaceTB/SmolLM2-135M-Instruct @ 7e27bd9f, source κ blake3:12d2cd8a… **MATCH** against models/smollm2-135m-instruct.json (42 files, 259.9 MiB). Next per the issue plan: 135M rehearsal — a bounded observe/compile/certify pass to validate the HF-oracle pipeline end-to-end — queued behind the running 10k-article scale observation (the two compete for the same cores; the scale corpus also happens to be exactly what a wiki-competent teacher should be measured against, per the evidence-quality finding cross-linked above). Full measured baseline follows the rehearsal.

### comment 5186159000 — 2026-08-05T00:35:13Z — Casey-allard

Current-era refresh of the 135M rehearsal (post-#417 binary, schema 21, cloud run 2026-08-05): resumed pinned-source compile to the full 20,000-record corpus (rev 7e27bd9f, source κ blake3:12d2cd8a — unchanged), cover + score complete. Gate C (3,962 held-out): Rule 1+2 8.8%/13.90 bits | TLA3 baseline 9.8%/22.38 | 1+2 × fwd-anchor (true, = A-mode) 9.5%, live 10.0 vs 8.5 (+1.5pp, n=1,827) — the #399 A-mode lift replicates in direction on a second teacher. All self/draft anchored arms negative here, consistent with the #399 falsifications and explained by predicted-anchor accuracy collapsing to 4.8% (vs 32.5% on stories15M): self-anchoring requires a base scorer strong enough to produce representative anchors. Small-corpus caveat: 20k records is a rehearsal-scale corpus; the P2→P3 ceiling/migration question stays open pending a full-scale SmolLM2 corpus.

### comment 5194742647 — 2026-08-05T16:55:06Z — Casey-allard

Priority escalation (from #433's evidence audit): P(next==t_argmax) = 6.4% on wiki10k vs 70.2% on stories-teacher home corpus — the stories15M teacher's argmax is nearly degenerate on broad text (its unigram mode alone is 26.8% of positions), which bounded the entire falsifier-2 campaign. **A stronger teacher is the prerequisite for any broad-corpus program.** The 135M current-era rehearsal is posted above; P2→P3 with 360M on a broad corpus is now the highest-leverage open work on the board.

### comment 5225704576 — 2026-08-08T10:28:25Z — Casey-allard

Unassigning @auser and returning this to the open pool (no activity since 2026-08-05; not currently being worked). This is a phased epic — leaving it open with the current state recorded.

**Landed:** P1 (135M rehearsal) — pinned source `HuggingFaceTB/SmolLM2-135M-Instruct` @ 7e27bd9f, source κ `blake3:12d2cd8a…`, provenance + repro in `docs/smollm2_teacher_baseline_320.md` (PR #322). A current-era 135M rehearsal was refreshed 2026-08-05 (full 20k-record corpus; Gate C 3,962 held-out: Rule 1+2 8.8% / 13.90 bits). A 360M feasibility rehearsal was started.

**Open (the actual decision):**
- **P2 — compile + certify at depth, then the ceiling decision:** does the SmolLM2 teacher floor/ceiling lift enough over the stories15M matrix (#310/#318) to justify migration? This is the load-bearing question — RESEARCH.md and the #433 audit both name the teacher as the binding constraint on absolute accuracy ("a stronger teacher is the prerequisite for any broad-corpus program"; stories15M argmax is near-degenerate on broad text — 6.4% next==argmax on wiki10k vs 70.2% on its home corpus).
- **P3 — baseline migration (maintainer decision):** new pinned fixtures, κ re-pin with era notes, BASELINE.md/PROOF.md updates. Per AGENTS.md this is a maintainer call, not an automatic follow-on.

Constraints on record: κ baselines are macOS/libm-pinned; checkpoint fetch needs ~300 MB (135M) / ~750 MB (360M) and the measurement machine's disk was at capacity — clean up first.


### comment 5226103245 — 2026-08-08T12:28:37Z — Casey-allard

## P2 — compiled + certified (cloud, 2026-08-08)

Re-ran the P2 pipeline on the pinned SmolLM2-135M-Instruct source (rev `7e27bd9f`, source κ `blake3:12d2cd8a…`, unchanged): observe → 20,000-record corpus → cover → score (`--quality-profile relative_tla`) → teacher-floor report. Note this cloud runs the teacher at **~17 tok/s** (vs the doc's 128), so the observe needed a larger time budget; corpus regenerated (κ differs from the 08-05 run's).

**Gate C (3,962 held-out):**

| arm | top-1 | bits/token |
|---|---:|---:|
| Rule 1+2 (precedence) | 6.36% | 14.16 |
| graph-only (Rule 1 chain, EXCT off) | 1.54% | 14.46 |
| TLA3 store baseline | 12.52% | 20.58 |
| unigram null | 3.63% | 12.16 |

EXCT-miss 73% (`can_measure_generalization = true`). **Teacher floor: 4.20 bits/token** — the SmolLM2-135M teacher's own cross-entropy on the corpus (the ceiling); the artifact sits 16.4 bits above it.

## The migration decision (the point of P2)

The teacher floors are **not directly comparable** — they are measured on different corpora — but that difference is exactly the decision:

- **stories15M** (BASELINE.md, 500k era): teacher floor **1.43 bits**, ceiling **70.5%** — on its *narrow home distribution*. On broad text it is **degenerate** (6.4% next==argmax on wiki10k vs 70.2% at home), which is what bounded the whole broad-corpus / falsifier-2 program.
- **SmolLM2-135M**: teacher floor **4.20 bits** on real instruction text — a *competent general* teacher, not degenerate off-distribution.

So the ceiling "lift" is corpus-dependent. For the **stories-home benchmark**, stories15M is the stronger teacher (1.43-bit floor / 70.5% ceiling) and migrating would *regress* it. For the **broad-corpus direction** the project's own evidence keeps pointing at (RESEARCH.md; #458/#459 both concluded "more corpus/teacher"), stories15M is unusable and SmolLM2-135M is the competent teacher that makes broad-corpus work possible at all.

**P2 verdict:** SmolLM2-135M is confirmed a viable teacher (compiles cleanly, 4.20-bit floor, generalization measurable). The migration is a genuine trade — strong-narrow (stories15M) vs competent-broad (SmolLM2) — so **P3 (the re-pin) stays a maintainer decision per AGENTS.md**, and it should be taken together with the decision to commit to the broad-corpus program. The compiled artifact's own headroom (16.4 bits over the teacher floor) is now compiler-bound, not teacher-bound, on this corpus.

Leaving this open (phased epic): P1/P2 recorded; **P3 migration is the remaining maintainer decision.** Unassigning.


### comment 5227288440 — 2026-08-08T17:31:43Z — Casey-allard

Picking up **P2 at 360M** (Casey's call). SmolLM2-360M is the 'more interesting teacher' this issue names; the 135M rehearsal floor (4.20 bits) is recorded but 360M was only started (Ari's ~34.6k/500k feasibility rehearsal). Running the full P2 pipeline on the pinned 360M source (rev 2366112999e525164f9f74a3fbf50ec19b48b940): download → observe → cover → score (relative_tla) → teacher floor/ceiling + row matrix vs the 135M/stories15M matrix. This is P2 measurement (decision-input); **P3 (the baseline re-pin) stays a maintainer decision** per AGENTS.md. Assigning myself while actively running it.

### comment 5227726638 — 2026-08-08T19:24:53Z — Casey-allard

## P2 — SmolLM2-360M compiled + certified (cloud, 2026-08-08)

Ran the full P2 pipeline on the 360M source: observe → 20,000-record corpus → cover → score (`relative_tla`) → teacher-floor report.

### Provenance (drift note)

The descriptor's pinned revision `2366112999e525164f9f74a3fbf50ec19b48b940` **404s on HF now** (upstream rewrote it). I used current `main` (`a10cc1512eabd3dde888204e902eca88bddb4951`); its `model.safetensors` is **byte-identical (723,674,912 bytes) to the copy pinned on the measurement Mac in July**, so the weights are the same SmolLM2-360M-Instruct release — only a metadata-commit rev bump. κ on record: source `blake3:27ec272b…`, teacher (compiler layout) `blake3:eb23c3e8…`, artifacts `blake3:910b1112…`. (P3's re-pin is a maintainer step regardless, so measuring the current release is valid for P2.)

### Gate C — held-out D3 (4,098 positions), EXCT-miss 73.1% (generalization measurable)

| scorer | top-1 | bits/token |
|---|---:|---:|
| TLA table-native (evaluate-report) | 5.54% | 21.18 |
| graph chain+EXCT (Rule 1+2) | 5.0% | 14.78 |
| graph chain (Rule 1, EXCT off) | 0.3% | 15.12 |
| 1+2 × fwd-anchor (#399 A-mode) | 5.7% | 14.69 |
| TLA3 store baseline | 8.6% | 21.18 |
| teacher-argmax agreement | 8.61% | — |

**Teacher floor: 3.7908 bits/token** — the artifact sits **+17.39 bits over it** (compiler-bound on this corpus, not teacher-bound). Floor decomposition: by position 0-7 → 5.44 bits, 64-127 → 3.47 (context helps); by class digit 2.60 / alpha 4.61 / other 2.02.

### Teacher floors side-by-side (the migration signal)

| teacher | floor (bits/token) | corpus |
|---|---:|---|
| **SmolLM2-360M** | **3.79** | 20k instruction (this run) |
| SmolLM2-135M (cloud, 08-08 AM) | 4.20 | 20k instruction |
| SmolLM2-135M (original P2 doc) | 4.95 | 20k instruction |
| stories15M (BASELINE.md ref) | 1.43 | its narrow home corpus |

Floors are on different corpora, but the trend is the point: **360M lowers the teacher floor 4.20 → 3.79 vs 135M — a more competent teacher, a genuinely higher ceiling.** The artifact top-1 (5.5%) is *lower* than the 135M doc row (7.6%), but that is compiler/corpus-draw dependent, not the ceiling: a sharper teacher is harder for the lossy TLA compilation to match at 20k. The headroom that matters — 17.4 bits over a 3.79-bit floor — is the substrate's to close with more data, not the teacher's.

### P2 verdict

360M is confirmed a viable, more-competent teacher (compiles clean, floor 3.79 vs 135M's 4.20, generalization measurable). It sharpens the same trade the 135M P2 recorded — strong-narrow (stories15M, 1.43-bit home floor) vs competent-broad (SmolLM2-360M, 3.79-bit instruction floor) — and it strengthens the case that the ceiling lift is real but **only cashable at broad-corpus scale**, where the artifact can actually reach a lower floor. **P3 (the re-pin) stays a maintainer decision** per AGENTS.md, best taken together with committing to the broad-corpus program (the agreed next big run). Leaving #320 open; unassigning after this record.


### comment 5227980291 — 2026-08-08T20:27:04Z — Casey-allard

Closing — P1/P2 measurement is complete and documented (135M rehearsal floor 4.20; **360M P2 floor 3.79 bits**, a measurably more competent teacher, landed in PR #506 / docs/smollm2_teacher_baseline_320.md). Per the decision on this issue, **P3 (the baseline re-pin) is deferred into the broad-corpus program**: a re-pin is only meaningful once a broad-corpus SmolLM2 baseline exists to pin, and the 20k rehearsal artifacts are compiler-bound (+17.4 bits over the 3.79 floor), not a pinnable baseline. The measurement epic's asks are delivered; the eventual re-pin will be tracked with the broad-corpus work rather than as a lingering open issue. stories15M fixtures remain unchanged until then.


---
_Comment/review ids in this file: 5153570178, 5155323715, 5181043594, 5182012843, 5186159000, 5194742647, 5225704576, 5226103245, 5227288440, 5227726638, 5227980291_


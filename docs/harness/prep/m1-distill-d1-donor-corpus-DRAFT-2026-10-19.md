# `distill-d1` — one-time offline donor corpus (gpt-oss-120b)

- **Status:** DRAFT pre-registration, **TEST FITNESS: PENDING**. No compute, no pod, $0 so far.
- **Line:** M1 corpus infrastructure (#2029). **Lab:** kimi (pm1). **Reviewer:** fresh-context adversarial reviewer subagent (owner direction 6255848646), plus a non-author seat.
- **Governing decision:** **D41**, ratified 2026-10-19. One donor `openai/gpt-oss-120b`, one corpus `distill-d1`, one pass. Clause 2 binds the generation receipt; clause 3 forbids donor execution in student preparation, training, evaluation, serving, grading or any panel, and forbids scoring donor output against the student; clause 4 leaves D11, D23, D25, D29, D30 and the D32 discrete path unchanged on the student side; clause 5 makes the corpus an instrument input; clause 6 expires the exception at seal; clause 7 limits cost to the prospective plan below.
- **Purpose (one line):** generate, bind and consume one hash-pinned 30M-token distilled corpus for the discrete student, without ever running a teacher inside the student path.
- **No capability claim follows.** The corpus is an instrument input, not model evidence. Its existence, size or quality proves nothing about the student. Whether distillation helps is a separate training sheet with a matched control and sealed panels (§10).
- **Scope:** this sheet authorizes **corpus generation and consumption-plumbing validation only**. It authorizes no student training, no model comparison, no donor evaluation, and no transform of any existing panel. Student training after the seal needs its own FIT'd sheet.
- **Rules:** D22 (scope of a negative), D24 (GPU-only model compute), D26 (Vast default), D27 (Rust), D29 (scanned modules), D31 (no ruling out on an instrument technicality), D40 (owner types the release), D41 (this exception). D23/D25/D28/D30 apply unchanged to everything on the student side; D41 is the only carve-out and it covers generation only.
- **Token counting convention:** every count in this sheet is **student tokens** under the pinned tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` (`docs/labs/rung0-pilot-release-2026-10-14/sheet.md:90-91`), unless labelled "donor tokens". Donor tokens are never the budget unit, because the student never sees them.

## Bar

The registered acceptance bars of this sheet are the exact gate thresholds: **G1–G14** (§3) for the production corpus and **P1–P8** (§4.1) for the pilot, including the go/no-go rule (§4.2: measured tok/s ≥ 300 to proceed; < 250 stops for owner re-plan). Any threshold change is a re-registration with a new FIT review.

## Base artifact

- **Decision:** D41 (ratified 2026-10-19), one donor `openai/gpt-oss-120b`, one corpus `distill-d1`, one pass; the exception expires at seal.
- **Donor base:** `openai/gpt-oss-120b`, pinned revision, released MXFP4 weights (per-file + combined SHA-256), Apache-2.0.
- **Student base:** pinned tokenizer `d36d3e87…`, vocabulary 4096, UORT v1 format, `PreparedCorpus::load` contract (`crates/uor-r4-discrete/src/corpus.rs:129-215`).
- **Lineage base:** none — `distill-d1` is a new corpus identity and never inherits `mw-balp`'s identity, panels or splits.

## Data split

| Split | Share | Purpose |
|---|---:|---|
| train | ≈ 90% | student training sheets (their own FIT) |
| dev | ≈ 5% | open development evaluation; ≥ 250,000 eligible targets for power |
| held-out | ≈ 5% | **sealed**; opened only by a registered final evaluation |

Assignment is by document hash with a fixed salt, bound in the manifest; no document crosses splits. The sealed slice is never read during preparation, generation-gate runs or development.

## 1. Corpus specification

### 1.1 Size

| Quantity | Value |
|---|---|
| Target | **30,000,000** student tokens |
| Floor | 20,000,000 |
| Ceiling | 50,000,000 |
| Document cap | ≤ **200** student tokens each |
| Expected documents at target | ≈ 150,000 (mean ≈ 190 tokens, including short turns) |

**Why 30M.** The discrete driver consumes 64 rows per batch, and the accepted K16 timing record puts a batch at 130.24 ms proposal + 0.47 ms carry (`docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md:25-31`). That is **489.7 target tokens/s**, so 30M is ~17.0 h per epoch and 50M is ~28.4 h. The ceiling is set by what the student can actually consume, not by what the donor can produce. 1B would be ~567 h/epoch and is out of scope until the occupancy work in §11 lands.

### 1.2 Mixture

| Slice | Share | Tokens at target | One-line justification |
|---|---:|---:|---|
| Conversation (multi-turn) | 35% | 10.5M | The student is a chat model; short alternating turns are the highest-frequency pattern at 192–512 context |
| QA / instruction | 20% | 6.0M | Short, self-contained, high signal per token, and cheap to verify |
| Code (small functions + tests) | 20% | 6.0M | Teaches structure, exact copying and bracket/indent regularity; execution gives a deterministic filter |
| Reasoning prose (short CoT) | 15% | 4.5M | Wanted by the objective, but the riskiest slice at this scale, so capped below code |
| Exposition / general prose | 10% | 3.0M | Fluency anchor; prevents a dialogue-only register collapse |

Shares are of student tokens after tokenization, measured on the kept set. A slice that fails its quality gate may be dropped; the remaining shares then renormalise and the actual shares are recorded, not silently rounded.

### 1.3 Document shape

- **≤200 student tokens**, including role prefixes, split at turn boundaries — never mid-turn and never mid-code-block.
- The current operating context is **C=192** (`Spec { vocabulary: 4096, blocks: 4, context: 192, kept_cells: 8 }`, `docs/harness/prep/m1-learned-attention-link-2026-10-17.md:74`), and the crate's hard bound is `MAX_CONTEXT = 243` (`crates/uor-r4-discrete/src/metadata.rs:5`). A 200-token document plus BOS fits C=192 and fits C=512 without re-cutting.
- The **planned C=512 upgrade requires raising `MAX_CONTEXT`** in source. This sheet does not depend on that change and does not authorize it; the 200-token cap is chosen so the corpus is usable at both widths whichever lands first.
- Documents shorter than 16 student tokens are dropped (they teach a prefix, not a continuation).

### 1.4 Prompt set

- Fixed **before** the first generated token, written to `prompts/prompts.jsonl.zst`, and hash-bound in the receipt.
- Drawn from repo-held seed prompts plus public instruction sets whose licences permit derivative training data (per-slice SPDX recorded). No prompt may be authored from a held-out panel answer.
- **Pre-screened** against `heldout-200-a.json`, `heldout-200-b.json` and every sealed panel using the same 8-word-window / 4–7-word-full-turn rule as §3.2; a prompt that matches is removed before generation, and the removal is recorded.
- Prompt IDs are stable and appear in every document's `meta.prompt_id`, so a document can be traced to its prompt without storing the prompt twice.

## 2. Donor binding

| Item | Binding |
|---|---|
| Donor | `openai/gpt-oss-120b` |
| Revision | pinned commit SHA (recorded at dispatch; not a branch or `main`) |
| Weights | released **MXFP4** files; per-file SHA-256 and a combined weights SHA-256 |
| Size | 116.8B total / 5.1B active (`huggingface.co/api/models/openai/gpt-oss-120b`) |
| Licence | Apache-2.0 (`huggingface.co/openai/gpt-oss-120b`: "Permissive Apache 2.0 license") |
| Engine | vLLM, version pinned and container **image digest** pinned; no runtime compilation of our code |
| Hardware | **one declared pod: 1× RTX PRO 6000, 96 GB**, via `uor-pod`, under the ≤ $8/h cap |
| Execution | local weights only; no API, no hosted endpoint, no hidden provider (D41 clause 1) |
| Sampling | temperature 0.7, top_p 0.95, top_k 20, max_tokens 512, `reasoning_effort: medium`, one seed per shard |
| Isolation | donor runs in its own container; no donor code, weights or Python enters the repository or any project artifact (D29) |

Sampling parameters are fixed at registration and are **not** tuned on generated outputs. Tuning on outputs would be a design decision made on the corpus, which is exactly what the pre-registration forbids.

### 2.1 Generation receipt — `uor-r4.donor-corpus/1`

New schema, modelled on the existing `FilePin` convention (`crates/uor-r4-discrete/src/corpus.rs:24-48`). Five blocks, all mandatory:

| Block | Fields |
|---|---|
| **Donor pin** | repo, revision, weights SHA-256 (per file + combined), quantization, total/active params, licence SPDX |
| **Generation pin** | prompt-set SHA-256, slice map, temperature, top_p, top_k, max_tokens, reasoning effort, per-shard seed, shard plan |
| **Run pin** | pod id, provider, GPU type and count, lease id, rate $/GPU/h, UTC start and end, engine version, image digest, driver/CUDA version |
| **Content pin** | per-shard: path, SHA-256, document count, student-token count; then document-count total, token total, and a **Merkle root over shard SHA-256 leaves** |
| **Readback** | destination store path, per-shard SHA-256 re-read after transfer, recomputed Merkle root, MD5/index record, verification timestamp and operator |

A receipt missing any field is an instrument fault (D31), not a corpus defect. The Merkle root is the seal identity: it is what the student-side manifest cites and what D41 clause 6 expires against.

## 3. Quality gates

Every gate has an exact pass rule. Gates run **before** the corpus is sealed; a gate that cannot run is `INSTRUMENT_UNREACHABLE` (D31) and the corpus is not sealed.

### 3.1 Provenance

| ID | Gate | PASS iff |
|---|---|---|
| **G1** | Donor bind | every weight file SHA-256 and the revision equal the registered values |
| **G2** | Receipt completeness | all five blocks present; no null, no empty field |
| **G3** | Readback | per-shard SHA-256 on the pod equals the SHA after transfer, and the recomputed Merkle root equals the recorded root |

### 3.2 Dedup and contamination

| ID | Gate | PASS iff |
|---|---|---|
| **G4** | Exact dedup | 0 duplicate document SHA-256 in the kept set |
| **G5** | Near dedup | residual near-duplicate rate **< 3.0%**. Rule: MinHash over 5-word shingles, Jaccard ≥ 0.8, one document kept per cluster |
| **G6** | Panel contamination | **0** kept documents match `heldout-200-a`, `heldout-200-b` or any sealed panel. Rule: any 8-word window equal to an 8-word window of a panel user turn, or containment of a 4–7-word panel turn as a whole word sequence (the `PanelExclusion` rule, `crates/uor-r4-training/src/bin/prepare-knowledge-corpus.rs:1-30`) |
| **G7** | Train overlap | **0** kept documents share a 12-word sequence with `mw-balp` TRAIN; matches are dropped before counting |

### 3.3 Slice correctness

| ID | Gate | PASS iff |
|---|---|---|
| **G8** | Code slice | **100%** of kept code samples execute and pass their tests; 0 kept samples unevaluated. Sandbox: disposable container on the pod, no network, read-only root, 10 s and 1 GiB limits per sample |
| **G9** | Math slice | **100%** of kept math samples pass a deterministic answer check; 0 unevaluated. Only problems with a machine-checkable final answer enter this slice |
| **G10** | Spot audit | per slice, 200 random documents: **malformed ≤ 2.0%** and **refusals ≤ 5.0%**. Fails → reject that slice. One re-draw of the slice is allowed; a second failure stops for owner direction |

Malformed = schema invalid, fails byte round-trip, wrong role order, or contains a template artefact. Refusal = the donor declines, or returns an empty/safety-only response.

### 3.4 Distribution

| ID | Gate | PASS iff |
|---|---|---|
| **G11a** | Slice shares | each kept share within **±3.0 pp** of §1.2 |
| **G11b** | Truncation | **< 1.0%** of kept documents end without a terminal finish reason (i.e. hit `max_tokens`) |
| **G11c** | Repetition | **< 2.0%** of kept documents have an 8-word n-gram repeated more than 3 times |
| **G11d** | Length | median kept document length in **40–120** student tokens; p99 ≤ 200 |

### 3.5 Consumption

| ID | Gate | PASS iff |
|---|---|---|
| **G12** | UORT admission | `PreparedCorpus::load` succeeds; vocabulary exactly 4096; first token is BOS 0; mask binary and exactly payload length; eligible-target count equals the manifest (`crates/uor-r4-discrete/src/corpus.rs:129-215`) |
| **G13** | Window admission | `batch(0, 192, 64)` returns 64 valid windows with in-document predecessors, and the same call at context 512 succeeds once `MAX_CONTEXT` is raised |
| **G14** | Seal | receipt, manifest and Merkle root written; store readback re-verified; the D41 exception is recorded **expired** at seal |

## 4. Pilot design

**The pilot is the first tranche of the same pass, not a separate generation.** Same donor, same prompt set, same sampling, same receipt; its documents are retained and enter `distill-d1` if they pass §3. D41 clause 6 stays intact: one pass, one corpus.

| Item | Value |
|---|---|
| Size | ~1,000,000 student tokens (~5,000 documents) |
| Split across slices | the §1.2 shares, at 1/30 scale |
| Cost | ≈ 42 min of generation at 400 tok/s + setup ≈ **$2** |
| Purpose | prove the pipeline end to end before spending ~21 GPU-hours |

### 4.1 What the pilot must prove

1. **Generation** runs on the declared pod at a **measured** tok/s, recorded per shard.
2. **Receipt** `uor-r4.donor-corpus/1` is emitted with all five blocks and a Merkle root.
3. **Round-trip readback** from the pod to the private store, SHA for SHA, with the MD5/index record.
4. **Rust tokenization** produces UORT from donor text with the pinned 4096-vocab tokenizer; no Python and no float.
5. **`PreparedCorpus::load`** accepts the artifact and reports the manifest's own counts.
6. **Window admission** at C=192 (and at C=512 once available).
7. **Gates G4–G11** run at pilot scale and report numbers, even where a rate is too small to be decisive.
8. **Cost accounting** is complete: pod hours, rate, storage, transfer.

### 4.2 Go / no-go for production

Proceed to the 30M tranche **iff all of**:

- P1–P8 pass;
- measured tok/s ≥ **300** sustained (30M ≤ 28 h);
- the projected all-in cost fits the owner-approved costed plan and the ≤ $8/h cap;
- no unresolved licence or redistribution question.

If measured tok/s is **< 250**, stop and re-plan size or donor with the owner; do not silently shrink the corpus below the 20M floor. If a **quality** gate fails at pilot scale, the corpus is rejected and a new pass needs a new owner decision (D41 clause 6).

### 4.3 Repair vs regeneration

- **Repair** (same prompt set, same sampling, same donor, same slice design): re-emit a corrupt or incomplete shard, record it in the receipt, re-verify G3. Permitted.
- **Regeneration** (any change to prompt set, sampling parameters, donor revision, quantization or slice design): a new pass, requiring a new owner decision. Not permitted inside this sheet.

## 5. Student consumption contract

### 5.1 What the student reads

| Item | Contract |
|---|---|
| Format | UORT v1, 64-byte header, `u16` tokens, vocabulary **exactly 4096**, BOS 0 |
| Mask | `response_mask.u8`, binary, exactly payload length; eligible positions are `mask==1`, non-BOS, with an in-document predecessor |
| Splits | train ≈ 90%, dev ≈ 5%, held-out (sealed) ≈ 5%, split by document hash with a fixed salt; assignment bound in the manifest |
| Lineage | new corpus identity `distill-d1`, with `tokens_sha256`, mask SHA, document count, token count, eligible targets, tokenizer pin and Merkle root. It never inherits `mw-balp`'s identity |
| Tokenizer | `d36d3e87…` for the pilot and the production corpus. A different tokenizer would be a different corpus |
| Dev slice | ≥ 250,000 eligible targets, so later training sheets have power without touching the sealed slice |

### 5.2 Teacher signal — hard targets only

- `CorpusBatch` carries exactly one target per scored row: `targets: Vec<u32>` (`crates/uor-r4-discrete/src/corpus.rs:113-118`). There is no field for a distribution, a weight or a rank.
- **No logits, no probabilities, no softmax-derived quantities.** Any consumption of a distribution requires normalisation (D23) or a new learning law; both are excluded. A donor probability array over ~200k donor tokens is also physically absurd to store at 30M positions.
- **No rank lists are consumed.** They may be stored as an evidence sidecar `ranks/*.jsonl.zst` (donor token IDs, ranks, decoded strings), hash-bound, and they are **never opened by training**. Two reasons: UORT has no rank field, and the donor vocabulary is many-to-one into 4,096, so donor top-k cannot be aligned to student tokens without a segmentation map that does not exist.
- What transfers is **behaviour**: format, register, instruction adherence, correctness and selection — not a distribution.
- Student rules unchanged: D32 discrete integer path, D11 serving, D23 no softmax, D25/D30 no matmul, D29 module scan. **D41 clause 4 grants no student-side exception.**

### 5.3 Where preparation runs

Tokenization and UORT conversion are CPU work but a **big** CPU job (30M tokens, per-document round-trip decode). It runs on the pod, not the laptop (D30 bullet 3: no big CPU runs locally; D24 applies to model compute only). Bounded `RAYON_NUM_THREADS`, bounded RAM, output order independent of thread count (`prepare-knowledge-corpus.rs` header).

## 6. Cost and minimum test power

**Minimum test power.** The pilot (1M tokens, ~5,000 documents) is the minimum generation that exercises every gate end to end with reportable rates. The production floor (20M tokens) is the minimum corpus that gives the dev slice ≥ 250,000 eligible targets at the §5.1 split, which later training sheets need for a powered comparison. Sizes below the floor are not a corpus; they are a pilot.

**These are assumption-based planning figures, not a quote, not a cap and not a measured cost.** The release packet records measured pod hours, the actual rate and all-in spend.

| Item | Assumption | Estimate |
|---|---|---:|
| Pilot, 1M tokens | 400 tok/s, $2.50/GPU/h, 42 min + setup | **~$2** |
| Production, 30M tokens | 400 tok/s → 20.8 h, $2.50/GPU/h | **$52** |
| Production + 20% retries/overhead | repairs, re-draws, audits | **$63** |
| 50M ceiling | 34.7 h | ~$87 |
| Fallback: 30M on Qwen3-30B-A3B / 1× 5090 | 550 tok/s → 15.2 h at $0.99/h, only if the owner redirects the donor | ~$15 |
| Student consumption, 3 epochs | 3 × 17.0 h at $0.99/h, current 489.7 tok/s | ~$50 |
| Student consumption after the occupancy fix | unknown until measured | lower, not projected |

Rate basis: the documented 5090 rate is $0.99/GPU/h (`docs/labs/compute.md:225`); the PRO 6000 has been observed at ≈$3.16/h for one card (`docs/labs/context-curve-2026-10-11/README.md:4`) and ≈$2.39/GPU/h on a 2× pod (`docs/labs/vsa-native-test-2026-10-09/README.md:19`). $2.50/GPU/h is a planning midpoint. The ≤ $8/h cap and ≤ 4 running pods are hard (`docs/labs/compute.md:364`).

**Charged cumulatively.** Preparation, pilot, production, audits, repairs and resumes all count against this plan. A self-set hour or step budget is an estimate, not a stop rule: a healthy run is renewed and the extension is recorded.

## 7. Steps

1. **FIT review** — a fresh-context adversarial reviewer subagent, plus one non-author seat; every finding posted and applied. Sheet SHA frozen after FIT.
2. **Owner costed-plan approval** — the §6 figures, the 30M target and the shard plan.
3. **PM dispatch** with the registration checklist (§8) bound.
4. **Casey's typed D40 release** at dispatch, typed personally with `UOR_LAB` unset. No lab types it (D40 §2); the lab stages the command and verifies the resulting record.
5. **Pilot** (§4), 1M tokens, gates P1–P8, measured tok/s.
6. **Measured sizing** — 20M / 30M / 50M chosen from the measured rate and the approved budget.
7. **Production tranche**, same pass, same pins.
8. **Gates G1–G14**, seal, readback, exception expiry recorded.

**Dependency note.** The owner's sequence places the driver math optimization **before** the pilot. This sheet can be FIT'd and merged before that work lands; steps 4–8 wait for the optimization rung's measured result. The optimization is not a gate on corpus *design*, only on *spend*.

## 8. Registration checklist — bound at dispatch

| # | Item |
|---|---|
| 1 | This sheet's SHA-256, post-FIT |
| 2 | D41 record reference and the owner's ratification id |
| 3 | Donor repo, revision commit, per-file and combined weight SHA-256, quantization |
| 4 | vLLM version and container image digest; driver and CUDA version |
| 5 | Prompt-set SHA-256, slice map, source licences per slice |
| 6 | Sampling: temperature, top_p, top_k, max_tokens, reasoning effort, per-shard seed |
| 7 | Student tokenizer full SHA-256 and vocabulary |
| 8 | Pod: provider, GPU type and count, lease id, rate $/GPU/h, cap check |
| 9 | Budget: token target, shard plan, max hours, max dollars |
| 10 | Store destination, readback method, MD5/index path |
| 11 | Gate table G1–G14 with thresholds |
| 12 | PM dispatch id and the D40 release record id |
| 13 | Student-side target: UORT lineage path and the pointer to the separate training sheet |

## 9. Stop rule

- Any **G** or **P** failure stops for diagnosis (D31). It is never a mechanism negative — there is no mechanism here.
- An **instrument fault** (missing receipt field, SHA mismatch, unreadable store, failed readback) is repaired and re-run; it does not consume the exception.
- A **corpus defect** (dedup, contamination, slice audit, distribution) rejects the corpus. A rejected corpus consumes the pass; a new pass needs a new owner decision (D41 clause 6).
- A **throughput shortfall** (< 250 tok/s) is re-planned with the owner, not absorbed by shrinking below the 20M floor.
- Owner caps (≤ 4 pods, ≤ $8/h), the 30 GiB host disk floor and the pod disk floor are hard stops. A healthy run is renewed, not cancelled.

## 10. What this sheet can and cannot decide

**Can decide:** that a corpus meeting §1–§3 exists, with bound provenance and a verified seal; and that the student pipeline consumes it without format or schema error.

**Cannot decide:**
- whether distillation improves the student — needs a training sheet with a matched control (same token budget and steps on `mw-balp` vs `distill-d1`) and sealed panels;
- which donor is better — no donor comparison is run, and running one would be a D28 comparator unless separately authorised;
- anything about the donor's capabilities — the donor is never evaluated;
- that the corpus is "clean" in an absolute sense — the gates bound dedup, contamination and format, not truth. Donor errors survive the filters and are expected.

**Named follow-ups, each its own sheet:** the distillation training/control sheet; a rank-list learning law (would need a new objective and a cross-tokenizer alignment); the occupancy/throughput work in §11; a C=512 `MAX_CONTEXT` raise.

## 11. Why the corpus is sized to the student, not the donor

The measured limiter is on the consuming side: 64 rows per 130.7 ms batch = **489.7 target tokens/s** (`docs/harness/prep/m1-batched-verified-coordinates-DRAFT-2026-10-18.md:25-31`). The heavy evaluation launch uses `rows × 16 = 1,024` threads in blocks of 128 — 8 blocks on a 170-SM GPU, **~4.7% occupancy**, with each thread running the full serial writer (`docs/integration/current-state.md:210`; `crates/uor-r4-discrete/src/device.rs:259-268`). Until that changes, a corpus larger than ~50M buys storage, not learning. BVG-K's 16.25× (`docs/integration/current-state.md:90`) is **updates per token**, not tokens per second, and Stage C2/T are still open — so it is not used to justify a larger corpus here.

---

TEST FITNESS: PENDING. This draft is submitted for fresh-context adversarial review and one non-author seat. Registration requires: every finding applied and posted; the §8 checklist bound; the owner's costed-plan approval; then PM dispatch and Casey's typed D40 release. **No pod is rented and no token is generated by this sheet alone.** The D41 exception is unspent until the release is typed, and it expires at the corpus seal.

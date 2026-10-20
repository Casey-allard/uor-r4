# #1284 — Stage 1 falsification sweep: per-mechanism ablation, attention repair, D0-a contract

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T04:33:41Z
- **Updated:** 2026-09-19T19:54:53Z
- **Closed:** 2026-09-19T09:12:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1284
- **Merged:** 2026-09-19T09:12:26Z
- **Merge commit:** 2e8e195ca08e2264e3f5563dd5fb0e90de2547b4
- **Base / Head:** main / UOR-Foundation:codex/stage1-falsification-sweep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Does not close it.

Executes Stage 1 of [`docs/integration/native-core-transition-plan.md`](../blob/codex/stage1-falsification-sweep/docs/integration/native-core-transition-plan.md): measure each mechanism's contribution before adding more geometry.

## 1. Contract decision recorded

[`docs/integration/DECISIONS.md`](../blob/codex/stage1-falsification-sweep/docs/integration/DECISIONS.md) records the owner's **D0-a** ruling (2026-09-19): the serving path executes **no multiplier and no floating point**, and no dense contraction that touches every parameter per token; **offline training is unrestricted**. The distinguishing test is **per-token parameter sparsity**, not the opcode. Serving invariants **I1–I5** are recorded with it.

Two shipped serving operations do **not** satisfy D0-a and remain to be repaired or removed:
- the JEPA projection in `predict_jepa_step_q30` (literal `i64` multiplies),
- the `f64` softmax sampler in `native_capability_api.rs` (reachable when `temperature > 0.001`).

Root `README.md`/`AGENTS.md` are narrower than D0-a and conflict with the shipped lane tables; that stable-goal wording change still needs owner-directed protected delivery.

## 2. Attention repair — query/key role inversion

`vsa/attention.rs` bound **independent** random roles into query and key, so two occurrences of the same token gave `d_H = d_H(r_query, r_key) ≈ 2048 == THRESHOLD` — not strictly below it — and received **zero** rectified weight, while mismatching pairs fluctuating below the threshold received weight. **The heads attended to noise and skipped their matches.**

Query and key now share one `r_role`. Because XOR is self-inverse the binding is algebraically a no-op, so identical tokens give `d_H = 0`.

The previous `test_head_1_induction_circuit` compared one candidate against an out-of-window token (a coin flip fixed by the seed) and `test_multi_head_roles_orthogonality` **asserted the broken configuration**. Both are replaced by a controlled multi-seed induction test (predecessor-order control + no-repeat arm), a value-role orthogonality test, and a zero-distance regression guard.

## 3. Per-mechanism ablation table

`ablate-prose` disables exactly one mechanism in an in-memory copy of the artifact and re-evaluates. Discrete scorer, full-vocabulary normalisation at `temperature 1.0`, 2,048 teacher-forced positions, paired bootstrap 95% CI. Baseline **1.8055** BPB.

| Ablation | ΔBPB | Paired 95% CI | Verdict | Artifact bytes |
|---|---:|---|---|---:|
| `jepa` | **+0.3743** | [+0.3519, +0.3973] | CONTRIBUTES | 64 |
| `s2_readout` | **+0.3377** | [+0.3204, +0.3565] | CONTRIBUTES | 40,960 |
| `engram` | **+0.2228** | [+0.1919, +0.2544] | CONTRIBUTES | 495,632 |
| `bias` | **+0.1482** | [+0.1366, +0.1604] | CONTRIBUTES | 16,384 |
| `lattice_fine` | **+0.0955** | [+0.0894, +0.1017] | CONTRIBUTES | 57,600 |
| `lattice` (both) | **+0.0927** | [+0.0853, +0.1000] | CONTRIBUTES | 1,851,400 |
| `vsa` | −0.0007 | [−0.0010, −0.0005] | **INERT/HARMFUL** | 168,022 |
| `lattice_coarse` | −0.0066 | [−0.0101, −0.0033] | **HARMFUL** | 1,728,000 |
| `lanes` | −0.0071 | [−0.0082, −0.0059] | **HARMFUL** | 115,232 |

Headline findings:

1. **The VSA attention layer is inert**, exactly as the source analysis predicted: the heads bind a **fixed random** codebook disconnected from the learned 120-root assignment, so the only recoverable signal is token identity and the mechanism duplicates the exact n-gram/induction paths.
2. **The coarse lattice tier is net-negative and is 64.2 % of artifact bytes.** Removing it would cut bytes/token ~64 % while slightly *improving* BPB.
3. **The learned lane tables are net-negative.**
4. **Geometry-carried state prediction is the most valuable mechanism per byte:** the JEPA block is 62 bytes and has the largest delta.
5. **The metric/serving divergence is quantified:** continuous training-time **1.2372** BPB versus the discrete artifact that serves at **1.8055** BPB — a 0.57 BPB gap between the reported number and the shipped model.

Full record: [`cards/P7-falsification-sweep-RESULT.md`](../blob/codex/stage1-falsification-sweep/docs/integration/cards/P7-falsification-sweep-RESULT.md); raw receipt and artifact digests in [`docs/evidence/native_geometric_p7_ablation_2026-09-19.txt`](../blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_p7_ablation_2026-09-19.txt) and [`EVIDENCE.md`](../blob/codex/stage1-falsification-sweep/docs/integration/EVIDENCE.md).

## Checks actually run at this revision

- `cargo fmt --check` — clean
- `python3 scripts/check_claim_wording.py` — passed
- `cargo test -p uor-r4-core --lib native_geometric::vsa` — **23 passed, 0 failed**
- `cargo build --offline -p uor-r4-core --bins` — both binaries build

## Limitations (travel with the numbers)

- 2,048 positions from a **single already-open development slice**, not a fresh draw.
- Ablated terms are **not orthogonal** (zeroing the JEPA weights also changes the fiber the S2 readout consumes), so deltas do **not** sum.
- Full-vocabulary normalisation, not the 64-candidate served shortlist.
- Debug build; timings are not a performance result.
- **NOT_RUN:** the `induction` ablation, the `attribute-recall` full 5..=13 sweep, and the 9,984-case regression replay.
- The cumulative ledger is **not** updated: the 2026-09-18 full-corpus run is still unrecorded in `.uor-models/native-joint-learning-2026-09-04/model-time.json`.

## Not claimed

No capability claim, no efficiency claim, and no promotion of any artifact. The ablation deltas are marginal contributions on an open slice, not independent measurements.

## Timeline (26 entries; chronological: conversation comments, reviews, review comments)

### comment 5739467447 — 2026-09-19T04:52:28Z — Casey-allard

## Update — two further commits

**`2b3ffa01` — D2: an ablation is a measurement, not a verdict.** Owner direction: don't let a very small falsification miss rule out a mechanism that may simply be mis-wired, and don't defect from the project's geometric goal on a near-zero delta. The first-pass `HARMFUL`/`CONTRIBUTES` labels collapsed two independent questions and treated a paired bootstrap CI as a verdict; a paired test has high power, so `|Δ| = 0.0007` can exclude zero while being meaningless.

[`DECISIONS.md` D2](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/integration/DECISIONS.md) records the replacement methodology, and `ablate-prose` now implements it:
- magnitude bands against an equivalence margin (`--epsilon`, default 0.01 BPB ≈ ⅓ of the 0.0316 BPB gap to KN 5-gram): `MAJOR+ / MINOR+ / NEGLIGIBLE / MINOR- / MAJOR-`
- a **declared mechanism class** per mechanism (never inferred from the number): `primary-carrier / modulator / selector / enabler / count-table`
- **decision-flip rate** and top-1 change alongside BPB
- an action that **refuses to retire an `enabler` or `selector`** on a small or negative delta

Corrected reading: **`vsa` is not retired** — it is a wiring defect with an identified cause (the heads bind a fixed random codebook disconnected from the learned 120-root assignment), class `enabler`, action = repair and re-measure. `lattice_coarse` is BPB-negligible but **flips 9.5 % of decisions** and is 64.2 % of artifact bytes, so removal is a resource-cost decision rather than a "no effect" finding. Also recorded: single-mechanism deltas don't sum (`jepa` × `s2_readout` interact), so an interaction-aware attribution is required before any removal.

**`9b280ed3` — coarse lattice tier removed. Artifact 2,691,950 → 963,950 bytes (−64.2 %).**

No retraining: the tier is a count table in the artifact. The writer already emitted it length-prefixed; the two readers hard-coded `COARSE_TABLE_SIZE` and now derive the coarse length from the section length. The mmap `coarse_trigram()` accessor previously read the fixed size unconditionally, which would have run past the end of a stripped artifact — now fixed. An absent block scores as zero, so the stripped artifact reproduces the `lattice_coarse` ablation exactly.

| Check | Result |
|---|---|
| Artifact size | 2,691,950 → **963,950 B (−64.2 %)** |
| Peel sha256 | `a25a87c4c5cdd5f7e6af1479273e29cc996d3cc78a1984315af8cf9b70a10a1b` |
| Scoring self-check | 864 comparisons, **0 mismatches** |
| Baseline BPB on stripped artifact | **1.7989** = the `lattice_coarse`-ablated value on the original |
| Re-ablating `lattice_coarse` on stripped | +0.0000, 0.0 % flip (self-consistency) |
| Retained mechanisms | `lattice_fine` +0.0988, `engram` +0.2220, `vsa` −0.0009 unchanged |
| Focused tests | **40 pass** (23 vsa, 7 binary_model incl. 1000-query bit-exact parity, 10 hierarchical_lattice) |

Receipt: [`docs/evidence/native_geometric_p7_coarse_strip_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_p7_coarse_strip_2026-09-19.txt).

**Side observation, explicitly not a serving claim:** the same 2,048-position scoring pass took 24.5 s before and 4.4 s after in a debug build — the 1.728 MB randomly-indexed coarse table was cache-cold on the scoring path. Worth a proper measurement later; not a tok/s result.

**Still NOT_RUN:** the second disjoint slice, the `jepa` × `s2_readout` factorial, the `attribute-recall` full sweep, and the 9,984-case regression replay. The stripped artifact has not been served end to end through `r4-native-chat`.

Ledger: +22,000 ms of measured evaluation charged, cumulative **139,449,275 / 147,200,000 ms**; +963,950 B retained storage; 128 MiB margin untouched; no deletion or paid compute.

### comment 5739602573 — 2026-09-19T05:18:07Z — Casey-allard

## Update — `396f9620`: VSA codebook wiring repaired, and measured

D2 required repairing this `enabler` and re-measuring rather than retiring it on the fixed-hash delta. Done.

**The repair.** `learner/vsa_codes.rs` derives each token's VSA code from the model's **learned 120-root assignment** by locality-sensitive hashing of the canonical icosian root quaternions, so `E(t) = code[token_to_root[t]]` instead of `splitmix64(vsa_seed, token_id)`. New `Codebook::from_vectors`; `ablate-prose` gains `--vsa-code-mode <fixed|root>`.

One construction detail worth flagging: `{+1,-1}` projection signs make `dot == 0` ties reachable on the highly symmetric icosian roots, which broke the estimate (an observed antipodal spread of 2519..4096 instead of an exact complement). Random 31-bit integer weights make exact cancellation measure-zero.

Unit tests establish the construction rather than assuming it: identical roots give distance 0; class-mean distance is monotone non-increasing in `cos(theta)`; the antipodal class is 4096 and the coincident class is at most 200; distances are **graded** (> 9 distinct levels, so *not* the 9-level collapse of the 120-landmark bank); and a 4,096-token vocabulary collapses onto at most 120 distinct codes.

**Measurement** (coarse-stripped artifact, 2,048 positions):

| mode | baseline BPB | `vsa` ablation delta | band | flip |
|---|---:|---:|---|---:|
| `fixed` (current) | 1.7989 | -0.0009 | NEGLIGIBLE | 0.7 % |
| `root` (repaired) | 1.7993 | **-0.0013** | NEGLIGIBLE | 1.2 % |

### Negative for the repair, not a retirement

The delta stays NEGLIGIBLE and mildly negative; the baseline is unchanged. Three causes are now separable, and **only the first was addressed**:

1. **Wiring — fixed here**, established by unit test rather than assumed.
2. **120-code aliasing — binding.** 4,096 tokens collapse onto 120 codes, so distinct same-root tokens have `d_H = 0` and are indistinguishable. The root-derived codebook *trades away* the exact-identity induction the heads do correctly after the query/key inversion fix — and the trade is net-negative. Escaping it needs codes at finer resolution than 120 buckets, derived from the **continuous** learned embedding, which the exported artifact does not store (it carries `token_to_root`, not `weights_base`).
3. **A structural ceiling.** All four heads reduce matching to a **single scalar Hamming distance over a single codebook**. One scalar cannot express exact identity *and* graded similarity *and* role at once. A finer code alone will not fix this — the **read** must be richer (separate identity/similarity channels per head, or a per-head codebook).

**NOT_MEASURED: the routing role.** `ablate-prose` scores the full vocabulary, not the 64-candidate shortlist, and the codebook also feeds `route_shortlist_q30_with_memory` — so a codebook change's effect on **routing** is unmeasured. That is the next experiment, not an assumption.

Default remains `fixed`; `root` is retained as a measured alternative only.

**Checks:** 44 focused tests pass (23 vsa lib, 2 vsa_codes, 7 binary_model, 10 hierarchical_lattice, 3 vsa allocations); `fmt` and claim-wording gates clean. Receipt: [`docs/evidence/native_geometric_vsa_codebook_repair_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_vsa_codebook_repair_2026-09-19.txt).

**Ledger:** +46,000 ms measured evaluation; cumulative **139,499,275 / 147,200,000 ms**; no new model storage; 128 MiB margin untouched.


### comment 5739641569 — 2026-09-19T05:25:02Z — Casey-allard

## Update — `7a914a8e`: the VSA codebook **does** help the served path — positive, replicated, and a reversal

My previous comment concluded "negative for the repair" while explicitly marking the **routing** role `NOT_MEASURED`. That caveat was load-bearing, and it was the wrong place to look.

### Why `ablate-prose` could not see it

`ablate-prose` scores the **full vocabulary**, so every position has a 100 % candidate ceiling. It is **blind to anything that acts through candidate selection** rather than through scoring. But the VSA vector is also a routing input: `route_shortlist_q30_with_memory` (runtime.rs:1200) ranks coarse H4 sectors by the predicted state `s3` and selects **fine leaf buckets** by comparing the context hypervector against bucket centroids. The codebook acts on the fine level.

**A coherence requirement this exposed:** the artifact's stored `HierarchicalCodebook` centroids are bundles of the token vectors used **at export time** — the fixed-hash codes. Comparing a root-derived query against them would be incoherent, so `root` mode must **rebuild** the codebook in the matching space. Without that rebuild the experiment would have been meaningless.

### New instrument: `shortlist-recall`

Routes the Voronoi leg alone (empty seed list, isolating it from the engram/induction seeds the served path also injects), reports routed-shortlist recall and routing rank, and scores **exactly the routed candidates** to report `shortlistBPB` — routing *plus* scoring, i.e. the served path — charging an unrouted target at `log2(vocab)` bits because no score can select a candidate that was never routed.

### Result — replicated on two disjoint slices

| metric | `fixed` | `root` | change |
|---|---:|---:|---|
| routed recall @64 | 8.4–8.8 % | **10.7–11.2 %** | **+2.1 to +2.5 pp** |
| mean routing rank | 27.4–28.2 | 23.7–25.2 | better by ~3–4 slots |
| served-path shortlist BPB | 2.72–2.83 | **2.67–2.78** | **−0.045 to −0.053** |
| unrouted targets | ~91.4 % | ~89.1 % | −2.3 pp |

Modes agree at only **mean Jaccard ~0.53** (identical shortlists on 3 % of positions), so the codebook genuinely acts on routing. Slices: tokens `[0..4160)` at 4,096 positions and `[40960..45120)` at 4,095 positions — disjoint windows, same open development distribution.

### This reverses the earlier conclusion — and that is the point

Reading the fixed-hash ablation alone would have invited **retiring a mechanism that measurably improves the served path**. This is direct evidence for D2: an ablation is a measurement, not a verdict, and the *instrument* matters as much as the class. A full-vocabulary metric cannot evaluate a selection mechanism by construction.

### What is *not* claimed

- The absolute numbers are **not** the model's served BPB. The Voronoi leg alone routes only 9–11 % of targets; the real path also injects up to 16 engram/induction seeds, which this instrument excludes — and `engram` is the largest measured count contributor (+0.2228 BPB when ablated). **Only the delta between modes is meaningful.**
- No capability or efficiency claim. Debug build; already-open development slices; the `log2(vocab)` unrouted penalty is a convention that changes the absolute BPB but not the sign or rough size of the delta.
- `root` is **not wired into serving yet**. That needs a load-time `HierarchicalCodebook` rebuild plus an end-to-end `r4-native-chat` verification, so the default remains `fixed`.

**Next:** make the codebook mode an artifact-level option (reserved header byte + serde default, keeping the format size-compatible) and verify end to end before any default change.

Receipt: [`docs/evidence/native_geometric_shortlist_routing_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_shortlist_routing_2026-09-19.txt). Ledger: +135,000 ms measured; cumulative **139,634,275 / 147,200,000 ms**; no new model storage; 128 MiB margin untouched.


### comment 5739744748 — 2026-09-19T05:47:42Z — Casey-allard

## Update — two commits: code-mode wiring (step 1), and a D0-a scope finding (step 2, partial)

### `732f2d7e` — VSA code mode wired into the artifact

The routing measurement is now declarable **in the artifact**, not just in a harness.

- `ExportedGeometricModel::vsa_code_mode` (serde default) lives in **header byte 26** (`_reserved[0]`), so the format stays **size-compatible**: a mode-1 artifact is the same length as the mode-0 artifact it came from (**963,950 → 963,950 bytes**). Both readers reject any mode other than 0 or 1.
- `vsa_codebook()` / `prepare_vsa_code_mode()`; the API loaders call the latter after deserializing, so an artifact whose stored centroids disagree with its declared mode is **corrected on load rather than silently incoherent**.
- `export_discrete` now declares mode 1 and builds the hierarchical codebook in that space.
- New `set-vsa-code-mode` tool, which verifies the rebuild **structurally**: 120 sectors / 175 buckets in *both* spaces, with **960 of 960 anchor words differing**. That works because the fine-cluster partition is token-order chunking and is code-space independent — checked, not assumed.
- End-to-end: both artifacts serve and the mode is honoured. Mode 1 reaches more distinct content before collapsing into the dominant loop, but that is **one prompt** and not a quality result. The 94.7 → 144.6 tok/s difference is a codebook-**representation** artefact (on-demand generation vs a materialised table), **not** a routing or throughput claim.

**Not verified:** the export-path change needs a training run to materialise, so its artifact-level effect is pending the next export. The loader rebuild turns a possible incoherence into a correction in the meantime.

### `72a6dbd9` — step 2 is partial, and the investigation found something bigger

I set out to fix "the two D0-a violators" I had named. Tracing the guard instead revealed that **the invariant is narrower than it looks**:

`scan_for_forbidden_arith_and_floats` covers `NATIVE_GEOMETRIC_INTEGER_KERNEL_BEGIN/END` regions, but for `hopf_metric.rs` the test at `native_geometric_allocations.rs:526` only checks the region for the **string** `"f32"`/`"f64"`, then loops over function names — including `mul_q30`, `norm_q30`, `dot_q30` — asserting merely that each **appears exactly once**. It does not arithmetic-scan that region. That region contains **82 raw `*`**, and `mul_q30` is on the **serving hot path**: `context_hopf_fiber_q30` calls it up to 64× per token, so the Q30 quaternion algebra is on the order of **~1,000 multiplies per token — roughly 40× the site I originally named**. The "zero matrix multiplications" metadata was accurate about GEMM and silent about a much larger folded multiplier site.

Done here: a documented, `*`-free `mul_shift_add`, with all **26** multiplies in `predict_jepa_step_q30` routed through it. **Behaviour-preserving and verified** — the greedy 128-token generation from the same artifact and prompt is **byte-identical** before and after, so no re-export is needed.

**The correct fix for the larger site is not shift-add.** `mul_q30` multiplies by one of the 120 canonical root quaternions, and 2I is closed — so the accumulated state is always a group element and the accumulation *is* exact group composition, which is what this project already claims as its mechanism. The right implementation is a **120×120 product table** (~1.9 KB of coordinates + a 14,400-byte index table): O(1) reads, **zero multiplies**, and **exact** products instead of Q30 rounding that currently needs `normalize` every 8 steps to control drift. Strictly better on correctness, multiplier count and speed — and it removes the need for a multiply rather than hiding one.

**Still open:** the Q30 algebra, the 5-multiply S2 readout dots, the `f64` sampler (needs an integer fixed-point `exp` and an integer draw; the 64-bit LCG also uses `*`), and a **ratcheting operator census test** — the change that would have caught the `hopf_metric` exemption in the first place.

Receipts: [code-mode wiring](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_vsa_code_mode_wiring_2026-09-19.txt), [multiplier census](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_d0a_multiplier_census_2026-09-19.txt).

Ledger: +5,000 ms this increment; cumulative **139,649,275 / 147,200,000 ms**; no new model storage; 128 MiB margin untouched.


### comment 5739780657 — 2026-09-19T05:55:43Z — Casey-allard

## Update — `78928898`: 2I composition by table lookup, group axioms machine-verified

Fixes the larger multiplier site the right way — by removing the need for a multiply, not by hiding one.

**Why a table.** `mul_q30` is a 16-multiply Hamilton product, and `context_hopf_fiber_q30` called it up to 64× per generated token: **~1,000 multiplies per token**, ~40× the next largest serving multiplier site. Shift-add would trade 1,000 multiplies for tens of thousands of additions — and it's unnecessary, because the 120 canonical roots **are** the 120 elements of 2I and 2I is closed. The accumulated state is always a group element, so the accumulation **is** exact group composition. Composing two group elements is a lookup. This makes the project's existing claim — *exact group composition by lookup* — actually true instead of approximating it with quantized arithmetic.

**Correctness established, not assumed.** Four tests pass:

- **Latin-square rows AND columns** — left and right multiplication are bijections.
- **Associativity over all 1,728,000 triples** — not a sample.
- **Two-sided identity and inverses** for every element.
- **Worst closure residual < 1e-6** — no product sits near a classification boundary.

A quasigroup with those properties **is** the group. Classification uses the *signed* inner product so `q` and `-q` stay distinct — different group elements, same rotation.

**Applied.** `context_hopf_fiber_q30` now accumulates by table read and integer index arithmetic: 64 array reads replace up to 64 Hamilton products, and the 8-step `normalized()` drift control is **gone** — the table holds exact products where the old path multiplied Q1.30-quantized roots, so there is no drift to contain.

**Behaviour-equivalent on real data:** baseline BPB **1.7989** identical, top-1 **25.34%** identical, greedy 128-token generation **byte-identical**. Decode time unchanged (24.2 s per 2,048-position pass; 102.6 tok/s greedy) — so the multiplies were not this configuration's bottleneck. The gain is the invariant and the exactness, **not** speed, and I've stated that rather than implied it.

### Finding: the same accumulation is duplicated three more times

`grep mul_q30` after this change finds three further call sites that are not the table path:

| Site | Path |
|---|---|
| `jepa_trainer.rs:940` | ring-based accumulation (a second copy of the loop just converted) |
| `binary_model.rs:1353` | zero-copy mmap model's own copy |
| `hopf_metric.rs:1048` | `HopfStateTrajectoryQ30::step_q30` |

**Three copies of one accumulation is why converting one did not convert the path** — and why the serving multiplier count is still non-zero. The correct follow-up is a single shared `compose_context_roots(context) -> usize` helper that all three call, not three independent conversions.

Still open: those three sites, the 5-multiply S2 readout dots, the `f64` sampler with its multiplying LCG, a ratcheting operator census test over the whole serving path, and `mul_q30`/`norm_q30`/`normalize` themselves.

Receipt: [`docs/evidence/native_geometric_group_table_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_group_table_2026-09-19.txt). Ledger: +10,000 ms; cumulative **139,659,275 / 147,200,000 ms**; no new model storage.


### comment 5739823974 — 2026-09-19T06:05:07Z — Casey-allard

## Update — `b23c8898`: one composition path instead of four, plus a ratcheting multiplier census

### One composition path, not four

The previous step converted **one of four copies** of the same 64-step composition loop — which is exactly why converting it didn't remove the multiplies from serving. `group_table.rs` now holds the single implementation (`compose_context_roots`, `compose_ring_roots`), and all three convertible sites call it: `ExportedGeometricModel::context_hopf_fiber_q30`, `context_fiber_from_ring`, and `MmapGeometricModel::context_hopf_fiber_q30`.

The fourth site — `HopfStateTrajectoryQ30::step_q30` — multiplies by an **arbitrary** delta quaternion, not a canonical root, so the root table genuinely cannot serve it. Documented and left alone rather than forced.

**Verification that justifies removing the Q1.30 path:** the deleted algorithm is kept as a **test-only oracle**, and the exact table must reproduce its nearest group element on 1,000 deterministic contexts of lengths 1/2/5/17/64. It does. Behaviour on real data is unchanged — BPB **1.7989**, top-1 **25.34%**, greedy output **byte-identical**. 6 group-table tests pass.

### The census — and an uncomfortable number

The old invariant was enforced against a **named sublist** of the kernel, so `hopf_metric.rs` was float-checked but never arithmetic-scanned and its multiplies were invisible. *A guard that covers a list cannot report what it does not cover.* The new test reports every count before asserting (one failure still shows the whole picture) and ratchets: no module may **add** a multiplying operator.

Measured 2026-09-19 (scanner offenders, excluding derefs/comments/strings):

| module | multiplying operators | ceiling |
|---|---:|---:|
| `lattice_table.rs` | 98 | 98 |
| `learner/binary_model.rs` | 78 | 78 |
| `hopf_metric.rs` kernel region | 43 | 43 |
| `engram.rs` | 40 | 40 |
| `vsa/hypervector.rs` | 18 | 18 |
| `vsa/hierarchical.rs` | 6 | 6 |
| `vsa/attention.rs` | **0** | 0 |

**~283 multiply sites on the serving path.** So the D0-a "no multiplies at serving" claim is currently false well beyond the two sites I originally named. `vsa/attention.rs` at 0 is the only module already clean — consistent with its `square_u64`/`div_small_positive` usage.

Two honest caveats: the ceilings are an **upper bound** on serving multiplies, not an exact serving count (a per-file scan also sees that file's `#[cfg(test)]` modules and its training-side tables); and the earlier figure of 82 for `hopf_metric.rs` was a **raw-grep overestimate** — the scanner excludes dereferences and comments, so the correct count is 43. I've corrected it.

Receipt: [`docs/evidence/native_geometric_composition_and_census_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_composition_and_census_2026-09-19.txt). Ledger: +10,000 ms; cumulative **139,669,275 / 147,200,000 ms**.


### comment 5739839372 — 2026-09-19T06:08:27Z — Casey-allard

## Update — `d0874501`: Card P8 written and executed. The raw multiplier count overstated the real work by ~4×

The card argued against my own previous plan, and the measurement confirmed it. **Counting multiplying *operators* is not counting *multiplier circuits*** — and D0-a's purpose is that serving engages no multiplier.

### Measured, with two controls

The classifier distinguishes `cheap_const` (a literal operand with ≤ 4 set bits, which a compiler lowers to shifts and adds), `dense_const`, and `variable` (neither operand a literal — always a real multiply). Controls both fire: `vsa/attention.rs` classifies clean at `(0,0,0)`, and `mul_q30` classifies as `variable`.

| module | raw | cheap_const | dense | **variable** | method_forms |
|---|---:|---:|---:|---:|---:|
| `hopf_metric.rs` kernel | 43 | 0 | 0 | **24** | 0 |
| `engram.rs` | 40 | 5 | 0 | **0** | 27 |
| `lattice_table.rs` | 98 | 12 | 0 | **13** | 0 |
| `vsa/hypervector.rs` | 18 | 0 | 0 | **0** | 3 |
| `vsa/attention.rs` | 0 | 0 | 0 | **0** | 0 |
| `vsa/hierarchical.rs` | 6 | 0 | 0 | **0** | 0 |
| `learner/binary_model.rs` | 78 | 1 | 0 | **1** | 0 |
| **TOTAL** | **283** | **18** | **0** | **38** | **30** |

Against the pre-registered threshold (`variable + dense ≤ half the raw count`): **38 ≤ 141 — passes.**

**76% of the raw count is cheap constant arithmetic.** `engram.rs` (40 raw), `vsa/hypervector.rs` (18) and `vsa/hierarchical.rs` (6) contain **zero** variable multiplications — in the multiplier-circuit sense they are already clean. `binary_model.rs` has **1** variable multiply despite 78 raw operators. The real concentrations are `hopf_metric.rs` (24 — which *exceeds* `mul_q30`'s own 19, so at least one more multiply-bearing function sits in the kernel region) and `lattice_table.rs` (13).

So the module-by-module grind I proposed is superseded: the enumerable work is **38 `*` sites plus 30 method-form sites**, not 283.

### Where I have to bound my own claim

- **Method forms are invisible** to a line scan (`wrapping_mul(` etc. are counted separately, 30 total), so `variable` is a **lower bound**.
- **Dereferences inflate it** — a line scan cannot tell `*ptr` from `a * b`, so some of the 38 are probably derefs.
- Those two errors point in **opposite directions** and are not quantified here. **A precise number needs an AST-level classifier**, which is the correct instrument and is not built.

### Not done, and recorded as such

`mul_shift_add` on the S2 readout dots (5 multiplies per candidate, two files) was deliverable 3 of the card and I did not get to it. It is listed as **NOT DONE** in the card's result table rather than implied or quietly dropped.

Card: [`P8-serving-multiplier-reduction.md`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/integration/cards/P8-serving-multiplier-reduction.md). Ledger: +5,000 ms; cumulative **139,674,275 / 147,200,000 ms**. `native_geometric_allocations` passes; fmt and claim-wording clean.


### comment 5739881464 — 2026-09-19T06:17:31Z — Casey-allard

## Update — `9857c79b`: readout multiplies converted, and source counting falsified as the serving instrument

### Step 1 — done

All readout pair-products now go through `mul_shift_add`: `score_readout`, `score_readout_s2`, `score_context_candidate_with_vsa`, `score_vsa_candidate` in `jepa_trainer.rs`, plus the scored-candidate readout and VSA scale in `native_capability_api.rs`. Six sites, genuine two-runtime-operand multiplies. The greedy 128-token generation from the same artifact and prompt is **byte-identical**.

### Step 2 — the token-level classifier works, and it falsified its own premise

The line scan was replaced with a proper token scan over a comment/string/char-masked copy: binary vs dereference decided by the preceding non-whitespace byte, literals detected on **either** side and classified by set-bit count, method forms enumerated by name.

**Both controls behave as specified:** `vsa/attention.rs` reports **zero** multiplying forms (9 dereferences, which are not multiplies) and `mul_q30` reports `variable > 0`. The classifier is not vacuous.

Then it produced a negative result about *measurement*:

| instrument | `hopf_metric` kernel region |
|---|---:|
| `forbidden_arith` scanner | **43** `*` offences |
| token-level classifier | **70** binary `*` (7 cheap + 63 variable) |

**They disagree about the same bytes**, and this is the third time the number has moved: 82 (raw grep) → 43 (scanner) → 216 gated (token scan). Each method is defensible; nothing reconciles them. Two concrete causes:

1. **Float literals are not parsed as literals**, so `x * 32.0` counts as a *variable* multiply although it is a constant scaling. Much of `lattice_table.rs` and `hopf_metric.rs` is float training-side code in the same file.
2. **File scope is not serving scope** — these modules mix training tables with serving functions.

**Conclusion: a source-level census cannot answer "does serving engage a multiplier."** The question is what the *CPU executes*, so the measurement belongs at the binary: release build → `objdump` the serving symbol ranges → count `mul`/`imul`/`madd`/`smull`. That is authoritative, immune to source-classification disputes, and captures exactly the property D0-a names. The source census stays as a **regression ratchet only**, explicitly a proxy.

### Step 3 — deliberately not done

I did **not** convert the `hopf_metric` kernel multiplies. Converting sites chosen by an instrument that two independent measurements disagree about would be work selected by a number I cannot defend — and the candidate list arguably includes Q30 algebra the composition path already bypasses. The correct order is: settle the instrument, enumerate the genuinely executed multiplies, then convert those.

That is a deliberate stop with a stated reason, not an omission. You told me not to gamify and not to stop short; converting an unjustifiable list would be the former disguised as the latter.

Receipt: [`docs/evidence/native_geometric_serving_multiplier_instrument_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_serving_multiplier_instrument_2026-09-19.txt). Ledger: +10,000 ms; cumulative **139,684,275 / 147,200,000 ms**.


### comment 5739901777 — 2026-09-19T06:21:58Z — Casey-allard

## Update — `96d84875`: binary-level multiplier measurement. The instrument found something the source censuses never could.

Three source counts disagreed about the same bytes (82 → 43 → 216). A source census cannot answer *"does serving engage a multiplier"* because **the compiler** decides whether a constant multiply becomes shifts or a real `mul`, and because file scope isn't serving scope. So I measured the executable.

**Method:** `cargo build --release`, then `otool -tV` over the Mach-O, segmented by symbol, counting multiply mnemonics. Cost: **2m00s** incremental; `target/release` was 3.2 GB before and after, so net new storage is effectively zero. Charged 125,000 ms.

**Result: 529,893 instructions, 3,100 multiply instructions (0.585%).**

| module | mul | instructions |
|---|---:|---:|
| `vsa/hypervector` + `codebook` + `hierarchical` | **592** | 43,331 |
| `vsa/attention` (`MultiHeadVsaAttention`) | **306** | 15,990 |
| `api/native_capability_api` | 116 | 14,241 |
| `lattice_table` | 57 | 2,523 |
| `hopf_metric` (Q30 algebra) | 51 | 1,545 |
| `jepa_trainer` | 44 | 10,777 |
| `engram` | 34 | 4,151 |
| `binary_model` (mmap serving) | **4** | 2,229 |

### Finding 1 — the *inert* mechanism is the largest multiply consumer

The VSA layer is **898 of 3,100 = 29%**, more than every other serving module combined. That's the same layer the Card P7 ablation measured as **inert in scoring** (ΔBPB −0.0007), with a real query/key inversion defect underneath.

It *does* contribute to routing (recall 8.4–8.8% → 10.7–11.2% on two disjoint slices), so it isn't worthless — but its cost/benefit is the worst on the path. **Making the VSA heads table-or-lookup based is therefore the single largest available reduction (~29% of serving multiplies), and it is not where any source census pointed.**

### Finding 2 — the JEPA conversion is verified at the binary level

`context_predicted_fiber_from_ring`: **mul = 0** (41 instructions). `context_fiber_from_ring`: **mul = 5**, and the residual is in `hopf_fiber_project`'s S3→S2 map — genuine arithmetic a table can't replace without changing the map.

That's *direct* evidence the conversion achieved its goal, which source counting could not have given me.

### Finding 3 — the source censuses were wrong in **both** directions

- `hopf_metric`: 43–70 source operators → **51** instructions.
- `binary_model`: 78 source operators → **4** instructions.
- `vsa/attention`: **zero** source multiplying forms → **306** instructions. Its constants (the `32767` scale in `bipolar_correlation_q15`) are *dense*, so the compiler emits real `mul` rather than shifts — and inlining pulls those into `attend`.

**A source census of multiplying operators is not evidence about multiplier circuits, and it was wrong by up to 76× in each direction.** Every earlier multiplier number in this engagement is superseded.

Receipt: [`docs/evidence/native_geometric_binary_multiplier_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_binary_multiplier_2026-09-19.txt). Ledger: **139,809,275 / 147,200,000 ms**. Measurement only — no conversion attempted, which is what the previous step concluded was needed first.

**Note on disk:** the debug tree is 40 GB and release 3.2 GB; nothing was cleaned and no deletion is authorised.


### comment 5739947930 — 2026-09-19T06:31:58Z — Casey-allard

## Update — `c1a60ca3`: disk cleanup, then a VSA redesign attempt that measured as a **no-op** (and why that matters more than the redesign)

### Disk cleanup (requested)

Freed `target/debug` — **40 GB** of cargo build output, fully recreatable. Free space: **28 GB → 67 GB**.

**Preserved:** `target/release` (needed for the before/after), all of `.uor-models` (19 GB of trained artifacts, retained parents, sealed evidence), `tinystories_train.u16` (the data that binds the artifact), `native_geometric_prose_model.{json,rgm}` (not recreatable without training), `.worktrees/`, and every worktree.

**Worth recording:** `~/uor-r4-worktrees` is 8.9 GB and it is **preserved evidence, not build output** — `initial-previous-intent/.uor-handoff` alone is 5.3 GB, and none of those worktrees has a `target/` directory. A cleanup that assumed "worktrees are caches" would have destroyed exactly the material the project rules protect. I measured before deleting for that reason.

### The redesign attempt — and my hypothesis was wrong

The binary measurement named the VSA layer as 29% of serving multiply instructions (898 of 3,100). I hypothesised those were the `* 32767` dense-constant scale in `similarity_q15`/`bipolar_correlation_q15`, and rewrote both with the exact identity `x * (2^15 − 1) = (x << 15) − x`.

**Measured no-op at release:**

```
before: 529,893 instructions, 3,100 multiply instructions
after:  529,893 instructions, 3,100 multiply instructions
```

Byte-identical totals, identical per-module distribution. **LLVM's instcombine already performs that strength reduction**, so `* 32767` was never a multiply instruction. I kept the change with the comment corrected — it is the exact identity and it guarantees the shift at `opt-level = 0` — but it is **not** claimed as a reduction.

### The finding that matters: static counting is not the executed count

The identical totals expose a flaw in the instrument I called authoritative, and it's the same *class* of flaw as the source censuses: **a static instruction count is not an executed-instruction count.** A multiply inside a branch serving never takes is counted as if it runs.

Concretely: `MultiHeadVsaAttention::attend` gets vectors via `Codebook::get`, whose on-demand path calls `Hypervector::from_seed` — SplitMix64 with the arbitrary constants `0xBF58476D1CE4E5B9` / `0x94D049BB133111EB`, which are *not* strength-reducible, so each emits a real `mul`, inlined into `attend`. **But with the learned-root codebook (`vsa_code_mode = 1`) the on-demand path isn't taken at run time** — so those multiplies are present in the image and possibly never execute. Static counting cannot tell.

**Four instruments have now addressed this one question, each failing for a different reason:**

| # | instrument | why it fell short |
|---|---|---|
| 1 | raw grep of `*` | counts comments, derefs, type syntax |
| 2 | `forbidden_arith` scan | counts operators, not circuits; misses method forms |
| 3 | token-level classifier | can't see compiler decisions; counts float scalings as variable |
| 4 | static binary disassembly | counts instructions **present**, not **executed** |

Only #4 was described as authoritative. The identical before/after totals falsify that.

### The instrument that would actually answer it

**Dynamic retired-instruction counting by class** — a CPU-counters trace over a fixed serving workload, reporting multiplies per token. That measures exactly what D0-a names, is immune to dead code and inlining, and gives a per-token number instead of a per-binary one. It is not built; it needs its own projection (traces are multi-hundred-MB and the workload runs are a machine charge).

**Until it exists, the honest statement is that this project does not know how many multiplies its serving path executes** — and every number it has published for that is an estimate from an instrument since shown to be wrong.

No further conversion: the VSA redesign was premised on those 306 being hot scalar multiplies; the measurement says the scalings are already shifts, so the premise is void.

Receipt: [`docs/evidence/native_geometric_vsa_lookup_noop_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_vsa_lookup_noop_2026-09-19.txt). Ledger: **139,819,275 / 147,200,000 ms**; free space 67 GB; model storage untouched.


### comment 5739955482 — 2026-09-19T06:33:38Z — Casey-allard

## Update — `1a342d49`: the dynamic harness **cannot be built on this hardware**, and my "29 % of serving multiplies" claim was a static artifact

### Feasibility checked before building — and it fails

| route | status |
|---|---|
| `xcrun xctrace` (Instruments CPU counters) | **not present** on this machine |
| `frida` (software DBI / Stalker) | **not installed** |
| Apple PMU via kpc | counters exist (`kern.num_static_scalable_counters: 90`, `kpc.pc_capture_supported: 1`) but **ARMv8 defines no architectural multiply event**, and Apple's implementation-defined events are undocumented |

**A hardware multiply count is unobtainable here, with or without root.** I proposed an instrument that does not exist. That's the fifth failed instrument for this question, and the first where the mistake was mine *before* writing any code — the check took two minutes and I should have done it before recommending the harness.

### What I did instead is sound: find the source rather than count it

Disassembling `MultiHeadVsaAttention::attend` gives an unmistakable signature:

```
eor x16, x12, x12, lsr #30     ← z ^= z >> 30
mul x14, x14, x25              ← z *= (register-loaded 64-bit constant)
mul x15, x15, x25
eor x16, x16, x16, lsr #27     ← z ^= z >> 27
mul x14, x14, x26
```

That is the **SplitMix64 finaliser**. Confirmed independently — `0xbf58`, `0x94d0`, `0x9e37`, `0x1331` all appear in the disassembly. `Hypervector::from_seed` is SplitMix64 (four multiplies per 64-bit word), reached from `attend` via `Codebook::get`'s fallback for tokens not in the table, inlined.

### Correction to my own claim

I said *"the inert mechanism is the largest multiply consumer — the VSA layer is 898 of 3,100 = 29 % of serving multiplies"* and built a redesign on it.

**That is a static artifact and it is wrong as a statement about serving.** Those multiplies are on the **on-demand** codebook path. Under `vsa_code_mode = 1` with a materialised table, `get()` hits the table and `from_seed` is never called — **the multiplies are in the image and do not execute.** Static presence is not execution: precisely the flaw I had identified in the instrument one step earlier, and then failed to apply to my own reading of it.

(Also recorded: 306 is the symbol-matched count for `attend`. A span extraction reported 1,906, but that span crosses other symbols and is not the method's own count. I never reported 1,906 as the method's, and it should not be inflated into one.)

### The sound substitute for a count

A **per-function zero-check** is sound where a count is not: if a function's address range contains no multiply mnemonic, that function *cannot* execute one. Sufficient, checkable with `otool` + symbol ranges, indifferent to dead code and inlining. It can't be a cargo unit test (needs a release build), so it belongs as a checked-in script plus a recorded procedure.

Under it, the current tree would **fail** the VSA attention symbol and **pass** `context_predicted_fiber_from_ring` (mul = 0).

### The actionable fix

The multiplies are in **on-demand pseudorandom hypervector generation** — not in the correlation arithmetic (the `* 32767` was already a shift) and not in the bundling. Since the hypervector bank is **arbitrary by construction**, the generator can be made multiplier-free: an **xorshift/rotate mixer** instead of SplitMix64's two arbitrary-constant `wrapping_mul` calls. That removes the possibility rather than excluding the path that uses it, and makes the zero-check passable.

It's a behavioural change (different codes), so it requires the anchor rebuild `prepare_vsa_code_mode` already does plus re-measurement of routing recall and BPB. **Not implemented** — it needs its own measurement and I won't claim it without one.

Receipt: [`docs/evidence/native_geometric_dynamic_instrument_feasibility_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_dynamic_instrument_feasibility_2026-09-19.txt). Ledger: **139,824,275 / 147,200,000 ms**; 66 GB free.


### comment 5740040578 — 2026-09-19T06:50:47Z — Casey-allard

## Update — `5423761b`: the zero-check instrument works, produced two real reductions, and Card P1 needs your authorization

### The instrument that finally works

After four counting instruments failed and a hardware multiply counter turned out not to exist on Apple Silicon, `scripts/serving_multiplier_check.py` replaces counting with a **per-function zero-check**: disassemble a `--release` binary, segment per symbol, assert **zero** multiply mnemonics for each declared serving symbol.

That's a **sufficient** condition — a function whose own range contains no multiply *cannot* execute one — and it's indifferent to dead code, inlining and compiler strength reduction: the three things that broke the counters. It also reports what it can't check: **seven declared serving names are inlined and have no release symbol**, so they are explicitly **not certified** rather than silently assumed.

### What it found immediately

| symbol | mul | status |
|---|---:|---|
| `context_predicted_fiber_from_ring` | 0 | **PASS** |
| `compose_ring_roots` | 4 | FAIL — "pure table lookup" |
| `binary_model` | 4 | FAIL (3 symbols) |
| `context_fiber_from_ring` | 8 | FAIL (`hopf_fiber_project`'s S3→S2 terms) |
| `score_token` | 34 | FAIL (lattice scoring) |
| `MultiHeadVsaAttention` | 306 | WATCH (SplitMix64 on-demand generation) |

A composition helper documented as pure table lookups carried **four multiplies**, because `% GROUP_ORDER` and `state * GROUP_ORDER` address a 120-wide table, and a modulo or multiply by a non-power-of-two constant is lowered to `umulh`/`madd`. **No source census could have found this** — the source contains no multiply in either place.

### Two measured reductions against the sound measure

1. `% GROUP_ORDER` → `.min(GROUP_ORDER - 1)`. The modulo was redundant (values are already range-guaranteed), so a branchless min is equivalent and multiplier-free. **4 → 2.**
2. Product-table row stride padded 120 → **128**, so the row base is a shift. Cost 960 padding bytes. **2 → 1.**

Behaviour unchanged: greedy 128-token output **byte-identical**, 617 tok/s.

**Self-inflicted regression, recorded:** the stride change broke three `group_table` tests and then a fourth assertion, because the tests used `GROUP_ORDER` as the stride. Fixed to `ROW_STRIDE`; 6/6 pass.

### Card P1 — not executed, and I need authorization

`ollama` **is** installed (`/usr/local/bin/ollama`), which makes P1 cheaper than I assumed — no need to clone or build `llama.cpp`. But **there are no weights**: `ollama list` is empty and the blob store is 4 KB (a bare manifest stub).

So P1 needs:
- **multi-GB downloads** (≈1.1 GB BitNet b1.58 2B4T, 2–6 GB per Q4 model) — 66 GB free, so storage is fine, but I'm not going to pull multiple GB of external weights without you saying so;
- **sudo for `powermetrics`** — required for J/token, and an agent shouldn't invoke sudo unprompted;
- **4–8 hours wall** for downloads plus three repeats per artifact. Projection to record before any run: **≥ 10 GB storage, 4–8 h wall, sudo for energy.**

It remains my top recommendation, because **the project has no trustworthy serving-efficiency number of any kind** — not multiplies/token, not tok/s, not J/token — and the only historical numbers were withdrawn as fabricated. Everything I've built here has been measurement infrastructure to make such a number mean something; P1 is the number itself.

Receipt: [`docs/evidence/native_geometric_serving_zero_check_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_serving_zero_check_2026-09-19.txt). Ledger: **139,854,275 / 147,200,000 ms**; 66 GB free.


### comment 5740138016 — 2026-09-19T07:11:57Z — Casey-allard

## Update — Card P1 (partial): the project's first measured incumbent baseline

You authorised the downloads and `sudo`. Two hard limits surfaced first, and I checked before spending anything:

- **`sudo` requires a password** → `powermetrics` is unavailable to an agent → **J/token is UNAVAILABLE**. Not a choice; a harness limitation.
- **No `llama-perplexity`** → **incumbent BPB is UNAVAILABLE** without building llama.cpp.
- `ollama 0.33.2` was already installed, so no clone/build was needed; `ollama list` was empty, so I pulled `qwen2.5:1.5b` Q4 (986 MB).

### Measured

**CPU-forced** (`num_gpu: 0`) — the only fair comparator for a CPU-only serving path:

```
run 1  35.88 tok/s   load 2.090 s   prompt 176.6 tok/s
run 2  36.36 tok/s   load 0.001 s   prompt 1099.3 tok/s
resident 1.4 GB   PROCESSOR 100% CPU
```

(ollama's default is the **GPU**: 34.67 / 35.25 / 34.28 tok/s, 1.2 GB. Only the CPU figure belongs in this comparison.)

Bandwidth sanity check: 986 MB of weights at ~61 GB/s gives a ~62 tok/s ceiling, so 36 tok/s is ~58% of it. The measurement is plausible, not anomalous.

### The number the project lacked

| system | decode tok/s | resident |
|---|---:|---:|
| qwen2.5:1.5b Q4 (ollama, **CPU**) | 36.1 | 1.4 GB |
| **UOR-R4 native (release, CPU)** | **617** | **~1 MB** |

~**17× throughput**, ~**1400× smaller resident**.

### Why that must not be spun

**The quality is not comparable, so the efficiency claim is not a win claim.** `qwen2.5:1.5b` writes coherent English with discourse structure maintained across sentences. The native model emits repetitive TinyStories fragments and scores 1.7989 BPB under full-vocabulary discrete scoring on an already-open slice. Efficiency at unequal quality is a fact about two different systems.

The honest formulation: **the native architecture's cost per token is 17× lower and its footprint 1400× lower, at a quality the incumbent exceeds.** Whether that cost advantage survives to *equal* quality is the open question — and it is the question a P3-style scaling test exists to answer.

### Mismatches that travel with the numbers

- **Metric mismatch:** ollama's `eval_duration` is **decode-only**; the 617 tok/s figure is total-request and **includes prompt ingest**. They are not computed alike. A decode-only native measurement is required before the 17× factor is quoted — it can move the figure, but not by 17×.
- One model, one prompt, one session, 2–3 repeats. The card also wants SmolLM3-3B and Qwen3-4B Q4 on a **sealed held-out set**; neither was used.
- **UNAVAILABLE, not omitted:** energy (sudo) and incumbent BPB (`llama-perplexity`).

### To finish P1

1. `sudo` for `powermetrics` → J/token for both systems.
2. A llama.cpp build with `llama-perplexity` → incumbent BPB on the same held-out text, which is what makes a **quality-matched** efficiency statement possible.
3. Pulls of SmolLM3-3B and Qwen3-4B Q4 (2–6 GB each).
4. A sealed held-out set, built once.
5. Decode-only timing for the native path, so both sides are computed the same way.

Items 1 and 2 block any quality-matched efficiency statement — the only kind that supports the energy objective.

Receipt: [`docs/evidence/native_geometric_p1_incumbent_baseline_2026-09-19.txt`](https://github.com/UOR-Foundation/uor-r4/blob/codex/stage1-falsification-sweep/docs/evidence/native_geometric_p1_incumbent_baseline_2026-09-19.txt). Model retained in `~/.ollama/models` (940 MB); 59 GB free; no cleanup performed.


### comment 5740620304 — 2026-09-19T08:57:00Z — Casey-allard

## Backward pass delivered; first trained instruction run is an honest negative

Commit `c2d4e141` on this branch. Full record: `docs/evidence/native_geometric_lowbit_chat_2026-09-19.txt`, plus a new top entry in `docs/integration/current-state.md` and a row in `EVIDENCE.md`.

**What now exists.** `LowBitCoreTrainer` (`learner/lowbit_core.rs`) — f32 masters, quantise-on-forward with the same rule `TernaryLinear::quantize` applies, straight-through estimator through the ternary step, BPTT through the recurrence and the `relu` mask, bias-corrected Adam mirroring the existing `AdamMoments`. Plus `LowBitCore::sequence_loss` / `forward_reference_f32` and `LowBitCoreTrainer::loss` / `next_token_accuracy`, so any reported loss is a statement about the function that serves. `learner/chat.rs` adds a byte-level tokenizer (vocab 259) and an instruction loader; `bin/train-lowbit-chat.rs` trains and prints raw generations.

**Learning is verified.** 10 `lowbit_core` + 7 `chat` tests pass; `cargo fmt --check` clean. The learning test uses a delayed-recall task the recurrence is required for (a no-recurrence control cannot exceed chance): loss **4.390 → 0.094**, **100 % recall**, control 0.901, `trainer.loss == core.sequence_loss`, and the trained `forward_i32` equals `float_forward_ste` elementwise.

**The end-to-end run does not answer.** dim 128, 4,000 steps, ≤24-byte sequences, procedural instruction corpus (not TinyStories): held-out loss **20.2727 → 3.9138**, top-1 **29.5 %** — but **response-only teacher-forced accuracy is 0/470** and free generation is a single repeating fragment (`854854854…`) that never emits `<|end|>`. The 29.5 % is instruction copying, not answering. No conversational capability.

**Blocker, measured.** A sweep over usable sequence length confines learning to ≈≤24-byte sequences; at 40 tokens held-out loss is ≈18.5, worse than uniform (`ln 259 = 5.56`). `h_t = relu(W_x + W_h·h_{t-1})` with random ternary `W_h` grows ≈ `√dim/2` per step with no contraction or normalisation, so the logits saturate and the response gradient is lost. Architecture, not the backward pass.

**Next action.** Stabilise the recurrence inside D0-b: `min(relu(·), MAX)` (smaller; preserves integer/float exactness because both paths clamp at the same integer) or `relu(W_x + (W_h·h) >> k)` (contractive, but moves the float reference into a fractional domain). Then re-run and require response accuracy to leave zero before any scale, tokenizer or chat-data change.

**Ledger.** Charged 433,000 ms measured model time; cumulative **140,329,275 / 147,200,000 ms** (≈114.5 min remaining). No extension used. Retained negative candidate: `.uor-models/native-lowbit-chat-2026-09-19/lowbit_chat.bin` (330,764 bytes). The 128 MiB stop margin is untouched. `References #820`. Not closing anything.


### comment 5740901861 — 2026-09-19T09:58:57Z — Casey-allard

## Follow-up: the dense low-bit recurrence is structurally falsified; a content-addressed core replaces it

Commit `7b35126d`. Receipt: `docs/evidence/native_geometric_lowbit_attention_2026-09-19.txt`. Design + scaling: `docs/integration/geometric-core-architecture-2026-09-19.md`.

**Two measured, independent causes of the chat run's failure.**

1. **Magnitude.** `h_t = relu(W_x + W_h·h)` with ternary `W_h` has spectral norm ≈ `2√(p·dim)`. Measured peak `|h|` reaches i32 saturation (~2.14e9) **between 16 and 32 steps** at `dim` 32/64/128. That is exactly the ~24-token workable window, and the collapse at 40 tokens (loss 18.47, worse than uniform `ln 259 = 5.56`).
2. **Burial.** Summing every past input into the same channels buries a remembered item. Delayed recall reaches delay 4 at `dim ≥ 128` and is **0.00 at delay 8 at every width** (800 and 3000 steps identical). 8× width buys no extra remembered item.

**The obvious repair is falsified.** Right-shifting the recurrent term stabilises the magnitude and destroys the memory in the same operation (delay-1 recall 1.00 → 0.16 at k=2 → 0.00 at k=3). Structural reason: a ternary matrix cannot be near-orthogonal, so the only shift that stabilises the recursion also erases what is stored. **Stability and memory are not jointly available from dense ternary mixing with a scalar decay.** Kept as an opt-in mechanism (`recurrent_shift = 0` is the original behaviour), not deleted.

**Replacement — `learner/lowbit_attention.rs`.** Content-addressed linear attention with ternary operands: `S_t = (S_{t-1} >> decay) + k ⊗ v`, read as `relu(q · S)`, then the ternary output map. `k` and `q` are unscaled ternary, so the outer product and the matched-filter read are **conditional adds/subtracts with no multiplier**. Integer path verified exactly equal to an `f64` reference at decay 0/2/4. `decay = 0` grows the state **linearly** (peak `|S| < 2^20` over 512 steps). Grounded in linear attention / RWKV (2305.13048) / HGRN2 (2404.07904) / MatMul-free LM (2406.02528).

**Result.** Induction (`[x,y,filler*delay,x] → y`), held out: **delay 16 = 1.00** at `dk=dv` 64 and at 128/256/512 — against the dense core's **0.00 at delay 8**. The read is a matched filter, so capacity buys horizon (`SNR ≈ √(dk/N)`). The new core also scales in width.

**The negative that matters more: training is fragile.** `dk=128, lr=0.05` collapses to the uniform predictor (loss exactly `ln 8`); `dk=256, lr=0.02` gives 0.38; `dk=64` delay 32+ gives 0.00 where delay 16 gives 1.00. An architecture that finds its solution only in part of the regime map cannot be scaled. Top open item.

**Scaling — the honest part.** Design doc §6: a coherent chat model of this family needs ~`10^9` parameters and `10^10`–`10^11` tokens → `10^5`–`10^6` hours of sequential BPTT on one M1. **The serving premise survives; the implicit premise that chat-scale training also happens on this laptop does not.** Three options are put to the owner: narrow the target to a domain-scoped local model, separate training compute from serving, or pursue a sample-efficiency result.

**Claims:** none. Still no chat capability; the new core is measured on synthetic induction, not language. No serving-multiplier claim for the new module — the construction is multiplier-free and the integer path is verified, but `scripts/serving_multiplier_check.py` was not run against it. Energy/token remains UNAVAILABLE.

**Next action.** Design doc §7.1–7.2: add a **power-of-two readout normalisation** (shift the read by the leading bit of the accumulated key mass — bit scan + shift, no divide), then re-measure the regime map and the induction horizon. Do not scale `dk`, add layers, or change the tokenizer until the regime map is flat.

**Ledger.** Charged 2,029,290 ms measured model time; cumulative **142,358,565 / 147,200,000 ms** (~80.7 min remaining). No extension used. `References #820`.


### comment 5742970667 — 2026-09-19T15:14:45Z — Casey-allard

## Geometric addressed memory built and measured; conditioning fixed; BPE queued

Two commits: `a2a2c867` (geometric mechanism) and `39a5e3b9` (conditioning). Receipts: `docs/evidence/native_geometric_geometric_attention_2026-09-19.txt`, `docs/evidence/native_geometric_lowbit_attention_2026-09-19.txt`.

### 1. The project's own math, as requested

Owner direction was to push the project's mechanism rather than fall back on linear attention. `learner/geometric_attention.rs` implements an **exactly-addressed associative memory over the verified 2I group**:

```
write:  S[addr(prev)] += value(cur)
read:   y = norm(S[addr(query)])   ;   logits = W_o · relu(y)
```

Address = table read, write = add, read = table read, normalisation = bit scan + shift. **Multiplier-free throughout, `O(dv)` per token against linear attention's `O(dk·dv)`.** Addresses are elements of the machine-checked 2I group (120 H4 roots composed through `learner/group_table.rs`); two composed factors give 120² = 14,400 addresses at 13.8 bits — the transition plan's `H4^k` product-code lever on the existing table.

**Measured**, context repetition (random run `R`, then `R`; the second copy needs memory of `prev → next`), held out:

| context | geometric (exact address) | linear attention |
|---:|---:|---:|
| 4 | **0.69** | 0.05 |
| 8 | **0.31** | 0.03 |
| 16 | **0.02** | 0.00 |

Exact addressing wins at every length — the direction theory predicts (cross-talk removal). **But both mechanisms fail this task at this scale and the exact-addressing variant fails less; a 0.05 baseline is at chance for 32 classes, so "beats linear attention" is a weak claim and is recorded as such.**

**Negatives first:** it does not solve the task; more training does not help (900 → 3000 steps: 0.69 → 0.66), so the ceiling is a mechanism limit; the address assignment is fixed, not learned; capacity is bounded by `n_addr`. Likely causes of the ceiling: ternary `dv=32` value/output resolution, and a first-order (bigram) memory.

### 2. Conditioning — fixed, and partially confirmed

The owner confirmed the power-of-two readout normalisation is D0-b-compliant. Implemented in `LowBitAttention` (serving, loss, trainer; shift constant under STE) and measured against the collapse recorded earlier:

```
dk=128, lr=0.05, norm_bits=0: acc 0.06  loss 2.079  (= ln 8, uniform predictor)
dk=128, lr=0.05, norm_bits=6: acc 0.38  loss 1.970
```

Design-doc prediction §7.1 is **partially confirmed**: normalisation changes the regime but 0.38 is not a solution. It had **no measurable effect** on `GeometricAttention` — both results recorded, not just the favourable one. Pinned as a deterministic assertion.

### 3. BPE move — confirmed and queued

The project BPE (`HfBpeTokenizer` over `tokenizer.json`) is **49,152 tokens** (+17 added). The shipped native model uses a **4096** vocabulary, so the plan is to wire `HfBpeTokenizer` with a frequency-reduced vocabulary at the existing 4096 resolution first (the output projection is `vocab × dv` and dominates per-token cost — 49k × 64 ≈ 3.1M adds/token, which would consume the remaining ledger in a single short run). Queued for the next training run, after conditioning.

### 4. Honest state

Still **no chat capability**. Everything here is synthetic-task measurement plus one exactness property. The geometric core's construction is multiplier-free and its integer path is verified equal to an `f64` reference, but `scripts/serving_multiplier_check.py` has not been run against it, so no served-binary multiplier claim is made. Energy per token remains UNAVAILABLE.

**Recommended next action, in order:** (1) raise `dv` 32 → 128 at fixed everything else for the geometric core — if accuracy rises with `dv` the limit is readout resolution and the mechanism scales, otherwise product factors are required; (2) learn the address factors (`H4^k`, Stage 3); (3) compose rather than choose — the exact-address memory (associative recall) and the matrix-state core (contextual integration) solve different problems, so a layer reading both is the natural next architecture; (4) then the BPE training run and raw generations.

**Ledger:** charged 315,000 ms this round; cumulative **142,673,565 / 147,200,000 ms** (~75.4 min remaining). No extension used. `References #820`.


### comment 5743373234 — 2026-09-19T16:11:41Z — Casey-allard

## Geometric memory: found and fixed the ceiling, and confirmed what remains

Three commits (`3a6f7506`, `e4be0473`, `8261ffa2`). Receipt: `docs/evidence/native_geometric_geometric_attention_2026-09-19.txt`.

### The `dv` diagnostic came back negative — and the reason was a genuine bug in the mechanism

Raised `dv` 32→256 with everything else fixed: **no effect** (0.69/0.72/0.75/0.70 at context 4). That sweep was confounded. The real ceiling was the **`relu` after the read**: with exact addressing the *selection* is already the nonlinearity, so a `relu` there only discards the retrieved value's sign half.

| context | relu | no relu |
|---:|---:|---:|
| 4 | 0.69 | **0.86** |
| 8 | 0.31 | **0.64** |
| 16 | 0.02 | **0.47** |

`relu=false` is now the measured default. Pinned as `selection_is_the_nonlinearity_not_the_read`.

### Re-measured under a sign-preserving read, resolution *does* scale

| `dv` | ctx 4 | ctx 8 | ctx 16 |
|---:|---:|---:|---:|
| 32 | 0.86 | 0.64 | 0.47 |
| 128 | **0.94** | 0.72 | 0.44 |
| 256 | 0.89 | **0.80** | 0.50 |

Pinned as `resolution_scales_the_mechanism`. Against linear attention's 0.05/0.03/0.00. **Best now 0.94 (ctx 4, dv=128) and 0.80 (ctx 8, dv=256).** Still not a solution and still not a language model.

### The remaining ceiling is a first-order address collision — confirmed by controlled experiment

Same run length, same alphabet, differing only in whether the queried token's bucket receives two different successors:

```
ambiguity clean=0.44   duplicated-key=0.14
```

A duplicated queried key costs **3× accuracy**. So the ceiling is addressing by one token — not capacity (`dv` ruled out), not budget. Pinned as `a_duplicated_queried_key_breaks_first_order_addressing`.

### Next mechanism, with its falsifiable prediction

The experiment says what kind of fix is needed: the address must be an **ordered word** over the context, not a set or a single token. This is exactly W33's ordered-operation result (`PLPL=LPLP`, `Ω=LP−PL`, `ker(Ω)` order-insensitive — `docs/integration/nemesis-w33-relevance.md`), and the project already has the exact machinery in the machine-checked 2I composition table.

**Address by the ordered pair `(e(t_{i-1}), e(t_i))` of 2I elements** — 120² = 14,400 ordered addresses, order-preserving by construction since `(a,b) ≠ (b,a)`. Prediction: the duplicated-key collapse shrinks and context-16 accuracy rises materially. This is a small, contract-compliant change to `GeometricAttention`'s addressing.

BPE-4096 remains queued for the next training run.

**Ledger:** cumulative **142,858,565 / 147,200,000 ms** (~72.4 min remaining). No extension used. `References #820`.


### comment 5743556761 — 2026-09-19T16:39:14Z — Casey-allard

## Ordered-word addressing: the collapse is recovered, and context copy reaches 100%

Commit `d7ee26fc`. Receipt: `docs/evidence/native_geometric_ordered_word_addressing_2026-09-19.txt`.

### The prediction from the last round is confirmed

Last round's controlled experiment showed a first-order address costs 3× accuracy when a token repeats with two different successors (`clean 0.44` / `duplicated 0.14`), and predicted an **ordered word** address would recover it. Held fixed: task, alphabet, seeds. Only the word length changed.

```
collision order=1 clean=0.44 dup=0.14 | order=2 clean=1.00 dup=0.98
```

**Duplicated-key 0.14 → 0.98.**

`GeometricAttention` now addresses by an ordered word of `order` tokens over the 120 elements of `2I`. `order = 2` gives 120² = **14,400 ordered addresses** with `(a,b) ≠ (b,a)` by construction; the address is a positional radix composition — table reads and index arithmetic, no multiplier.

### Context copy now solves where the matched filter scores zero

| context | order=1 | **order=2** | linear attention |
|---:|---:|---:|---:|
| 4 | 0.86 | **1.00** | 0.05 |
| 8 | 0.64 | **1.00** | 0.03 |
| 16 | 0.47 | **1.00** | 0.00 |

### Two ablations moved back to `order = 1`, explicitly

Both are true and both are now *unresolvable* at `order = 2` because the task saturates at 1.00: the `relu`-after-read cost (order=2: 0.95 vs 1.00, below the 0.1 threshold) and resolution scaling (order=2: 1.00 vs 1.00). Retained at `order = 1` where they measure 0.31 vs 0.64 and 0.64 vs 0.80. Recorded rather than silently weakened — a saturated task is not evidence of absence.

### A defect found and fixed mid-change

The refactor's first run returned 0.00 everywhere, including at `order = 1` where 0.86 was known. Cause: `tail_word` read index `len` instead of `len − 1`, so `forward_i32` and the accuracy metric read the wrong bucket — **training was correct and the measurement was not.** All results above are post-fix.

### Research synthesis, and SpiralCore indexed (owner request)

- The project's **September-13 attention synthesis** had already selected H1 and named the **directed relative element `r(i,j) = inverse(g_i)·g_j`** — the same ordered-word idea this session derived from measurement. The two converge independently.
- **D0-a vs D0-b conflict recorded:** that synthesis calls a published MatMul-free model's ternary accumulation "a matrix product under this project's stronger rule". That was the D0-a reading, superseded by owner-signed **D0-b**, which permits bounded integer/ternary linear maps with no multiplier in the kernel. `LowBitAttention` is legal under D0-b and would not have been under D0-a.
- **W33/NEMESIS** supply no replacement attention rule (their dossier says so), but W33's ordered-operation *principle* transferred and worked.
- **SpiralCore v68 mathematics extracted and indexed**: mechanical HTML extraction, SHA-256 verified against the preserved copy — 33 sections / 49,066 chars (dodecahedral network, six H2 decagons, addressing schema, E8 operator subnet routing, LADA ports, stabilizers, Bell 2-of-6 codec, Clifford complement, FBS binder tree, route witnesses, effective resistance). Extract at `research/spiralcore-v68/spiralcore-v68-mathematics-extract.txt`; ingested as **34 items / 33 edges**; retrieval verified through the knowledge service. Faithful extraction, not endorsement.

### Honest limits

Still **not a language model**: one layer, `dv = 32`, synthetic task. The task is now **saturated at 1.00**, so it can no longer discriminate — the next measurement must be harder. Addresses are still fixed, not learned; `order ≥ 3` needs a sparse store. No serving-multiplier claim (`serving_multiplier_check.py` not run). Energy/token UNAVAILABLE.

**Next action:** (1) harden the measurement — raise alphabet and run length until `order = 2` stops scoring 1.00, then compare `order = 2` vs `order = 3` there; (2) the **graded group kernel** (`w[class(q⁻¹g)]`, the other half of the project's `r(i,j)` formalism) so a query can match a *near* word; (3) then BPE-4096 and a real instruction-data run.

**Ledger:** cumulative **143,073,565 / 147,200,000 ms** (~68.8 min remaining). No extension used. Also repaired a duplicated `## Not done` heading in the ledger introduced by repeated anchor-inserts. `References #820`.


### comment 5743619189 — 2026-09-19T16:47:25Z — Casey-allard

## Addendum (`0183e967`): the saturation caveat is closed

The previous commit flagged that the copy task saturated at `order = 2` for a 32-token alphabet, so it could no longer discriminate. Difficulty is now raised by shrinking the alphabet until ordered **pairs** repeat with different successors (context 16, `dv = 64`, deterministic seeds):

| alphabet | order=1 | order=2 |
|---:|---:|---:|
| 4 | 0.34 | **0.62** |
| 8 | 0.25 | **0.84** |
| 16 | 0.42 | **0.97** |
| 32 | 0.55 | **1.00** |

Ordered-pair addressing wins at **every** difficulty and the margin grows as words become unique (+0.28, +0.59, +0.55, +0.45). At alphabet 4 — only 16 possible ordered pairs for a 16-token run — both ceilings are gone and `order = 2` still doubles `order = 1`. Pinned as `ordered_words_win_at_every_difficulty`.

One counter-intuitive but consistent detail: `order = 1` also improves with a larger alphabet (0.34 → 0.55), because single tokens then repeat less often. Those are two different collision mechanisms and the data shows both.

Tests: `geometric_attention` 15/15; `cargo fmt --check` clean. Ledger **143,143,565 / 147,200,000 ms** (~67.6 min remaining).


### comment 5743916079 — 2026-09-19T17:27:41Z — Casey-allard

## Spherical-harmonic grounding: Peter–Weyl on 2I, and a graded kernel that works

Two commits (`0fd09066`, `17b3eaa2`). Receipt: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

You noted some original ideas were based on spherical harmonics. The **exact finite form** of that here is **Peter–Weyl**: conjugation-invariant functions on a finite group — functions of the *relative* element `r(q,g) = inverse(q)·g`, the descriptor the September-13 synthesis named — form a space whose dimension is the number of conjugacy classes, spanned by the irreducible characters. That is this group's harmonic band count.

Computed from the project's **own verified table, not cited**:

```
2I conjugacy classes = 9, sizes (sorted) = [1, 1, 12, 12, 12, 12, 20, 20, 30]
```

with conjugation-invariance checked over all 14,400 `(g,h)` pairs. So a graded read kernel `w[class(inverse(q)·g)]` is the **maximally compact rotation-invariant kernel this group admits: nine ternary weights rather than 120.**

### Mechanism and measurement

`graded_read`: `y = Σ_g w[class(q⁻¹g)]·S[g]` with **ternary** `w`, so the read stays conditional adds/subtracts. Weight on the identity class reproduces `S[q]` exactly.

Corrupt the query address by a fixed group element and weight the kernel on its class; averaged over five corruption elements, held out, deterministic (`vocab` 120, `dv` 64, 900 steps):

```
graded kernel over 5 corruptions: clean exact=0.55 soft=0.15 | corrupted exact=0.13 soft=0.23
```

**A class-function kernel recovers a corrupted query — 0.13 → 0.23 (≈1.8×)** — because it pools over group-near stored elements where the exact read looks in exactly one wrong bucket. It pays with clean accuracy (0.55 → 0.15). The kernel is **hand-set, not learned**, so this is a lower bound on the trade a trained kernel could make. Both numbers reported.

### Two defects found by the new checks — both recorded

1. **The identity of 2I is element 1, not element 0.** `exact_kernel` weighted an arbitrary singleton class. The first graded measurement (clean 0.19; corrupted 0.13 → 0.23) was therefore **invalid and has been re-measured**; the numbers above are the corrected result.
2. **A vacuous test.** `graded_exact_kernel_equals_the_bucket_read` initially compared two empty vectors and passed. It now asserts a non-empty bucket *and* a query-dependent read, and its token list was changed so the queried element is genuinely a predecessor.

This is the **second time in two sessions that a measurement defect looked like a mechanism result.** The non-vacuity guard is the general lesson and is now in the test.

### Honest limits

Not a language model: one layer, synthetic tasks. The kernel is hand-set; whether a *learned* kernel finds a better trade is untested. Corruption is a single fixed group element; multiple/out-of-set corruptions untested. Ternary kernel weights cap a class weight at `±1` or `0` (larger integers would need a multiply). No serving-multiplier claim; energy/token UNAVAILABLE.

**Next action:** (1) **learn the kernel** — nine ternary weights, gradient in closed form `∂L/∂w[c] = Σ_{g∈c} Σ_j dnum_j·S[g][j]`, write path unchanged; prediction: a learned kernel keeps most clean accuracy while retaining the corruption benefit, and if it cannot, the trade-off is structural and should be recorded as such. (2) Then **BPE-4096 and a real instruction corpus**.

**Ledger:** cumulative **143,433,565 / 147,200,000 ms** (~62.8 min remaining). No extension used. `References #820`.


### comment 5744067485 — 2026-09-19T17:49:00Z — Casey-allard

## Learning the harmonic filter: attempted, negative, reverted — and *why* is the result

Commit `379af88e`. Receipt §8–9: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

On your direction I implemented the learned filter — nine ternary weights, gradient `∂L/∂w[c] = Σ_{g∈c} Σ_j dnum_j·S[g][j]`, read as a group convolution with STE + Adam — and measured it against the fixed exact filter under the same protocol:

```
fixed   kernel=[0,1,0,0,0,0,0,0,0] clean=0.42 corrupted=0.13
learned kernel=[1,1,1,0,0,0,0,0,0] clean=0.14 corrupted=0.14
```

**Trained on clean addresses only, the filter spreads over neighbouring classes and loses clean accuracy without gaining corrupted accuracy.** The reason is the principled part: **the training objective contains no address corruption, so robustness to it cannot be learned from that objective.** The optimiser does exactly what it is asked — it adds filter mass only while it helps the clean loss, and spreading never does. The spread filter's benefit is real (0.13 → 0.23 hand-set, previous round) but must be **chosen**, or trained with corrupted addresses present in the objective.

The change also regressed five `geometric_attention` tests in the trainer's read path. Since the experiment is a clear negative, I **reverted** it rather than carry a regressing change; the file is back at its verified 18/18 state and the working tree is clean. **The diagnosis is retained; the regressing code is not.**

### Where your wider concept already lands (and what is missing)

Your framing — harmonic compute with Lie-group packaging, storage and backpropagation in a superposition over that basis — maps as follows:

| Concept | Status |
|---|---|
| **Superposition storage** over the group: `S = Σ_g c_g·δ_g`, read = convolution against a filter | **present** (classical superposition over a Lie-group basis — not quantum, and described as such) |
| **Backpropagation in that basis** | **present** — the suffix-sum gradient *is* the adjoint of the convolution |
| **Lie-group packaging** | **partly** — exact composition via the verified table; the ordered word packs context into a group element; the *element assignment* is still a fixed function of the token id |
| **Harmonic compute** | **partly** — the filter is a band-limited (conjugation-invariant) class function, the maximally compact one; a general *non-class* group-algebra filter and multiple bands are missing |

### Next honest steps, in order

1. **Put corrupted addresses in the training objective** and re-test the learned filter — the diagnosis predicts that is exactly what makes it work.
2. A **general non-class filter** (a full group-algebra element, not merely a class function).
3. **Learned packaging** of the element assignment.
4. Then **BPE-4096 and real text**.

**Ledger:** cumulative **143,668,565 / 147,200,000 ms** (~58.9 min remaining). No extension used. `References #820`.


### comment 5744272456 — 2026-09-19T18:16:56Z — Casey-allard

## Corruption in the objective: the diagnosis is confirmed — and the learned filter still is not worth it

Commit `65b88da6`. Receipt §10: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

Last round I concluded that learning the harmonic filter failed because **the objective contains no address corruption**. That was testable, so I tested it: an optional `corrupt_frac` displaces the query address by a random group element during training, so a spread filter is what makes the answer recoverable. The filter is now learnable (`learn_kernel`, own clip group) with defaults preserving all previously verified behaviour.

```
corrupt 0.0: kernel=[1,1,1,0,0,0,0,0,0] clean=0.11 corrupted=0.12
corrupt 0.5: kernel=[1,1,0,1,1,1,0,1,0] clean=0.27 corrupted=0.13
```

**The diagnosis is confirmed:** corruption in the objective changes what the filter learns (spreads 3 → 7 classes) and raises corrupted accuracy (0.12 → 0.13). Clean accuracy also rises (0.11 → 0.27) — a regularisation effect from training through displaced addresses.

**But the learned filter still is not worth it on this task.** The **fixed exact filter reaches clean 0.42** on the same architecture and budget, so every learned variant is worse on clean and only marginally better under corruption. Honest reading: **a spread filter pays only if address corruption is part of the deployment distribution.** +0.01 under corruption against −0.15 clean is not a trade to make by default.

### Two more defects found and fixed

Both were invisible in aggregate numbers and appeared only as regressions in tests that had nothing to do with the change:

1. **`order = 2` addresses are pair indices, not group elements** — the group inverse lookup indexed the 120-entry table with a value up to 14,399: an out-of-bounds panic in six tests. Now guarded to `order = 1`. **This was the cause of last round's five failures**, so the revert was correct but the cause is now understood and fixed.
2. **The filter's gradient was folded into the matrices' global clip norm**, which changed the clip applied to the matrices and altered every existing training result. The nine filter weights are now clipped in their own group.

### State

`geometric_attention` **19/19 pass**; `cargo fmt --check` clean; working tree clean. The learned filter and corruption augmentation are retained as **options** whose defaults are the previously verified behaviour, so nothing regressed.

### Next (unchanged order)

1. **General non-class group-algebra filter** — the filter is currently a class function (conjugation-invariant); a full group-algebra element is strictly more expressive and is the missing half of "harmonic compute".
2. **Learned packaging** of the token→element assignment (Stage-3 `H4^k` product codes over `nearest_h4_root`) — the representation change, which you flagged as possibly the higher-leverage one.
3. Then **BPE-4096 + real instruction corpus**.

**Ledger:** cumulative **144,223,565 / 147,200,000 ms** (~49.6 min remaining). No extension used. `References #820`.


### comment 5744372738 — 2026-09-19T18:31:34Z — Casey-allard

## General group-algebra filter: no gain — and that localises the bottleneck

Commit `d0927a3c`. Receipt §11: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

Step 1 of the recommended order was the missing half of "harmonic compute": the filter is now either a **class function** (9 slots, conjugation-invariant) or a **general group-algebra element** (120 slots, one per group element), selectable by `class_filter`. The general case contains the class case as a subspace, so it is strictly more expressive.

```
class filter:   slots=9   non_zero=3  clean=0.11
general filter: slots=120 non_zero=5  clean=0.11
```

**A 13× increase in filter capacity changes nothing.** Training uses 5 of 120 slots and reaches exactly the same clean accuracy; both remain far below the **fixed exact filter's clean 0.42** on the same budget.

This is a clean negative that does real work: **the filter is not the bottleneck.** Filter capacity, filter expressiveness *and* conjugation-invariance are all immaterial to clean accuracy here. Whatever limits the learned variants sits **upstream of the read** — in the stored representation or the readout. That is now the measurement's own conclusion rather than my preference, and it makes **learned packaging of the token→element assignment** the next step on evidence.

### Running tally of this session's mechanism findings

| Question | Answer |
|---|---|
| Can the dense low-bit recurrence hold memory? | No — i32 saturation in 16–32 steps; burial at delay ≥ 8; decay falsified |
| Does exact addressing beat a matched filter? | Yes, at every length — and ordered words reach **1.00** where the matched filter scores 0.00 |
| Is the read activation helping? | No — `relu` after an exact-address read destroys the retrieved sign half |
| Does the harmonic (Peter–Weyl) kernel do anything? | Yes — it recovers a corrupted address 0.13 → 0.23 |
| Can the filter be *learned*? | Yes, but it loses to a fixed exact filter (0.27 vs 0.42 clean); robustness is not in the objective |
| Does more filter capacity help? | No — 120 slots behave exactly like 9 |

### State and next

`geometric_attention` **20/20 pass**; `cargo fmt --check` clean; working tree clean; nothing regressed. New behaviour remains behind options (`class_filter`, `learn_kernel`, `corrupt_frac`) whose defaults are the verified behaviour.

**Next, on evidence now:** (1) **learned packaging** — the token→element assignment (Stage-3 `H4^k` product codes over `nearest_h4_root`), since the filter is ruled out as the limit; (2) **BPE-4096 + real instruction corpus** — the synthetic tasks are nearly exhausted as a signal.

**Ledger:** cumulative **144,513,565 / 147,200,000 ms** (~44.8 min remaining). No extension used. `References #820`.


### comment 5744521547 — 2026-09-19T18:57:08Z — Casey-allard

## Relational generalisation: the first task where the group is load-bearing

Commit `f1127069`. Receipt §12: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

Last round I argued that learned packaging **cannot be tested on a copy task** — relabelling group elements is a symmetry of the architecture, so with no collisions and no meaningful proximity, every injective assignment behaves identically. The task has to reward group *structure*. So I built one.

**Task.** Token `w < 120` has meaning = its own element of `2I`; token `120+j` is a relation whose element is `j+1`. The fact `(w, h_j) → compose(w, h_j)` is **defined by the group**, so an unseen pair still has a well-defined answer. Each sequence lists several facts and then queries one pair; the answer is **not adjacent** to the query and appears nowhere else in the prompt; held-out pairs are never used as a fact during training.

```
context lookup:      seen-query=0.39  unseen-query=0.00
composed dictionary: seen-query=0.39  unseen-query=0.42
```

**A context-address lookup scores 0.00 on unseen relational facts — it cannot generalise** (the falsification half). The composed read — package the vocabulary by meaning, then address `compose(elem(w), elem(h))` — answers unseen facts at **0.42 against 0.00**, with **no generalisation gap at all** (0.42 unseen vs 0.39 seen), because the query is *computed* rather than retrieved.

**This is the first result in the session where the group structure is load-bearing rather than decorative.**

### The remaining ceiling is the readout, and that is now the single binding constraint

Both mechanisms sit at the same ~0.4 on facts they can reach — the identical limit measured twice already: the fixed exact filter's 0.42, and the learned filter's inability to beat it. Composition removes the *generalisation* gap entirely; what remains is mapping 120 value vectors to 120 classes. Every mechanism tested this session now converges on the same wall.

### A task leak caught, and recorded

An earlier version of this task was a literal repeat of each fact. It scored **1.00 on "unseen" facts** — because the answer sat directly beside the query and could be copied. A leak, caught by a failing test. That is the **second task-design error this session**, and like the first it initially looked like a success.

### Tally

| Mechanism question | Measured answer |
|---|---|
| Dense low-bit recurrence holds memory? | No — i32 saturation in 16–32 steps; burial at delay ≥ 8; decay falsified |
| Exact addressing vs matched filter? | Exact wins at every length; ordered words reach **1.00** vs **0.00** |
| Read activation? | `relu` after an exact read destroys the sign half |
| Peter–Weyl kernel? | Works — corrupted-address recovery 0.13 → 0.23 |
| Learned filter? | Loses to a fixed exact filter; robustness isn't in the objective |
| More filter capacity? | No effect (120 slots ≡ 9) |
| **Composition vs lookup on unseen facts?** | **0.42 vs 0.00 — the group carries capability** |

### Next, on evidence

**Fix the readout.** It is now the one wall every mechanism hits: 120 value vectors → 120 classes is not reliably decodable at `dv = 64` with ternary weights. D0-b permits up to **4 bits per weight**, and the project's own **icosian/E8 quantiser** (Stage 6) is the natural way to spend that resolution. Prediction: readout resolution, not filter capacity or addressing, is what moves the ~0.4 ceiling — falsifiable by sweeping weight bits and `dv`.

Then BPE-4096 + real instruction corpus.

**Ledger:** cumulative **144,833,565 / 147,200,000 ms** (~39.4 min remaining). No extension used. `References #820`.


### comment 5744736423 — 2026-09-19T19:33:47Z — Casey-allard

## Four readout levers ruled out — and two more diagnostic defects caught

Commit `b3097ede`. Receipt §13: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

Last round I recommended fixing the readout on the theory that its **resolution** caused the ~0.3–0.4 ceiling. Falsify-first, that was tested *before* building anything: an **unquantised readout** is an oracle for "would more precision help?" — and D0-b permits up to 4 bits/weight, so a 4-bit shift-and-add readout would have been the right build if the oracle had shown headroom.

```
serving readout:    ternary=0.28  unquantised=0.28
relational budget:  steps=900 0.28 | steps=4000 0.28
read width:         dv=64 0.28 | dv=256 0.28
filter capacity:    9 slots 0.11 | 120 slots 0.11
```

**All four levers are flat.** Weight precision, training budget, read width and filter capacity each fail to move the ceiling. So the **4-bit kernel build is not recommended on this evidence**, and the reason is recorded rather than left implicit — that's the point of testing the cheap oracle first.

### Two diagnostic defects, both of which would have produced a wrong conclusion

1. **The first oracle changed only _training_**, while evaluation still went through the core's ternary readout — so it never tested serving resolution at all. Rebuilt to compare quantised vs unquantised **at evaluation, on the same trained masters**. *Third diagnostic defect this session.*
2. **The diagnostic dictionary included relation tokens**, whose elements (1..8) deliberately collide with word elements — corrupting eight of 120 buckets. Found and fixed.

### What this rules *in*

The ceiling is structural in the readout **mechanism** — a single linear map over one read vector decoding 120 classes — not in its width, precision, the filter, the addressing, or the budget. The natural next candidate is a **non-linear / table decode**: the project's own **nearest-root decoder** over the 120 canonical roots, which is exact, multiplier-free by construction (compare + table read, not a contraction), and is listed in the project synthesis under *exact finite geometric actions*. That is a different **kind** of readout, which is what the evidence now points at.

### Session tally

| Lever | Verdict |
|---|---|
| Dense low-bit recurrence for memory | Falsified (i32 saturation 16–32 steps; burial) |
| Exact addressing vs matched filter | Exact wins everywhere; ordered words **1.00** vs **0.00** |
| Read activation | `relu` after an exact read destroys the sign half |
| Peter–Weyl class-function kernel | Works — corruption recovery 0.13 → 0.23 |
| Learned filter | Loses to a fixed exact filter; robustness isn't in the objective |
| Filter capacity | Flat (120 slots ≡ 9) |
| Composition vs lookup on **unseen** facts | **0.42 vs 0.00 — the group carries capability** |
| Readout resolution / budget / width | **Flat — none is the constraint** |

Six defects were caught this session by tests rather than by reading; four of them initially looked like mechanism results.

**Tests:** `geometric_attention` **25/25**; `cargo fmt --check` clean; working tree clean.

**Next:** build the **nearest-root / table decode** readout — a different kind of decode, multiplier-free by construction. Then BPE-4096 and real text.

**Ledger:** cumulative **145,623,565 / 147,200,000 ms** (~26.3 min remaining). No extension used. `References #820`.


### comment 5744863152 — 2026-09-19T19:54:53Z — Casey-allard

## Open/closed route abstention works; the nearest-prototype decode is the fifth flat lever

Commit `d82d9f04`. Receipt §14: `docs/evidence/native_geometric_spherical_harmonic_kernel_2026-09-19.txt`.

I returned to the recommended track and folded in the two ideas from your routing discussion that could genuinely help. One worked; one is another flat lever.

### Open/closed route state — the good one

The state already carried a per-bucket **write count** and never used it. A bucket with count zero silently produced a uniform distribution — i.e. a **confident guess**. `logits_or_abstain` now returns `None` for a closed route:

```
route state: seen ok/abstain/wrong=25/0/39   unseen=0/64/0
```

**Every unseen query now abstains (64/64) with zero confident wrong answers**, and seen accuracy is unchanged at 0.39. That converts hallucination into honest abstention, and it matches your own chain-traversal rule — eviction must be *proved* before an abstention is offered. Here the proof is exact: the bucket was never written. It also gives the abstention a typed meaning (open / closed) rather than a threshold on a score.

### Nearest-prototype decode, in reduced form — flat

A nearest-prototype/table decode is `argmax_r (2·num·p_r − ‖p_r‖²)` — which is **linear in `num` with a per-class constant**. So the full nearest-root build is a linear readout **plus a bias**, and the cheap falsification was to add the bias first:

```
nearest-prototype reduced form: plain=0.28  with-bias=0.28
```

**Flat.** The decode's geometry is not the missing piece, so the elaborate nearest-root version is **not** worth building on this evidence — the same reasoning that retired the 4-bit kernel. Testing the reduced form first saved that build.

### Five levers ruled out

Weight precision, training budget, read width, filter capacity, decode geometry — all flat on the relational ceiling. It is not explained by any of them.

### What the next step must be

**Diagnose the failure structure, not tune further.** The read is provably correct — an unseen pair composes to a valid element that *is* in the dictionary — so the failure is in decoding one read vector into 120 classes. The next experiment should inspect structure rather than a scalar: is the read vector actually distinct for every word; are errors concentrated on a subset of the vocabulary; is the effective dimensionality of the read space far below `dv`? Those are cheap and they localise the mechanism.

**Tests:** `geometric_attention` **27/27**; `cargo fmt --check` clean; working tree clean; everything new is behind options defaulting to the verified behaviour.

**Ledger:** cumulative **145,888,565 / 147,200,000 ms — ~21.9 minutes of model time remaining.** The structural diagnosis above fits in that; anything larger needs an owner-recorded extension. `References #820`.



---
_Comment/review ids in this file: 5739467447, 5739602573, 5739641569, 5739744748, 5739780657, 5739823974, 5739839372, 5739881464, 5739901777, 5739947930, 5739955482, 5740040578, 5740138016, 5740620304, 5740901861, 5742970667, 5743373234, 5743556761, 5743619189, 5743916079, 5744067485, 5744272456, 5744372738, 5744521547, 5744736423, 5744863152_


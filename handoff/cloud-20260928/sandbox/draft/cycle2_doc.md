# Hyperbolic geometry, cycle 2: real hierarchies, the radius quantizer, code, and the Rust read

2026-09-26 · Requested by the owner · References #820 · Branch `claude/blissful-wozniak-girwwq` (kept separate from `main`)

**Status.** An evidence note, not a decision record. It continues the [geometric-attention note](geometric-attention-2026-09-26.md). The owner asked to keep advancing toward the geometric language model; this cycle tests hyperbolic geometry, the strongest result of that note, on real data, and brings it into the project's Rust model.

Labels: **Measured** (scratch code in the review sandbox, one seed unless stated), **Derived**, **Hypothesis**. Model code stays in Rust; the scratch experiments are not committed.

## 0. Findings

1. **Only hyperbolic keys learned a real hierarchy.** On the nested scopes of this repository's own Rust code, with 192 scopes stored:
   - hyperbolic keys with 16 dimensions answered 77.5% of the queries that need the hierarchy;
   - dot product answered 0.4%, even with three times the training;
   - Hamming codes and cosine did no better than the trivial rule (§2).
2. **The owner's quantizer works in hyperbolic space.** Keys stored as a quantized radius plus a quantized direction kept most of the full-precision accuracy (§3):
   - on the synthetic tree, 36 bits per key solved 96.7% of the non-root queries at 192 nodes, against 98.5% at full precision and 53.5% for full-precision dot product;
   - on the code tree, 36 bits kept 87% of full-precision hyperbolic accuracy and 68 bits kept 92%, while full-precision dot product solved almost none;
   - the radius is decisive: with a 3-bit direction code, 3 radius bits raised 35.4% to 81.2%.
3. **Hyperbolic attention helps language models most on code.** On this repository's code it beat dot-product attention by 0.020 bits/byte; on prose, by 0.005 on each of two seeds. Half the heads hyperbolic captured nearly all of the gain. At this tiny size a third recurrent layer still beat any attention layer (§4–§5).
4. **The project's Rust model now has an optional hyperbolic read** (commit `57a4c5e`).
   - Existing dot-product models are verified bit-identical.
   - The integer runtime refuses Lorentz models until an integer arcosh table exists (§6).
5. **A correction** (§1). In the tree task the root is always stored, so some earlier rows reflected only the always-root rule. The earlier synthetic headline holds; this note reports accuracy on the non-root queries.

## 1. A correction to the tree results: the always-root baseline

In the tree task the root is always stored, so a model that always answers with the root's value is right whenever no other ancestor of the query was stored. Measured on the exact evaluation data:

| Tree | Stored nodes 48 | 96 | 192 |
|---|---:|---:|---:|
| Synthetic (4-ary, depth 5) | 52.7% | 26.3% | 3.6% |
| Code scopes (this repository) | 91.9% | 84.2% | 72.2% |

Raw recall therefore has to be read against this baseline. This note reports **accuracy on the non-root queries**, which is what hierarchy-aware retrieval has to earn. Where a run predates that metric, the note gives the excess over the baseline, (recall − baseline) / (1 − baseline), labelled as an estimate.

Effect on the earlier note's §4b: its headline holds, because at 192 stored nodes the synthetic baseline is only 3.6% (hyperbolic 98.5% against dot product 54.1%). The runs that reported 52.7% / 26.5% / 3.8% (8-bit Hamming) or similar learned only the trivial rule. The note's §4b table now carries this baseline row.

## 2. A real hierarchy: the scopes of this repository's own code

**Data.** Rust sources of five crates (training, graph-format, router, integer, tokenizer), parsed into nested brace scopes below a root, crate and file level, ignoring braces in comments, strings and character literals: 6,284 scopes, 2,129 of them internal, depth up to 13, heavy-tailed branching (median 1 child, maximum 93). Task and model as in the earlier §4b: each query is a leaf scope, and the target is the value stored at its deepest stored ancestor.

**Measured** (48 · 96 · 192 stored scopes; one seed; "estimate" marks the excess over the always-root rule where the non-root metric was not recorded):

| Scoring | Width | Recall | Non-root accuracy |
|---|---:|---:|---:|
| Always-root rule (baseline) | — | 91.9% · 84.2% · 72.2% | 0% |
| Dot product | 4, 8, 16 | 91.7–91.8% · 84.2–84.3% · 72.2–72.3% at every width | 0.3% · 1.7% · 0.4% (16 dimensions) |
| Dot product, three times the training | 16 | 91.9% · 84.4% · 72.4% | 0.0% · 1.4% · 0.4% |
| Cosine | 8 | 86.1% · 80.3% · 70.0% | below the rule |
| Cosine | 16 | 90.3% · 84.6% · 75.0% | below the rule · ≈ 3% · ≈ 10% (estimate) |
| Hamming | 16 bits | 91.7% · 84.2% · 72.2% | ≈ 0% (estimate) |
| Hyperbolic | 4 | 94.5% · 89.2% · 75.7% | ≈ 32% · 32% · 13% (estimate) |
| Hyperbolic | 8 | 97.8% · 94.8% · 88.2% | **75.9% · 71.2% · 66.5%** |
| Hyperbolic | 16 | 98.6% · 96.9% · 92.9% | **83.2% · 81.4% · 77.5%** |

**Reading.**
- **Only hyperbolic keys learned the real hierarchy.** Dot product, Hamming codes and cosine learned the always-root rule and little or nothing more. Dot product stayed there with three times the training, so this is not a matter of training length.
- **Hyperbolic accuracy rose with width:** 13%, 66.5% and 77.5% of the non-root queries at 192 scopes, for 4, 8 and 16 dimensions.
- **The real hierarchy is harder than the synthetic one** (irregular branching, depth up to 13). At 8 dimensions and 192 stored nodes, hyperbolic keys solved 66.5% of the non-root queries here, against 98.5% on the synthetic tree.

## 3. The owner's quantizer on hyperbolic keys: a radius plus a direction code

Hyperbolic keys lose the hierarchy when coded as sign bits, because the radius carries the depth. The owner's original quantizer keeps the radius: store a quantized radius (log-spaced levels) and a quantized direction, and keep the query at full precision.

**Measured on the synthetic tree** (8 dimensions; accuracy on the non-root queries at 48 · 96 · 192 stored nodes):

| Hyperbolic key code | Bits per key | Non-root accuracy |
|---|---:|---:|
| Direction as sign bits, radius dropped | 8 | below the trivial rule |
| Sign bits + 2 radius bits | 10 | ≈ 5% · 4% · 2% (estimate) |
| 3-bit direction per coordinate, radius dropped | 24 | 60.4% · 51.9% · 35.4% |
| 2-bit direction + 3 radius bits | 19 | 64.2% · 60.0% · 46.1% |
| 3-bit direction + 3 radius bits | 27 | 87.1% · 85.0% · 81.2% |
| **4-bit direction + 4 radius bits** | **36** | **98.5% · 97.8% · 96.7%** |
| Full-precision hyperbolic keys | 128 (fp16) | 99.2% · 99.2% · 98.5% |
| Full-precision dot-product keys | 128 (fp16) | 87.7% · 77.1% · 53.5% |

**Measured on the code tree** (non-root accuracy at 48 · 96 · 192 stored scopes):

| Key code | Bits per key | Non-root accuracy |
|---|---:|---:|
| 16 sign bits, radius dropped | 16 | below the trivial rule |
| 16 sign bits + 3 radius bits | 19 | ≈ 15% · 14% · 7% (estimate) |
| 8 dimensions: 3-bit direction + 3 radius bits | 27 | 34.6% · 40.1% · 33.9% |
| **8 dimensions: 4-bit direction + 4 radius bits** | **36** | **65.4% · 64.8% · 58.0%** |
| **16 dimensions: 4-bit direction + 4 radius bits** | **68** | **79.2% · 75.3% · 71.4%** |
| Full-precision hyperbolic, 8 dimensions | 128 (fp16) | 75.9% · 71.2% · 66.5% |
| Full-precision hyperbolic, 16 dimensions | 256 (fp16) | 83.2% · 81.4% · 77.5% |
| Full-precision dot product, 4 to 16 dimensions | up to 256 | 0.3% · 1.7% · 0.4% (16 dimensions); 0.0% · 1.4% · 0.4% after three times the training |

**Reading.**
- **The radius is decisive.** With the same 3-bit direction code, 3 radius bits raised non-root accuracy at 192 nodes from 35.4% to 81.2%.
- **36 bits per key are enough on the synthetic tree.** They matched full-precision hyperbolic keys and far exceeded full-precision dot-product keys.
- **The code tree needs a little more precision, and gets it.** At 192 scopes, 27 bits kept about half of full-precision hyperbolic accuracy. 36 bits kept 87% of it (58.0% against 66.5%), and 68 bits at 16 dimensions kept 92% (71.4% against 77.5%). Full-precision dot product solved almost nothing.
- This is the owner's original quantizer, keep the radius and code the direction, used where the radius carries meaning.

## 4. Language models trained on code

**Setup.** As in the review's §6.3, but trained on this repository's Rust source bytes: 25.5 MB for training and 0.74 MB held out, split by file. Three layers of width 128; the first two are diagonal recurrences, and the third is either a third recurrence or a 4-head attention layer without position encoding. One seed.

| Layers 1–2 → layer 3 | Bits/byte, 512-byte windows | At the 128-byte training length |
|---|---:|---:|
| diagonal → diagonal (no attention) | **1.403** | **1.454** |
| diagonal → dot-product attention | 1.463 | 1.484 |
| diagonal → hyperbolic attention | 1.443 | 1.478 |
| diagonal → mixed: 2 hyperbolic + 2 dot-product heads | 1.445 | 1.479 |

**Reading.**
- Hyperbolic attention beat dot-product attention on code by 0.020 bits/byte (0.006 at the training length), about four times its edge on prose (§5). Code is hierarchical; this is the direction the tree results predict.
- Half the heads hyperbolic captured nearly all of the gain (1.445).
- At this size a third recurrent layer still beat any attention layer (1.403). Attention's job is recall at range, which these short-window byte models barely need.

## 5. Two seeds on WikiText

| Attention (third layer) | Seed 0 | Seed 1 | Mean | At the training length (seeds 0, 1) |
|---|---:|---:|---:|---:|
| Hyperbolic | 1.8990 | 1.8992 | **1.8991** | 1.9202, 1.9232 |
| Dot product | 1.9019 | 1.9057 | 1.9038 | 1.9220, 1.9285 |

Hyperbolic was better on both seeds, by 0.005 bits/byte on average: small but consistent. The seed spread was 0.0002 for hyperbolic and 0.004 for dot product.

## 6. The hyperbolic read in the project's Rust model

Commit `57a4c5e` on this branch (an agent in an isolated worktree wrote it; the lead reviewed the core and merged it).

- **Config.** `ReadGeometry { Dot (default), Lorentz }` on `JointConfig` (crates/uor-r4-integer/src/config.rs, re-exported by the training crate). The field defaults to Dot when absent and is never serialized for Dot, so existing configs, checkpoints and packed manifests keep byte-identical JSON, hashes and contracts. A Lorentz model is selected in the campaign JSON with `"read_geometry": "lorentz"` (the training CLI has no model flags).
- **Score.** Both read paths (full, and bounded admission) call one `read_scores` helper. Dot is the old expression. Lorentz lifts query and key to the hyperboloid, clamps z = q₀k₀ − ⟨q,k⟩ to at least 1 + 10⁻⁶, and scores −exp(`read.lorentz_log_beta`)·arcosh(z), computing z² − 1 as (z − 1)(z + 1) for precision near 1. Age bias, NoRead competition, value mixing and admission are unchanged. Lorentz checkpoints bind the geometry in their numerical contract.
- **Guards.** Quantization and packed export refuse Lorentz models, and the integer runtime refuses a Lorentz manifest with the typed `IntegerError::UnsupportedReadGeometry` before any contract, parameter or table access. No integer arcosh path exists yet.
- **Checks (Measured, by the agent on identical sources).**
  - Dot bit-identity: 41 SHA-256 digests recorded before the change matched exactly after it. They cover configs, contracts, initial parameters, outputs in both read modes, gradients, sessions, safetensors, quantized outputs, hard-model and hard-parameter files, packed reload, and the Full/Recent64/Orthant64/ExactCache64 admission policies.
  - Tests: integer 25/25 and training library 73/73, including 5 new ones (Lorentz forward/backward finiteness and gradients in both paths, an F64 reference for the score, Dot identity for absent metadata, the Lorentz checkpoint round trip, the integer refusal).
  - `cargo fmt --check` passes; no clippy warnings in touched files; the claim-wording check passes.
  - A trial merge with Codex's open PR #1398 had no textual conflicts and compiled. Integer 25/25 and training 75/75 passed when built with `UOR_BUILD_SOURCE_COMMIT` set, which one of #1398's new tests requires; without it that test fails on its own source-binding precondition, not because of this change.
  - Rerun by the lead in the branch checkout: the five new training tests and the integer crate (25/25) pass. The one compiler warning (an unused `mut` in joint_bounded_campaign.rs:103) is in a file this change does not touch and is already on `main`.
- **Not run.** Metal and accelerate backends, any real training or language evaluation with a Lorentz read (so there is no Rust model-quality result yet), and integer bundles against real artifacts.
- **Known issue for the first training run.** At initialisation all distances are about 5 with little spread, so the learned NoRead score takes most of the read mass (0.94–0.996 on the test fixture, against 0.05–0.62 for Dot). The first run should add a learned distance offset or initialise the NoRead bias lower.
- **Integer follow-up (Derived).** Ranking needs only z, since arcosh is monotone. The weights need exp(−β·arcosh z) = (z + √(z² − 1))^(−β), which an artifact-bound lookup table on quantized z can supply. Index log₂ z plus 8–10 mantissa bits, about 22K entries, with a separate table in z − 1 near z = 1, where arcosh ≈ √(2(z − 1)) is steep. Seal the tables like the existing exp, tanh and sigmoid tables, and test parity against the F32 path.

## 7. What this changes

1. **Hyperbolic heads become the default geometry of the read.** They are better on hierarchies and code and slightly better on prose, at the same cost: one Lorentzian inner product per candidate, with arcosh from a table.
2. **Keys are stored as a radius plus a direction code**, the owner's quantizer. About 36 bits per key at 8 dimensions kept most of the full-precision accuracy on both trees, against 128–256 bits for fp16 keys.
3. **Next measurements**, each small:
   - a. **The first Rust training run with the Lorentz read** on the D8 learner.
     - Add a learned distance offset, or initialise NoRead lower (§6).
     - Run a dot-product control at equal budget, three seeds.
     - Data: TinyStories on the owner's machine, or this repository's code here.
   - b. **The integer arcosh table** (§6), so the integer runtime can serve a Lorentz model.
   - c. **Hyperbolic admission.** Build trained advertisements (geometric-attention note §5) from Lorentzian centroids of radius-plus-direction codes, and test them at 4k–16k candidates.
   - d. **Three seeds** for the headline comparisons in §2 and §3.

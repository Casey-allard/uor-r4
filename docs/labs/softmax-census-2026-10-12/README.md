# The D23 softmax census: where normalized exponentials actually live, and the checker for compliance claims

References #2028 (owner direction of 12 October), [D23](../../integration/DECISIONS.md#d23---no-softmax-anywhere-in-the-active-project),
[#2251](https://github.com/UOR-Foundation/uor-r4/pull/2251) (the rule's delivery). Lab: Kimi, session
`kimi/s1`, 2026-10-12. Source work only: no training, no pod, no model run; claimed on
[#2028 comment 6124257305](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6124257305).

> **One line.** D23 says to inspect the complete executed output/objective/gradient path before claiming
> compliance. This record is that inspection for the whole project: **every active executed path — the Line A
> objective, the stack's serving mixture, the D11 serving engine, the native learner's objectives and the
> training aids — computes a normalized exponential today**, and there is an executable checker
> (`scripts/softmax_path_check.py`) so a compliance claim carries evidence instead of inspection alone.

## The instrument

`python3 scripts/softmax_path_check.py [--path NAME] [--self-test]` scans an executed path's source for
banned ops (softmax / log-softmax / log-sum-exp normalization / softmax cross-entropy / exp-normalize
forms), skipping test modules and comments, and prints CLEAN or the file:line hit list. Scalar exponentials
that are not a softmax (sigmoid/tanh gates, exp tables as tables) are recorded separately so a reviewer
confirms the classification rather than trusting it. Current verdict at this record's head:

| executed path | verdict | banned-op sites |
|---|---|---:|
| `stack-objective` (Line A `train` / `dialogue-train`, CPU + CUDA + Metal) | VIOLATIONS | 14 |
| `stack-serve` (host serving mixture and scoring) | VIOLATIONS | 14 |
| `d11-serve` (the integer engine) | VIOLATIONS | 7 |
| `native-learner` (Line B `native_geometric`) | VIOLATIONS | 21 |
| `training-aids` (surrogates, distillations, comparators) | VIOLATIONS | 63 |

The checker is the operative definition for compliance claims; the map below is the narrative reading of
where those sites sit and what each one does (full class-by-class enumeration in the session's sweep).

## The map

**1. The Line A objective (every `train` / `dialogue-train` run).** `StackModel::weighted_loss` → the
`CrossEntropy` op: `row_log_sum_exp` in the forward (`geometric_stack.rs:17764`), `exp(v − lse)` in the
backward (`:17947`); the same structure in the CUDA kernels (`cuda_stack_kernels.rs:414, 446`) and Metal
(`metal_stack_kernels.rs:314, 380`). The pointer arm's `PointerMixture` op is the same shape with the copy
mixture inside (`:18564-18574`, backward `:18778`). **There is no non-softmax objective in Line A today.**
The turn compiler's `softmax_ce` (`geometric_turn_compiler.rs:480-493`) is the compiler path's own.

**2. The read.** The fused read computes a causal row softmax with the NoRead slot inside
(`geometric_stack.rs:15995-16004`; CUDA `read_softmax_warp` at `cuda_stack_kernels.rs:1279-1331`; Metal
`metal_stack_kernels.rs:837-870`). **The softmax-free alternatives exist and are served**: the
rank/Hamming-rank/learned-rank weightings (`rank_weights`, "no exponential enters the forward weights",
`geometric_stack.rs:15585-15600`), the Q31 integer rank read in the D11 session (`stack/session.rs:227-275`,
`2495-2507`), and the export gate that already refuses to export a softmax read to the D11 engine
(`stack_export.rs:268-321`, with the parity test `tests/stack_softmax_free_export.rs`). They are a
configuration, not the default.

**3. The serving mixture (host).** `next_scores_pointer_scores` computes `lse = row_log_sum_exp(&logits)`
and mixes `(1−g)·exp(z−lse) + g·p_copy` (`geometric_stack.rs:11269-11273`) — the served model's `softmax(z)`
at the head; `read_span_probe` (`:10492-10497`) and the LSE-based scoring (`:11124, 18177`) likewise.

**4. The D11 engine.** The sealed `exp` table exists precisely to normalize at serving: read masses
(`model.rs:3086, 3590`), vocabulary masses (`:3118, 3811`), the served pointer mixture
(`stack/session.rs:2781-2817`), and the served read row (`:2511-2526`). The **sampler itself is clean**
(greedy argmax and the Q48 ticket draw, `sampling.rs:101-148` compute no normalized exponential) — but the
masses it draws from are softmax products (`softmax_into`, `model.rs:4541-4579`).

**5. Training aids.** Starling surrogate (softmax-over-kept gradient, `geometric_stack.rs:4458-4465`),
readout self-distillation (`:4416-4436`) and selector KL (`:4534-4549`), the compiler `Softmax`/
`SparseSoftmax` heads (`relation_compiler.rs:353-362`, `temporal_compiler.rs:186-554`), the κ teacher
(`kappa_llama.rs:909, 1228`), and the surrogate STEs across the geometric_* credit modules — all prohibited
by D23 as teachers, auxiliary losses or comparators, whatever their forward does.

**6. The native learner (Line B).** The Line-B objective family is softmax-shaped throughout:
`addressed_attention/training.rs:97-195` (categorical weights + CE with softmax derivative), the pooled
episode objective's log-mean-exp (`cross_state_completion.rs:640-718`), the low-bit/integrated/JEPA/prior
learners (`learner/*` — ~21 checker sites), and the blocked sampled law (`whole_path_learning.rs:1749`,
already BLOCKED under D23).

**7. Already softmax-free (the replacement inventory).** The rank-family reads and their D11 serving path
(above); greedy decoding; the copy-span oracle's uniform forced attention (`geometric_stack.rs:11245-11260`);
hard native bank/occurrence generation (`geometric_bank_generate.rs`, "no action softmax"); the A3 sieve in
`mqar-bench`; the E8 orbit memory's gate mixing (`e8_memory.rs`); the VSA integer rectified-quadratic scoring
(`vsa/attention.rs`); and rank/margin objectives — `rank_softplus` in the frozen-map-fit rank loss
(`cross_state_completion.rs:586-630`) and the raw-score margin API (`geometric_generate_learning.rs:1421`) —
which exist today only as diagnostics, not as wired objectives. Note for the replacement design: `softplus`
and `sigmoid` are scalar exponentials, which D23's carve-out does not automatically prohibit; a rank/hinge
objective family is the in-tree starting point that stays inside the rule.

**8. Retired scope (not in the active-path verdict).** `uor-r4-lut` (frozen comparator): 9 sites. The
`uor-r4-graph-*` crates: zero.

## What this means for the next source task

D23's replacement must cover, in one design: a softmax-free **objective** (class 1 — the largest gap; no
wired non-softmax objective exists), a softmax-free **output distribution** for serving (classes 3–4 — the
head and the pointer mixture), the **read** (already solved as a configuration, class 2), **sampling**
(clean, class 4/6), and the **training aids** (class 5 — every current surrogate/teacher is prohibited, so
starling-class reads need either a non-softmax teacher or none). The copy-span oracle and the rank objectives
are the only delivered building blocks that are clean by construction.

## Verification and limits

- The checker is a source scan, not a model run; it is evidence about source, and a compliance claim still
  needs the complete executed path reviewed per D23 (the checker's profiles are the named starting points).
- `--self-test` passes; the checker's per-path verdicts were cross-read against an independent line-by-line
  sweep of the same tree (the session's census sweep, ~150 read hits).
- Retired crates (`uor-r4-lut`, `uor-r4-graph-*`) are outside the active-path verdict by construction.

## Cost

| | |
|---|---|
| compute | laptop CPU only (source scan and review); no pod, no training, no model run |
| external | none |

## Next

The replacement learning law is the owner's next source task; when it lands, this checker is the compliance
gate for its TEST FITNESS review, and the Kimi lab's held runs (span oracle, bench re-baseline) re-register
against the compliant objective.

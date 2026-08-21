# S1 representation-redesign instruments — Q1 / D1 record (#822)

- **Issue:** #822 — "TRACKER/S1: persistent prompt-conditioned predictive state"
  (programme #820). The S1 stage verdict of 2026-08-21 is **REVISE**: the five-arm
  mechanism space at the current representation is exhausted
  ([`docs/prompt_arms_bakeoff_834.md`](prompt_arms_bakeoff_834.md)), and S1's
  remaining work executes the approved redesign RFC
  ([`docs/s1_redesign_rfc_822.md`](s1_redesign_rfc_822.md)) on the tracker itself.
- **Date:** 2026-08-21.
- **Status:** Q1 is ANSWERED (§2). The D1 instrument's run contract (§3, posted to
  #822 before the run) and its teacher-grounded outcome (§4) are recorded below.
  This record is append-only; the D2 instrument will extend it.
- **Claim language:** normative per
  [`docs/formal_vocabulary.md`](formal_vocabulary.md). Every labeled statement
  carries one claim class (**Definition**, **Objective**, **Guarantee**,
  **Assumption**, **Empirical Criterion**) and, where applicable, a status.
- **Execution scope. Definition.** Everything in this record is **reference-only /
  off-serving-path** in the sense of
  [`docs/conformance_execution_scope_830.md`](conformance_execution_scope_830.md):
  offline reference mathematics over the attested #833 bundle's recorded corpus,
  grounded in recorded teacher labels. No deployed-serving behavior changes; no
  serving claim is made or implied. Binds existing capabilities only — no new
  capability, no `model/ids.toml` row, no `CONFORMANCE.md` change.
- **Evidence files:** the D1 harness
  `crates/uor-r4-api/tests/joint_conditional_run_822.rs` (ignored; run with
  `cargo test -p uor-r4-api --release --test joint_conditional_run_822 --
  --ignored --nocapture`) and its CID-bound record
  [`docs/joint_conditional_822_result.json`](joint_conditional_822_result.json).

## 1. What does not move

The frozen gates cited by the RFC §1 bind every instrument in this record: the
20‰ `CAUSAL_FLOOR_PERMILLE` promotion gate on the #833 protocol; the ≥ 25‰
off-serving lower bound as the bar for **opening** a lowering track (Q3,
adopted 2026-08-21); the #886/#887 lowering calibration; and the deployed
invariants (P-4 operation classes, allocation-free steady state, bounded
capacities, deterministic bytes, typed errors, `no_std` boundaries, witness
replay). This instrument changes nothing deployed.

## 2. Q1 — what prefix did the recorded `t_argmax` labels consume? (ANSWERED)

**Question (RFC §4-D3).** If the teacher labels were produced from short
prefixes, every evidence table fits a suffix-conditioned teacher, and the
measured "suffix-locality" partly mirrors the observation protocol.

**Empirical Criterion (answered, Empirical).** Each recorded label is the
teacher's argmax after consuming the **full article-token prefix up to that
position — full causal attention over positions `0..=pos` — hard-capped at
`--sequence-length 128`** (the observe-text default). The deployed 8-token
`compiler::WINDOW` never bounds the teacher during observation; it keys only
the emitted sample (`sample_id`/`shard_of` routing).

Producer-code evidence (read at main `6fc46149`):

- `crates/uor-r4-graph-compiler/src/observation_text.rs::produce_article_records`:
  one `oracle.reset()` per article, then
  `for pos in 0..positions { oracle.step(tokens[pos], pos, &mut logits) }` with
  `positions = min(tokens.len() - 1, seq_len)`; the v4 record's top-8 (whose
  `[0]` becomes `Corpus::t_argmax` per `compiler::load_corpus_bytes`) is
  encoded from those logits. Teacher-forced: the sampled token is discarded,
  `next` is the actual next text token.
- `crates/uor-r4-model-source/src/lib.rs::Llama::layer_forward`: attention
  spans `st.att[h*seq_len .. h*seq_len + pos + 1]`, KV caches sized
  `n_layers x seq_len x kv_dim` — the whole cached prefix, every step.
- seq_len plumbing: `Teacher::load_with_sequence_length(source,
  options.sequence_length)` with the graph-CLI observe default
  `sequence_length: 128` (`.min(max_position_embeddings)` = 8192 for
  SmolLM2-360M-Instruct); the #833 run-contract wording concurs
  ("dense Simple-Wiki 0..2999 seq-len-128 re-observe").
- Doc correction shipped with this record: the `observation_text.rs` module
  header claimed the text stream is "tokenized BOS-prefixed"; the text path
  prepends no BOS (`TokenizerKind::encode_lossy` inserts no specials) — the
  BOS wording belongs to the autoregressive `observe` driver, which starts
  from `oracle.bos_token()`.

Corpus-byte confirmation (attested #833 bundle, `corpus.meta` CID
`blake3:aa9d1767...`, 360,924 x 88-byte records): `span_start` spans 0..=127
with exactly 2,994 records at `span_start = 0` (one per story); per-story
position counts max out at exactly 128 with **2,556/2,994 stories (85.4%) at
the cap**; mean label prefix length (`span_start + 1`) = **62.7 tokens**.

**Consequences.** (1) The labels are not window-conditioned: the teacher
consumed up to 16x the deployed 8-token key (mean ~63 tokens), so the measured
suffix-locality (#874/#875/#891/#894) is a property of the **compiled key
space**, not an observation-protocol artifact at the window scale — D1/D2
remain fully meaningful. (2) The label-side ceiling is **128 tokens of
document prefix**, not the teacher's native 8,192: conditioning beyond 128
tokens is absent from every recorded label. No contract in this series may
promise conditioning beyond that bound. (3) If D1 and D2 both report
sub-floor, the honest §6-step-4 recording is that the S1 claim as measured is
additionally bounded by <=128-token label conditioning, and the D3
re-observation decision goes to the maintainer with its own contract.

## 3. D1 — joint conditional keys: run contract (posted to #822 before the run)

**Objective.** Replace the bag's *unconditional* per-token tables with joint
conditional tables keyed by `(content token, 2-token suffix)` — learn
`P(answer | t in window, suffix)` residuals relative to `P(answer | suffix)`,
the quantity the #891 falsifier identified as never represented (subtracting
the *global* marginal added nothing: CR-vs-Ψ −1.3‰ [−2.4, −0.1]).

Pre-registered arms (λ fixed at 1.0 before evaluation; the λ-sweep is
exploratory, not the verdict): reproduction arms `base`/`psi`/`cr` on the
exact #891 candidate sets as harness gates; new arms `joint` (PRIMARY), `mix`
(the RFC-named backed-off fallback), `joint-narrow`, and the §4-D4 comparison
arms `d4pos`/`d4skip` on a shared widened candidate set (suffix ∪
content-top-32 ∪ joint-top-32); planted nulls `swap` (different-story
evidence under this window's suffix) and `keyshuf` (this window's tokens
under the swap partner's suffix key — the conditioning-specificity null).
Bounded tables throughout: cap 64 per key (#835 discipline).

```text
metric to move:      teacher-grounded held-out top-1 vs the suffix-local floor; base 246.6permille
                     (recorded #875/#891); best prior content arms: psi +17.5 [15.9,19.0], CR +16.2 [14.6,17.9]
reachability ceiling: base-miss mass = 1000-246.6 = 753.4permille overall; joint keys can fire only on
                     known-suffix positions (46,128/72,130 = 639permille of held-out; #893's 26,002
                     novel-suffix positions fall back joint->base, mix->psi-bag). Ceiling >> the 25permille
                     bar; the run decides.
instrument + verdict: in-run reproduction gates must PASS before the verdict binds — base 246.6+-0.05,
                     psi 264.1+-0.05, cr 262.8+-0.05, minimal-pairs total 4,722 exact, psi-follow 10,
                     cr-follow 13, base-follow 0 (docs/conditional_residuals_834_result.json); joint-table
                     anti-vacuity (nonempty, >0 covered positions); both nulls change >=1 prediction.
exit rule:           PRIMARY = joint-vs-base paired 95% lower bound. >=25.0permille -> SELECT (Q3-adopted
                     lowering-track opening bar; the 20permille floor unchanged as the promotion gate).
                     Fallback consultation order (pre-declared): mix consulted only if joint is below bar.
                     Lower bound <=0 AND 0 joint minimal-pair follows -> NO ARM. Otherwise REVISE —
                     the [20,25) floor-clearing-but-below-opening-bar case is recorded explicitly as such.
if positive:         file an #836-shaped lowering-track issue for the selected arm (built-capability order
                     judged at implementation; #886-style deployed-fidelity spot-check pre-planned). D2
                     still runs afterward — independent evidence for the same §6-4 decision.
if negative:         record here and on #822; proceed to the D2 instrument (RFC §6-3). §6-4 then consumes
                     Q1 + D1 + D2 together. The branches differ.
cost estimate:       offline, zero teacher compute; ~2 min table build + multi-arm eval over 72,130
                     positions, est. 10-30 min wall, ~2 GB peak, Mac-local. Blocks nothing.
```

## 4. D1 — outcome (teacher-grounded run, 2026-08-21): SELECT (backed-off mix)

The run completed 2026-08-21 (elapsed 35.9 s; bundle
`.uor-models/compiled/smollm2-360m-broad-clean`; result_cid
`blake3:41be22f04af4662277e65029e558442dfdc02dd93494db369765a8606ede0900`;
full numbers in
[`docs/joint_conditional_822_result.json`](joint_conditional_822_result.json)).

**Harness gates: PASS.** base 246.6‰, Ψ 264.1‰ (+17.5 [15.9, 19.0]),
CR 262.8‰ (+16.2 [14.6, 17.9]), minimal pairs 4,722 with follows Ψ=10 /
CR=13 / base=0 — all reproduce the recorded #875/#891 values exactly.
Double-run: 2,000 positions identical. Anti-vacuity: joint tables 1,429,685
keys (mass 2,118,411); joint coverage 46,128/72,130 positions — exactly the
known-suffix count, so the 26,002 novel-suffix positions have no joint
support, matching #893's decomposition; mean supported tokens 2.85 of 7.41
offered.

**Pre-registered verdict: `SELECT (backed-off mix)`** — the consultation
order was `joint` first, `mix` only because `joint` fell below the bar:

| arm | top-1 | paired Δ vs base [95% CI] |
|---|---|---|
| base (suffix floor) | 246.6‰ | — |
| joint (strict, PRIMARY) | 253.4‰ | +6.8 [+4.8, +8.9] |
| **mix (pre-declared fallback)** | **277.2‰** | **+30.6 [+28.6, +32.5]** |
| joint-narrow | 253.8‰ | +7.2 [+5.1, +9.2] |
| d4pos (comparison) | 217.9‰ | −28.7 [−31.8, −25.7] |
| d4skip (comparison) | 296.2‰ | +49.6 [+46.8, +52.4] |
| swap null | 213.4‰ | −33.2 [−35.3, −31.1] (changed 33,360) |
| keyshuf null | 130.4‰ | −116.3 [−118.9, −113.7] (changed 39,734) |
| trivial prior | 49.1‰ | — |

**Empirical Criterion (met, Empirical).** The mix arm's paired lower bound
**28.6‰** clears the Q3 opening bar (25‰) and sits entirely above the frozen
causal floor (20‰) — the first arm in the S1 record whose paired lower bound
does either, off-serving. The λ-sweep is flat around the pre-registered
λ = 1 (250/253/248/239/230 for λ = 0.5/1/2/4/8 on the strict joint arm) —
no tuning cliff. Minimal pairs: joint follows 83 and mix 78 of 4,722
(vs Ψ's 10, CR's 13, base's structural 0) — same-suffix disambiguation is
real and an order of magnitude beyond the bag arms.

**Reading (why the strict arm is sub-floor while the mix clears).** The
strict joint arm can only fire on the 639‰ known-suffix slice and pays base
everywhere else; its supported-token mass is thin (2.85/7.41). The mix keeps
the joint conditionals where they exist and the Ψ-bag where they do not, so
it collects both the conditioning gain and the bag's novel-suffix reach. The
two nulls certify the mechanism: foreign content under the true suffix
(swap, −33.2‰) and true content under a foreign suffix (keyshuf, −116.3‰)
both collapse, so the (token, suffix) PAIRING — not candidate widening or
table mass — carries the signal. The #891 falsifier is thereby answered: the
useful subtrahend was the suffix-conditional, and representing it turns the
conditioning increment positive.

**Comparison-arm note (recorded, not a verdict).** `d4skip` — the milder
(token, last-token) conditioning — measured **+49.6‰ [46.8, 52.4]** on the
shared candidate set: denser support (a 1-token conditioning key) beats both
the 2-token joint and the mix. This is design input for the lowering track
(key granularity is a free parameter of the same mechanism class); it was
pre-registered as a comparison arm only, sits outside the SELECT consultation
order, and does not alter the verdict. `d4pos` (distance-bucketed unigrams)
is strongly negative (−28.7‰): order structure without conditioning hurts.

**Contract disposition (positive branch taken).** The #836-shaped lowering
track is filed as **#897** (trigger-gated on the RFC §6-4 decision point;
the 20‰ floor stays the promotion gate end-to-end; the key-granularity
question — 2-token joint vs 1-token d4skip-style conditioning — is named
there as the first design decision). D2 (region-conditional evidence) still
runs per §6-3; §6-4 then consumes Q1 + D1 + D2 together.

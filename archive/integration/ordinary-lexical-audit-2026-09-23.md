# Decisive audit of the ordinary/grounded lexical artifact — September 23, 2026

This is an independent, focused follow-up to the [principal review](ordinary-lexical-principal-review-2026-09-23.md)
of [PR #1353](https://github.com/UOR-Foundation/uor-r4/pull/1353) and
[PR #1354](https://github.com/UOR-Foundation/uor-r4/pull/1354). It **preserves** the sealed roots
`ordinary-lexical-1/2/3` and their artifact `fe3e8a638cd50a19741065cb00ac787b63bb59f8ad5776fd8bfcb162c37a912a`.
All new evidence is written to newly claimed roots (`olx-audit-1..5`, `olx-channel-probe-1/2`,
`olx-channel-1`). The **model source `learner/transferable_lexical.rs` is unchanged**
(`sha256 03f83eb8366d8fa710637e1941a2639c96ef603968cb0e8b31ce86d56fb2b250`, the principal's reviewed hash);
only the runner `crates/uor-r4-core/src/bin/ordinary-lexical.rs` gained two modes, `--audit` and
`--local-channel`, which load the sealed artifact **from bytes in a fresh process**. The delivered result
and its dated next step remain historical; this audit narrows claims, it does not create a new capability.

The original audit record is the sealed root
`/Users/casey.allard/uor-r4/.uor-models/realtext-prior-2026-09-20/olx-audit-4`
(receipt `sha256 a313c9f7739ffe64acf3d0c031ae6b51661c62502cd08ef31286d28f673b8618`, manifest
`sha256 8997120905d937a5f3b5e21d91da33ae8602befba07079673f39ca5dac52126e`, `0` unlisted files), produced by
executable `sha256 1cb7dce2cfc8271e003d6ec4d031616a1687ee53261dcacc4bf34ee4645b9ab4` from runner source
`2ebf3454745b346d190789af3e5fa975c3391bad76d5b372197d3aeaf2ed7f73`. `olx-audit-1/2/3` are preserved
earlier attempts of the same audit under intermediate source revisions.

**Principal reporting correction:** the separate sealed `olx-audit-5` replay preserves the same
artifact and panels and computes Generate-conditional NLL by renormalizing over Generate rows. Its
receipt is `sha256 2fd33edf787b53c771df0c7d7c4fd2f7c87ce2894d2faf1a4d427affdcb00647`, manifest
`sha256 026b86d8e00415d2e609ec2ba186b554b7d95c03257032f056dc1115efd7374a`, with zero unlisted
files. It was produced by executable `sha256 9a74fa3c9fff1e42e4fde0f79765f2127b267f51d4750bb34d28bc30c7f76d89`
and corrected runner source `sha256 2ffa551f568c29c37c16d353410b75a6e118bd5efd795a954c0c79d08f2da386`.
The earlier sealed reports remain immutable; their `token_conditional_bits_per_target` label actually
meant **Generate-action NLL**, because `score_example` normalizes Generate targets over legal Generate
and Stop rows.

## 1. The saved artifact replays end to end in a fresh process

A fresh process deserialises `artifacts/model.tlx` through `TlModel::from_bytes` and executes **every**
qualification panel on that loaded instance, then compares each row with the stored seal by
`served_actions`, `served_tokens` and `state_digest`:

| Panel (on the deserialised artifact) | Result vs stored |
| --- | --- |
| authored training panel (32 temporal + 4 class) | **36 / 36 identical** |
| held-out compositions | **3 / 3 identical** |
| class panel (both swaps) | **4 / 4 identical** |
| preflight A | **32 / 32 exact** (route-key rule contradicted on 16) |
| preflight B | identical to stored |
| prose Generate-action bits/target | **7.372198274238398 == 7.372198274238398** |
| generation tokens (3 prompts) | identical |
| mixed session phases | identical |

The principal review's lifecycle objection — the headline panels were evaluated on the *in-memory*
quantised model while the child process replayed only 36 fitted cases — is now closed: the serialised
artifact, reloaded in a separate process, reproduces the in-memory instance exactly on every panel.
This is a serialization/reproducibility result, not a language-quality result.

## 2. The post-copy decision depends on the fingerprint, not on the emitted token

The delivered preflight B changed the evidence fingerprint `m` **and** the copied-token feedback
together, so its 2/2 did not isolate dependence on the token emitted by `Copy`. The audit holds the
selected-source fingerprint, typed facts, prefix and the forced `Copy` action fixed and changes **one**
variable at a time. On the loaded artifact:

| Arm (forced `Copy`, length-1 owned span) | Decision |
| --- | --- |
| actual A (fingerprint A, feedback A) | `Generate(808)` |
| actual B (fingerprint B, feedback B) | `Generate(1361)` |
| **swap feedback only** (fingerprint A, feedback B) | `Generate(808)` |
| **swap fingerprint only** (fingerprint B, feedback A) | `Generate(1361)` |
| no-copy control (same feedback, preceding action `Generate`) | `Generate(808)` |
| identity erased, A / B | `Generate(583)` / `Generate(583)` |

For this token pair and top-1 decision, classification is `decision_tracks_fingerprint = true`,
`decision_tracks_feedback = false`, `no_copy_event_matches_actual = true`, and
`identity_erased_alias = true`. **The post-copy top-1 choice follows the selected-source evidence
fingerprint `m` and does not change when only emitted-token feedback is swapped in this pair.** This
does not prove that feedback never changes lower-ranked logits or another context. The delivered 2/2
is therefore a content/fingerprint result, not a demonstrated role for emitted-token feedback.
The class words remain declared labels, not semantic
claims, and the identity-erased arm is a diagnostic, not a serving mode.

## 3. The prose deficit is not the Stop denominator, clamp saturation, or lost local identity

Measured on the same 5,376 development targets with explicit denominators:

- **full-action NLL = Generate-action NLL = 7.372198 bits/target** over 5,376 scored Generate
  actions. Their equality is automatic because the development windows contain no Stop targets;
  it was **not** an independent denominator control. The corrected, genuinely
  **Generate-conditional NLL is 7.371855 bits/target**, only 0.000343 lower. All are measured
  on the same loaded artifact and 5,376 development targets.
- **mean Stop probability 2.28e-4**; `Stop` is never chosen on development. The measured
  0.000343-bit normalization difference is far smaller than the gap to the token-only references;
  Stop normalization is **not** the material deficit on this panel.
- **hidden-state clamp saturation 0.0** and **fingerprint clamp saturation 0.0** — clamping is not the
  cause.
- Generate-action loss **by context frequency** (fit-split count of the exact `(prev, cur)` context):
  8.49 / 8.41 / 7.74 / 7.05 for counts 0 / 1 / 2–4 / 5+ — higher at rare contexts but elevated in every
  bucket, so the deficit is not confined to unseen contexts.
- Generate-action loss **by position** (bucket `i/8`) is roughly flat between 7.10 and 7.59.
- **local identity**: swapping only the penultimate token (same last token) changes the state digest
  **100 %** of the time and the top-1 next token **56 %** of the time; swapping the last token changes
  top-1 93 %. These sampled swaps show sensitivity, **not** injective retention of exact token IDs;
  they refute the stronger assertion that the state/readout is wholly blind to the penultimate token.
- **checkpoint before the final grounded-only phase: UNAVAILABLE** — the runner never persists
  intermediate training state, so phase-2/phase-3 checkpoint comparison is not possible.

## 4. A matched bounded local-channel comparator does not close the gap

One ordinary learner (the retained `TlModel`/`TlTrainer`), one split, one schedule, one seed, one
architecture; the **only** change between arms is the bounded local channel handed to the readout
(none, versus the last two tokens with the retained fixed order rotation). Both arms train on the same
full-context task and are scored on the same 5,376 development targets (2,000 steps, batch 224, lr 0.02,
seed 13 — matching the delivered run's step count and targets-per-step). Sealed root
`olx-channel-1` (receipt `sha256 9a631d20dbcde8f7bf404d2b690ec701a424f4bd6f3d1a3c3ca5c92184a74514`):

| Arm (same data, steps, batch, seed, architecture) | dev bits/target | gain vs count (ref − arm) |
| --- | ---: | --- |
| tuned interpolated `(prev,cur)` count reference | 5.1217 | — |
| delivered artifact (window objective, prefix recurrence) | 7.3722 | −2.2505 |
| **recurrence-only** (full-context per-position objective) | **6.2039** | −1.0822 [−1.2822, −0.8957] |
| **recurrence + direct two-token channel** | 6.6732 | −1.5515 [−1.7519, −1.3613] |

The direct two-token channel makes development loss **worse** by 0.469 bits
(two-token − recurrence-only CI `[+0.410, +0.527]`, excludes zero), and both arms remain behind the
count reference. On this matched evidence the principal's "give the shared path a direct bounded
last-two-token channel" hypothesis is **not supported as a sufficient remedy in this training
configuration**. The channel may be redundant or harder to optimise; this comparison does not
distinguish those causes or retire the mechanism family. The sealed `olx-channel-1` receipt calls its
negative recurrence-only-minus-two-token interval `two_token_minus_recurrence_only`; the interval
above reverses that mislabeled sign without altering the original receipt.

The informative positive is the **recurrence-only** arm: a plain training-formulation change (per
position, full preceding context, larger target batch, no grounded-only final phase) recovers about
**1.17 bits/target** on the same targets against the delivered 7.3722 — **without any geometry**. The
delivered deficit is therefore substantially an optimisation/curriculum/batching effect, exactly the
cause the principal said to repair first. **Disclosed difference:** this per-position formulation feeds
the preceding tokens with the `OBSERVE` event throughout, whereas the delivered window objective feeds
them `OBSERVE` for the frozen prefix and `GENERATE` afterwards; the two arms are self-consistent and
matched to each other, but their absolute values are not a like-for-like replay of the delivered path.
This arm is a **prose-only diagnostic** learner: it does not retain the grounded Copy/Stop behaviours
and is **not** a successor artifact.

Timing on separate samples, kept separate from energy: rollout-only 702.8 µs/token, measured
tokenize + rollout + decode 695.2 µs/token (sampling noise makes the latter lower; it also excludes
source selection and session bookkeeping), dense packed-coefficient inspections
611,814 per generated step, artifact 466,711 B. Energy remains **UNAVAILABLE**.

## Limits and claim boundaries

The class words are declared labels; the identity-erased and no-copy arms are diagnostics. The
local-channel arms are prose-only and carry no grounded behaviour. `--audit` and `--local-channel` are
new **source**; the artifact-producing model source is unchanged, so the delivered artifact and its
version remain valid. The `olx-channel-1` seal records its source revision label but **not** the exact
executable hash, because the shared target binary was rebuilt afterwards for the audit seal; the audit
seal records the executable hash. Sampling, donor adaptation through a compatible emission interface,
broader conversation, executed Rust and whole-path energy remain open. No general prose, chat, reasoning
or geometric-advantage claim follows.

## One evidence-supported next step

Do **not** add geometry yet. Repair the shared path's training formulation first: train the retained
`TlModel` on the per-position, full-context objective with the larger target batch **while keeping the
grounded Copy/Stop supervision**, confirm on open development that it retains the ~1.17-bit prose
recovery and still passes the authored/held-out panels, and only then attempt the remaining gap to the
retained count reference. A signed H4/shared relative-transport operator becomes the first geometric
candidate **only** on a witnessed order/role/scope or distant-interference failure, and only against an
information- and compute-matched ordinary control.

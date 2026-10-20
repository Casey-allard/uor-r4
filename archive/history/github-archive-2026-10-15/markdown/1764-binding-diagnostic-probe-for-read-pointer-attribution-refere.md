# #1764 — Binding diagnostic probe for read/pointer attribution (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T21:53:28Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-10-05T21:59:51Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1764
- **Merged:** 2026-10-05T21:59:51Z
- **Merge commit:** 4a1ebd0b5bf55a4bd9a10ca1d41c53945b58a47b
- **Base / Head:** main / UOR-Foundation:claude/binding-probe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Diagnostic only: no model, training or serving change. Owner decision: "diagnose binding, then fix" (#820 comment 6003048397).

## What it adds

- `StackModel::read_span_probe(ids, spans)` (`geometric_stack.rs`). At the last position of one window it returns:
  - every geometric read layer's per-head attention mass on declared source sets;
  - the pointer gate and the pointer's full attention;
  - the generator softmax and the decoded mixture.

  Each source set is an auxiliary value channel. It shares the fused read's exact scores, age bias, admission and NoRead normalization, as the existing binding label does, and it is removed before `read.out`. So hidden states and logits do not change. `BindingCapture.target` becomes `Option`, with no behaviour change for the existing label paths.
- `binding_probe.rs`: the pure parts.
  - Word and token span matching over a decoded window, using chat-grade's word rule.
  - The `exact` check parser and pass rule.
  - Asked-key extraction.
  - Disjoint span priority.
  - The **classification rule**. It was fixed in commit 497a2f3, before any model was probed, and is documented in the module.
- `binding-probe` binary. It answers each `exact` memory row exactly as `chat-grade reply` does: `reply_panel` over `greedy_reply`, with the model's own earlier replies in the history. It probes two positions:
  - the **marker**, where the history ends in the assistant marker;
  - the **decision** query: the history plus the greedy reply up to the first expected or distractor value token.

  It classifies at the decision query, or at the marker when the reply names no value. `replies=` cross-checks the regenerated replies against a chat-grade record.

### Pre-registered rule

`E` / `D` is the maximum attention mass on the expected / distractor span over all 4 read layers × 16 heads and the pointer. `o_E` / `o_D` is the decoded probability of a spelling's first token. τ = 0.10.

- Mass pattern:
  - `expected` if `E ≥ τ` and `E ≥ D`;
  - `distractor` if `D ≥ τ` and `D > E`;
  - else `neither`.
- Miss classes:
  - (c) readout if the pattern is `expected`;
  - (a) binding-swap if the pattern is `distractor`, or if it is `neither` and `o_D > o_E`;
  - else (b) unreached.
- Mapping (owner's): majority (a) → Step 7a read-key content; (b) → Step 7b exact learned join; (c) → copy-gate/readout fix.

## Checks

- `cargo test -p uor-r4-training --lib read_span_probe binding_probe binding`: all pass (1 + 4 new, 15 existing binding tests).
  - The model test checks that the probe equals `read_binding_masses` for sets before the query on every read layer and head, with and without a pointer.
  - It checks that the probe's mixture equals `next_scores`.
  - The toy-history test covers span matching: split tokens, repeated keys, region restriction, disjointness.
- Release CUDA build at 497a2f3 on 1× RTX 5090. Regenerated last replies equal the stored Step 6c/6 chat-grade replies on every compared row: v4 40/40 for C-s1 and C-s2 (no stored v4 replies for A), v3 30/30 for all four.

## Results (Step 6c C-s1/C-s2, Step 6 controls A-s1/A-s2; 96M `rrarrarrarrarr`, width 1024, context 384, pointer dim 32 Dot)

#### conversational-v4 (blind), 40 exact memory rows

| model | pass | miss | (a) swap | (b) unreached | (c) readout | majority | misses naming distractor / neither | (a,b,c) at tau 0.05 | at 0.25 | pass mass pattern E/D/N |
|---|---|---|---|---|---|---|---|---|---|---|
| C-s1 | 24 | 16 | 7 | 0 | 9 | c_readout | 2 / 14 | 7/0/9 | 6/2/8 | 22/2/0 |
| C-s2 | 21 | 19 | 9 | 0 | 10 | c_readout | 4 / 15 | 9/0/10 | 10/2/7 | 21/0/0 |
| A-s1 | 12 | 28 | 13 | 1 | 14 | none (tie) | 5 / 23 | 12/0/16 | 13/8/7 | 10/2/0 |
| A-s2 | 13 | 27 | 19 | 2 | 6 | a_binding_swap | 2 / 25 | 19/1/7 | 14/12/1 | 12/1/0 |

Read mass at the classified query, mean over rows of head-mean / head-max (layer 11 = last read; layers 2/5/8 are below 0.04 head-mean on either value everywhere):

| model | set | L8 expected | L8 distractor | L11 expected | L11 distractor | L11 question | L11 NoRead | pointer gate | pointer on expected | pointer on distractor | pointer top = expected |
|---|---|---|---|---|---|---|---|---|---|---|---|
| C-s1 | misses | 0.010 / 0.116 | 0.011 / 0.101 | 0.070 / 0.427 | 0.061 / 0.321 | 0.264 / 0.860 | 0.396 / 0.999 | 0.029 | 0.092 | 0.196 | 0.06 |
| C-s1 | passes | 0.032 / 0.432 | 0.014 / 0.160 | 0.275 / 0.952 | 0.050 / 0.299 | 0.071 / 0.444 | 0.459 / 0.999 | 0.271 | 0.849 | 0.107 | 0.88 |
| C-s2 | misses | 0.006 / 0.070 | 0.012 / 0.162 | 0.068 / 0.366 | 0.074 / 0.354 | 0.263 / 0.812 | 0.357 / 0.999 | 0.083 | 0.119 | 0.166 | 0.11 |
| C-s2 | passes | 0.034 / 0.394 | 0.020 / 0.196 | 0.297 / 0.922 | 0.042 / 0.277 | 0.092 / 0.453 | 0.432 / 0.999 | 0.213 | 0.756 | 0.078 | 0.76 |
| A-s1 | misses | 0.014 / 0.136 | 0.018 / 0.177 | 0.033 / 0.247 | 0.055 / 0.225 | 0.402 / 0.892 | 0.379 / 0.996 | 0.104 | 0.000 | 0.134 | 0.00 |
| A-s1 | passes | 0.039 / 0.481 | 0.013 / 0.093 | 0.210 / 0.861 | 0.078 / 0.405 | 0.095 / 0.450 | 0.483 / 0.999 | 0.025 | 0.367 | 0.044 | 0.33 |
| A-s2 | misses | 0.006 / 0.067 | 0.007 / 0.060 | 0.016 / 0.098 | 0.026 / 0.159 | 0.500 / 0.959 | 0.311 / 0.996 | 0.035 | 0.003 | 0.166 | 0.00 |
| A-s2 | passes | 0.036 / 0.448 | 0.020 / 0.125 | 0.188 / 0.808 | 0.052 / 0.331 | 0.118 / 0.433 | 0.436 / 0.994 | 0.166 | 0.600 | 0.091 | 0.62 |

#### conversational-v3 (development), 30 exact memory rows

| model | pass | miss | (a) swap | (b) unreached | (c) readout | majority | misses naming distractor / neither | (a,b,c) at tau 0.05 | at 0.25 | pass mass pattern E/D/N |
|---|---|---|---|---|---|---|---|---|---|---|
| C-s1 | 15 | 15 | 9 | 0 | 6 | a_binding_swap | 8 / 6 | 9/0/6 | 9/1/5 | 13/2/0 |
| C-s2 | 20 | 10 | 7 | 1 | 2 | a_binding_swap | 6 / 4 | 7/1/2 | 7/1/2 | 20/0/0 |
| A-s1 | 6 | 24 | 13 | 1 | 10 | a_binding_swap | 7 / 17 | 14/0/10 | 11/9/4 | 5/1/0 |
| A-s2 | 10 | 20 | 15 | 3 | 2 | a_binding_swap | 6 / 14 | 14/0/6 | 15/3/2 | 8/2/0 |

Read mass at the classified query, mean over rows of head-mean / head-max (layer 11 = last read; layers 2/5/8 are below 0.04 head-mean on either value everywhere):

| model | set | L8 expected | L8 distractor | L11 expected | L11 distractor | L11 question | L11 NoRead | pointer gate | pointer on expected | pointer on distractor | pointer top = expected |
|---|---|---|---|---|---|---|---|---|---|---|---|
| C-s1 | misses | 0.011 / 0.128 | 0.024 / 0.285 | 0.120 / 0.527 | 0.159 / 0.591 | 0.145 / 0.621 | 0.413 / 0.999 | 0.387 | 0.200 | 0.612 | 0.20 |
| C-s1 | passes | 0.027 / 0.391 | 0.015 / 0.182 | 0.276 / 0.895 | 0.053 / 0.252 | 0.090 / 0.565 | 0.448 / 0.999 | 0.398 | 0.746 | 0.119 | 0.73 |
| C-s2 | misses | 0.008 / 0.091 | 0.019 / 0.193 | 0.088 / 0.408 | 0.127 / 0.496 | 0.186 / 0.597 | 0.420 / 1.000 | 0.189 | 0.270 | 0.325 | 0.30 |
| C-s2 | passes | 0.030 / 0.381 | 0.016 / 0.178 | 0.316 / 0.960 | 0.030 / 0.188 | 0.081 / 0.403 | 0.438 / 0.999 | 0.453 | 0.912 | 0.036 | 0.90 |
| A-s1 | misses | 0.010 / 0.104 | 0.012 / 0.099 | 0.043 / 0.254 | 0.069 / 0.306 | 0.346 / 0.829 | 0.385 / 0.997 | 0.081 | 0.078 | 0.137 | 0.08 |
| A-s1 | passes | 0.033 / 0.362 | 0.010 / 0.089 | 0.249 / 0.876 | 0.053 / 0.262 | 0.143 / 0.597 | 0.426 / 0.998 | 0.014 | 0.764 | 0.219 | 0.83 |
| A-s2 | misses | 0.005 / 0.039 | 0.010 / 0.087 | 0.020 / 0.124 | 0.056 / 0.253 | 0.410 / 0.871 | 0.349 / 0.996 | 0.084 | 0.085 | 0.157 | 0.10 |
| A-s2 | passes | 0.028 / 0.331 | 0.016 / 0.105 | 0.211 / 0.821 | 0.079 / 0.427 | 0.092 / 0.429 | 0.474 / 0.998 | 0.354 | 0.734 | 0.133 | 0.80 |

**Verdict under the pre-registered mapping.**

- On blind v4, both candidate models have a narrow (c) readout majority:
  - C-s1 9/16, C-s2 10/19;
  - pooled a 16 / b 0 / c 19 of 35.

  The mapping therefore points to a **copy-gate/readout fix**.
- The distribution is close to even between (a) and (c). (b) unreached is about 0 for C at τ = 0.05 and 0.10, and 2 at 0.25. Step 7b's premise, that the value is not reached, is therefore not supported.
- The controls differ: A-s1 is tied, A-s2 is (a). On development v3, (a) is the majority for every model.

**Scope and caveats.**

- Most C misses on v4 (14/16 and 15/19) name neither value, for example " Hamid ate.", " Yours.", " The zoo is at the zoo.". They are therefore classified at the marker. There, "readout" means: the layer-11 read had already reached the expected span, but the reply never produced a value.
- In those misses the pointer is effectively off:
  - gate 0.03–0.08;
  - pointer mass on the expected value 0.09–0.12, less than on the distractor (0.17–0.20).

  In passes the gate is 0.21–0.27, with 0.76–0.85 on the expected value.
- Layers 2/5/8 put almost no mass on either value. Binding, where it happens, is in layer 11, plus partly layer 8 in passes.
- The max-over-64-heads statistic almost never yields `neither`. Several marker classifications are close calls, for example mem-04 with D 0.15 vs E 0.13. The full per-row JSON keeps every layer, head and kind.
- In 4 v4 rows and 6 v3 rows, an expected and a distractor spelling share a first-token candidate. `o_E` vs `o_D` cannot separate them there, but this only matters for the `neither` branch.

## Illustrative rows (C-s1, v4)

- **(c) readout, mem-40.** Tent "63", then corrected to "38". Asked: which tent this year. Reply " Yours.".
  - At the marker, layer 11 head 13 puts 0.89 on "38".
  - The pointer puts 0.995 on "38", but its gate is 0.006.
  - The greedy token is " Your".
- **(a) binding-swap, mem-11.** Odette's boat is painted turquoise and called Pelican. Asked: the boat's name. Reply " Odette is named turquoise.".
  - At the decision query, layer 11 head 9 puts 0.88 on "turquoise".
  - The pointer puts 0.998 on "turquoise" (gate 0.025).
  - `o_D` = 0.66.
- **(a) by a close call, mem-04.** "Hamid ate a burrito…", then "won a stuffed walrus". Asked: what did Hamid eat. Reply " Hamid ate.".
  - At the marker, D 0.15 vs E 0.13 on the best heads.
  - The pointer's top source is the asked key " H" (0.69).

## Artifacts and cost

- Reports are sealed `report_output` roots at `/workspace/uor-r4/diag-binding-20261005/{C-s1,C-s2,A-s1,A-s2}-{v4,v3dev}/` on the EUR-NO-1 volume, plus `*.log`.
- The `*-v3` roots are sealed error records. v3 has 160 requests, above the 128 limit, so it was rerun as `v3dev` from the a/b split.
- A copy is at `icloud:UOR-R4/results/claude/diag-binding-20261005.tar` (md5 12718c88…).
- Compute: 1× RTX 5090 pod in EUR-NO-1 for about 8 minutes (the probe runs took about 2 minutes), plus about 1 minute on a pod the tool deleted after a failed bootstrap. About $0.17 in total.
- The v4 panel files come from branch `claude/panel-v4`; they are not on main yet.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6003902691 — 2026-10-05T21:58:58Z — Casey-allard

Exact-head check at 497a2f35 (Claude lab): rustfmt clean on all four touched .rs files (the only rustfmt diff reachable via lib.rs is the pre-existing relation_compiler.rs formatting on main, untouched here); `cargo test --release -p uor-r4-training --lib binding` 19 passed. The probe method is read-only (no forward-pass change) and its regenerated replies equal the stored chat-grade replies on every compared row (v4 40/40 C-s1/C-s2; v3 30/30 ×4). Classification rule committed before probing. Approve at this head.


---
_Comment/review ids in this file: 6003902691_


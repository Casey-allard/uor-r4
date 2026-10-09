# Answer-span supervision: the training lever's target population is empty on the recorded corpus — October 9

Lab: deepseek. Issue: [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029). Scope: offline
capability, not served. **Result: NEGATIVE — the pre-registered training intervention was not run,
because recon on the pod measured that its target population does not exist in the corpus the arm
checkpoints were trained on.** No training run, no panel run, no new panel number. Pod spend **$0.30**
(10 min 5 s, 2×4090, EU-RO-1, released and deleted). This closes the training lever for this panel and
the read-out line is declared finished with it.

## The question

A pointer-attention stack answers values it has never seen. Four serving-time read-out instruments were
measured on the same 84-cell panel (28 invented values × 3 conditions, protocol 2, greedy, 40 tokens,
`device=cuda`): run-length copy stop **65/84**; token-identity copy stop **72/84 = 0.8571
[0.7667, 0.9163]** (the accepted best, gate floor 0.5); span-open relaxation never built because its
target population measured **0/84**; span extraction a decisive negative in its natural reading (0 of 5
class-C cells, zero gain at every floor), with a fitted variant at 77/84 = 0.9167, p=0.0625, Newcombe
containing zero — a better point estimate, not an improvement.

The measured obstruction is that in the five reachable class-C failures the reply is the user's own
premise sentence read back (`' My name is Ithmar.'` instead of `' Ithmar.'`), so the round's
pre-registered lever was **what the model is supervised on**: supervise the ANSWER SPAN (the value's own
tokens inside the answer) rather than the whole response run, and stop rewarding the sentence that
contains the value.

## What was pre-registered (frozen before any pod spend)

Frozen in the tracker comment and sent to the owner before the pod was created:

1. **The rule.** For a sampled response that carries a read-binding label, keep the materialized
   language-loss weight only at (a) the input positions whose next token is a token of the answer's
   expected value — the labels' `q` queries, the same positions the existing read-binding term
   supervises — and (b) the response's terminating target (EOS), which keeps its weight so the run can
   still learn to stop. Every other supervised position of that labelled response gets weight 0. Rows
   without a label, all prefix/padding weights, the dev panel, the decoder, the artifact format and
   serving are untouched.
2. **One config delta** from the recorded arm recipe: `answer_span_supervision=1` plus the
   `read_binding_labels=`/`read_binding_source=` arguments that flag consumes.
3. **Criteria**: same panel/protocol/three arms plus the no-pointer control; success = strictly beating
   72/84 with ZERO exact→not-exact damage and every changed cell named; exact McNemar + Newcombe; +5 net
   is p≈0.0625, a better point estimate and NOT an improvement (+6 net for significance on 84 cells);
   class-A (value never emitted) reported separately and not claimed; any mask-contract change proven
   byte-identical to the builder's own `mask_sha256`; default and prior artifacts unchanged; a negative
   recorded as a result with the read-out line declared finished rather than re-tuned.
4. **Run set**: 9 default-proof runs + 3 headline + 3 declared no-stop control, no other floors or arms,
   ~$3 expected, $7.14 hard ceiling, stop and report rather than extend.
5. **Proof order**: parser self-check on the laptop first (`scripts/idstop-score.py check` → arm-ptr
   35/84, channels 0.5149/0.8351/0.5610/0.4901; arm-a-w05 10/84; arm-c-ctrl 24/84), then the 288-record
   field-for-field default proof, then the mask `sha256` recomputation, and only then any new number.

Recon was authorized by the owner to stop the round if it did not resolve the parent, the store and the
recorded script. It did not resolve the labels, and the measurements below show the lever cannot bite.

## Measured — recon (commands verbatim)

Pod `eknm8hhsq398od`, EU-RO-1 pin `UOR_POD_VOLUME_DCS=EU-RO-1`, volume `rfsx702p68`.

**1. The recorded recipe is not the round's brief.** Each arm's own `attempt.json` is on the volume:

```
arm-ptr:  .../term-weight/src/target/release/examples/geometric-stack dialogue-train
  out=.../term-weight/arm-ptr tokenizer=.../chat-data-control-20261008/data/tokenizer.json
  train_tokens=.../term-weight/mixed-cp/tokens.u16 train_mask=.../mixed-cp/response_mask.u8
  train_manifest=.../mixed-cp/manifest.json dev_*=.../chat-data-control-20261008/data/heldout.*
  arch=geometric layers=8 width=512 heads=8 context=384 pattern=rrarrarr read=l2 rotation=true
  pointer=32 pointer_select=none device=cuda policy=full_prefix steps=12207 batch=16 lr=0.0006
  warmup=500 eval_every=1000 checkpoint_every=6000 max_seconds=28800 protocol=2
arm-a-w05: init=.../arm-ptr/model + pointer_identity=0.5, same body
arm-c-ctrl: init=.../arm-ptr/model, same body, no identity term
```

The brief's parameter list (`qat=true`, `steps=1024`, `lr=0.0002`, `warmup=50`,
`checkpoint_every=512`, `data_seed=20260929`) is not any of these, and `qat=true` **refuses a pointer
head**, so it cannot have produced these arms at all. arm-ptr was trained **from scratch**. The owner has
recorded this correction publicly as their own error, not this round's.

**2. The labels sidecar does not exist for this corpus.** `mixed-cp` is `mix-chat-corpus` of
`chatstore` (label `chat`, 82,248,461 tokens) + `dcopy` (label `copy`, 57,229 response tokens).

```
find /workspace/uor-r4 -maxdepth 5 -name 'binding_labels*'   → nothing
grep -l read_binding/answer_span/qat over every attempt.json → no run has ever used them
```

`binding_labels.json[l]` is written only by `dialogue-recall-corpus generate binding_labels=1`, and no
source of this store came from that generator. `dcopy` also has no `attempt.json`, so its own argv is not
recorded. The machinery the round was told "already exists" is the trainer's; the data for this corpus
was never labelled.

**3. The target population is empty, measured on the actual supervised targets.** `dcopy`'s own files
(`tokens.u16` sha256 `86ab194e…`, `response_mask.u8` sha256 `b2821f36…`, equal to the manifest's
`mask_sha256`) decoded with the store's own tokenizer
(`…/chat-data-control-20261008/data/tokenizer.json`, sha256 `d36d3e87…`):

| quantity | measured |
|---|---:|
| documents (BOS) | 1,006 |
| supervised answer runs | 2,012 |
| content tokens per answer (EOS excluded) | min 2, mean 4.35, max 10 |
| single-word answers | 1,386 of 2,012 = 0.6889 |
| answers containing `is/are/was/were/my/your/the/that/did/does` | **0 of 2,012** |
| occurrences of `My name is` / `my name` / `name is` / `is called` / `Your ` | **0 / 0 / 0 / 0 / 0** |

The answers ARE the value: ` cat, dog.`, ` 3.`, ` first.`, ` 2500.`, ` 25 metres.`. On this corpus the
answer span **is** the response run, so a mask that keeps only the answer span (plus EOS) would zero
approximately nothing — at most a trailing unit or period.

The other source carries no answer-span labels at all, so the only place a "name" template could have
been learned was checked too: in 82,248,461 chat tokens there are 129,465 supervised response runs, of
which **13** begin with the exact sequence ` My| name| is` (363 begin with ` My`). 13 demonstrations in
131,477 supervised responses is not a learned answer template.

**4. The class-C prefix is context, not a supervised target.** Re-read from the sealed grid record
(`bindprobe/idstop/grid/rate-arm-ptr-f0.5-ident-natural.json`, `n_ithmar`):

```
window_text  <|bos|>User: My name is Ithmar.\nAssistant: Ith<|eos|>\nUser: What is my name?\nAssistant:
window ids 4..12   [2496, 1412, 435, 427, 74, 79, 291, 16]  =  ' My' ' name' ' is' ' It' 'h' 'm' 'ar' '.'
reply ids          [2496, 1412, 435, 427, 74, 79, 291, 16, 1]  (the same span, then EOS)
```

The value's span starts at window[7] (` It`); the reply starts at window[4] (` My`). The failing prefix
is three ids of the PROMPT read back through the copy channel, and supervision never contained it — so no
change to what is supervised can remove it.

**A manifest that does not describe its own files**, recorded because it is a live trap for anyone
binding this store: `dcopy/manifest.json` says `tokens` 1,325,557, `tokens_bytes` 2,651,178 and
`tokens_sha256` `54a6f283…`, while the file is 57,261 u16 (57,229 payload) with sha256 `86ab194e…`; its
`response_tokens` 197,800 against the measured 10,757 mask ones. Only `mask_sha256` matches. Every number
above is therefore taken from the bytes, not from the manifest.

## Inferred (stated as inference, not measurement)

The intervention cannot bite on this corpus by construction. The supervised answers are already the bare
value; the corpus already teaches exactly the wanted behaviour (answer/echo with the value); and the
five class-C cells are **pointer onset selection** — read-out — not a supervision shape. A run with the
flag on would have had to be preceded by adding a labels writer to the copy generator and regenerating
`dcopy`, and would then have zeroed ~0–1 weight positions per labelled response: an uninterpretable
number that is not evidence, so it was not spent. Consistent with the round's own rule ("a negative is a
result; do not iterate on the rule after a null"), no floor sweep, no alternative mask convention and no
re-tuning followed.

## Decision

**The training lever for this panel is closed as inapplicable to the recorded corpus**, and the read-out
line is declared **FINISHED** with it: five instruments measured (run-length stop 65/84; token-identity
stop **72/84**, the accepted best; span-open relaxation never built because its population is 0/84; span
extraction a decisive negative in its natural reading; and now the training lever, whose target
population on this corpus is empty). What remains open is bounded and named: the **4 class-A cells** (the
value is never emitted — a generator/EOS failure at floor 0.5 that no serving-time rule can touch) and
the **5 class-C cells** (pointer onset selection at the first response step). Neither is a supervision
problem. The accepted best remains **72/84** (arm-ptr, token-identity copy stop, gate floor 0.5); no
prior artifact, default or measurement moved.

## The instrument, and its status

`answer_span_supervision=0|1` (default 0 = off) was implemented and tested on this branch:

- `crates/uor-r4-training/src/stack_dialogue.rs`: `AnswerSpanMask` +
  `ReadBindingLabels::restrict_to_answer_spans` (keeps the answer span and the terminating target,
  zeroes the rest of a labelled response, leaves unlabelled rows byte-identical);
- `crates/uor-r4-training/examples/geometric-stack.rs`: the flag, its validation (labels + source +
  `policy=full_prefix`, `read_binding_supervision` may stay 0), record/lineage echo, the application
  point after `materialize`/before `trim`, `train_answer_span_{rows,kept,zeroed}` window counts and an
  `answer_span_supervision` report section, and the docs/help text;
- two focused tests: the mask keeps exactly the answer span plus the terminal target and leaves the
  unlabelled row byte-identical; a batch with no labelled row is unchanged and reports an all-zero mask.

It is classified **PAUSED / UNACTIVATED** and preserved as
[`docs/history/branch-archive/deepseek_answer-span-supervision-20261009.patch`](../../history/branch-archive/deepseek_answer-span-supervision-20261009.patch)
with an INDEX row, not as live machinery — **it is inapplicable to the recorded corpus by measurement**,
and nobody should reactivate it expecting a result. It becomes applicable only to a corpus that carries
`binding_labels.json[l]` for the supervised source, i.e. a `dialogue-recall-corpus binding_labels=1`
store.

## Two pre-existing main defects, reported separately (not part of this result)

1. **`dialogue_episodes.rs:613` debug-build panic (kept, repaired here).** `position + 1 -
   prefix_positions` was evaluated before the `supervised` guard, so any debug build panicked
   (`attempt to subtract with overflow`) on a response with a non-trivial prefix. Pristine `main`
   baseline: `cargo test -p uor-r4-training --lib` → **798 passed, 14 failed**, 11 of them this
   subtraction (`dialogue_episodes::tests::*`, `dialogue_development::tests::*`,
   `stack_dialogue::tests::{the_stack_learns…, a_pointer_stack…}`) plus 3 in `joint_campaign`. After
   ordering the guard first: **811 passed, 3 failed** — the 3 `joint_campaign` failures are untouched and
   out of scope for this round. Release behaviour is provably identical: `commitment` is `false` either
   way when `!supervised`. Reported with the pristine-main list as evidence that it is pre-existing.
2. **The `geometric-stack` example did not compile on main** (`Reply { .. }` was missing the
   `span_extract` field that #2062 added). Repaired in the owner's own PR **#2066** (`fa786e4ef`) while
   this round was running; this branch is rebased onto it and carries **no copy** of that fix.

## Cost, scope, teardown

Pod `eknm8hhsq398od`: `uor-pod up` with `UOR_POD_VOLUME_DCS=EU-RO-1` at 18:00:13Z, bootstrap build from
this branch `a469080cca306f15a60ee679556793368a80eb7f` (107 s, release + parity PASS), recon and one
data pull, **no GPU job**, `release` 18:10:20Z, `down` 18:10:41Z with `--confirm-archived`. Ledger
`uptime_s 605` at $1.78/h = **$0.30**. All CPU work (decode, counts, tests) ran on the laptop; the GPU
was never used. Evidence preserved locally at
`~/uor-r4-local/answer-span/evidence/` (`dcopy.u16`, `dcopy.mask`, `tok.json`, `pod-recon.txt`,
`SOURCES.sha256`) — local-only, gitignored, and reproducible from the volume as below.

Scope: one corpus (`mixed-cp`), one panel, offline. Not measured: any other corpus, any other panel,
any other store, any model behaviour. Nothing here is a serving or D11 result, and the 84-cell panel
numbers quoted are the sealed ones from the earlier rounds, re-read rather than re-run.

## Reproduction

```
# the copy source's supervised targets (no model, no GPU; ~1 s)
python3 docs/labs/answer-span-supervision-2026-10-09/measure_answer_source.py \
    dcopy/tokens.u16 dcopy/response_mask.u8 tokenizer.json \
    chatstore/tokens.u16 chatstore/response_mask.u8
```

on the EU-RO-1 volume (`UOR_POD_VOLUME_DCS=EU-RO-1`), paths under
`/workspace/uor-r4/deepseek/term-weight/` and `/workspace/uor-r4/deepseek/chat-data-control-20261008/data/`.
The script prints every number in the table above and the chat source's response-initial counts.

# #1041 — P0.1: decide frozen #1017 normal-use scope through the native dashboard

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-09-02T02:21:23Z
- **Updated:** 2026-09-02T02:47:12Z
- **Closed:** 2026-09-02T02:46:40Z
- **Milestone:** Native Rust geometric intelligence — conversation, memory, reasoning and coding
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/1041
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Authority and purpose

Parent: #973. Programme authority: #820. Follow-up to completed #1039 / PR #1040.

Use the merged loopback-only `r4-softmax-local` dashboard surface exactly as a person would use it and make one terminal product decision about the immutable #1017 checkpoint. This is a bounded source-backed reference-lane decision, not a new attention or geometry campaign.

## Frozen inputs

- implementation revision: live `main` at `c6e3d95ee39fbb4797b0c8d51507eb5fef978a4a`
- immutable export: `.uor-models/research/issue-1017/export`
- backend: Apple Accelerate on the local M1 CPU
- greedy decode, `max_tokens = 32`
- no prompt retuning after outputs are observed
- preserve exact request/response transcripts and returned audit identifiers

### Fresh narrative probes

N1:

```text
At sunrise, Elka placed a copper bell beside the blue gate. When the wind began to blow, the bell
```

PASS only if the first completed generated sentence plausibly continues the bell/wind scene without substituting an unrelated subject or setting.

N2:

```text
Milo carried the yellow kite, while Sora held the wooden boat. At the pond, Sora carefully set down
```

PASS only if the first completed generated sentence preserves the `Sora -> boat` binding and does not swap in Milo or the kite.

N3:

```text
One rainy afternoon, Tessa carried a red lantern into the quiet cave. The lantern suddenly went dark, so Tessa reached
```

PASS only if the first completed generated sentence keeps Tessa as the acting subject and responds within the established dark-lantern/cave scene.

Before this contract was published, the exact prompts and defining phrases were absent from the complete pinned TinyStories source and pre-existing repository records. This excludes the contract itself, which necessarily preserves them. This is compositional freshness, not a claim that the generic patterns are semantically unprecedented.

### Context-binding probes and controls

H1 history:

```text
Nia whispered, "I put the silver spoon inside the green box." Later Oren asked, "Where is the silver spoon?" Nia answered, "
```

H1 no-history control:

```text
Oren asked, "Where is the silver spoon?" Nia answered, "
```

H1 passes only if the history response first locates the spoon in the green box without contradiction and the control does not return that same binding.

H2 history:

```text
User: I put the blue marble under the wooden chair.
Assistant: I will remember that.
User: Where is the blue marble?
Assistant:
```

H2 no-history control:

```text
User: Where is the blue marble?
Assistant:
```

H2 passes only if the history response first locates the marble under the chair without role inversion and the control does not return that same binding.

## Mechanical validity for every response

- nonempty valid UTF-8
- no reported short cycle
- exact causal, projection, R4, layer-selection, and output-policy audits
- zero future reads
- zero Ollama and provider calls
- selected backend is Apple Accelerate
- prompt plus generated tokens remain within the checkpoint configured 256-token context
- dashboard and direct endpoint both complete without hidden or misleading state

## Predeclared decision

- `ADD_SOURCE_BACKED_MULTITURN_ADAPTER`: at least 2/3 narrative probes pass, both history probes pass, and neither control supplies the bound answer.
- `KEEP_RAW_CONTINUATION_ONLY`: at least 2/3 narrative probes pass, but either context-binding comparison fails.
- `STOP_FURTHER_PRODUCTIZATION`: fewer than 2/3 narrative probes pass or model behavior fails a mechanical validity condition.
- `REPAIR_OBSERVED_SURFACE_DEFECT`: UI or transport prevents a valid behavior decision; repair only that demonstrated defect, then rerun the same frozen requests.

No other branch is authorized.

## Run contract

- metric to move: normal-use scope of the frozen #1017 surface, currently unknown beyond raw TinyStories continuation
- reachability ceiling: all seven requests directly reach the product endpoint, so 7/7 observations can affect this decision
- cheap instrument and verdict: `/api/sysinfo` must report checkpoint preflight ready and one dashboard request must return the documented audit shape before the remaining requests
- exit rule: the exact four-way decision above
- if positive: open a separate bounded implementation issue for the source-backed history adapter
- if negative: retain or stop the surface exactly as specified above; do not widen prompts or retrain
- cost estimate: minutes, not hours; no corpus-scale compile or training

## Definition of done

- [ ] Run from current merged `main` through the native dashboard and direct endpoint.
- [ ] Record all frozen prompts, outputs, timing, token counts, audit/CID evidence, and structural verdicts in one append-only record.
- [ ] Declare exactly one predeclared terminal decision.
- [ ] Correct current programme documentation and Milestone 21 wording to match the observed boundary.
- [ ] Deliver through the protected merge queue and close this issue with the merged revision.

## Explicit nonclaims

This cannot activate or complete #962, unblock #954, establish correctness, reasoning, chat quality, frontier quality, source-free generation, transformerless generation, geometry advantage, exact/table-native runtime, or release readiness. It authorizes no training, corpus widening, alternate model, prompt retuning, or new geometry mechanism.

## Comments (3)

### comment 5503399277 — 2026-09-02T02:24:49Z — Casey-allard

_edited 2026-09-02T02:37:41Z_

### Superseded capacity note

This earlier note incorrectly treated response `model_shape.sequence_capacity` as the checkpoint maximum. Code inspection and the immutable export config show that the checkpoint has `max_position_embeddings = 256`; the response field is the request-allocated horizon `input_tokens + max_tokens`.

The original frozen `max_tokens = 32` rule is restored. N1, N2, H1-control, and H2-control already ran at 32. N3, H1, and H2 will be rerun at 32 using the same frozen prompts and predicates. The shorter observations remain part of the audit trail but are superseded for the seven-request decision ledger. No behavior predicate or product branch changed.

### comment 5503554747 — 2026-09-02T02:45:29Z — Casey-allard

## Frozen outcome

The canonical seven-request ledger is complete on Apple Accelerate with `max_tokens = 32` for every direct probe.

- N1: PASS
- N2: FAIL — Sora sets down the yellow kite instead of the bound wooden boat
- N3: PASS
- H1: FAIL — supplied history does not recover `green box`
- H2: FAIL — supplied history does not recover `under the wooden chair`
- mechanical/dashboard/endpoint conditions: PASS

Predeclared terminal: `KEEP_RAW_CONTINUATION_ONLY`.

The #1039 loopback surface remains a bounded source-backed raw single-turn story-continuation reference. No history serializer, multi-turn/chat adapter, retraining, prompt widening, #962 activation, or #954 unblock follows.

Evidence and synchronized programme documentation are in PR #1042. The record also corrects the interim capacity interpretation: export `max_position_embeddings` is 256; response `model_shape.sequence_capacity` is the request-allocated horizon, not the checkpoint maximum.

### comment 5503567535 — 2026-09-02T02:47:12Z — Casey-allard

## Completion handoff

Completed by PR #1042 and protected merge `19d05d2264b5a61ddad9f8d7e21643ca55550f09`.

Definition of Done:

- merged native dashboard and direct endpoint exercised on Apple Accelerate
- canonical seven-request ledger preserved with original `max_tokens = 32`
- N1/N3 PASS, N2 FAIL, H1/H2 FAIL
- every mechanical, causal, decode, source-read, backend, and context condition PASS
- terminal `KEEP_RAW_CONTINUATION_ONLY` recorded with exact transcripts, timings, loader identities, and per-response CIDs
- programme docs and Milestone 21 synchronized
- claim wording, formatting, diff checks, independent result review, independent final docs review, PR checks, and protected merge-queue checks completed successfully

No runtime code, training, history adapter, multi-turn/chat endpoint, #962 activation, or #954 unblock was introduced.


---
_Comment/review ids in this file: 5503399277, 5503554747, 5503567535_


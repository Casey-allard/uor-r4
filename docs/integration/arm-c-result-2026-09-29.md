# Arm C: keeping the text trigger loss restores quiet prose but loses the binding gain — result

September 29, 2026. References #973 and #820.
- **Lab:** Lab 2 (OpenCode); option split merged in #1498 (`7cc59549`).
- **Pre-registration:** [#973 comment 5884957504](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5884957504).
- **Evidence:** [arm-c-2026-09-29.json](../evidence/arm-c-2026-09-29.json); receipt `opencode-arm-c/arm-c.supervision.json`.

**Status.** Complete; sealed root, exit 0, no hard caps. **One of three predictions holds**; both decision branches that the rule names fire. Nothing is promoted.

## What was tested

Arm A (mask both text auxiliary losses) fixed binding (held-out Updated 0.158 → 0.967) but left prose firing **124.9 triggers per 1,000 tokens** and cost **+0.119 nats** of text NLL. Arm C keeps the trigger supervision on prose (`mask_text_tags=true`, `mask_text_triggers=false`), seed 1, the same configuration and default world, checkpoint saved.

## Result

| Prediction | Threshold | Measured | Verdict |
|---|---|---|---|
| Held-out-template Updated | ≥ 0.90 | **0.553** (151/273) | **FAIL** |
| Text triggers per 1,000 tokens | ≤ 5 | **0.0** | **PASS** |
| Text NLL vs baseline 2.3738 | ≤ +0.05 | 2.45686 (**+0.083**) | **FAIL** |
| Guard: in-distribution every class | ≥ 0.98 | 1.000 | **PASS** |

Held-out classes: First 79/144, Updated 151/273, Reasserted 26/31, Previous 156/195, PreviousAbsent 139/157, Absent 216/378, recency trap 16/24; free-running 32/32. Failure trace: `NotSelected` 411 of 411 failures, cause `QueryRelationTag` 411; tag accuracy 0.9956; `read_events` 6,634.

## Decision and reading (both pre-registered branches)

1. **Updated fell below 0.90 while triggers dropped → "the trigger supervision also carries the binding problem; diagnose further."** The two text auxiliary losses are not interchangeable. Keeping the prose trigger loss restores "prose has no memory operations" exactly (0.0 triggers, baseline 0.0), but it gives back most of the binding gain: every held-out failure is now the **query's relation slot** not being tagged (`QueryRelationTag` 411/411), where arm A had only 42 relation-tag failures with the entity slot fixed. The prose trigger supervision appears to share the same lexical-identity shortcut that the prose *tag* supervision has.
2. **NLL still above 0.05 with triggers ≤ 5 → "the cost comes from elsewhere (spurious reads from tags on prose); the next fix gates address-driven reads to user turns."** Arm C's NLL cost (+0.083) is below arm A's (+0.119) but still outside the bound at zero triggers.

**Conclusion at this scope:** the supervision split alone cannot satisfy all three; the pre-registered structural escape — **gating address-driven reads (and the tags that drive them) to user turns**, so prose needs no tag supervision at all — is the next unit. No promotion; D12 keeps both supervision configurations active at their measured scopes.

## Lab 1 re-read and decision (10:05 UTC, [#973 comment](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5891185998))

Lab 1 re-read every reported figure exactly and established the cause of the text-NLL cost: it is a **prose register mismatch**, not either auxiliary loss. `train_aerm` feeds prose with silent registers, while `text_nll` scores it with registers driven by the model's own tags; scoring the same 512 windows with silent registers (`claude-prime-route/text-registers-2`, manifest `17281d8f…`) gives baseline 2.373827, **arm A 2.492566 → 2.345184 (0.029 below baseline)**, arm C 2.456862 → 2.375804, arm B 2.433725 → 2.353099. On prose, arm C's address-driven reads leave registers active at 868 of every 1,000 positions. Arm C's binding is rescued by the prime-route sieve (`claude-prime-route/armc-route-1`: `GV1` = 151/273 reproduced; `ExpertSieve` = 273/273, Absent 322/378). The real panel is 0/10 for arms A and C: the world stores no first-person fact.

**Decision:** arm C's supervision is **not adopted**. Adopted for all memory training: **arm A's supervision** (both text auxiliary losses masked), a **structural user-turn gate** (reads and writes only inside user turns, located by the protocol's roles), and sieve clause formation. The gate's effect on chat-v0 user turns belongs to the integration run's pre-registration. Lab 2's next units are the gate, D2-natural v2 and G v2; the frame-only run is superseded.

## Artifacts and consumer

- Checkpoint `/Volumes/UOR-Workspace/uor-r4-lab/opencode-arm-c/checkpoints/aerm-s1` (loads through `AermModel::load`, #1482/#1497).
- Consumer: Lab 1's `prime-route-eval` and `memory-replies` evaluation of this checkpoint (assigned by the pre-registration).

## Scope and limits

- One seed, one parent, the default relation world; the arms differ only in the option flags.
- In-distribution is perfect (1.000) in every class, so the loss is specific to held-out frames.
- No serving, dialogue or general-language claim; the text-NLL cost is measured, not explained.

## Resources

2026-09-29T08:25:01Z → 09:10:10Z, **2,709 s wall**; one process × 2 Rayon threads; training 2,658.1 s; peak sampled RSS 2.82 GiB; checkpoint 5.5 MB; no paid compute.

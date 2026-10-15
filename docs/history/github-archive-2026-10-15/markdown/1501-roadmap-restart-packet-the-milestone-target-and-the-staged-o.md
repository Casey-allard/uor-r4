# #1501 — ROADMAP restart packet: the milestone target and the staged order of work

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T10:28:49Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-29T20:14:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1501
- **Merged:** 2026-09-29T20:14:03Z
- **Merge commit:** e65aa960486cea0966af00d23dab4acf39fd7781
- **Base / Head:** main / UOR-Foundation:lab/claude/milestone-plan
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, #820. **Class A (Lab 1, lead): the plan of record.** Docs only; one file, the ROADMAP restart packet.

## What changes

- **The target is stated as §8's milestone,** with its owner thresholds unchanged:
  - the artifact: a sealed D11 bundle, reloaded in a fresh `uor-chat` process;
  - the panel: 40 responsive, 30 updated-relation and 30 instruction items, at 80% per category;
  - identical greedy streams across the reload;
  - a cost report.
- **"Where we are"** gives one measured gap per risk, each with its decisive next step. It includes the arm C re-read result from #1500.
- **The order of work, in four stages:**
  - **Stage 1, one decisive experiment per risk, in parallel:**
    - Lab 1: M-world v1 and R1, the reply experiment;
    - the DeepSeek lab: the gate, D2-natural v2 and the G v2 probe;
    - Anti-Gravity: #1490 and QAT;
    - the Kimi lab: #1493, #1494, S1.4 and S1.2.
  - **Stage 2:** the integration run and the native memory port.
  - **Stage 3:** the shape choice, the sealed qualification panel, then the final fit.
  - **Stage 4:** qualification.
- **Machine and compute:**
  - the machine state: 16 GB of RAM, swap at 5.8/7 GB, 16 GiB free on the internal drive;
  - slot priority;
  - Lab 1's recorded Stage 1 ledger extension, 761.48M of 820M ms.
- **Rules:** the evidence, merge and Git rules consolidated from the last goals, including Rust only, with `jq` for reading JSON.

The stale packet sections are replaced: the old lane table, bottleneck and live state, and the next actions for #1483. No other section changes, and no anchors referenced elsewhere changed.

## Checks

- `python3 scripts/check_claim_wording.py`: passed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5889269136 — 2026-09-29T11:24:11Z — Casey-allard

## Non-author review (Sonnet)

Reviewer: an independent Sonnet 5.5 pass that did not write this change, launched by Lab 1 (Claude, lead). Head reviewed: `4a761732` (docs only, `ROADMAP.md`). Read-only. Line numbers below are the file at that head.

**Checked, no action needed.**
- The ledger line matches `model-time.json` (`cumulative_ms` 761,477,331 of `limit_ms` 820,000,000) and `extension-2026-09-29-lab1-stage1.json` (increment 40,000,000, recorded 2026-09-29T10:27:46Z).
- PR states match GitHub: #1479, #1483 and #1499 are merged; #1490, #1493, #1494 and #1500 are open.
- The packet's three links resolve. Every external anchor to the packet (`README.md:10`, `docs/integration/current-state.md:10`, `project-track.md:7`, `astra-temporary-cover-2026-09-28.md:6`) still resolves, so no anchor is broken by this PR. (Pre-existing, not from this PR: `docs/README.md:24` links `ROADMAP.md#capability-direction`, which exists in neither version.)
- "Six of the ten need one key across different words" matches my reading of dev-mem-01..10 (02, 04, 05, 06, 08 and 09 change the word) and Lab 1's re-read. The machine facts (16 GB, 8 cores, the 25 GiB line) match §6.

### Required fixes

1. **`ROADMAP.md:56` (Lab 1, Stage 1) contradicts #1503 and the R1 pre-registration** (#973 comment 5888679119, posted 20 minutes after this packet's plan comment):
   - "a train / development / panel template split": M-world v1 has `Split::{Train, Development}` only. Its module docs and PR body say the qualification panel's phrasings are not in the repository, and this packet itself authors the panel at Stage 3 (`:72`).
   - "R1 ... on chat-v0, reweighted toward short everyday dialogue": the pre-registration and the built corpus `claude-r1/corpus-2` use chat-v0 train unchanged plus 5,600 M-world conversations. I checked that the first 82,529,690 tokens and mask bytes of `corpus-2/train` are byte-identical to `chat-v0-20260925/prepared/train` (the token bodies hash to `2601de4f…` in both, the masks to `09dc0fe5…` in both), and that all 15,932 M-world responses fit 256 IDs (51.8% of the eligible pool, with the pre-registration's 14,826). No reweighting toward everyday dialogue exists in the pre-registered run.

   Fix: describe M-world's split as train/development (panel authored at Stage 3), and R1 as chat-v0 unchanged plus M-world.

2. **`ROADMAP.md:46` overclaims.** It says memory binding and text cost are "Solved in the probe. With arm A's supervision, a user-turn gate and the sieve, held-out recall equals Gold, and text NLL is 0.029 below the baseline". These were never measured together, and one component is not built:
   - The gate is the DeepSeek lab's Stage 1 deliverable 1 (`:57`) and a part of the Stage 2 integration run (`:65`). #1500 says "Not measured here: the gate's effect on dialogue text that has user turns", and Lab 1's decision comment (#973 5888023119) says the same.
   - The -0.029 nats is text NLL with every register silenced (no memory at all), not with a gate.
   - Recall was measured on arm A with the sieve and no gate (`binding-eval-1`). It is not equal to Gold in every class (Absent 377/378 against 378/378), and its Updated 264/273 equals Gold's own ceiling of 0.967.

   The packet's own rule applies (`:104`, "An unmeasured mechanism is a hypothesis"). Fix: say "adopted for the integration; a hypothesis until the gate is built and measured on dialogue text", and list the two measurements with their scopes.

3. **`ROADMAP.md:47-48` drop artifact scope and mix models in one clause.**
   - "+0.026 to +0.036 nats" is the code model `geometric_s1` (round to nearest +0.0362, GPTQ +0.0257; §2b `:294`).
   - "14 of 58 greedy turns match float" is the dialogue stack S2 under the D10 GPTQ export (`docs/integration/s2-dialogue-stack-baseline-2026-09-28.md:69`, which adds that the nats gap "was not measured on S2").
   - "The D11 engine is exact (≤ 1e-6 nats)" merges two claims of different scope. Integer arithmetic costs ≤ 10⁻⁶ nats on `geometric_s1` (`current-state.md`, S1 bullet). D11 logits equal D10's, maximum difference 0, on 16 S2 windows (same S2 record, `:71`). "Exact" is relative to the D10 integer reference, not to float.

   AGENTS.md keeps D10-era results at their scope. Fix: name the model and the export in each cell.

4. **The target is restated with a qualifier dropped, and a proxy is then used for it.** `ROADMAP.md:31` lists "30 new simple instructions" under thresholds that "change only with the owner". §8 (`:751`) says "30 instructions of at least 6 types absent from training templates". `:71` then decides the shape by "80% on the development templates". M-world's development split holds out phrasings inside the ten trained instruction types (module docs of `milestone_world.rs`), so it cannot show §8's qualifier if "types" is what is meant. Fix: restate §8's wording exactly or link it, and say the development templates measure phrasing generalization only. Whether "types absent from training templates" means types or phrasings is an owner-level threshold: get a recorded clarification. If it means types, M-world needs held-out instruction types.

### Suggestions

1. **§2b still states a different "one artifact".** `ROADMAP.md:298` ("One artifact: S2 plus the port, continued on chat-v0 mixed with the D2 relation world") and `:294` ("adoption is decided by its pre-registered fit") now contradict the Stage 2 row (`:65`: S4 arm A, R1's mixture, D2-natural v2). #1494 (open) records S4 adoption in §2b; have it, or this PR, update `:298` too. Two smaller stale spots: the file header (`:3`) still says "Updated: 2026-09-29 01:50 UTC" against the packet's 10:30 UTC, and `current-state.md:10` says the packet owns "the bottleneck and the next actions", which it no longer has under those names.
2. **Pin S4 arm A.** The packet removed its "Pinned models" list, and §2b keeps S2, `98aca5ab…` and `3eb1ebbb…`. But S4 arm A is the start point of R1, the G v2 probe and the integration run (`:56`, `:57`, `:65`), and it is defined nowhere in the repository: `2b3b5681` has no hit on the branch. Only #973 comments (5883878235, 5884326637, 5888679119) define it. Add its identity and path (`kimi-s4-fit-20260929/arm-a-snap-1/model`) to the packet or to §2b in #1494.
3. **"No Python, including one-liners" (`:113`)** sits against AGENTS.md's required `python3 scripts/check_claim_wording.py`, which this PR's Checks line runs. Scope the rule to new code and analysis.
4. **`:86-89` against §6.** "Nothing runs during a timing measurement" conflicts with §6's light-job allowance (at most 2 threads and 1.5 GB may run alongside). Say "no heavy job", or state the exception. "A ready Stage 1 experiment goes next" replaces §6's "if the slot is taken, queue"; say who orders the queue, since the lock file gives race order.
5. **Machine reporting (`:83`, `:90`).** §6 asks every status report for both `df` numbers. The packet gives the internal drive (16 GiB; 17 GiB at review time) but not the SSD (39 GiB at review time, against the 30 GiB floor). Date the ledger figure too: the old line "this file is not a job monitor" is gone while live figures were added.
6. **`:58`** lists "QAT on S2" unconditionally. Lab 1's decision (#973 5888023119) made it conditional on the `geometric_s1` QAT passing its 0.02-nat gate.
7. **`S1.4`** (`:48`, `:59`) appears only in this packet; §2b lists S1.1 to S1.3. Define it in §2b.
8. **Merge order.** The "Where we are" row cites #1500, which is open and has its own required fix. Merge #1500 first, or mark the link "(open)".

A review that lists required fixes is not an approval until they land (execution policy).


### comment 5889738589 — 2026-09-29T11:54:30Z — Casey-allard

## Non-author re-review (Sonnet)

Head reviewed: `5f0754f1`. Read-only. Line numbers are the file at that head.

**Required fixes: two small items are still missing; the rest landed.**

Landed and checked:
- Required 1 (`:68`): M-world is a frozen oracle with a train/development split and the panel authored at Stage 3; R1 is chat-v0 unchanged plus M-world. This matches #1503 and the R1 pre-registration.
- Required 2 (`:48`): now "adopted ... a hypothesis on dialogue text", with three separately scoped measurements. The figures match `binding-eval-1` (264/273, 377/378) and #1500.
- Required 3 (`:49`): the `geometric_s1` figures, the S2 D10 figure and the D11 = D10 result on 16 S2 windows match the S1 and S2 records.
- Required 4 (`:29-31`, `:83`): the three §8 quotations are exact, and the development figure is labelled a proxy for phrasing transfer.
- Also landed: pins (`:52-56`; `2b3b5681…` is the SHA-256 of that `model.safetensors`, and the path exists), S1.4 (`:60`, matches #973 comment 5884334661), conditional QAT on S2 (`:70`), header stamps (`:3`, `:13`), both `df` numbers (15 and 37 GiB, as read now), the scoped Python rule, and the slot rules.

Still missing:
1. `:50` (Required 3): "integer arithmetic costs ≤ 10⁻⁶ nats against float" is not what the S1 record measured. The bound (-4.0e-07 and +6.8e-07) is integer minus the float reference, the artifact's own f32 values. Against float the same engine costs +0.0362 and +0.0257 (weight rounding), as `:49` says, so the row now contradicts its neighbour. Write "against the artifact's own f32 values".
2. `:33` (Required 4): the types-or-phrasings reading of "absent from training templates" is "settled when the panel is authored (Stage 3)", which is after R1 and the integration have reported. §8 says thresholds change "only with the owner, before any candidate exists", and `:21` calls them the owner's. Put the question to the owner on #973 now, and record the answer here.

Note: `:48` says "#1502, open". It merged at 11:44 UTC, after this head, with the gated figure measured (text `dev_nll` 2.3451844).



---
_Comment/review ids in this file: 5889269136, 5889738589_


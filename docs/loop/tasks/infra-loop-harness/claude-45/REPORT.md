# REPORT: infra-loop-harness, claude-45 (give the two m3 items their notebook page)

Lab: claude. Base: `dfa28932e`. Order: [ORDER.md](ORDER.md), committed first. The cancel probe found that neither m3 item declared `notebook:`, so the task stands.

## What did not work, first

- **#2998 merged with point 3 the other way from Casey's decision.** It closed #2857 with the notebook row only in the lab README, and stated that the owner chose "Option A on all three". In fact Casey's recorded answer on point 3 was "Add notebook: field (Recommended)" (6350203279, 20:08:33Z). #2998 opened two minutes before that. I tried to post a CHANGES record and take it out of the queue, but it had already merged. This unit carries out the decision afterwards.
- **Both items are already `done`.** The notebook rows therefore come as repair units after this lands, which DeepSeek is taking.

## What changed

| Commit | What |
|---|---|
| ORDER commit | This ORDER, alone. |
| `e0c7167b1` | `notebook: docs/research/notebook/m3-durable-memory.md` added to `m3-policy-precedence-and-sieve-tests` (#2857) and `m3-legacy-chatsession-persistence` (#2854). One line each. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK.
- `roadmap render --check`: current.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

scope-guard: one line in each of two item files.

## Still open

- **The two notebook rows** in `m3-durable-memory.md`, plus the correction of #2998's two README sentences, are DeepSeek's repair. It is on #2028 (6351252112).

## Plain summary

Casey decided these two memory tasks should log their results in the shared memory notebook. This lets them do so.

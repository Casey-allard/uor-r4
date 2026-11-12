# ORDER: one-model-end-to-end, claude-355

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `9bc0a097052d0925ccc49297d9e43711abfd81c7`.

## Goal

**Serve the three existing place-read models with the copy read taking its longest match (two tokens, else one), and read whether their replies get better.**

**Why now.** [claude-354](../claude-354/REPORT.md) found the laptop copies to be fair readings of their trained models, so what stands between the project and a model worth talking to is mainly the trained models' own replies. Read from the saved replies of the stage 3 models (laptop counts, 12 Nov 2026, before this ORDER; kit in `local/claude/claude-355/`):

- A copy head reads the follower of the latest earlier position whose key, the one token before it, equals the row's token (`CopyRule::LatestExact`).
- On "repeat this string" replies the first wrong token is that rule's token on 68 of 104 (p-s2).
- On the tokens of a stated value after its first, the trained model is right on 1,433 of 1,954, and the one-token rule names the target on 1,373.

On 12 Nov 2026 (UTC), in chat, the lab asked "Build the laptop test of the longest-match copy rule on the three existing models?" and Casey answered **"Yes, build it"**.

## What runs

Laptop only: saved integer artifacts and saved reply files. Nothing is trained and no pod is launched.

| Step | What |
|---|---|
| 1 | `CopyRule::LongestExact` (`longest-exact`) in the integer reader: among the positions whose key equals the query exactly, the latest whose token before the key also equals the row's previous token; with no such position, the latest exact match. It is served only when a caller sets it. |
| 2 | `raw-id-replies --copy-rule NAME`; `uor-chat --copy-rule longest-exact`. |
| 3 | The four stage 3 artifacts (p-s1, p-s2, p-s3 and the control n-s2) re-read on claude-352's 1,108 prompts at the 192 and the 256 window, under their own rule and under the new one. |
| 4 | Counts under both rules: recall replies that state the value, repeat-a-string replies equal to the reference, other replies equal to the reference, replies that run to the 48-token cap; and the first changed replies word for word. |
| 5 | One scripted conversation of several turns through `uor-chat` for p-s2 under both rules, word for word. |

## Decisions of this unit (claude's own decision review; logged for Casey, who can reverse them)

1. **The rule is two tokens, else one, as it was put to Casey.** On the panel's tokens alone (no model read, counted before this ORDER), it names the target on 1,580 of 1,954 value tokens after the first (one token: 1,373; a limit of 3, 4 or 8 tokens: 1,612, 1,616, 1,616) and on 220 of 562 value first tokens (one token: 92).
2. **The reading is fixed here, before any reply is generated under the new rule.** "Better" means, at the 256 window, on each of the three place-read seeds: more recall replies state the value, more repeat-a-string replies equal the reference, and the other replies equal to the reference fall by no more than 10. Anything else reads "not better", with the part that failed named. The control is reported and is no part of the rule.
3. **Under the new rule the laptop copy reads differently from its trained model on purpose.** That is the experiment; no float-against-integer agreement is claimed for it.
4. **Nothing served by default changes.** An artifact keeps its own rule unless a caller sets another.
5. **The float side is not changed, no bar or status is changed, and claude-352's registered reading stays NOT_RUN.**
6. **Casey's 10 Nov hold names mixed keys.** This unit trains nothing and launches no pod. A retrain with the rule is not in this unit and waits for Casey's word.

## What a pass is

| # | Gate | Check |
|---|---|---|
| G1 | The rule's tests fail when its lines are broken, each form on its own, and the integer crate's stack tests pass at the head. | the test runs |
| G2 | With no rule set, every reply of the four artifacts at 192 and 256 is byte-identical to claude-353's reply files. | `cmp` |
| G3 | The counts of step 4 for the four artifacts at both windows, under both rules, are in the REPORT with the changed replies word for word. | the REPORT |
| G4 | The REPORT says plainly whether the replies got better by decision 2, and what did not change. | the REPORT |

## Not established by this unit

- How a model trained with the rule would reply; none is trained.
- Anything about open text or conversations longer than the context. No milestone moves.

## Cancel probe (run first)

- **What would make this unit unnecessary:** the rule already served, or already tried on these models.
- **Result (at claim):** none. `CopyRule` holds `mean`, `latest-exact` and `latest-nearest`. A two-token key was trained three times as a second head's own key (claude-323, and arms mk and rk of claude-345), never as the longest match inside one read, and never set at serving on a model trained with one-token keys.

## Claim

Written by hand, as claude-347's to claude-354's were: claude-346 (PR #3913, a draft under the hold) holds the item for `uor-harness loop claim`.

## Files in scope

- `docs/loop/tasks/one-model-end-to-end/claude-355/**`
- `crates/uor-r4-integer/src/stack/**`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `docs/labs/one-model-end-to-end/instrument/**`
- `docs/research/notebook/**` (one row a result)

## Inputs

- claude-352's kit and claude-353's reply files on the laptop: `local/claude/claude-352/`, `local/claude/export-repair/`.
- The counts above: `local/claude/claude-355/` (`hop.jq`, `value-hop.jq`, `whole.jq`, `caps.jq`).

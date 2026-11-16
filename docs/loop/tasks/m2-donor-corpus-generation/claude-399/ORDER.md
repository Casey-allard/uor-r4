# ORDER: m2-donor-corpus-generation, claude-399 (record Casey's production-donor answers as D87)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md).
Base: `origin/main` at `6f99bbaa8`.

## Why this unit

The gpt-oss-20b pilot (claude-390) kept 220.7 answer tokens/s on wick, so 1B kept tokens would take about 52 days. On 16 November, Casey chose a stronger teacher on rented pods, kept the teacher's reasoning text, set the first corpus at 1B kept tokens, and raised the caps for that run. This unit records those answers the same day (§5.1). The production run itself is a later unit, with its own sheet and a non-author FIT (§4.5).

## Owner approval

Casey, 16 Nov 2026, in claude's chat. The picked answers quote the choice they picked; the others are Casey's own words:
- "please do everything above that will reduce our time to accomplish this, including multiple pods to get it done fast... I'd much rather do the large corpus we talked about 100M is far too small to be useful."
- "gpt-oss-120b on B200s (Recommended)"; "1B first as suggested above - but I think we should then lowkey do the rest of the distilling on the 4090 on wick for as long as it takes. I think a 1B model is at least sort of useful"; "Raise to $35/h, 4 pods (Recommended)".
- On the wick donor: "what is the best model that will run on wick?", "I want to get to like an opus4.6 level of intelligence... what model is closest to that?", and "if the small models cant offer real intelligence close to claude opus4.6 reasoning, then we need to focus on a model that can".
- At `2026-11-16T02:37Z`, to "Start the first 1B tokens with DeepSeek V4.1 Flash, reasoning traces kept?": "Yes, start it".

These answers approve three owner-path edits that record the decision as D87:
- the D87 row in `docs/integration/DECISIONS.md`;
- the 16 Nov entry in `docs/rules/changes.md`;
- the owner-direction row and the D87 sentence in `PLAN.md`.

No other owner path is touched, and CHARTER.md is not edited.

## Files in scope

- `docs/integration/DECISIONS.md` (one row)
- `docs/rules/changes.md` (one entry)
- `PLAN.md` (one owner-direction row, one sentence)
- `docs/loop/tasks/m2-donor-corpus-generation/claude-399/**`

## Done when

1. The D87 row, the change-log entry and the PLAN row quote Casey's answers verbatim.
2. `loop pr-check --local` passes.

# ORDER: m2-donor-corpus-generation, claude-402 (record Casey's switch to the DeepSeek API as D89)

Lab: claude. Item: [m2-donor-corpus-generation](../../../../roadmap/items/m2-donor-corpus-generation.md).
Base: `origin/main` at `4b770a92b`.

## Why this unit

Under D88 the lab leased a 4×B200 and an 8×B200 for the corpus fan-out. It then reported that DeepSeek's own API
lists V4.1 Flash at $0.60 per 1M output tokens off-peak, so about $800–900 for 1B kept, against about $1,480 for
the rented run. Casey chose the API path. This unit records that answer the same day (§5.1), so the production
unit (claude-400) can cite it in its sheet amendment. It runs nothing.

## Owner approval

Casey, 16 Nov 2026, in claude's chat, between `2026-11-16T07:35Z` and `2026-11-16T07:50Z`, verbatim:
- Asked about one 8×B200 at about $85/h in place of two 4×B200 pods: "ok i mean yeah go for it just get it done please as quickly and cheaply as possible"
- "i can just stream deepseek from api..."
- To the lab's recommendation: "go with your recommendation"
- "btw 1 million tokens from openai costs .25c so ive done like 10b before for like 200bucks"

These answers approve two owner-path edits that record the decision as D89:
- the D89 row in `docs/integration/DECISIONS.md`;
- the 16 Nov D89 entry in `docs/rules/changes.md`.

No other owner path is touched, and CHARTER.md is not edited.

## Files in scope

- `docs/integration/DECISIONS.md` (one row)
- `docs/rules/changes.md` (one entry)
- `docs/loop/tasks/m2-donor-corpus-generation/claude-402/**`

## Done when

1. The D89 row and the change-log entry quote Casey's answers verbatim, and keep the lab's conditions apart from the owner's terms.
2. `loop pr-check --local` passes.

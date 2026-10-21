# ORDER: infra-loop-harness, claude-20 (record the labs' P-1 and P-2 results)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `9c47d30d4`.

## Goal

On 21 Oct, Kimi, DeepSeek and Antigravity joined the loop and posted their probe results on #2028. Record them on [the loop page](../../../README.md), in the P-1 and P-2 rows and in the per-client line under "Subagents", each with a link to its comment. Codex has not joined yet, so it still owes both probes.

The planted fault: **a result that its lab's #2028 comment does not support.** The REPORT ties each recorded fact to its comment.

## Cancel probe (run first)

Do the loop page's P-1 and P-2 rows still say that the probes are owed by kimi, antigravity and deepseek? Result at `9c47d30d4`: yes, both rows say so, and line 189 calls those three clients unverified. The task stands. Class: instrument.

## Files in scope

- `docs/loop/README.md`: the P-1 and P-2 rows, and the per-client line under "Subagents".
- `docs/loop/tasks/infra-loop-harness/claude-20/**`

## Done when

- Each of the three labs has its P-1 and P-2 result in the rows, each with a link to its #2028 comment, and Codex is still listed as owing both.
- The per-client line says which clients fire hooks inside subagents.
- Claim-wording, links and the copyright check pass. `loop pr-check --local` fails only `scope`.

## Stop and ask if

- A lab's comment and the page would disagree on something already recorded.
- Any file outside scope needs to change.

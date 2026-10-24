# REPORT: object-store-consumer-m3, deepseek-63 (record the owner-registered M3 thresholds)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2873. Base: `690fafc4c`. `References #2873`; item stays `ready`.

## What did not work, first

- **The blocker I was about to escalate was stale.** While checking the FIT prerequisites for the frozen panel, the README's "awaits owner ratification" line looked like an open owner action; reading #2028 showed the owner had already registered card `6339008955` at 2026-10-23T02:36:36Z ("Yes, register all six (Recommended)"). The right move was a record repair, not another decision request.
- **The registered wording differs slightly from the sheet's table** (e.g. the registration says "UR4SESS1 restart byte-identical 64/64" where the table says "token-for-token identical continuation"). The repair quotes the registered text verbatim and states it governs, instead of silently rewriting the table.
- No model, FIT or GPU work was attempted; registration releases nothing.

## What changed

| File | Change |
|---|---|
| `docs/labs/object-store-consumer-m3/README.md` | card marked **owner-registered** (2026-10-23T02:36:36Z, comment `6356034555`); status line no longer asks for ratification and names only the real remaining prerequisites; §3 retitled **Owner-Registered Acceptance Thresholds** with the verbatim registered bar text and a note that it governs; panel-freeze pins and stop conditions unchanged |
| this folder | ORDER and REPORT |

## Measurements

| Check | Result |
|---|---|
| owner registration source | #2028 comment `6356034555` at 2026-10-23T02:36:36Z: "Yes, register all six (Recommended)", card `6339008955` registered exactly as proposed |
| threshold changes | none (text records the existing registration) |
| code/model/compute | none |

## Checks run at the head

`git diff --check`, `loop report --check`, `loop pr-check --local`, `loop gates --full` (see closing line). No Rust, data, bar or hold change.

## Skills used

- `solution-integrity`: verified the owner registration in the decision comment before claiming an owner action, and quoted the registered text rather than paraphrasing it.
- `scope-guard`: the lab README and this folder only.

## Still open

- **M1/M2 model selection, a non-author `TEST FITNESS: FIT` review of the frozen configuration, and Casey's typed release** are still required before any on-model serving. The item stays `ready`; the frozen panel pins and the registered thresholds are unchanged.

## Plain summary

- The M3 durable-memory thresholds were already owner-registered on 23 October; the sheet still asked for that registration. This repair records the owner's quote, time and link, quotes the registered bar text verbatim, and keeps the sheet DRAFT for the model, FIT and typed release that remain. No threshold or compute changed.
# REPORT: m7-parity-library-cli-browser, kimi-13

Lab: kimi. Order: [ORDER.md](ORDER.md).

## What did not work, first

- `uor-harness panel show m7-parity` on the installed J3-build binary ignores the id argument and prints every panel; the new panel was verified by grepping the full listing instead. No harness defect filed: the listing itself rendered the panel correctly, and the `panel show` semantics belong to the D30 port ledger, not this task.
- No other attempt or check failed. No GPU, no model compute, no code change: this task is registration and documentation only.

## What changed

- `docs/harness/panels/m7-parity.json` (new, 1 file): frozen panel registering the bar `m7-parity-prompts-v1` — the fixed 8-prompt panel (token ids `0..40`, listed verbatim), 48 tokens per prompt with the fresh-process `32 + 16` save/restore split, byte-exact `UR4SESS1` snapshots across the Rust library, `uor-chat` CLI and browser worker, greedy integer argmax at temperature 0, `lorentz` read mode, pinned synthetic stand-in (`mk-synthetic-stack` seed 5, sha256 `d60f88d6…92df`), and the accepted-M4-artifact substitution rule. Bar `current` is 0/8 with a note that the synthetic stand-in pass (8/8 prompts, 384/384 tokens, 16/16 snapshots, 3/3 tamper controls, #3008) validates the harness and is not a milestone result.
- `docs/roadmap/items/m7-parity-library-cli-browser.md`: Done-when names the registered bar and keeps the accepted-artifact condition; the item stays `ready`.
- `docs/products/web-client.md`: the parity row now says the bar is registered; re-execution on an accepted M4 `UORLUT01` artifact remains.

Size by class: 3 docs/registry files changed, 1 new; no code, no tests (nothing executable changed). The ORDER gave no numeric estimate; the change is minimal by construction. (The `m4-lawful-likelihood-bpb` scope line that was first drafted into this task moved to kimi-12 / #3019, because `pr-check` reads scope at the merge base.)

## Measurements

Nothing was measured; there is no runnable behaviour to measure. The one verification is the registry render: `uor-harness panel show m7-parity` (full listing grepped) prints the panel with `frozen: yes`, all fields, and the bar line `m7-parity-prompts-v1: … 0/8 -> 8/8 prompts [higher is better]`.

## Checks run at the head

- `uor-harness panel show m7-parity` (registry render): panel present and complete — see above.
- `python3 -m json.tool docs/harness/panels/m7-parity.json`: valid JSON.
- `uor-harness loop report --check`: result recorded in the PR body.
- `uor-harness loop pr-check --local`: result recorded in the PR body; `scope` requires kimi-12 (#3019) merged into the base first, since the rule reads the item's scope at the merge base.

## Skills used

None.

## Still open

- Re-execution of the identical protocol on the accepted M4 `UORLUT01` artifact (#2857 via `m4-export-uorlut01-cli`); the item `m7-parity-library-cli-browser` (#2867) stays `ready` for it.
- The M7 acceptance panel `m7-distribution` stays deliberately unfrozen with no bars; this registration does not change it.

## Plain summary

The project must prove that its model gives exactly the same answers whether it runs as a Rust library, as a command-line program, or inside a web browser. The harness that checks this was built and passed on a stand-in model, but nobody had written down the official measuring stick: which test prompts, how many tokens, which sampling settings. This change writes that measuring stick down in the project's registry, exactly as the builder requested and the item instructed ("ask the PM to register it"), so that when the real exported model exists, everyone re-runs the identical check and the result counts toward the milestone.

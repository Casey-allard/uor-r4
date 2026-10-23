# ORDER: m7-parity-library-cli-browser, kimi-13 (PM registration of the m7-parity-prompts-v1 bar)

Lab: kimi. Item: [m7-parity-library-cli-browser](../../../../roadmap/items/m7-parity-library-cli-browser.md) (#2867). This is a PM duty the item delegates to the PM ("Ask the PM to register the prompt list and token count as the bar"), not an item claim: the item stays `ready` and the re-execution on an accepted M4 `UORLUT01` artifact remains open. Stacked on kimi-12 (#3019), which adds this item's scope entries for the panel file and the item file; this PR's `pr-check` scope passes only after #3019 merges, because the rule reads scope at the merge base.
Base: `efa44691e`.

## Goal

Register the delivered comparison protocol as the M7 parity bar, exactly as requested in the revised bar-registration request (#2867 comment 6352825431, superseding 6352106589): the 8-prompt `m7-parity-prompts-v1` panel (token IDs in `0..40`), 48 generated tokens per prompt with the fresh-process `32 + 16` save/restore split, `UR4SESS1` snapshot byte-equality (plus SHA-256) across the Rust library, `uor-chat` CLI and browser worker, greedy integer argmax at temperature 0 (lowest id on ties), and the `lorentz` read mode on the pinned synthetic stand-in (`mk-synthetic-stack` seed 5, sha256 `d60f88d6…92df`), substituting the accepted M4 `UORLUT01` artifact when one exists. The milestone bar itself (bit-identical tokens and snapshots across the three hosts, set by #2035) is unchanged; this registers the measurement protocol the item left unpinned.

## Cancel probe (run first)

`ls docs/harness/panels/ | grep -i parity` and `grep -c "not yet registered" docs/roadmap/items/m7-parity-library-cli-browser.md` at the base: no parity panel file exists and the item still says the bar "is not yet registered" (1 match). The work is needed; the task stands.

## Files in scope

- `docs/harness/panels/m7-parity.json` (new)
- `docs/roadmap/items/m7-parity-library-cli-browser.md` (Done-when names the registered bar)
- `docs/products/web-client.md` (status line: bar registered; re-execution on the accepted artifact remains)
- `docs/loop/tasks/m7-parity-library-cli-browser/kimi-13/**` (this ORDER and the REPORT)

Anything else is a stop-and-ask.

## Done when

- `docs/harness/panels/m7-parity.json` parses through the harness registry (`uor-harness panel show m7-parity` or equivalent renders it) and carries the frozen protocol: the 8 prompt rows with their exact token IDs, the 48/32+16 token counts, the sampler policy, the comparison rule and the synthetic stand-in pin, with one bar `m7-parity-prompts-v1` whose note states that the synthetic stand-in pass (8/8 prompts, 384/384 tokens, 16/16 snapshots, 3/3 tamper controls on #3008) validates the harness and is not a milestone result.
- The item's Done-when names the registered bar and keeps the accepted-artifact condition.
- `docs/products/web-client.md` no longer lists PM bar registration as pending.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the head, scope included (requires #3019 merged into the base, per the note above).

## Stop and ask if

- Registering the bar would change the milestone bar itself (owner-only), rather than pin the delegated protocol.
- Any file outside the list above needs a change.
- The registration request is withdrawn or contradicted on #2867 before this merges.

# REPORT: infra-loop-harness, kimi-1 (scope lines on ready items)

Lab: kimi. Order: folded into [kimi-3/ORDER.md](../kimi-3/ORDER.md) as history; this folder's original ORDER.md was removed so the PR adds exactly one ORDER, which `loop review --check` and `loop pr-check` both require.

## What did not work, first

- `uor-harness loop claim infra-loop-harness` refused with "item infra-loop-harness has no scope: line" — the very gap this PR closes. Fell back to `team claim --branch kimi/item-scopes --lab kimi` plus this hand-written ORDER, as the joining instructions say until scopes land.
- The ORDER said "eight open loop items" for the `Closes #` step edit; the buggy sentence ("open the PR with `References #<the loop issue>`") existed only in the seven per-PR items (3, 7c, 8, 9, 10, 11, 12). The umbrella item's own step already carries the right semantics ("the last PR says `Closes #<this issue>`"), so it was left unchanged.
- `pkg/` and `studio/` do not exist at this base (they exist only in the stale owner checkout). Scopes for the M7/M8/desktop-client items point at `crates/uor-r4-integer/**` and `web/**`, which exist; `src-tauri/` and `models/` are new folders the scopes name prospectively, not existing paths.
- `xtask roadmap render` rewrote `docs/roadmap/README.md` byte-identical: the rendered table does not show `scope:`, so the page carries no visible diff. Render --check confirms current.

## What changed

- `docs/roadmap/items/*.md`: a `scope:` line on each of the 51 ready items (loop items 3, 7c, 8–12 first, then the A8 items DeepSeek is on, then the rest), each written from the item's own stated paths, none root-wide, all inside the validator's glob rules.
- The seven open loop PR items now open their PR with "`Closes #<this item's issue>` and `References #2904`".
- `docs/loop/tasks/infra-loop-harness/kimi-1/`: this ORDER and REPORT.

Size by class: docs 58 insertions, 7 deletions across 51 item files; no code, no tests, no generated files. Size against the ORDER's estimate: the ORDER set no estimate (size is reported, never capped, D48).

## Measurements

- `grep -l '^status: ready' docs/roadmap/items/*.md | xargs grep -l '^scope:' | wc -l` → 51 of 51 ready items scoped (was 0 at the base).
- `cargo run -q --locked -p xtask -- roadmap check` → "87 items OK (ready 51, held 26, decision 4, done 6)".
- `cargo run -q --locked -p xtask -- roadmap render --check` → README current.
- `uor-harness loop next` before the change: "any: 0 ready scoped item(s), none claimable". After: see "Still open" — the decision is re-measured on fresh main after the merge, not in this branch.

## Checks run at the head

- `xtask roadmap check` at `952a6e6ba`: pass (87 items OK).
- `xtask roadmap render --check` at `952a6e6ba`: pass (current).
- `uor-harness loop gates --quick` at `952a6e6ba`: fmt-claim-wording, fmt-harness, claim-wording all GREEN.
- `uor-harness loop report --check` at the head: PASS.
- `uor-harness loop pr-check --local` at the head: fails only `scope` — the item has no `scope:` at the merge base until this PR lands (the known, expected failure named in the ORDER); order, commit-order, report, owner-paths and rule-sources all PASS.

## Skills used

- scope-guard: the written edge (the ORDER's file list) and "size is checked before content" (`git diff --stat` before review). The ORDER, the loop page and the PM lane note were the opener.

## Still open

- The 51 scopes are PM first drafts from each item's own text. A claiming lab that needs a path outside its item's scope stops and asks, and the scope is widened or narrowed by a decision then, per the ORDER template.
- `loop next` on fresh main should stop saying "0 ready scoped item(s)" once this merges; verified after the merge with `loop close --verify`.
- DeepSeek and Antigravity have in-flight branches that touch item files; they merge these edits forward (noted in the lane instructions).

## Plain summary

The loop's claim command refuses to hand out work until every roadmap item names the files its work may change (its "scope"). No item had one, so every lab was stopped. This change writes that file list into all 51 ready items, fixes seven loop setup items so their pull requests will close their own issue (they only referenced the umbrella issue before), and re-renders the roadmap page. The roadmap's own checker passes on all 87 items.

References #2904.

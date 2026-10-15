# #2080 — Freeze held-out memory acceptance panel conversational-v5* for #2029 criterion 1

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T20:00:15Z
- **Updated:** 2026-10-09T20:04:32Z
- **Closed:** 2026-10-09T20:02:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2080
- **Merged:** 2026-10-09T20:02:11Z
- **Merge commit:** 7d7cb5492e4aa9cba0afa483b0db5e8d6733fce5
- **Base / Head:** main / UOR-Foundation:panel-freeze-v5
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

Acceptance criterion 1 of #2029 (`>= 34/40` `check_pass` on the chat-grade v4 memory panel) can no longer be measured on v4: models have replied to it and its misses were inspected by name (`mem-11`, `mem-20`, `mem-29`, six `no_value` rows), so v4 is development evidence. A v4-equivalent panel must be drawn and sealed **before** a candidate is selected — freezing it later would repeat the problem it exists to solve. This is that draw.

**References #2029** (does not close it — criterion 1 is not met and cannot be until a candidate runs the frozen acceptance once).

## Panel

`data/panels/conversational-v5*`: 64 rows, ids `conv-v5-*`, tier C.

| file | sha256 |
|---|---|
| `conversational-v5.json` | `e766cfe9a311eb85c465dba7a9d5b74961f8ccb75cd9f93b3b3e9337b992c750` |
| `conversational-v5-a.json` | `9345b60d1762be56d8b488521887574bfc54ce1e588e31be386c012a2f9f6e3d` |
| `conversational-v5-b.json` | `4daa10eee83020014df4322d3593c419d838f1bb0865c9f3305f825e0177813d` |
| `conversational-v5-checks.tsv` | `737c4dfd5a65fe49c207afe40e88bf354c7cf9dd45fa173c54e639bb0aa0dbc8` |
| `conversational-v5-swaps.tsv` | `6e3ae51469863239ff30b733af4ec0423cf230213ae2673fe71b0e79a9a484c8` |
| `conversational-v5-provenance.tsv` | `60e72c498294c61d997f643f4305bad8b936d78fbd456217498cadd11754b254` |

`multi_turn_memory` 40 (`exact`, `history=recall`) + `unknowable_or_impossible` 24 (`abstain_exact`, 12 single-turn, 12 multi-turn). Same six-column checks format, same binding-swap control, same `RowCheck::passes` rule, same manifest seal as v4. 34 memory rows list the distractor's key words; the six update rows cannot, as in v4.

## Provenance: values are drawn, not chosen

320 values drawn by the deterministic rule `panel-v5-draw/1` (`scripts/panel_v5_draw.py`, 602 lines, re-runnable byte for byte) from named, hashed sources:

- `/usr/share/dict/propernames` `626d634b…` — personal names (127 after filters)
- `/usr/share/dict/web2` `be41ad97…` — place names (28 after filters)
- `research/…/ptb/ptb.train.txt` `fcea919f…` (committed in-repo) — attestation filter only
- `rule:panel-v5-number-draw` — two-digit numbers absent from every earlier panel

Each value's source, source sha256 and `sha256(SEED|slot)` digest is in `conversational-v5-provenance.tsv`; the source line, pool index, pool size and rule are in `docs/labs/panel-freeze-2026-10-09/provenance-draw.tsv`. Frames, keys and the scene catalogue are authored in the script and are not values. **No model was loaded, run or consulted, and `chat-grade` was not invoked; pod spend zero.**

**Declared deviation:** v5 draws only names, places and two-digit numbers — the classes with a named, non-invented source here. v4's wider subject range is not reproduced; the row shape, split, check rule and control set are.

## Frozen acceptance criteria (written before any model sees it)

1. **Primary:** `>= 34` of the memory rows at `check_pass` on the frozen `exact` check — not `acceptable`. Per category, every failing row named.
2. **Secondary, not gating:** `acceptable` (unstable judge, rows named), the 24 unknowable rows, and the whole control set.
3. **One declared run**, passing `checks=data/panels/conversational-v5-checks.tsv` (the frozen binary embeds only the v2/v3/v4 checks; without `checks=` every v5 row is unchecked and a multi-turn row is a hard error). The report records the checks path + sha256.
4. Parent baseline measured before any candidate number is believed.
5. This piece meets no criterion; a new panel makes a milestone measurable, it does not move it.

## Checks run (all passing)

- `shasum -a 256 -c MANIFEST.sha256` from `data/panels/` → **26 files, 26 OK**. (The root-relative form `shasum -a 256 -c data/panels/MANIFEST.sha256` cannot work for any of the 26, because the manifest has always listed bare filenames.)
- `python3 scripts/check_panel_v4_novelty.py` (unchanged) → **pass**, 140 v4 value words and 43 names against a 2,420-word vocabulary from 23 files, no collision.
- `python3 scripts/check_panel_v5_novelty.py` (new) → **pass**, 75 v5 value words and 109 names against 2,142 words from 22 earlier files, and v4's 155 value/name words against the v5 files.
- `python3 scripts/check_panel_v5_conformance.py` (new; Python port of `validate_checks`, `abstention_fault`, `fabricated_specifics`, `copy_replies`, `check_only_controls` and the M-world leak rule) → **pass**: bare expected spelling 40/40, binding swap 0/40, echo last/history 0, copy-first 20/40 and copy-last 20/40, constants 0/0/0 on memory and 24/24/0 on unknowable, adversarial abstentions 0/24, **zero M-world phrasing leaks over 1,834 patterns**.
- `python3 scripts/check_claim_wording.py` → pass.

## Limitations

- **`chat-grade check` was not run** (CPU-only, no-grader constraint), so the authoritative structural validation — including the context/token-position check against the ladder tokenizer — remains and **must** run before the acceptance run. The port is not the binary; if they disagree, the binary wins and the panel is repaired under a new name.
- No model has replied to any v5 request, so the panel's difficulty is **unmeasured**; no equivalence to v4's 31/40 is claimed beyond the shared rule and shape.
- The two dictionaries are machine-local, pinned by sha256 rather than committed.
- The v4 checker reads every `data/panels/` file as a bag of words; row ids are therefore zero-padded to three digits and the in-panel provenance file avoids bare numbers. Recorded in the lab record.
- STATUS.md, ROADMAP.md and #2028 unchanged: no served model, BPB headline or milestone moves.

## Delivery

Canonical record `docs/labs/panel-freeze-2026-10-09/README.md` (question, identity, provenance, frozen criteria, seal, limitations, Next), newest-first `docs/integration/current-state.md` entry ending in **Next:**, and `data/panels/README.md` updated — v4 and v3 status lines now point to v5, and the v4 novelty counts are corrected for the enlarged vocabulary. One short-lived branch; delete the branch and worktree and run `uor-hygiene --apply` after merge.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


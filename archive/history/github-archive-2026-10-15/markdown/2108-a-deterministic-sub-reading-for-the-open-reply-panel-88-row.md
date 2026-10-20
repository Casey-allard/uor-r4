# #2108 — A deterministic sub-reading for the open reply panel: 88 row checks, and a canned reply cannot pass one (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T23:17:55Z
- **Updated:** 2026-10-09T23:21:51Z
- **Closed:** 2026-10-09T23:19:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2108
- **Merged:** 2026-10-09T23:19:32Z
- **Merge commit:** 7cc3fa4b6c19408250b08e53034fed4f0f710113
- **Base / Head:** main / UOR-Foundation:reply-deterministic-checks
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The headline is the coverage, and it is a finding

Measured on the 232 requests **before a single check was authored**:

| | rows | share |
|---|---:|---:|
| **panel** | **232** | 100% |
| **exactly determinable — checks built** | **88** | **37.9%** |
| — ill-posed rows (`abstain_exact`) | 67 | 28.9% |
| — content rows (`any`) | 21 | 9.1% |
| **no judge-free correct answer exists** | **144** | **62.1%** |

The 144 are open-ended requests — *"What are the key components of a successful marketing campaign?"*, *"Develop a catchphrase for a bike shop."* — where any fluent on-topic reply is as correct as any other. **This is stronger than Result 6's claim.** Result 6 said the instrument is judge-only and the judge's line is not in the text. This says something more basic: **for most of this panel there is no judge-free correct answer to check, because the request is open-ended.** A panel two-thirds of which cannot be scored deterministically cannot gate a milestone no matter how good its judge is.

**So the panel can carry a deterministic SUB-READING, not a replacement for 116/232 — and criterion 1 remains NOT MET. A new instrument is not a met milestone.**

## The canned-reply control PASSES — run by the grader, not by me

`chat-grade check constants=<file>` applies the frozen `RowCheck::passes` through `check_only_controls`, so the control arm **is** the instrument rather than a reimplementation of it:

| control reading | value |
|---|---|
| total canned-constant passes, all categories | **0** |
| `Hello! How can I help you today?` (18.6% / 23.9% of accepted rows) | **0 of 88** |
| the other twelve memorised strings | 0 of 88 each |

The greeting that carries a fifth to a quarter of the accepted set passes **none** of the deterministic checks. That is the measurement Result 7 predicted.

## What was built

| file | sha256 | contents |
|---|---|---|
| `data/panels/reply-panel-checks.tsv` | `ad4e9a14…` | 88 row checks |
| `reply-panel-checks-provenance.tsv` | `01de5bb2…` | 88 provenance rows |
| `reply-panel-canned.txt` | `ef8bb147…` | 13 observed memorised replies |

**21 content rows, kind `any`:** the reply must contain the row's required content — `ask-02` → `honey`, `ask-06` → `cow|chicken|pig|horse|sheep|goat|duck|animal`, `heldout-043` → `96`. **The anchors differ per row and none occurs in any canned reply, so a single canned string cannot pass across rows**: it would have to contain every row's content word at once. That is the mechanism, not a promise.

**67 ill-posed rows, kind `abstain_exact`:** selected by chat-grade's own recorded `heldout-ill-posed-v3-ids.txt` rather than by hand; the request's text does not contain the material it refers to, and the check rejects a fabricated specific.

**Authoring discipline, auditable:** anchors were written from the *request*, never from the reply; the observed replies were used only to build the control list. The builder drops any anchor present in the row's last user turn or in a canned reply — one drop is recorded (`heldout-132`'s `lemon` is in the request).

## Structural validation

`check_panel` **pass** · **88 of 88 rows checked**, `checks_without_loaded_request: []` · **worst-case context position 168** of 384 (`follow-02`) · `shasum -a 256 -c MANIFEST.sha256` from `data/panels/` → **29 files, 29 OK** · both novelty checkers re-run unchanged with the new files present and **both pass**.

Recorded: the **frozen macOS artifact `a5071cfe…` predates the `check` subcommand** (usage line `extract|grade`), so validation used a source build of `chat-grade` at current main (5m37s, CPU, no model); `check` is dispatched at `chat-grade.rs:216`. A small real gap in the frozen artifact; it does not affect the sealed runs, which used `grade`.

## Frozen target and first reading

Frozen **before any model is run against these checks** (the panel is development evidence either way): primary is `check_pass` over the **21 content rows**, the component a canned reply cannot pass; the 67 ill-posed rows are reported secondarily and read against their best constant, as v3 does for its clarify and unknowable categories.

**First reading, retrospective** on the sealed 29M replies: **2 of 88** — 2 of 21 content rows (`ask-03` contains "hydrated"; `ask-06` names a horse) and **0 of 67 ill-posed rows**. The misses are substantive, not formatting: `ask-01` answers "Why do leaves change colour?" with *"The fall is change color in the fall."*; `heldout-103` answers "write the primes 1-12 and add them" with *"1, 2, 3, 4, 5, 6, 7, 8, 9, 10."*

The 0/67 is a real diagnostic **with one measured limit**: 27 of 67 replies name a fabricated specific and 8 of 67 ask a clarifying question, but `abstain_exact` does not credit a question. It is used because the alternative is unsafe as it stands — **a `question`-kind clarify check would be passed by the canned greeting on all 67 rows**.

## The unfinished part, named

`forbid`/`keys` are **not expressible for a single-turn row** with the current grammar: they are parsed only for `exact`, which `validate_checks` requires to be a multi-turn recall row, and for `abstain_exact`. The fix is an additive kind — **`reply_exact`**: `exact`'s rule without the recall requirement, permitted with `history=none`. That is the `Next:`.

## Recommendation

**Restate criterion 1's reply half as a deterministic sub-reading over the 37.9% that is determinable, and give the open-ended remainder no gate at all** — rather than build a second instrument for it. A gate there would be the judge again, and Step 0a already recorded that the lever for this panel is the float model's corpus/knowledge/capacity.

## Delivery

New record [`docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md`](docs/labs/reply-panel-deterministic-checks-2026-10-09/README.md); newest-first [`current-state.md`](docs/integration/current-state.md) entry ending in **Next:**; stale docs fixed (`data/panels/README.md` inventory, the open reply panel record's Next, the ladder runbook's chat-grade §6); `scripts/reply_panel_checks_build.py` and `scripts/reply_panel_checks_read.py`. `check_claim_wording.py` passes; `scripts/__pycache__/` not committed. **STATUS/ROADMAP/#2028 unchanged — checked, not assumed:** 43/232 unchanged, criterion 1 still **NOT MET**.

**References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


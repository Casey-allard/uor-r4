# #1753 — conversational-v4: blind held-out memory panel (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T19:54:19Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-10-05T21:24:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1753
- **Merged:** 2026-10-05T21:24:44Z
- **Merge commit:** dc9a4697f335edcc897fc684b1efa7987f3bb600
- **Base / Head:** main / UOR-Foundation:claude/panel-v4
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Do not merge until reviewed at the exact head.

## What

`conversational-v4`: a fresh held-out panel for the Step 6c acceptance run, authored blind on 2026-10-05. **v3 becomes a development panel** (its misses were inspected). The blindness rule was kept: I did not open the dialogue-recall training generator, anything under the local Step 6 directories, or #820 comments after 2026-10-05 18:00 UTC, and I did not search for the templates of any training generator.

The panel uses the same format and checking code as v3 (`chat-grade` request JSON, a 6-column checks TSV, a swaps TSV, `exact` / `abstain_exact`):

| Category | Rows | Turns | Check |
|---|---|---|---|
| `multi_turn_memory` | 40 | 13 with 2, 23 with 3, 4 with 4 | `exact` + binding-swap control |
| `unknowable_or_impossible` | 24 | 12 single, 12 multi-turn (related facts given, the asked one not) | `abstain_exact` |

- Memory rows cover pets, food, family, school, jobs, places, times of day, months, numbers, money, colours, toys, plants and animals. The question forms are what / which / who / where / when / how old / how much / whose, plus "Tell me again ...", "Do you remember ...?" and corrections. In 22 rows the distractor is the same type as the answer (5 of these are updates or corrections). In the other 18, the other stated values are a different type. Every row forbids its stated distractor values, and 16 rows also list the distractor's key words. Copy-first / copy-last pass 19 / 20 of 40.
- Files: `data/panels/conversational-v4{,-a,-b}.json` (a = rows 1–32, b = rows 33–64), `conversational-v4-checks.tsv`, `conversational-v4-swaps.tsv`, `MANIFEST.sha256`, and a new README section.
- `chat-grade.rs`: embeds the v4 checks and swaps (`EMBEDDED_CHECKS` now holds 3 files; `EMBEDDED_SWAPS` is now an array whose files are merged). It adds a check-only control `expected_value`, which replies with each expected spelling on its own; a row counts as passing only when every spelling passes. That changes reports additively. New test `panel_v4_rows_checks_and_check_only_controls`. The M-world leak test now also covers v4 (renamed `panels_v3_v4_share_no_m_world_phrasing`). The v3 test's swap-id assertion is now filtered to `conv-v3-*`.
- `scripts/check_panel_v4_novelty.py`: the value and name novelty check.
- Local copies of the five v4 files are in `~/uor-r4-local/ladder/panel/`, with lines added to its `README.txt` and `SHA256SUMS.txt`. They are byte-identical to the repo files.

## Validation, run at this head (CARGO_TARGET_DIR=~/.cache/uor-claude-target)

`cargo fmt -p uor-r4-training --check` passes. `cargo test --offline --release -p uor-r4-training --bin chat-grade`: 16 passed, 0 failed.

`chat-grade check requests=conversational-v4-a.json,conversational-v4-b.json tokenizer=<ladder tokenizer> context=384` exits 0 with `check_panel: pass`. The worst case is 347 positions (`conv-v4-mem-23`). The check-only controls:

| | memory (40) | unknowable (24) |
|---|---|---|
| **binding swap** | **0 / 40 pass** (every swap fails) | – |
| **expected value** (each spelling as a bare reply) | **40 / 40 pass** (every expected value passes) | – |
| echo last turn / echo history | 0 / 0 | 0 / 0 |
| copy first / last stated | 19 / 20 | – |
| constants 1, 2, 3 | 0, 0, 0 | 24, 0, 0 |
| adversarial abstentions | – | 0, 0, 0 |

As on v3, constant 1 ("I'm not sure. Can you tell me more ...?") passes every unknowable check, so that category can only be read against its best constant.

<details><summary>Full <code>chat-grade check</code> JSON (the <code>checks_without_loaded_request</code> list of v2/v3 ids is dropped)</summary>

```json
{
 "categories": {
  "multi_turn_memory": 40,
  "unknowable_or_impossible": 24
 },
 "check_only_controls": {
  "multi_turn_memory": {
   "adversarial_abstentions": [
    {
     "reply": "Yes! I can't wait to play with you!",
     "result": {
      "check_pass": 0,
      "checked_rows": 0
     }
    },
    {
     "reply": "I'm not sure. Sam is the one.",
     "result": {
      "check_pass": 0,
      "checked_rows": 0
     }
    },
    {
     "reply": "I don't know, but they are under the bed.",
     "result": {
      "check_pass": 0,
      "checked_rows": 0
     }
    }
   ],
   "binding_swap": {
    "check_pass": 0,
    "checked_rows": 40
   },
   "constants": [
    {
     "check_pass": 0,
     "checked_rows": 40
    },
    {
     "check_pass": 0,
     "checked_rows": 40
    },
    {
     "check_pass": 0,
     "checked_rows": 40
    }
   ],
   "copy": {
    "copy_first_stated": {
     "check_pass": 19,
     "checked_rows": 40
    },
    "copy_last_stated": {
     "check_pass": 20,
     "checked_rows": 40
    }
   },
   "echo_history": {
    "check_pass": 0,
    "checked_rows": 40
   },
   "echo_last": {
    "check_pass": 0,
    "checked_rows": 40
   },
   "expected_value": {
    "check_pass": 40,
    "checked_rows": 40
   }
  },
  "unknowable_or_impossible": {
   "adversarial_abstentions": [
    {
     "reply": "Yes! I can't wait to play with you!",
     "result": {
      "check_pass": 0,
      "checked_rows": 24
     }
    },
    {
     "reply": "I'm not sure. Sam is the one.",
     "result": {
      "check_pass": 0,
      "checked_rows": 24
     }
    },
    {
     "reply": "I don't know, but they are under the bed.",
     "result": {
      "check_pass": 0,
      "checked_rows": 24
     }
    }
   ],
   "binding_swap": {
    "check_pass": 0,
    "checked_rows": 0
   },
   "constants": [
    {
     "check_pass": 24,
     "checked_rows": 24
    },
    {
     "check_pass": 0,
     "checked_rows": 24
    },
    {
     "check_pass": 0,
     "checked_rows": 24
    }
   ],
   "copy": {
    "copy_first_stated": {
     "check_pass": 0,
     "checked_rows": 0
    },
    "copy_last_stated": {
     "check_pass": 0,
     "checked_rows": 0
    }
   },
   "echo_history": {
    "check_pass": 0,
    "checked_rows": 24
   },
   "echo_last": {
    "check_pass": 0,
    "checked_rows": 24
   },
   "expected_value": {
    "check_pass": 0,
    "checked_rows": 0
   }
  }
 },
 "check_panel": "pass",
 "checked_rows_by_category_and_kind": {
  "multi_turn_memory/exact": 40,
  "unknowable_or_impossible/abstain_exact": 24
 },
 "checked_rows_by_history": {
  "none": 12,
  "recall": 40,
  "topic": 12
 },
 "checks": {
  "embedded": [
   {
    "embedded": "data/panels/conversational-v2-checks.tsv",
    "rows": 84,
    "sha256": "5589462f3ed8ebf13c1b2bb71ecdeddd866924c5d8e7de634579cef6cbe72184"
   },
   {
    "embedded": "data/panels/conversational-v3-checks.tsv",
    "rows": 84,
    "sha256": "be82781146e500fe948e7a5a3062d09212f1d4c524c3537c1850faf1657c30c3"
   },
   {
    "embedded": "data/panels/conversational-v4-checks.tsv",
    "rows": 64,
    "sha256": "b4238e112e5e66cfd773de75f71ae8eb39c9077c172fe8ff0f4ba8b33cc5793e"
   }
  ],
  "rows": 232
 },
 "context": 384,
 "excluded": 0,
 "max_new_tokens": 64,
 "multi_turn": 52,
 "requests": 64,
 "worst_case_history": {
  "id": "conv-v4-mem-23",
  "positions": 347
 }
}
```
</details>

### Novelty check: `python3 scripts/check_panel_v4_novelty.py` (exit 0)

The script checks every word of every v4 expected and forbidden value, and every proper name in the v4 turns. Each word is compared, using chat-grade's word rule, with all earlier `data/panels/*` files (the README is read without its new v4 section) and with `~/uor-r4-local/ladder/panel/*.json`. It does not check distractor key words or the answer-class lists of the unknowable rows; those are ordinary words by design and are never stated in a v4 turn.

```json
{
 "earlier_files": 31,
 "earlier_vocabulary": 2520,
 "v4_rows": 64,
 "value_words_checked": 140,
 "proper_names_checked": [
  "abernathy",
  "amara",
  "april",
  "bodhi",
  "clementine",
  "dario",
  "duluth",
  "ezra",
  "fernando",
  "galway",
  "halifax",
  "hamid",
  "ingrid",
  "juniper",
  "kwame",
  "lindqvist",
  "linnea",
  "lisbon",
  "marzipan",
  "mireille",
  "mochi",
  "nico",
  "nutmeg",
  "odette",
  "odette's",
  "okonkwo",
  "oskar",
  "pelican",
  "pernilla",
  "prague",
  "pudding",
  "quill",
  "rafael",
  "rosalind",
  "saoirse",
  "september",
  "soren",
  "street",
  "teodor",
  "thea",
  "tofu",
  "tucson",
  "yusuf"
 ],
 "collisions": [],
 "result": "pass"
}
```

## Limits

- The checks have the same lexical limits as v3, and the rule is strict: a correct reply that also names the distractor fails.
- No closed-class lists were added to `forbid`. A hedge that names an unstated value of the same type is therefore not caught.
- The corpus leak check (`chat-grade leak corpora=...`) was not run on v4. Only the unit test against the M-world string literals was, and six turns were reworded during authoring until it passed.
- No model has replied to v4.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6001919448 — 2026-10-05T19:56:23Z — Casey-allard

Exact-head check at edcf770a (Claude lab): `rustfmt --check --edition 2021` clean on chat-grade.rs; `cargo test --release -p uor-r4-training --bin chat-grade` 16 passed; `chat-grade check` on v4-a+b with the ladder tokenizer, context 384: `check_panel: pass`, binding swap 0/40, expected value 40/40, echo 0/40, constants 0/0/0 (memory) and 24/0/0 (unknowable), copy-first/last 19/20 of 40, worst-case history 347 positions; novelty script exit 0. Note for acceptance design: the copy controls pass ~half the memory rows, so a Step 6c bar must sit well above 20/40. Approve at this head.


---
_Comment/review ids in this file: 6001919448_


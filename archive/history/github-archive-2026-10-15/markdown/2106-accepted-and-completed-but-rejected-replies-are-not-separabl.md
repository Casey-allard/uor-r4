# #2106 — Accepted and completed-but-rejected replies are not separable without the judge (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T23:02:01Z
- **Updated:** 2026-10-09T23:49:19Z
- **Closed:** 2026-10-09T23:05:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2106
- **Merged:** 2026-10-09T23:05:22Z
- **Merge commit:** 18da9585f4e81746cf5379df3bef588aa076f3b2
- **Base / Head:** main / UOR-Foundation:reply-accepted-vs-completed
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

**Result 7, a supplement to Result 6** — the comparison Result 6 could not make. Result 6 profiled the failures and contrasted **completed against cut**; the accepted set was available too, so this compares **accepted against completed-but-rejected** on fourteen judge-free properties, both directions, with the cut confound removed entirely: only replies ending in terminal punctuation, **27 accepted against 64 rejected**. CPU only, no pod, no generation, no grading, **$0**.

## Reconciliation with Result 6 — no correction needed

64 rows have `no_terminal=0` (Result 6's count) and 125 are cut. Of those 64, **62 are below the 64-token cap and 2 reach it** (`heldout-124`, `heldout-139`, both `fluent_only`). Result 6's 64 and the earlier 62 differ by exactly those two rows and by nothing else; **no conclusion changes under either definition.** Also worth recording: **16 of the 43 accepted replies are themselves cut**, so `no_terminal` means "was cut", not "failed".

## The finding: nothing separates the sets

Fourteen properties, Bonferroni-corrected. **Two survive — and they are the same observation:**

| property | accepted (27) | rejected (64) | p |
|---|---:|---:|---:|
| reply **is** `Hello! How can I help you today?` | 8 | 1 | **0.0002** |
| reply contains a question mark | 10 | 4 | **0.0006** |

Eight of the ten accepted replies containing a question mark **are that greeting**. Remove that one canned reply from both sides and **nothing separates the sets at all**:

| after removing the one canned reply | accepted (19) | rejected (63) | p |
|---|---:|---:|---:|
| median tokens | **14** | **20** | 0.45 (Mann-Whitney) |
| contains a question mark | 10.5 % | 4.8 % | 0.33 |
| starts hello / hi / hey | 5.3 % | 11.1 % | 0.67 |
| under 15 tokens | 52.6 % | 44.4 % | 0.60 |
| over 40 tokens | 26.3 % | 25.4 % | 1.00 |

Acceptance is also flat across length buckets: 1–10 tokens 30.0 %, 11–20 21.7 %, 21–30 15.4 %, 31–40 20.0 %, 41–70 23.8 %. The other twelve properties — request type, restatement, content overlap, self-reference, repetition, digits, code fences, shorter-than-request — are in the [record](docs/labs/open-reply-panel-2026-10-09/README.md).

## One correction to the natural reading of Result 6's length finding

Result 6's length contrast is **completed versus cut**, and it is correct as stated — cut replies ran to the cap and are long. It should **not** be read as "short replies are rejected": against accepted replies of the same terminal-punctuation class the rejected ones are **longer** (median 19.5 tokens against 9, Mann-Whitney p = 0.043 before the canned reply is removed, 0.45 after), and the shortest replies in the panel are accepted most often. **Length is not the discriminator; it looks like one only when the comparison is against the cap.**

## The instrument statement, now measured

The only property that predicts acceptance in this population is that the reply **is** one specific memorised string. Strip it and two sets of replies that differ by **3× in verdict** are statistically indistinguishable on all fourteen properties. That is a stronger statement than "the failures are diffuse": **the accepted set is not distinguishable from the completed-but-rejected set by anything measurable without the judge.**

And the string is not a curiosity:

| artifact | occurrences | accepted | acceptance rate | share of all accepted rows |
|---|---:|---:|---:|---:|
| 29M `chat-29m-B-lr5e-4` (43/232) | 9 | 8 | 88.9 % | **18.6 %** |
| 100M `chat-100m-C` (46/232) | 19 | 11 | 57.9 % | **23.9 %** |

Both are 3–5× the panels' base acceptance rates (18.5 % and 19.8 %). **Between a fifth and a quarter of the accepted rows on this panel are the same memorised greeting.**

## Consequence

A second, independent reason for Result 6's conclusion that the reply-panel half cannot carry criterion 1, plus one requirement on the replacement: **the deterministic row checks must make a canned reply unable to pass**, and any future number on this panel should disclose the canned-reply count beside it. `Next:` is rewritten to point at building that replacement — not at classifying the failure set a third time, and not at the decoder.

**Criterion 1 remains NOT MET and 43/232 is unchanged**; STATUS/ROADMAP/#2028 unchanged — checked, not assumed.

## Delivery

- **Result 7** in [`docs/labs/open-reply-panel-2026-10-09/README.md`](docs/labs/open-reply-panel-2026-10-09/README.md); `Next:` rewritten.
- Newest-first `docs/integration/current-state.md` entry ending in **Next:**.
- `scripts/open_reply_panel_completed_classify.py` — the comparison, with the multiplicity correction and the population reconciliation built in.
- `check_claim_wording.py` passes; `scripts/__pycache__/` not committed.
- Evidence bundle `icloud:UOR-R4/results/deepseek/reply-panel-accepted-vs-completed-2026-10-09.tar` (197,632 B, md5 `104f7011b813dc8abd1fbd15a1045740`).

**References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


# #1628 — Add post-epoch-eval.sh: the post-run measurement chain in one command

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T17:32:47Z
- **Updated:** 2026-10-02T17:34:11Z
- **Closed:** 2026-10-02T17:34:10Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1628
- **Merged:** 2026-10-02T17:34:10Z
- **Merge commit:** 47c292041a0133944245f04441cc08a777707cf1
- **Base / Head:** main / UOR-Foundation:codex/post-epoch-eval-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

The whole post-run measurement chain for a chat arm, in one command: **export → derived development panel → disjoint held-out panel → read against every prior arm.**

The goal it serves is a full in-domain epoch followed by a specific measurement. **Reconstructing that by hand hours later is how this lane has previously ended up quoting an unvalidated number** — twice from an instrument that was wrong (a reply rubric that missed correct answers, a "held-out" panel that shared the training templates). This script makes the measurement deterministic and attaches the interpretation to the number.

Two behaviours are deliberate:

- **It refuses outright when the arm has no sealed report.** A partial fit must not be measured as if it were a result — the previous run in this lane died before writing its report and could not be sealed at all.
- **It runs the disjoint held-out panel unconditionally** and prints the reading next to it: *a high derived-panel score with a low disjoint score is template memorisation, not capability.* That panel is the only instrument here that has caught a template artefact, and the arm most at risk of one scores **10/10** on the derived panel while scoring **0/40** on the disjoint one.

## Exact-head evidence

**Reviewed head:** `74927ccb`.

**Validated end to end on a real arm, not syntax-checked** — and that validation caught two defects that are fixed in this commit:

1. a **relative root was not resolved** against the reports directory, so the script refused a valid arm;
2. **`$REPORTS` was referenced before assignment**, which under `set -u` aborted the script outright.

Both would have failed *hours into a queued run* rather than at author time. That is precisely the failure this script exists to prevent, so it is worth recording that it caught its own.

**The end-to-end run reproduced the winning arm exactly:**

| | expected | reported |
|---|---:|---:|
| pooled response NLL | 2.097073 | **2.097073** |
| first-four NLL | 1.417128 | **1.417128** |
| memory (derived panel) | 4/10 | **4/10** |
| distinct replies | 38/38 | **38/38** |
| **disjoint held-out** | — | **0/40** |

The disjoint 0/40 matters: it confirms the template-detection instrument still works after being wired into the chain.

**Also exercised:** the refuse-path on an arm with no sealed report (`REFUSING: no sealed report … the run did not finish its fit`).

The validation regenerated only **derivable** directories (`integer/`, `replies-1/`) inside the sealed root; the sealed `dialogue/report.json` was untouched (timestamp unchanged) and the root has been restored to its sealed state. The authoritative copy in the cloud store is unchanged.

## Scope

An orchestration script. **No model behaviour changes and no capability claim.** It labels every result it prints as **geometry-on-principle, not a demonstrated advantage**, and prints the matched measurement — **1.9981 geometric vs 2.0111 transformer (Δ −0.0130, inside the ±0.021 seed spread) = parity** — so nothing read through it can imply a geometric advantage that has not been measured.

References #1512, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


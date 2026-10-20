# #2090 — Geometry: verified connections map, pictures of the pre-registered mechanisms, and repo-wide alignment

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:07:04Z
- **Updated:** 2026-10-09T21:09:11Z
- **Closed:** 2026-10-09T21:08:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2090
- **Merged:** 2026-10-09T21:08:25Z
- **Merge commit:** 65f2eb0468b50004dc0482c12e5671d5f6869e0a
- **Base / Head:** main / UOR-Foundation:claude-connections
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner request (9 October): every mechanism and connection documented with pictures, and every page aligned.

**Verified connections:**
- **`docs/figures/geometry/verify_connections.py`:** an exact integer/Z[φ] script, 29 of 29 claims PASS. It covers:
  - primes → semiprimes → K6 → the 15 half-turn axes;
  - Cl(0,6) bivectors (SpiralCore v63);
  - octonions/Fano as a signed XOR;
  - 2I (120 icosians, shells 1,12,20,12,30,12,20,12,1);
  - A5 acting on the six five-fold axes;
  - Z[φ];
  - E8 = 2I ∪ φ·2I;
  - Hopf (2I → 30 points, fibres of 4).

  The prime↔axis labelling is flagged as a choice (12 inequivalent).
- **`connections.svg`** plus a new `docs/geometry.md` § 10 *How the geometric pieces connect*: a conversion table (forced vs chosen, code location, model use) and two cautions. Half-turns don't compose like bivectors, and the prime↔axis labelling is chosen.

**Pictures of the pre-registered mechanisms** (§ 11):
- `exact-2i-lanes.svg` (E1);
- `octonion-signed-binding.svg`;
- `hamming-rank-read.svg`.

Each is marked "pre-registered, not measured".

**Alignment:**
- **README:** the VSA "inert" line is re-scoped (#2070/#2071/#2077), there is an experiments-in-flight table, and the credits are updated (Mark (N3mesis); Matthew's SpiralCore v69).
- **ROADMAP:** M4 is in progress, with acceptance item 0 (no softmax at runtime), and the experiments table.
- **STATUS:** the experiments table.
- Existing status figures were checked; nothing in them is contradicted.

Checks at this head: verify script 29/29 PASS; `generate.py` deterministic across 13 figures; claim wording passes; all links resolve.

References #2029 #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6089272069 — 2026-10-09T21:07:12Z — Casey-allard

Exact-head self-review at `5178a02edcf2211087d3373fe7ec315aef1548b0`: verify script 29/29 PASS, figures regenerate with no diff, claim wording passes, links resolve; doc tables match the script's claims.


---
_Comment/review ids in this file: 6089272069_


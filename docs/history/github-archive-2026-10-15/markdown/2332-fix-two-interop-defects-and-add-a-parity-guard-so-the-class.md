# #2332 — Fix two interop defects, and add a parity guard so the class stops recurring (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T14:03:13Z
- **Updated:** 2026-10-13T14:34:39Z
- **Closed:** 2026-10-13T14:31:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2332
- **Merged:** 2026-10-13T14:31:51Z
- **Merge commit:** a82e33d206849cc7572efcd3ee9ebb748fc0126e
- **Base / Head:** main / UOR-Foundation:deepseek/interop-defects
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: no model line - tooling · count 0/3 · headline 37/232 → 37/232 (unchanged)

Tooling only. Fixes both defects in Codex's report (comment `6146770185`), routed by the PM as high priority.

## Result

Decision: **KEEP** (tooling).

**Both defects are real, and both are the same shape: two implementations that are each self-consistent and disagree.**

### Defect 1 — the generic receipt command created a record its own checker rejects

`cmd_receipt` defaulted `--kind` to `receipt`; the authoritative kind set did not include it. So **every invocation of `uor-harness receipt` without an explicit kind emitted a record the validator refuses** — including the onboarding exercise. Codex found it as `unknown kind receipt` failing advisory CI on #2328.

**The fix is in the kind set, not the default, and the reason matters:** `receipt` is documented as a kind in the Rust source, the generic command exists for exactly this, and `parity.py` already drives it. Renaming the default would have been a second name for the same thing.

> **A command must never be able to create a record its own checker rejects.**

Correcting the Python set exposed a **second real bug**: `status.rs::ALL_KINDS` was **silently dropping one receipt the Python reader counted** — caught by parity the moment the first set was fixed:

```
rust 110 (… other 42) != python 111 (… other 43)
```

### Defect 2 — two answers to "how long is a check-in good for"

`cmd_presence` held `const TTL_MINUTES: i64 = 30` while the presence module and the heartbeat used **5**. So the command a lab reads to judge whether a peer is still working said thirty minutes, while the check-in said five.

```
$ uor-harness presence
4 lab(s) known (stale after 5m) · live tier: ~/.local/share/uor-r4/harness
```

## The guard, so this class stops recurring

New `parity.py` check `constants_agreement` compares what the two implementations **tell labs**:

- the presence TTL, Rust `presence --json` against `multilab.PRESENCE_TTL_MINUTES`; and
- the `--kind` default parsed from `main.rs` against the authoritative kind set.

**Disagreement here is invisible without a check** — each side is self-consistent, so only the lab reading one of them can tell. **That is why both defects survived to be found by a human instead of by CI.**

### On the already-emitted row

Codex noted it could not be corrected without violating the append-only guard and left it alone. **That was the right call**, and with `receipt` now a legitimate kind the row is **valid where it stands** — no correction, no shrinking, guard intact.

```
cargo fmt --check                              -> clean
cargo clippy --all-targets -- -D warnings      -> 0 findings
cargo test -p uor-r4-harness                   -> 113 passed
python3 -m unittest discover -s tests/harness  -> 319 passed on 3.15 AND 3.9
python3 scripts/harness/cli.py check           -> 0 errors
python3 scripts/harness/parity.py              -> all eleven comparisons ok
```

## Checklist

- [x] Checks run at this exact head (above)
- [x] Tooling only; no capability or headline changed
- [x] Record: docs/labs/interop-defects-2026-10-13/README.md
- [x] Branch deleted after merge

### Stated rather than hidden

- **`constants_agreement` reads the Rust default by regex** on `main.rs`. It catches the current spelling; a refactor that computes the default differently would make the check **silently skip** — quieter than a failure, which is the wrong direction for a guard.
- **Only two constants are covered.** `RECEIPT_KINDS`, the panel kinds and the claim staleness windows are still unpinned across the two implementations.
- **The `receipt` kind is now accepted but barely used.** Nothing routes it and the roll-up shows it as `other`; it exists for the generic command and for rows already written.
- **Kimi's item (2), the unread-note surfacing, is still not started** — this unit took the two defects the PM routed as high priority.

References #2028


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _


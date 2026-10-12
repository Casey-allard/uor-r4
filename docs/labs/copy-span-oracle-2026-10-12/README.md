# The copy-span oracle: the forced-copy oracle's attention half, implemented

References #2029 (M1; pre-registered in [#2029 comment 6119323722](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6119323722)).
Lab: Kimi, session `kimi/s1`, 2026-10-12. Source only: no training, no pod, no panel run.

> **One line.** The gate oracle (the attribution record's conditions D/E) forced the copy **gate** and showed
> the copy branch is not the answer path — but it never forced the **attention**: whether a *perfect address*
> would help stayed open, and its own record named the span hook as the next instrument. This PR builds it:
> `StackModel::set_copy_span_oracle` replaces the pointer's attention by the uniform distribution over a
> supplied window span in both serving mixtures, and `chat-grade reply copy_oracle=span checks=<panel checks>`
> supplies each memory row's span by locating the frozen check's expected value in that row's own history.

## What landed

- **`StackModel::set_copy_span_oracle(Option<(usize, usize)>)`** — a serving-time field, never saved, no
  parameter, no gradient, no training path, on the pattern of `set_read_silence` (#2191) and the gate floor
  (#2187). With a span set, the pointer's computed attention (scores, selection and route alike) is replaced
  by `1/(end − start)` on `[start, end)` in `next_scores_pointer_scores` and in `read_span_probe`; the head
  and the gate are untouched. A span outside the scored window is an error, not a silent fallback; a span on a
  model without a pointer head is refused at the setter; `None` is the historical decoder bit for bit.
- **`chat-grade reply copy_oracle=span checks=<file>`** — per row whose frozen check is `exact`, the expected
  value's *last* occurrence in the row's current history is located by decoded text (per-id concatenation, so
  token boundaries are exact) and set as the span for that row's replies. Rows without an `exact` check, and
  rows whose value is not stated in the window, run plain. Every turn's span (or its absence, with the reason)
  is recorded in `replies.json` under `copy_oracle_spans`; the checks file is bound by sha256.
- **Tests:** `the_copy_span_oracle_replaces_the_pointers_attention` (oracle flips the copied token to the
  span's content at gate ≈ 1; clearing restores the model's own attention bit for bit; an out-of-window span
  errors; empty/reversed spans are refused), `the_copy_span_oracle_needs_a_pointer_head`, and
  `the_oracle_span_is_the_values_last_occurrence` (the span search: last occurrence wins, a value crossing a
  token boundary covers both tokens, an unstated value has no span).

## The run it prepares

The pre-registered N/O1/O2 reading: oracle-off must reproduce the artifact's recorded 23/40 exactly (the
identity check), then O1 (span, learned gate) and O2 (span + gate floor 0.9) separate *addressing* from
*gating* from *emission* as the copy path's bottleneck. It follows TEST FITNESS and the artifact note below.

**Artifact status (declared):** the pre-registered artifact, the order-3 address arm S2 (`09e9dd3d…`), is not
on the laptop and not in the cloud-store index; the local fallback named in the pre-registration (the
read-binding-scaled S2) has reports locally but no weights. The instrument is artifact-agnostic; the run's
artifact will be bound by sha256 in its own report, and if neither recorded artifact is recoverable the
pre-registration will be amended with a fresh CPU-trained pointer model and its own measured baseline.

## Verification at the PR head

- `cargo test -p uor-r4-training --lib`: **919 passed, 0 failed** (with `UOR_BUILD_SOURCE_COMMIT` set, which
  also carries the three `joint_campaign` tests diagnosed on #2028 comment 6119168767).
- `cargo test -p uor-r4-training --bin chat-grade`: 20 passed, including the new span-search test.
- `cargo check -p uor-r4-training --bins`: clean; `cargo fmt --check -p uor-r4-training`: clean.
- The oracle-off path is structurally untouched: the override arms activate only on `Some(span)`.

## Cost

| | |
|---|---|
| compute | laptop CPU only (builds + focused tests); no pod, no training, no panel run |
| external | none |

## Next

The N/O1/O2 run on the bound artifact, after TEST FITNESS, as the pre-registration's three decision branches.

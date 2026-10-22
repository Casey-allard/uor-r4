# REPORT: m1-faster-writer, claude-34 (design note and host-only writer controls)

Lab: claude. Base: `fb6df367b`. Order: [ORDER.md](ORDER.md), committed first (`4600ba301`, filled in `1e345783a`). The cancel probe found no host bucketed writer and no per-cell token index in `crates/uor-r4-discrete`.

## What did not work, first

- **Clippy `-D warnings` cannot pass on this crate as it stands.** With the pinned toolchain, `cargo clippy -p uor-r4-discrete --all-targets` reports 67 warnings on `origin/main` and the same 67 on this branch, none of them in the new test file. Fixing them means touching code outside this unit, so the ORDER now says "no new finding" instead of `-D warnings`.
- **The cited "≈5×10⁶ comparisons per row" source (`finite-relational-head.md` §4.2) was not found on `main`.** The design note reproduces its order from the kernel: V + K·V `token_cell` calls, at up to 2×16 root matches each.
- **The index had to keep more than an ABSENT flag.** To match the reference when the moved token is itself the ABSENT one, it stores each token's base cell and the ABSENT count. Under a move, MAX is returned when the count after the move is above 0.
- **The kernel's extra `token_cell(target)` call feeds only the loss, so it is not mirrored.** The reference's count is therefore V·(1 + admitted).

## What changed

| Commit | What |
|---|---|
| `4600ba301`, `1e345783a` | The ORDER: claimed by `loop claim`, then filled. |
| `64cca9a21` | `crates/uor-r4-discrete/tests/writer_bucket_controls.rs`, test-only with no `src/` change and no CPU fallback. It has three parts. **A host reference** of `discrete.cu:270-345`: the roots, `AXES`, `GENERATORS` and `IDENTITY` from the crate's generated `geometry.rs` (via `include!`, as `device.rs` does); `token_cell`; `address_distance`; `token_distance`; `cell_precedes`; K admission; ABSENT returning MAX; and the `kind==1` move. **A host bucketed writer:** a CSR index, one-pass admission sorted by distinct distance values with ids ascending, an id-ordered bucket walk, and the `kind==1` move. **A pruned serving variant** that skips a cell only when its `address_distance` is strictly greater than the best so far. Six tests. |
| this commit | This REPORT, and the ORDER's clippy clause. |

The design note is on #2840 (comment 6346787358).

## Measurements

The counters are host only and per row. The bucketed writer makes 0 `token_cell` calls (1 under `kind==1`) and exactly 256 cell-distance evaluations, against the reference's V·(1 + admitted) calls. No GPU timing yet; that comes in the CUDA unit.

## Checks run at the head

- `cargo test -p uor-r4-discrete --test writer_bucket_controls`: 6 passed. The suite covers:
  - a seeded sweep over small vocabularies;
  - a cell-id tie;
  - a token-id tie;
  - an ABSENT token;
  - a `kind==1` move across cells;
  - the counter test.
- **Negative gates** on scratch copies, each restored:
  - flipping the token-id tie rule in the bucketed walk fails 4 tests;
  - pruning on `>=` in place of `>` fails 3 tests (the cross-cell tie, the move and the sweep).
- `cargo fmt -p uor-r4-discrete -- --check`: clean. Clippy: no finding in the new file; the crate's 67 warnings exist on `main`.
- `loop report --check` and `loop pr-check --local`: see the PR body.

## Skills used

- **solution-integrity:** the bucketed writer is proven against a faithful host copy of the kernel, ties and moves included, never against itself.
- **scope-guard:** one test file, with no `src/` change.

## Still open

- **The prep sheet** (step 4), with the bar's form proposed on #2840: 0 `token_cell` calls per row, 256 cell evaluations, and token distances at most the admitted members, reported on the pinned 4,096 and on 16,384 pieces. The owner registers the bar.
- **After FIT:** the CUDA writer with the index, built by nvcc in its own module, and the finite CUDA qualification on the GPU.
- The crate's existing clippy debt, as a separate item.

## Plain summary

The design shows the writer can find its token without rescanning the whole vocabulary on every row. These tests prove, on the computer and against an exact copy of today's writer, that the faster method picks the same token in every tie and edge case.

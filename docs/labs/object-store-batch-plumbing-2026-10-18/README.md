# Object-store batch plumbing for sync/repair  2026-10-18

Follow-up to the merged opt-in FETCH batch (#2607). No protocol change, no model-path code.

## Defect

`objpeer --batch` was parsed for every subcommand but read only in `pull`: `sync` and
`repair` go through `pull_series`, which hardcoded `None` (the derived default batch), so
`objpeer sync --batch 32768` was accepted and silently ignored. The usage line also omitted
`join/sync/repair`.

## Change

- New `peer::pull_series_with_batch` (same 1..=65535 clamp as `pull --batch`);
  `pull_series` delegates with `None` and `sync`/`repair` pass their `--batch` through.
- The CLI rejects flags a subcommand ignores: `--batch` only on `pull|sync|repair`,
  `--sessions`/`--concurrent` only on `serve`.
- Usage string lists `join|sync|repair` and which subcommand owns each flag.
- Tests: end-to-end `sync --batch 1024`, `repair --batch 1024` in the existing repair case, and
  rejection cases for `verify --batch`, `pull --sessions`, `join --concurrent`.

## Verification (host-only)

    cargo test --offline -p uor-r4-object-store   60 passed, 0 failed (CLI 21)
    cargo clippy --offline -p uor-r4-object-store --all-targets -- -D warnings   clean
    cargo fmt                                    applied

## Limits

- Batch stays opt-in; the local default (4096) is unchanged and the caller chooses a WAN batch.
- No remote/WAN measurement here (F4 remains: latency-emulating proxy + pipelined FETCH).
- No auto-batch heuristic; a new default would need its own evidence.

No model compute. Record only.

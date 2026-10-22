# ORDER: infra-loop-harness, claude-49 (a faithful archive copy's links are reported, not failed)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8. Base: `origin/main` once #3006 (the scope widening) has merged.

## Goal

Kimi's tool-gap card (#2028, 6351470799): PR 8's §4.7 archive copies are byte-identical copies of live files. They carry 19 relative links that resolve from the live location but break at the archive depth. The links gate's `MOVED` class needs an R100 rename, and §4.7's stub prevents one. Claude's answer (6351752514) is a new class, `COPIED`.

A NEW broken link is reported, not failed, when **all** of these hold:
1. its source is under `archive/`;
2. the source did not exist at REV, so it was added in this change;
3. the source's bytes equal a file that existed at REV (the original);
4. the same link, resolved from the original's directory, names a path that exists now.

Every other NEW broken link still fails. A new archive file with genuinely broken links fails, and so does an edited copy.

## Cancel probe (run first)

At the base, does the links gate fail on a newly added byte-identical archive copy whose link resolves from the original? If it already reports it, the task is void. Class: instrument.

## Files in scope

- `xtask/src/docs_links.rs` (from #3006)
- This task folder

## Done when

1. **Unit tests in `docs_links.rs` (`#[cfg(test)]`):**
   - a faithful copy's link that resolves from the original is `COPIED`;
   - a copy whose link does not resolve from the original either fails;
   - a non-identical file under `archive/` fails;
   - a file that existed at REV is not `COPIED`.
2. **Planted fault:** with the byte-equality condition removed, a test fails.
3. **Output:** a count line, `copied (archived copy)`, beside `moved (immutable source)`, and each case printed as `COPIED`.
4. **Checks pass:** `cargo test -p xtask`, clippy `-D warnings`, `report --check` and `pr-check --local`.

## Stop and ask if

- Finding the original needs a full blob scan that is too slow for the gate. The fix is then an index by blob id, and if that still does not work, stop.

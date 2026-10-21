# ORDER: infra-loop-pr02-item-keys, claude-12 (loop PR 2: item keys, Cancel if, done-when class, validator)

Lab: claude. Item: [infra-loop-pr02-item-keys](../../../../roadmap/items/infra-loop-pr02-item-keys.md) (#2905). Setup row: 2. Base: `e61746628`.

## Goal

Loop PR 2 from the build plan. The owner amended D52 on 21 Oct 2026, so Claude builds every setup PR while it runs alone; that amendment is in #2919. The item's done-when reads:

> `roadmap check` accepts and validates `scope:` (globs), `size:`, `lane:`, `rank:`, `notebook:`, `done_class:` (`host` or `gpu`) and a `## Cancel if` section, as the loop page's scope section lists them; they become required once the PM backfills the ready items. The planted fault turns the check red: **an item scoped `**` is refused.**

## Cancel probe (run first)

Does `xtask roadmap check` already validate any of these keys?

Result at `e61746628`: `xtask/src/roadmap.rs` has `REQUIRED_KEYS`, `SECTIONS` and `AREAS`, and no `scope`, `size`, `lane`, `rank`, `notebook`, `done_class` or "Cancel if" handling. The task stands. Class: instrument.

## Files in scope

- `xtask/src/roadmap.rs`
- `xtask/src/roadmap_sync.rs`: only if the issue body must render the new keys or section.
- `xtask/tests/**` and the test module in `roadmap.rs`.
- `docs/roadmap/README.md`: only if the renderer's output changes.
- `docs/loop/README.md`: only the scope section, and only if a key needs one sentence of definition.
- `docs/loop/tasks/infra-loop-harness/claude-12/**`

No item file is backfilled here; backfilling is the PM's step. No new `Cargo.lock` entry.

## Done when

- `cargo xtask roadmap check` passes on today's 87 items, because the new keys are optional.
- An item that carries the keys is validated:
  - `scope:` is a list of globs, and a root-wide `**` is refused;
  - `done_class:` is `host` or `gpu`;
  - `size:`, `lane:`, `rank:` and `notebook:` are well formed;
  - a `## Cancel if` section, when present, is not empty.
- The planted fault is a test showing that an item scoped `**` is refused. It fails with the refusal set aside, and passes with the refusal restored.
- `cargo test -p xtask` and `roadmap render --check` pass, as do claim-wording, links and the local copyright check.

## Stop and ask if

- A key's meaning is not settled by the loop page's scope section.
- Any file outside scope needs to change.

# REPORT: infra-k16-normal-close-accounting, deepseek-7 (K16 normal close and final resource accounting)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Base: `44ace17dc`. Closes #2827.

## What did not work, first

- **The item's `How to start` step 4 names two files outside its own `scope:` line.** It says to add a dated addendum under line 24 of `docs/labs/k16-four-role-execution-2026-10-18/README.md` and to edit `docs/integration/NOW.md:69-81`, but the scope is only `docs/harness/ledger/**`, `docs/labs/infra-k16-normal-close-accounting/**` and `docs/reference/storage.md`. I did not edit them; I wrote the record in the scoped folder and asked for a scope widening or a follow-up front-door sync as three `loop ask --scope` cards (`local/deepseek/deepseek-7/asks/1.md`-`3.md`).
- **The probe found the close was already complete, contrary to the item's summary.** The front-door correction record (`docs/labs/k16-frontdoor-close-sync-2026-10-19/README.md`) says normal close is complete: release 00:38:29Z, native down 00:38:33Z, after-down custody 131,072 B. Three front-door sentences (four-role README:24, the `docs/labs/README.md` final-custody box, NOW:81) still said pending, and the item's opening text repeats them.
- **The accounting was only in issue comments, not in the repository.** #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) records the whole lifetime 34,633 s at $0.908/h = $8.735212222 resource-cost proxy (not an invoice), storage/transfer unmeasured, no remaining idle lease/cost and no replacement allocation; #2029 [6257225762](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257225762) records the after-down closure custody. `budget show` reported `repo cumulative ledger: ABSENT under docs/labs/**/resource-ledger.json` before this change.
- **The committed ledger shards are per-lab and the `receipt` CLI writes only the caller's live shard without cost fields.** Appending a cost row to `docs/harness/ledger/codex-2026-10.jsonl` would put words in another lab's shard, and the CLI cannot carry `cost_usd`. The scope's `docs/harness/ledger/**` glob was therefore left unwritten; the machine-readable accounting is the `resource-ledger.json` chain entry that `budget show` already reads, beside the record.
- **The item's step 6 asks for a notebook row that its scope cannot allow.** `docs/research/notebook/m1-discrete-attention.md` is outside the three scope globs, and the scope rule only allows it when the item names it in a `notebook:` field (no item currently does). The row is not added; it is `loop ask --scope` card 4, and the front-door sync should add it once the item carries the field or the scope widens.
- **`uor-harness check` warns that this branch is not in the team claim registry.** The loop workflow claims by draft PR plus ORDER (`loop claim`), and writing `docs/harness/claims.json` to silence the old `status.branch_claims` warning would be an out-of-scope change (the loop PR 12 review had a claim record dropped for exactly that reason). The warning is left visible and named here.
- **`check` also reports other labs' orphan branches.** They are other labs' worktrees and branches; the rules say not to delete another lab's work, so none was touched.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/labs/infra-k16-normal-close-accounting/README.md` | new record | +90 | dated close and accounting record: lease/pod, create/release/down times, 34,633 s at $0.908/h = $8.735212222 labelled proxy (not an invoice), storage/transfer unmeasured, after-down custody (131,072 B, MD5 `9f7368ac…`, 13 files), one source per fact, and the front-door gap stated |
| `docs/labs/infra-k16-normal-close-accounting/resource-ledger.json` | new data | +42 | machine-readable close/accounting (pod, times, `uptime_s`, rate, `approx_gpu_cost_usd`, custody), read by `budget show` as the repository resource-ledger head |
| `docs/reference/storage.md` | docs | +8 | pointer to the record for the after-down closure custody, storage/transfer unmeasured |
| `docs/research/notebook/m1-discrete-attention.md` | docs | 0 | the K16 row is **not** added: the file is outside this item's scope and the item has no `notebook:` field to allow an append-only row; asked as `loop ask --scope` card 4 |
| the item's `status:` line and `deepseek-7/ORDER.md` | docs | small | item done; ORDER filled with probe, done-when and stop-and-ask |

**Size:** about 150 changed lines, against the ORDER's 180-line estimate (0.8).

## Measurements

- **Close:** pod `vast-56571294`; created 2026-10-18T15:01:18Z; released 2026-10-19T00:38:29Z; native down 00:38:33Z (before the 00:39:22Z expiry); no force/discard/renewal after release; no remaining idle lease or cost.
- **Accounting:** whole lifetime 34,633 s = 9.620278 h at the quoted $0.908/h gives **$8.735212222** — a resource-cost proxy, explicitly **not an invoice**; storage and transfer **unmeasured**.
- **Custody:** `k16-afterdown-closure-0038z.tar`, 131,072 bytes, MD5 `9f7368accd2d29395f7833cd0be79fb5`, 13 retained files, manifest SHA-256 `9dd0ee27…`, full cloud readback reported complete.
- **Machine-readable:** `uor-harness budget show` now reads `repo cumulative ledger: 1 resource-ledger.json file(s)` with `infra-k16-normal-close-accounting/resource-ledger.json` as the head; the chain stops at the head because it names no `previous_ledger` (a new accounting boundary, not a rewrite of earlier ledgers).
- **Gates and checks:** `loop gates --quick` GREEN on three; `cargo run -q --locked -p xtask -- roadmap check` 87 items OK; `uor-harness check` has only the named warnings above.

## Checks run at the head

- `cargo fmt -p uor-r4-harness --check`: clean (no Rust changed).
- `cargo test -q --locked -p uor-r4-harness --no-fail-fast`: every target green (run alone; no Rust changed).
- `cargo clippy -q --locked -p uor-r4-harness --all-targets`: no warnings (no Rust changed).
- `UOR_LAB=deepseek uor-harness loop gates --full`: GREEN on the five gates.
- `UOR_LAB=deepseek uor-harness loop gates --break all`: CAUGHT on the five gates.
- `UOR_LAB=deepseek uor-harness loop report --check`: PASS.
- `UOR_LAB=deepseek uor-harness loop pr-check --local`: PASS (or the named scope expectation until the repaired tool side is on `main`).

## Skills used

- **session-close**, read from its private local install: the dated record shape, sources beside claims, and what carries to the next session.

## Still open

- **A K16 notebook row is still missing** for the same scope reason (card 4); it is a one-line follow-up for the front-door sync.
- **The three front-door sentences are still stale** (`docs/labs/k16-four-role-execution-2026-10-18/README.md:24`, the `docs/labs/README.md` final-custody box, `docs/integration/NOW.md:81`). They are outside this item's scope; the record is the pointer a front-door sync can use. Asked for in three `loop ask --scope` cards and on #2827.
- **`docs/harness/ledger/**` was not written**: see the negative above; the machine-readable accounting lives in the scoped `resource-ledger.json` that `budget show` reads, and a committed cost receipt cannot carry a cost through the CLI without writing another lab's shard.
- **`check`'s team-claim warning** stays visible by design rather than editing `docs/harness/claims.json` out of scope.
- **The local compute ledger is machine state**, not committed evidence; the committed sources are the two #2029 comments and the front-door record linked in the record.

## Plain summary

- **The K16 pod is closed and its books are recorded.** `vast-56571294` was released at 00:38:29Z and confirmed down at 00:38:33Z on 19 October, before its lease expired; nothing was forced, discarded or renewed afterwards.
- **What it cost:** 34,633 seconds (9.620278 hours) at the quoted $0.908 per hour, which is **$8.735212222** as a labelled resource-cost proxy — not an invoice. Storage and transfer charges were never measured, so this record says `unmeasured` instead of inventing a number.
- **What happened to the bytes:** the after-down closure custody archive is 131,072 bytes with 13 retained files and a recorded MD5 and manifest, and its cloud readback was reported complete.
- **Where it lives now:** a new dated record under `docs/labs/infra-k16-normal-close-accounting/` with a machine-readable `resource-ledger.json` beside it (read by `uor-harness budget show`), a pointer from `docs/reference/storage.md`, and the machine-readable accounting. The three older sentences that still call the close pending, and the one-line notebook row, are named and asked about as `loop ask --scope` cards 1-4, because editing them is outside this item's scope.
# K16 normal close and final resource accounting — 22 October 2026

This dated record closes the custody and cost bookkeeping for the K16 four-role execution
([canonical execution record](../k16-four-role-execution-2026-10-18/README.md)). Every figure below
names its source; this record repeats no scientific result and invents no invoice.

## Normal close

| Fact | Value | Source |
|---|---|---|
| Lease/pod | `vast-56571294`, Vast.ai, 1× RTX 5090, Vast Alberta CA | `up` event in the owner's append-only compute ledger (`~/.local/share/uor-r4/compute/ledger.jsonl`), also recorded in #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) |
| Lab and session | `codex`, `cf1-k16-qual-20261018-145509z` | same ledger events |
| Created (pod up) | 2026-10-18T15:01:18Z | same ledger `up` event |
| Released (normal close) | 2026-10-19T00:38:29Z | same ledger `release` event; #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048); [front-door correction record](../k16-frontdoor-close-sync-2026-10-19/README.md) |
| Native down | 2026-10-19T00:38:33Z, before the 00:39:22Z lease expiry | same ledger `down` event with `uptime_s` 34,633; #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) |
| Force, discard or renewal after release | none | #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) |
| Remaining idle lease or cost | none; no replacement allocation | #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048) |
| Provider state at close | fresh GPU list empty; no discrete-stack PID; all 16 native jobs terminal, all four registered trajectories exit 0 | #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048); [canonical record](../k16-four-role-execution-2026-10-18/README.md) |

The session that held the lease released it and confirmed the guarded native down before the
expiry; no peer resource was touched. The same comment records that custody of DATA and the late
operator payloads had already passed full cloud readback.

## Final resource accounting

| Item | Value |
|---|---|
| Resource lifetime | **34,633 s = 9.620278 h** (whole lifetime, create to native down) |
| Quoted rate | **$0.908 per hour** |
| Final cost, labelled proxy | **9.620278 h × $0.908/h = $8.735212222** — a resource-cost proxy, **not an invoice** |
| Storage and transfer | **unmeasured** |
| Remaining idle lease or cost | none |
| Replacement allocation | none |

The lifetime is the whole resource lifetime of that pod, not an increment to add to earlier
same-resource estimates. Storage and transfer have no measured figure in the records, so none is
invented here.

Machine-readable form, read by `uor-harness budget show`: [resource-ledger.json](resource-ledger.json).

## After-down closure custody

| Fact | Value |
|---|---|
| Archive | `k16-afterdown-closure-0038z.tar` |
| Size | 131,072 bytes |
| MD5 | `9f7368accd2d29395f7833cd0be79fb5` |
| Retained files | 13 |
| Manifest SHA-256 | `9dd0ee27140a789f1778ca5758b961f0f0c2944258a7d838d5bd951f3d55911d` |
| Readback | full cloud readback reported complete |

Source: #2029 [6257225762](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257225762)
(`[Codex/Sol cf1]...` separate after-down closure custody comment).

## What this record does not do

Three front-door sentences still describe the close as pending and are outside this item's scope
(`docs/harness/ledger/**`, `docs/labs/infra-k16-normal-close-accounting/**`,
`docs/reference/storage.md`):

- `docs/labs/k16-four-role-execution-2026-10-18/README.md:24` — `Normal close: NOT_PERFORMED ...`,
  `Final resource accounting: PENDING actual cutoff`.
- `docs/labs/README.md` — the final four-role custody box: "final rental accounting remains
  pending".
- `docs/integration/NOW.md:81` — "Normal close and final resource accounting are not yet
  recorded."

This record is the pointer a front-door sync can use to replace those sentences with the facts
above. Editing them needs a scope widening or a follow-up front-door item; it is asked for, not
done out of scope.

The owner's local compute ledger is machine state, not committed evidence; the committed sources
are the #2029 comments and the front-door record linked above.

## Sources

- Item: [infra-k16-normal-close-accounting](../../roadmap/items/infra-k16-normal-close-accounting.md) (#2827).
- Canonical execution: [K16 four-role execution and final custody](../k16-four-role-execution-2026-10-18/README.md).
- Close correction: [front-door correction: normal close complete](../k16-frontdoor-close-sync-2026-10-19/README.md).
- Resource closure: #2029 [6257200048](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257200048).
- After-down closure custody: #2029 [6257225762](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6257225762).
- Current plan: [NOW.md](../../integration/NOW.md) lines 69-81, whose K16 sentence this record supersedes for close/accounting only.
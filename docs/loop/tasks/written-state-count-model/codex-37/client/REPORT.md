# [codex] D37 ordinary artifact client comparison

## Negatives first

On each of the two fixed prompts, both fitted arms produced the same text,
` We need answer. Need parse. User`, and the same 12 IDs
`[799,767,2505,16,2237,71,266,962,375,16,3184,2728]` without EOS. Neither
answer was correct. For copper the expected IDs including EOS were
`[581,2195,1]`, and the first mismatch was index 0, expected 581, actual 799;
for silver they were `[3585,322,1]`, and the first mismatch was index 0,
expected 3585, actual 799. This is a two-prompt product observation, not a
panel score or a claim of conversational ability. Exact text and raw-ID
trajectory/trace replay matched for both prompts on each arm.

## Observations and scope

On Wick, the registered run exited zero on 21 November 2026. Both artifacts
were written and opened by the ordinary D11 writer/reader. Each arm matched
all 677 retained original R CUDA target bucket and Q48 mass probes, as well
as the recorded four Pick blocks and provenance. The disk reader's complete
4,095-mass vector and argmax were checked against the ordinary `uor-chat`
entry point on 71 original prefixes per arm, representing 65 records and 71
distinct nonempty prefixes. The CUDA tape supplies target bucket and mass,
not the full vector; the full-vector check is native reader versus client.
A changed-distance planted negative at original ordinal 2 changed both the
bucket and mass on both arms. Grading plants covered matching IDs, wrong
first ID, and a missing EOS in `grading-finite-v2/`.

The run used only the original frozen fit and registered authored prompts,
not DEV-tune or DEV-test rows, and performed no fitting or GPU work. The
selected A1s candidate weights SHA-256
`b6b3da9e94ac33d0d8dc6c192c3117afc4fdc1f8e9c83dbf051010f5c2f31bb0`
matched the original A1s fit weights used here. This identity does not
turn the original-prefix checks into held-out quality evidence.

| Arm | D11 SHA-256 | D11 bytes | Tokenizer bytes | Combined bytes |
|---|---|---:|---:|---:|
| a1 | `167be3be18b748f48c545ed48b125c37a77acd2354c1756a4cfa1cf283150784` | 3,998,932,622 | 265,393 | 3,999,198,015 |
| a1s | `3e9de13fbc67cf61627d87bcc1a44c74d21d786e85450ec4fa21669522eb8855` | 3,998,972,048 | 265,393 | 3,999,237,441 |

Observed process start was 13:59:36 EST; the final root manifest was written
at 14:25:48 EST, approximately **26 minutes 12 seconds** by those timestamps.
This is wall time for the full serial run, including custody checks, packing,
text replays and 142 fresh client processes; it is not a throughput benchmark.

## Custody

The parent-bound GO, FIT, sheet, readbacks, coverage, exact original-prefix replay
outputs, text outputs, and manifests are under `evidence/`. The two large
artifacts are intentionally omitted from Git; complete inventory and bytes
remain on Wick at
`/mnt/plexpool/uor-r4-corpus/runs/20261120-pruned-r-fit-37/client/result-ordinal-v1`.
The full root and arm manifests were verified there, including both D11 files.
Root `results.sha256` SHA-256:
`8a6b13549204649ccf4de37a336d84bcf73d759bbe677924820b742bd156433d`;
arm manifests: a1 `91f3cbe7fe5a6e246cf3c38fd636221cda59ed97db54e720d9d15190c5e310f9`,
a1s `4140d3394d131a335b9e04e7f4619f5b3fa56b91c07072764cbeeadddc6dd9d7`.
The artifact entries in the checked-in arm manifests require the Wick files
for full verification. The source-tree manifest and tool hashes are bound by
`evidence/stage.sha256`; the full source manifest remains on Wick in the
parent client directory. FIT and GO bind the exact run and source digests.

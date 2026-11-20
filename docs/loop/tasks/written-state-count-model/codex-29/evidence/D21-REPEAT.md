# D21 hardware repeat on an independent RTX 4090

On November 20, 2026 UTC, both D21 seeds completed the original
frozen-inference dispatcher twice on Vast `vast-61553201`,
RTX 4090 `GPU-42c7cce0-a287-d13a-f8a3-005ea3f1809a`, driver
`595.99.02`, SM 8.9. The original two-seed run used a different RTX
4090 and driver `580.95.05`. The repeat restored the original binaries,
models, tokenizers and blind probes from the Wick-qualified portable
archives. It checked the original stage, source and executable manifests
before a fresh finite qualification. Finite and paired held dispatch each
recorded `done 0`. The held phase took 14:33.17 including per-seed cold
repetitions and the ordinary writer checks. [D21 FIT and limits](../../codex-21/FIT.md) · [Original results](../../codex-21/evidence/paired-findings.md)

| Bound item | SHA-256 |
|---|---|
| Fresh hardware and runtime receipt | `bfcd7ee546efadf18ab9876e94d1b3cfd84fc031bbb6e0f28af06f5aa7622bfc` |
| Original stage input manifest, both seeds | `b5cc4fa2fa2b392ecdb002d3368869d85b978a1c2784d3eae9ee98767e2b7b25` |
| Original executable manifest | `119577b27e13c0614c2c42e1c5f9ca9fe025faa4edf4824b626013ab8f9d9e08` |
| Fresh finite manifest | `68a3265fdc0a8475b0bf15bb9f8916ff10d7ce4cc88cd74df8d673e7617ec449` |
| Fresh GO receipt | `dc511d084a74a2b9b1d5fdbea932220580f463dbd101ee52371b4dcfc9771ad6` |
| Seed 1 semantic cold manifest, original and both repeats | `e0afe8f20ee2cb28dc84ecdedb8b57f066574fbb4d460c33d37f422dd58f7a27` |
| Seed 2 semantic cold manifest, original and both repeats | `5907aa9322ec4755e52fa589f780b82789146ed4e2f071c1bc1ef8749c415524` |
| Results archive, 99,980,164 bytes | `726ddda0480147d95425d94df92ccc786486df16adb3e02315cabd49bc6d3a89` |
| Authorization archive | `803859e54aac619a47c86e5afcdff2e2ef07074ad42ea97835f0e4c2d3ee23e4` |

The repeat verified all 851 raw and 426 semantic files for each of the
four executions on-device and again after extracting the workstation
archive. All four semantic manifests and summaries were byte-identical to
their corresponding originals. The semantic manifest covers the role
decisions, original value candidate packets, traces, released sources,
fixed writer and ordinary client token outputs. Device metadata and
timing remain in the raw receipts.

Each seed repeated the same 96 supported prompts and six unsupported
controls. The ordinary reference recovered 48/96 requested spans and
eight exact replies through EOS. The ordinary clause selector recovered
0/96 and produced zero exact replies, with 20 new wrong-source writes
against its paired reference. Supplied-endpoint ordered scoring recovered
16/96 exact replies, rising to 24/96 with the earlier-connector veto.
The eight gains were the same prompts as in the original run. The veto
retained 24 wrong-source writes and introduced zero new ones. Forced gold
matched all 96 supported tapes through EOS. The ordinary arms released no
source on the six unsupported controls. These are repeated observations
on the original panel, with no fresh transfer or timing comparison.

The D21 session held the dedicated pod from 01:38:29Z through release at
02:00:49Z. At the offered rate of $1.314/h, the 1,340-second D21 window
is estimated at $0.4891. GPU 0 was released to the D29 conformance work
only after the workstation archive and raw files verified. The subsequent
D29 lease is separate.

The full results and authorization archives are under
`~/uor-r4/local/codex/rival-admission-29/d21-repeat/results/` and
`/home/callard/uor-codex/20261120-d21-repeat-codex29/` on Wick. Both
archive hashes were rechecked on Wick. Reproduce by restoring the original
portable stage with its manifest, running D21's finite qualification, then
using the unchanged held dispatcher under the new device-bound pre-run
receipt. Validate each run's `raw.sha256` and `cold.sha256`. Compare
the latter with both original seed manifests above. [Finite entry point](../../codex-21/dispatch/finite.sh) · [Sealed dispatcher](../../codex-21/run/dispatch.sh) · [Original custody](../../codex-21/evidence/storage.json)

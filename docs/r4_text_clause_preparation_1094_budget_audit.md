# Independent budget audit — #1094 preparation freeze

2026-09-03. **Release: `NOT_ADMITTED`.** This read-only audit binds the
recorded original consumption and the remaining policy allocations. It does
not admit another preparation attempt, withheld access, comparison or replay.
Source was inspected at `6f21fc5f4c40b9620c9fec5e95a39097f812ae73`.

The reviewer read the public receipts and the original named comparison and
curator metadata files. No withheld directory listing or payload read, Python
package import, readiness/preparation invocation, model work or deletion was
performed. The later writing of this document is documentation work only.

## Recorded consumption and the missing terminal-write interval

The [#1085 specification](integration/clause-segmentation-1085.md) fixes
120 seconds preparation integrity, 120 seconds execution, 120 seconds replay,
360 seconds cumulative, 3 GiB peak RSS, 128 MiB new corpus/results and at most
6,400 logical row forwards. Independent input authoring and code review are
reported separately. An unavailable result requires separate repair; it does
not permit automatic retry or a renewed budget.

The original [stop](r4_text_clause_adapter_1094_evidence/prepare-stopped.json)
remains `UNAVAILABLE_REFERENCE_REPLAY`. Its later
[resource snapshot](r4_text_clause_adapter_1094_evidence/prepare-stopped-resources.json)
records:

| Quantity | Recorded value |
|---|---:|
| Preparation elapsed | 0.2602927920015645 seconds |
| Cumulative elapsed | 0.2602929580025375 seconds |
| Coordinator peak RSS | 65,601,536 bytes |
| Model loads / logical forwards / optimizer updates | 0 / 0 / 0 |
| Execution / replay | `NOT_RUN` / `NOT_RUN` |

The two time fields differ because `Budget.snapshot()` calls its elapsed-clock
property separately. The larger value is the conservative recorded debit if
one common observed figure is needed. Both observations precede serialization
and writing of the final resource receipt. That write and the subsequent exit
interval have no later elapsed observation. Recorded elapsed is therefore a
lower bound on completed preparation time; the following arithmetic produces
upper bounds on remaining time, not certified spendable allowances:

```text
120 - 0.2602927920015645 = 119.7397072079984355 seconds
360 - 0.2602929580025375 = 359.7397070419974625 seconds
```

The worker RSS value zero in the original receipt means no worker measurement
arrived. It is not a successful worker resource observation.

**Declared bookkeeping policy for the new freeze:** quarantine the entire
original **120-second preparation allocation**. No further timed `prepare`
attempt is admitted. At most **120 seconds execution plus 120 seconds replay**
remain, both still subject to separate source and release admission. This
conservative allocation policy does not assert that the original attempt took
120 seconds or establish an upper bound on its unobserved terminal-write tail.
The recorded lower-bound measurements remain unchanged.

## Exact original byte inventory

The original comparison directory currently contains exactly these seven
files, each matching its public retained copy and manifest:

| File | Logical bytes |
|---|---:|
| `authoring-input-preflight.json` | 3,032 |
| `bindings.json` | 58,987 |
| `preparation-progress.jsonl` | 581 |
| `preparation-started.json` | 863 |
| `prepare-stopped-resources.json` | 279 |
| `prepare-stopped.json` | 1,262 |
| `worker.sb` | 1,519 |
| **Total** | **66,523** |

The original corpus commitments account for sealed bytes without inspecting
their contents. The stopped-comparison byte ledger is:

```text
3,397,265 frozen corpus/selection/policy
+   1,565 population.json
+      48 harmless isolation-probe.txt
+  66,523 original comparison receipts
= 3,465,401 bytes consumed

134,217,728 - 3,465,401 = 130,752,327 bytes remaining
```

The final resource snapshot says `new_bytes: 3465122`; adding its own
279-byte file produces the same **3,465,401-byte** total. Future counted
corpus/results/receipt additions must reduce the remaining allowance; another
output directory cannot reset this debit. Delivery copies and the separate
ledgers below must be inventoried explicitly rather than silently counted
twice or silently folded into a different contract.

## Separate ledgers and admission blockers

The curator's successful independent generation recorded
0.27163095799915027 seconds and 79,626,240 native Darwin RSS bytes, with zero
model forwards/updates. Its initial Python-version failure has no elapsed-time
measurement and wrote no payload. Both original attempt receipts remain in the
[public curation record](r4_text_clause_adapter_1094_curation.json). Its
`resource_scope` explicitly separates authoring from timed preparation and
model comparison; no population regeneration is admitted here.

[#1096 readiness](r4_isolated_runtime_readiness_1096.md) had a separate
one-attempt/60-second/16-MiB contract. The approved attempt recorded
2.058100333000766 seconds and zero loads/forwards/updates. Both original freeze
folders total 165,740 bytes; originals plus public evidence/index copies total
339,014 bytes. These observations neither replenish #1094 nor silently debit
its comparison ledger. Readiness is evidence for the exact pinned runtime
configuration, not a successful #1094 preparation or model comparison.

The inspected `campaign.prepare()` starts a new `Budget` with zero carry and
repeats authoring preflight and readiness. `campaign.run()` carries only the
new `preparation-closed.json` phase time. Those paths do not preserve the
original stop and this quarantine policy. Invoking unchanged `prepare` in a
fresh directory is therefore not admitted.

The future coordinator must assemble explicitly typed preparation evidence
from bound historical receipts, preserve the original unavailable terminal,
enforce historical resource accounting and bind independent review before
release. It must not manufacture `COMPARISON_PREPARED` as though the stopped
preparation had completed. A new assembly status needs its own defined
meaning and reader in the coordinator. Until that implementation and its
independent reviews exist, release remains `NOT_ADMITTED`; no withheld file
hash/read or model work is admitted by this audit.

## Independently checked identities

| Source or receipt | SHA256 |
|---|---|
| #1085 specification | `85f928fec94fa0f6793cff4c35e1fc8c9cba691739d34db272465766c7c9dab1` |
| Original preparation start | `0a0eeff9238eb28ca745e8e4010654fc1f436f65f6c86aa5eedae8bd996c94d7` |
| Original unavailable stop | `87bd3082ce9b4da5e5227a3b82f6515773cf5f113de689ff6251a2a45340fad5` |
| Final original resource snapshot | `02f907640d6830e17c136e06078a54390ee4173862043926bee7f8f7670eaf18` |
| Original authoring preflight | `f79df8623038961d899ba727d99bb69b39754b1878d10bbed9da0bfe03e5ee82` |
| Original population commitment | `ad5bf0fdecb66b0de9e28c98941cf0fb2c6f737c7e1be3cbf48570822c65ba30` |
| Original selection commitment | `892e3239773e8a14e72ee650dc12c98ee4e1a5b432b69365a60cef8b15c9b5fa` |
| #1096 admitted manifest | `4acd2b7ec00ac8874573e2d6e52e5087b376bc0ebcd52aedd4464aa28979c644` |
| #1096 readiness result | `439aa149d6f128844490c4a9002bfe2ffb52fdeeaad067e8e3cb16447b24b930` |
| Inspected `campaign.py` | `e0e6c8d1387800cf587e82928accb988a9e13fed31a9dbb73763104069d56023` |

The source locations are `Budget` at line 269, `prepare` at line 427, `run` at
line 837 and terminal resource writing in `main` at lines 968–990 of
`tools/r4-softmax-trainer/src/r4_softmax_trainer/text_clause_adapter/campaign.py`
at the revision named above. This is a resource/provenance audit, not a
mathematical proof or a new capability measurement.

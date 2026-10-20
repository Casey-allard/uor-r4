# Independent admission source audit — #1094 retained assembly

2026-09-03. **`SOURCE_INSPECTED`; execution release `NOT_ADMITTED`.**
The independent `/root/retained_admission_audit` reviewer read the preparation
contract, its budget/source reviews, the original #1094 metadata, accepted
#1096 metadata, and `campaign.py`, `contract.py`, `worker.py` and `readiness.py`
from `6008e3527cb119d12af4abd99fe86a3d4ebe5a53`. This initial audit identifies
implementation admission requirements; it does not approve an implementation
or an execution envelope before those objects exist.

The knowledge-index query `retained-evidence preparation` returned the accepted
contract record `kb:8a5bbc50fb01f693e6b2b354a1b988275d22fa44c8b4cd30c471be0d41793b6b`,
with origin at that protected revision. The reviewer fetched it and followed
the original repository file, SHA256
`8fa572877490755dbfd6ffbd1cbb28bbdc2553486914549803973cf62995ac97`.
The index is a discovery aid; the committed contract governs.

## Decision-bearing implementation requirements

| Boundary | Required behavior and reason |
|---|---|
| Assembly side effects | A separate assembly module must not import `campaign`, which imports the adapter, or invoke `prepare`, a worker, runtime probes, model construction, authoring rows or corpus generation. It verifies historical evidence and committed metadata only. |
| Source identity | Bind the coordinator's committed closure and every executing worker source path. Compare actual bytes with committed bytes, and compare the complete path set so a missing/new source file cannot hide outside the closure. Preserve worker, adapter, curator, policy, learned model and arithmetic sources. A changed coordinator commit alone does not establish those identities. |
| Output identity | Create one exclusive output directory outside the corpus tree. Reject ancestor/descendant overlap and path aliases that could let output accounting traverse withheld content or redirect a profile/binding. Count all retained output files and existing temporary oracle bytes without traversing the corpus. |
| Evidence identity | Verify exact old start/stop/resources/authoring, population/selection commitments, #1085 specification and all accepted #1096 references. Preserve the original unavailable terminal; assembled evidence has its own schema and status and never manufactures `COMPARISON_PREPARED`. |
| Historical resources | Carry the policy debit of 120 seconds and 3,465,401 original bytes, with zero historical forwards. Count the corpus once. Keep #1096 and authoring ledgers separate. Execution/replay snapshots, stops and completion must retain history. A fresh output directory cannot replenish it. |
| Future clocks | Start execution accounting before fresh identity and release checks. Both sequential execution arms share one 120-second phase; fresh-process replay shares its own 120-second phase and inherits all execution consumption. Receipt writes and post-worker checks remain inside the phase/cumulative caps. |
| Launch identity | Bind the exact generated profile and its complete literal/path delta from #1096, including removal of the old manifest/profile grants. Use an exact clean environment, accepted launcher/resolved interpreter, both aliases, 18 runtime files, hardware and five assets. Verify runtime files/aliases before and after every attempted worker, including failure cleanup, while preserving the original cause and partial evidence. |
| Child observation | The unchanged ordinary worker verifies source/assets, fixed runtime and its existing single denied sentinel before model work. Parent runtime checks plus that child receipt must not be described as a fresh #1096 four-probe readiness observation. |
| Release | Require the distinct retained-release schema/status bound to the exact assembly digest, closure, profile, environment, commitments and 120-second carry. Reject old approval statuses, missing fields and altered bindings. Reject existing start/stop/completion markers before withheld hashing or model work. An assembly or code-review receipt is not execution permission. |
| Terminal evidence | Write the exclusive execution-start receipt before the first withheld hash/read. Preserve partial streams and work counts on failure. A stopped receipt overrides completion; no automatic retry or source/profile relaxation follows a failure. |

The old coordinator's `Budget.snapshot()` reads the public population manifest
and traverses only its chosen output. Retained execution should consume bound
historical byte components instead of recomputing a fresh corpus/output ledger.
Its old `run()` accepts only legacy prepared/review statuses, while
`arm_process()` inherits the parent environment. These are concrete source
boundaries to change; the model-comparison and refusal criteria stay frozen.

## Minimum bounded checks

Synthetic inputs can establish the admission and resource behavior without
touching the real sealed population or starting the worker. The useful checks
are: exact valid metadata assembly; changed historical/source/profile/approval
identity refusal; output/corpus overlap and alias refusal; existing terminal
marker refusal; historical time/bytes and execution-to-replay carry; phase,
cumulative, byte and forward cap refusal; clean environment independent of
injected parent settings; and pre/post worker-identity checks on a harmless
stub's success and failure. At every denied admission, an instrumented fake
payload reader and worker launcher must show zero calls. Synthetic fixtures
must be separately named and must not emit approval for the real envelope.

These are proposed minimum checks, not a report that they ran. Any subsequent
review must identify the actual source revisions and exact checks observed.
No model/runtime/prepare/run invocation, withheld read/hash/traversal, permission
change, fit, comparison, replay or broad QA was performed by this audit.

## Original research inspection and limits

The reviewer reread the pinned local originals referenced by the
[#1085 source audit](integration/clause-segmentation-1085-sources.md), and
independently rehashed all three objects:

| Original | Pin; SHA256; inspected scope |
|---|---|
| Mark / NEMESIS 3D Studio, *Integration of Hypercomplex Geometries as UOR Structure Carrying Substrates* | `0d106967843c2c96477cf3e57aeff213e7db1c97`; `697d48b70a1499a1fd70d8f1a4c285606a198a3831250425ae11439f37b395cc`; 99,374-byte PDF, extracted pp. 1–2 |
| W33 `analysis/w33_fractal_microvm_runtime.py` | `5674aa2e74c7de81864556b1bbc90aa9a1b1bd3d`; `875b53408cc5312b60b5a6254dbac80a9a1324c89cdf24936488d7a4744e90ca`; lines 38–58 and 314–329 |
| UOR-ADDR `crates/uor-addr/src/composition/canonicalize.rs` | `165b51e3e2113ee5d032730cde709335d4fe9b60`; `d9032fc9bc95a4f86ddbb8c0db3753865ee6de36a8f915fc446597243b8a6d89`; lines 68–118 |

NEMESIS separates state mapping, transition fidelity and primitive
interpretation; that distinction helps state what this handoff actually binds.
W33 supplies an example of explicit admission, canonical serialization and
immutable digest-addressed records. UOR's axis check illustrates typed digest
compatibility; its commutative G2 ordering cannot replace ordered clauses or
role framing. None proves this coordinator's admission correctness, isolation,
budget completeness, parser correctness or model-output preservation. No
external source was vendored, no upstream witness was executed, and no new
mathematical proof is asserted.

The original #1094 unavailable result, #1096 runtime-only readiness, #1079 weak
control and #1082 descriptive limits remain unchanged. #1094 and #973 remain
open, with #954 blocked. Implementation and exact-envelope review remain
separate decisions from the withheld empirical comparison.

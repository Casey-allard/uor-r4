# Independent preparation-contract review — #1094

**2026-09-03 — `CONTRACT_ACCEPTED`; execution release `NOT_ADMITTED`.**
The independent `/root/preparation_contract_review` agent reviewed the four
documents below against source and retained receipts at
`6f21fc5f4c40b9620c9fec5e95a39097f812ae73`. The reviewer did not author the
contract, budget audit or source audit. No blocking defect remains in this
specification. This review accepts the frozen contract for protected delivery;
it does not issue `ACCEPTED_FOR_RETAINED_EVIDENCE_COMPARISON` or authorize any
preparation, readiness, withheld access, model comparison or replay.

## Exact reviewed documents

| Document | Bytes | SHA256 |
|---|---:|---|
| [Preparation contract](r4_text_clause_preparation_1094.md) | 11,943 | `8fa572877490755dbfd6ffbd1cbb28bbdc2553486914549803973cf62995ac97` |
| [Machine-readable contract](r4_text_clause_preparation_1094_contract.json) | 12,091 | `fa0fbde6fda045bfa770837fd3eda612329bf34d2abaee34a5b143c751c65780` |
| [Budget audit](r4_text_clause_preparation_1094_budget_audit.md) | 7,707 | `b4c1c03af0fe5c38f1ea11ef225c73954b6de4514637a97d9c4405ffee1596d3` |
| [Source audit](r4_text_clause_preparation_1094_source_audit.md) | 10,589 | `37198e6c635f4f09575b94b04e3cba8073bd2ab35e9b70f277436783b0c20e5d` |

These hashes bind the reviewed document bytes independently of the eventual
delivery commit. A change to any of these four files requires review of the
changed content and a renewed hash binding. This record does not bind later
roadmap, ledger, GitHub or knowledge-index transport updates as though they
were an executable release envelope.

## Checks and findings

The reviewer parsed the machine-readable contract using standard-library JSON
and independently checked **all 19 source/receipt references** against exact Git
objects at the stated base revision. Every byte length and SHA256 matched.
The sources include the #1085 comparison specification, original #1094
start/stop/resources/authoring/bindings/curation records, accepted #1096
manifest/bindings/profile/result/start/approval/result review, and current
campaign/worker/adapter/curator/profile-generator source. No model asset or
withheld payload was opened to perform those checks.

The original stop remains `UNAVAILABLE_REFERENCE_REPLAY`: its authoring
receipt records 320/320 valid rows, 16/16 refusals and two schema probes exact
with zero model work; Python startup was denied before a worker identity
receipt arrived. The separate #1096 result records `ISOLATED_RUNTIME_READY`
for its exact manifest, with all four harmless denials and zero model
loads/forwards/updates. Its 18 runtime-file identities and two interpreter
symlinks are requirements for the future handoff. The new contract does not
reinterpret either result as raw-text model-output preservation.

The reviewer checked `Budget`, `prepare`, `arm_process`, `run` and terminal
resource writing in `campaign.py`, and the worker's binding and optional
readiness-identity paths. They support the stated blockers: ordinary
preparation repeats authoring/readiness with fresh accounting; a fresh output
folder drops old output bytes; execution consumes only the legacy prepared
status and its phase time; the launcher inherits its environment; and `run`
also performs replay. The four-probe #1096 identity event is optional and
distinct from the ordinary worker's single denied-sentinel receipt. The
proposed assembly and release schemas are explicitly unimplemented and absent.

Exact decimal arithmetic independently confirms both recorded-time
remainders. The source writes its last receipt after taking the snapshot, so
those remainders are upper bounds rather than spendable measured allowances.
The contract quarantines the entire original 120-second preparation
allocation as a policy debit. It explicitly preserves the measured values and
does not assert a 120-second elapsed observation or recover an upper bound on
the unobserved terminal-write interval. The future 120-second execution and
120-second replay allocations require carry-aware enforcement; their
availability is not an execution approval.

The independent byte arithmetic agrees:
`3,397,265 + 1,565 + 48 + 66,523 = 3,465,401`, leaving
`134,217,728 - 3,465,401 = 130,752,327` bytes before new material. The contract
carries retained output history, counts the corpus once, and keeps #1096 and
independent input authoring explicitly separate. The budget auditor's original
folder inventory is retained as that auditor's observation; this reviewer
checked the public receipt bindings and arithmetic without repeating an
original-folder or sealed-population inventory.

The assembly must occupy an exclusive output directory outside the sealed
corpus tree. It may bind committed metadata and historical evidence without
rerunning authoring or readiness. Future executable source, absolute paths,
profile bytes and delta, clean environment, runtime identities, original
population commitments and the full debit must be bound before independent
release. Fresh runtime/source/release checks are charged to execution before
withheld hash/read or model work. A historical approval cannot substitute for
the new exact-envelope release receipt.

The reviewer compared the future criteria with the pinned #1085 specification:
the accepted reader/core and vocabulary/query/four-fact context remain fixed;
the population remains 1600 valid rows, 80 refusal rows and 16 boundary
controls; full soft-output/logit/decision equality and all fourteen consumed
role decisions remain required; refusals remain model-free; and fresh-process
replay and the original terminal actions remain unchanged. No fit, tolerance
relaxation, population repair or automatic retry is admitted. #1079's weak
control and #1082's descriptive limitation remain historical evidence;
#1094 and #973 remain open and #954 remains blocked.

## Limits and release decision

This is static specification/source review plus identity and arithmetic
checking. The NEMESIS/W33/UOR source discussion is limited to provenance and
admission design; this review does not establish any of those research claims
as a proof of the proposed mechanism. There is no new mathematical proof,
parser measurement, runtime startup, model result or execution feasibility
measurement here. Broad QA is `NOT_RUN`; protected checks are delivery
transport rather than scientific evidence.

The reviewer executed no project package, adapter, curator, readiness worker,
`prepare` or `run`; inspected no withheld directory or payload; changed no
sealed permissions; and deleted no evidence or user material. Only this review
document was authored by the reviewer.

**Release remains `NOT_ADMITTED`.** The next action is to implement the
retained-evidence assembly and its carry-aware launch gate within #1094, then
obtain independent review of the exact implemented envelope before any
withheld execution. This contract review completes no empirical DoD and
activates no additional issue.

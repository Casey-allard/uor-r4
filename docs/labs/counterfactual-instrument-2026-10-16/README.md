# Counterfactual decomposition instrument — source delivery

Codex/Sol cf1 implements [PM commission6220832102](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6220832102)
on `69cb1f59d5d95950b766826453dbbfb2f843b219`, following the merged
[design sheet](../../harness/prep/m1-counterfactual-decomposition-2026-10-16.md)
and [PM-seat design FIT6219037898](https://github.com/UOR-Foundation/uor-r4/pull/2545#issuecomment-6219037898).
**CONSTRUCTION MERGED in #2550 (`98f29796f`).** Source/host checks and exact-head CUDA compile/strict-clippy gates passed; GPU behavior,
resource parity, diagnostic dose and all new measurements remain **NOT_RUN**.
Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE.

The source adds a separately compiled integer diagnostic module generated from
the existing CUDA helpers/writer/counterfactual body. U's CUDA source and
launch are unchanged. D retains the same model operations and serial launch,
with per-lane `clock64` spans, nested writer phases, executed-work counts,
visited/eligible masks and terminal fault records. No allocation pool, cache,
parallel launch or mechanism optimization is introduced. Composite identities
bind both PTX modules and all new source; the old loader refusal stays intact.

The standalone timing driver adds a distinct pin/record/command family for
matched U/D pairs. It compares all sixteen candidate losses, status and five
proposal words; excludes warmups; preserves raw records before attribution;
checks epochs/snapshots/original bytes; and emits no rate. The evaluation-only
bridge authenticates the original sealed checkpoint and exact target source/
ordinary-plus-diagnostic PTX pair, changes only source identity in memory and
records the unchanged payload. It cannot resume training or import Q4.

The [execution packet](execution-packet.md) carries all six binding FIT nits:
pair order, lane-total denominator, prospective resource-parity disposition,
all-lane/all-sample stability, the verbatim enclosing-profile limitation, and
H1 magnitude NOT_APPROVED. Its [pin template](pin-template.json) deliberately
refuses execution until exact CUDA identities are captured and reviewed.

[Compile-only receipts](compile-gates.md) preserve the first build-script
failure and its source correction: check101, clippy NOT_RUN, no model call;
ordinary PTX hash retained, both modules emitted before the resource cubin
target error. Its132s lease cost$0.033293333 estimate and MD5-before-teardown
archive are explicit; subsequent source-head transcripts attach to PR #2550.

Local host checks before the amendment: six decomposition/source tests and 30 instrument tests pass;
one pre-existing optional archived-byte metadata test is ignored. These test
receipt parity/accounting, clock/phase boundary routing, resource-log parsing,
source-anchor refusal, bridge scope/payload preservation and dose/pair order.
A C++ syntax-only inspection of generated source (clock instruction stubbed
for the host parser, never executed) also passes. No CPU model/reference or
CUDA model execution. Actual CUDA compilation/strict-clippy evidence is a
separate pre-merge gate and must be attached at the exact head.

Cost before that gate: $0 new GPU; inherited Codex 18-lease estimate
$2.230435278 plus PM's separately recorded ~$0.086 recipe-verification lease,
not invoices; storage/transfer UNMEASURED. Compile lease cost and archival
receipt will be posted with its transcript, not inferred from this source.
Line: M1 qualification instrumentation, count0/3, suspended headline37/232,
all unchanged; no mechanism acceptance or count reset.

**Next:** a separately accepted final pin/control/parity/execution packet after
protected source delivery. No diagnostic/optimization/100-batch timing/training on this authority.

PM [resource amendment6222308938](https://github.com/UOR-Foundation/uor-r4/pull/2550#issuecomment-6222308938) approves removing all D register caps for source + compile gates only. The superseded U+16 ceiling and all194→126→103 calibration receipts remain visible. New metadata tests cover exact runtime/AOT register and local/stack consistency, zero spills, launchability, shared equality and the local allowance. A fresh exact-head gate regenerates identities for PM delta review; no diagnostic or model/parity execution is released.

Amendment host validation: seven focused decomposition/source tests PASS; standalone instrument30 PASS/one pre-existing ignored; strict host clippy, claim wording, D23/D25 and whitespace PASS. New test exercises the recorded126-register/+560-byte zero-spill metadata and fail-closed bounds; this is not a CUDA runtime-resource observation.

Post-merge exact pins, accepted final gate/cleanup/cost receipts and outstanding execution prerequisites are consolidated in the [execution-readiness record](../counterfactual-execution-readiness-2026-10-16/README.md). Its prospective pin grants no diagnostic release.

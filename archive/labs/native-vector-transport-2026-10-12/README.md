# Exact signed-axis transport shared by SpiralCore and E8 table construction

References #2029, #2030 and #2032. Codex session
`codex/native-vector-transport-20261012`, claimed at
[#2029 comment 6136085443](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6136085443).
Base `8773ac8dd367370ce637f876771ffd91155f87b6` after protected delivery of #2259.

## Owner direction and scope

The owner clarified in the current Codex chat: only geometry, remove all active
softmax/hidden normalized-exponential and dense-matrix implementations; use
E8, SpiralCore, icosian operators and FSB routing, with flock/murmuration for
attention. Ternary and arithmetic modulo 256 are possible substrates, not renamed
dense models. Both the language stack and compiler/exact-store/emitter must
train directly into, carry tokens through and consume one native geometric
vector representation through the same writer. Training and M2 rental holds remain.

This source unit removes generic contraction code from the already-typed
SpiralCore signed-permutation composition and E8 static-root table construction.
It is an exact operator/compiler dependency, **not a completed shared writer,
learning law, full project-wide removal, GPU kernel or model result**. Kimi's
Hamilton/gate/head work and Claude's flock work remain theirs; no peer source,
worktree, branch, process, artifact or lease was removed.

## Source change and retained identity

`SignedMatrix8` retains the same private canonical array/wire representation.
Its `checked_mul` now composes the signed-axis indices and signs directly,
rather than executing the generic 8×8 multiply/sum loop. The axis decoder rejects
empty, repeated, multi-entry and non-ternary rows. `apply_root_key` applies the
same exact coordinate permutation/sign action with checked negation; it is used
by both E8 orbit construction and all 64 action-image table construction.
No matrix product, floating arithmetic or numeric multiplier is needed by these
new composition/application bodies. This is not an instruction audit of the
whole binary or a throughput/energy claim.

The table builder remains non-model static geometry preparation. Model execution
must use an admitted GPU path; the host root-key utility is not a CPU forward,
reference or fallback. Runtime E8 address transport already consumes bounded
lookup tables; those tables and their canonical ordering must remain unchanged.
The removed source is restorable through the indexed
[patch](../../history/branch-archive/native-vector-transport-2026-10-12/generic-contraction-removal.patch)
(`git apply --reverse` at this source head), never an execution permission.

## Verification

Source-check commands and exact-head receipts are posted on the protected PR.
Current local source checks: core library PASS, 30.02 s, training library
PASS, 3 m 31 s, and 3 static-algebra tests PASS, 0.11 s after 5 m 34 s test compilation.
The existing operator/composition kappa reference labels pass unchanged.
Those times describe debug/source validation, not model throughput.
The changed core and training libraries are compiled offline with a per-unit
`CARGO_TARGET_DIR`, 2 Cargo jobs, incremental/debug info off. Selected tests cover
**static algebra/table construction only**: the existing complete 64-element
composition/inverse/associativity/canonical-digest witness; independently
constructed 240 doubled E8 roots preserved bijectively under every action and its
inverse; malformed actions and signed-byte negation overflow return errors.
No model, learned parameter, corpus, forward/backward, native response or GPU
reference is executed. Device/model checks remain NOT_RUN while held.

Initial sparse checkout omitted existing compile-time inputs (root src, then
features/model and a docs/evidence contract). These were setup failures, not
source PASS or model negatives; the owned checkout was fully materialized,
with no edits to the omitted inputs or another lab's workspace. Every failure
log remains in the closeout archive. Scope ends once the named operator/type/
identity risks are resolved; no broad model suite or new numerical campaign.

## Dimensionality and writer boundary

The historical MiniLM clause probe is separate inactive evidence:
[clause_probe.rs](../../../crates/uor-r4-training/src/clause_probe.rs) sets
`HIDDEN = 384` and `BLOCKS = 48`, and [its registration](../../research/minilm-phase0/PREREG.md)
splits teacher-derived rows into 48 separate 8D E8 blocks. This is post-teacher
quantization for retrieval, not direct native learning or a requirement for 384D.
The tokenizer's token IDs, embedding width, number of geometric blocks, context
length and exact output identity are different quantities.

Use the existing 8D geometry as the structural building block; the number of
blocks and precision need a prospective capacity/cost decision once the shared
path exists. One 8D block being sufficient for useful language is UNVERIFIED.
The E8 root/address catalogue is not the vocabulary; exact token/occurrence/
version identity must survive independently of coincident geometric addresses.
A shared geometry/writer contract must bind the same learned vectors/operators,
transport frame, tokenizer and native artifact in both callers, with genuine
credit into the vectors. A new schema or two adapters alone cannot qualify it.

## Resources, decision and next dependency

Source/preparation/build/static-geometry checks/review/delivery only; no model
ledger reset, no GPU/pod/rental. Projection ≤ 1 h, 2 jobs, ≤ 8 GiB RAM; checkout hydration
extends combined source/build storage 1.5 → 2.8 GiB under standing local allowance,
while the target reservation stays ≤ 1.2 GiB and the 30 GiB + 128 MiB disk floor wins.
Full elapsed/failure costs and actual target size are preserved at closeout.

**KEEP source if the exact identities/checks pass; no capability promotion.**
M1 memory 20/40, reply 37/232 and M2 saved 437/512 + opened 0/128 remain unchanged;
fresh qualification NOT_RUN. The unified integration line advances 6/3 → 7/3
for this owner-directed source dependency, with no count reset. The existing
[owner pivot](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6111898445)
and current removal direction govern the work; no held experiment resumes. Source/unit successes
are not D22 trained-in mechanism or fresh-transfer KEEPs. This explicitly
owner-directed removal is one source dependency, not a new experiment line.

**Next:** preserve the existing operator/frame identities in the shared native
vector/writer implementation and the GPU Hamilton/rank/flock path. All remaining
prohibited execution sites still need removal/replacement and whole-path review;
source landing or this static witness releases no training or M2 rental hold.

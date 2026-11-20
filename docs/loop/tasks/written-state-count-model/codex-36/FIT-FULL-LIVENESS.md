# [codex] Independent full-C count FIT with phase-aware memory admission

**FIT PASS for full-C count materialization with the revised producer on Wick**
conditional on exact inputs, a distinct GO, an idle exclusive GPU lock and
the producer's live post-context headroom check. The earlier
[Wick resource HOLD](FIT-FULL.md) applies to the earlier resource formula
and executable. This FIT covers the spool and preparation of the exact
cap-selected native count component for P1(b). The R-only fit and paired
DEV-test have their own admission and evidence gates.

The [revised producer and plan bindings](FIT-FINITE-LIVENESS.md) were verified
on Wick. The admitted build is
`d7687d9fb76fb1d54abf22c62d51521b2ab14b298c3ff76cdab84ac61935b3f0` and the
full [phase-aware plan](full-fit/revised/plan-c.json) is
`a2d032239fd0bcba5e0f1612dab182fddd01a6b9b1fdaf23e2e03fd6a0e6562e`.
The plan binds TRAIN
`cd3adfe949ee34a62bc8d5e48592e091fcece796618d7910d1667d014a4bae2a`
and the independent census
`28d06be01c22439fb715d6ba516845c8ee5a0530c0b0c09ec9d062461d4a4a53`.
The device is Wick RTX 4090
`GPU-7c813ad0-2151-bb1d-5efb-a3340b9a6a41`, matching the pinned sm_89 AOT
count cubin `88da8f1ebcb2f665b5bd6e32ffc29545e858f8db1ab19c88a612debccf010305`.

The revised C128 finite replay's GO SHA-256 is
`1f266949c4c9249430312bd49b483160e0f2e346a4ea181c8c66072a69c0a082`.
Its sealed receipt is
`1094398383937070308485b73fe439698f01ea30556eaf8635bd955b0e669b97`.
Every order executed all 29 nonempty ranges with a maximum of 6,492 targets.
All eleven raw and eleven adjusted count rows, the AOT image stamps and all
16,459,406 serialized bytes matched the first finite replay. Independent cold
readback matched the archive and all 11 native Q48 probes.
[Revised GPU artifact cold witness](verify/evidence/prod-c128-gpu-liveness.json)

The full-C GPU admission requires **23,333,931,156 free bytes after CUDA
context creation**, including 512 MiB for driver and allocator overhead.
The revised finite run reported 24,838,406,144 device bytes free at that point,
leaving 1,504,474,988 bytes over the full-plan threshold in that observation.
Its measured host availability of 46,194,059,374 bytes also exceeded the
35,496,674,672-byte full plan. The full invocation must recheck both values
under its own lock. Keep 148,096,362,864 bytes for the canonical spool, and
retain enough room for the pruned candidate at the 3,912,936,331-byte compact
cap, plus receipt and filesystem headroom. An allocation refusal or I/O
failure stops the run and preserves any unique prefix as diagnostic material.

A successful count run requires exact TRAIN framing and the plan's complete
range coverage, the registered size, all raw and adjusted cardinalities and
spectra, and full canonical order digests. The historical order-ten raw
digest entry shares the adjusted domain. The order-ten raw and adjusted
follower words coincide and must be written as one shared native field, with
the actual raw-domain digests retained in the output receipt. Upon successful
readback, the streaming pruner must keep every order-zero through order-three
group, apply the same support threshold to all higher orders, fit the
explicit compact byte cap and reproduce the selected words and Q48 probes
through the cold reader. Record elapsed time, memory and resulting output
identity. Any count, identity or capacity failure leaves the later fit
unreleased until diagnosed.

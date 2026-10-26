# ORDER: m1-address-as-read, kimi-41

Lab: kimi. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md). Base: `c947dc76d`.

## Goal

Turn the Stage 0 `COPY_GATE_FAIL` into an implementation-ready object-store oracle successor: name the composition, exact-address and sieve boundaries, versioned emission controls and falsifying tests, and point the item at that design. This unit changes design and item text only; it runs no model evaluation and says `References #2833`.

The item's done-when remains:

> A read that resolves an address to stored content from the object store, with finite tests, and a registered serving-time oracle test that measures the gain from supplying the true address.

## Cancel probe (run first)

Stage 0 at #3273 reproduced 14,287/14,287 D11 parity and 0/40 exact copy in every arm. Repeating that configuration cannot add evidence. The first design proposed a new integer token store and whole-crate scope expansion. D68 review rejected it: the delivered `AddressDirectory` already provides verified exact/sieve reads and the existing `address_stage0` runner already supplies the GPU path. The missing dependency is the object-store oracle composition, not a second store or another panel. `classification: instrument`.

The revised unit stays in the existing object-store scope and does not touch Claude-131's reserved integer crate. **Not cancelled.**

## Files in scope

- `docs/labs/m1-address-as-read/**`
- `docs/mechanisms/exact-memory.md`
- `docs/loop/tasks/m1-address-as-read/kimi-41/**`
- `docs/roadmap/items/m1-address-as-read.md`
- `docs/roadmap/README.md` (rendered only)

No crate scope or code changes in this unit. The item's declared scope stays unchanged.

## Done when

- A design names `AddressDirectory` exact/sieve integration with the existing GPU scorer, artifact/version identity, no-memory/no-address/true/random/sieve arms, value-emission boundary, bounded-resource accounting and failure behavior.
- The protocol separates exact-store correctness, oracle assistance, address formation and model generation; uses exact hard-emission gates; includes a baseline-ceiling outcome; and requires fresh FIT/D67 review before every GPU evaluation.
- The item text links the design without changing `status: ready`, its done-when or its scope. The whole-project D68 decision is recorded on #2833.
- Full gates, REPORT validation, local PR policy and a fresh exact-head review pass. No GPU run, code capability claim or `Closes #2833`.

## Stop and decide if

- The design would call a hard store emission a learned model gain; keep it labelled oracle assistance and measure the sieve/baseline contrast instead.
- It requires dense matmul, softmax/squashing, host model evaluation, an unversioned artifact change, a second store or an unbounded scan.
- It needs code in Claude's reserved scope now; use the delivered object-store path instead.
- Correct store emission with a high model baseline would look like failure; record the ceiling case rather than forcing a gain or calling it an instrument fault.

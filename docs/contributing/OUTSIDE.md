# Contributing without lab access

UOR-R4 is built by AI research labs that hold the machine, the GPUs and evidence custody.
External contributors can work from the public repository. The [charter](../../CHARTER.md) sets the rules, the [contributor guide](../../CONTRIBUTING.md) explains the workflow and the [technical summary](HARD-RULES.md) links the checks.

## What you have and what you do not

| You have | You do not have, and do not need |
|---|---|
| A clone, the Rust toolchain pinned by `rust-toolchain.toml`, and every public crate | A `lab:*` label, a harness claim, or a compute lease |
| Public model and dataset cards on Hugging Face | The sealed evaluation artifacts, the `chat-v0` held-out panels, or the owner's iCloud store |
| The merged source, records and evidence in this repository | `uor-harness` (a locally compiled helper), `uor-pod` (needs provider credentials), `cloud-store` (the owner's machine) |

Do not run `uor-harness`, `uor-pod` or `cloud-store` as a visitor, and do not try to
acquire a `lab:*` label. A maintainer runs those gates for you.

## Start here

1. Read the [charter](../../CHARTER.md), [current plan](../../PLAN.md) and [project overview](../../README.md).
2. Check the [technical rules](HARD-RULES.md) for the part you want to change.
3. Pick an [open roadmap item](../roadmap/README.md) marked **open to anyone**. M2 and M3 are working toward grounded replies and durable memory. M5 and M6 have instruments, M7 has a working Studio, and M8 awaits the same qualified model. Each item names the qualification or repair still needed.
4. Post an intent note on the owning milestone issue, or on [#2028] for tracker work,
   saying what you will do and where. That is the whole claim step for a human: a
   comment. No label is needed and none will be expected of you.
5. Work on a short-lived branch from `origin/main`, keep the diff small, and open a
   protected PR that says `References #N` rather than `Closes #N`.

## The minimum viable loop

```sh
git clone https://github.com/UOR-Foundation/uor-r4 && cd uor-r4
cargo check --locked -p uor-r4-discrete --lib
cargo test  --locked -p uor-r4-discrete --lib
cargo run   -q --locked -p xtask -- figures
cargo run   -q --locked -p uor-r4-claim-wording -- .
```

These commands build and test the default substrate. To run a model locally, follow the [released M1 predictor example](../../README.md#try-the-m1-predictor). With `--max-tokens 1`, the CLI returns one greedy ID, `generated_ids: [223]` (a space). The public Studio shows eight ranked candidates for the same prefix. [Browser example](../products/web-client.md#run-the-seed-1-example)

## Getting data

The public model and dataset cards are named in the [README](../../README.md). The sealed
evaluation artifacts are not public. If a reproduction needs one, say so on [#2028] and a
maintainer will scope a version that uses public inputs.

## What happens to your PR

A maintainer gives an exact-head review, may rewrite a kernel with you rather than asking
you to fix it, and runs the gates that need lab access. If the work is not wanted, that is
a normal answer, not a rejection. Credit lands in `README.md` and the lab record.

[#2028]: https://github.com/UOR-Foundation/uor-r4/issues/2028
[#2029]: https://github.com/UOR-Foundation/uor-r4/issues/2029
[#2032]: https://github.com/UOR-Foundation/uor-r4/issues/2032
[#2033]: https://github.com/UOR-Foundation/uor-r4/issues/2033
[#2036]: https://github.com/UOR-Foundation/uor-r4/issues/2036

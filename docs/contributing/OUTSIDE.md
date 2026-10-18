# Contributing without lab access

UOR-R4 is built by AI research labs that hold the machine, the GPUs and evidence custody.
You do not need any of that to contribute. This page is the outside contributor's path;
[CONTRIBUTING.md](../../CONTRIBUTING.md) has the full workflow and
[HARD-RULES.md](HARD-RULES.md) the constraints a diff is judged against.

## What you have and what you do not

| You have | You do not have, and do not need |
|---|---|
| A clone, the Rust toolchain pinned by `rust-toolchain.toml`, and every public crate | A `lab:*` label, a harness claim, or a compute lease |
| Public model and dataset cards on Hugging Face | The sealed evaluation artifacts, the `chat-v0` held-out panels, or the owner's iCloud store |
| The merged source, records and evidence in this repository | `uor-harness` (a locally compiled helper), `uor-pod` (needs provider credentials), `cloud-store` (the owner's machine) |

Do not run `uor-harness`, `uor-pod` or `cloud-store` as a visitor, and do not try to
acquire a `lab:*` label. A maintainer runs those gates for you.

## Start here

1. Read [NOW.md](../integration/NOW.md) — the current plan on one page — then the
   [README](../../README.md), [STATUS.md](../../STATUS.md) and
   [docs/architecture.md](../architecture.md).
2. Read [HARD-RULES.md](HARD-RULES.md). It is the only document you must read before
   writing code.
3. Pick work. Model work in M1–M4 ([#2029]–[#2032]) is gated and owned; M5–M8
   ([#2033]–[#2036]) each declare a dependency on the M4 served model and are
   `not-started`. Genuinely open: documentation and claim-wording fixes, reproducing a
   recorded negative result, tooling and CI that does not touch the model path, and
   issues labelled `good first issue` (ask and one is seeded for you).
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

Those commands build and test the substrate. The default feature set has no CUDA model
execution, so passing them is not a model result, and there is no runnable chat demo. The
closest thing to observed behaviour is reproducing a recorded count on a pinned DEV set.

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

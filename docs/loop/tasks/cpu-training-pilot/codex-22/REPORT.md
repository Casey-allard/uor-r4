# REPORT — carry-hold receipt retention and platform coverage

Lab: codex. [ORDER](ORDER.md). Maintainer follow-up to [PR #3889](https://github.com/UOR-Foundation/uor-r4/pull/3889).

## What did not work

At `d824de345`, the batch success test expected a Linux descriptor operation on macOS, and both optional test hooks could truncate existing receipts. The next review at `0f4c31185` found that two open flags in the Linux reader used fixed numeric values across architectures. In this verification, the default Linux cross-build also failed to find Linux C headers for BLAKE3’s NEON implementation.

## What changed

Both platform tests use the same valid two-arm fixture. Linux tests the ordering and byte-identical repeat. Other targets require the documented Linux-containment refusal and verify that no report exists. The two optional receipt hooks call the same create-new writer. Its regression first retains a receipt, attempts to replace it with a shorter one and compares the saved bytes to the original.

The manifest reader now passes `libc::O_NOFOLLOW` and `libc::O_NONBLOCK` to the operating system. The CPU crate declares `libc` for Linux targets using the existing workspace lock at 0.2.186. Its regular-file and opened-descriptor containment checks still follow the open.

## Measurements

On Apple silicon macOS all eight tests of the audit binary passed. A separate invocation exercised the actual single-report retention hook twice against the same destination. The second invocation failed with an existing-file error, and byte comparison confirmed the original SHA-256 `c2c83d0cbf500e46400e750327470ab40b6ee9cca3c868ad2fd77c9660074fc3` was preserved.

## Checks run at the head

- `cargo test --locked --offline -p uor-r4-cpu-train --bin cpu-carry-hold-audit` on macOS.
- `cargo check --locked --offline --target aarch64-unknown-linux-gnu -p uor-r4-cpu-train --tests --features blake3/pure` checked the Linux test code with the portable hash backend.
- `rustfmt --check --edition 2021` on the changed Rust file and `git diff --check` passed.

The host and AArch64 checks above passed again after the flag correction. The locked libc source provides different definitions for AArch64 and MIPS64 as required by their respective Linux ABIs.

An independent native run used the committed source at `14caa4eafc233a64ad637600c6427e1149e0cbcc` on Ubuntu 24.04.5 LTS, Linux 6.8.0-142-generic AArch64, rustc and Cargo 1.97.1. `cargo test --locked -p uor-r4-cpu-train --bin cpu-carry-hold-audit` ran all 10 tests. All passed, with zero ignored or filtered cases. The run exercised FIFO, symlink and non-regular-file refusals, retained receipt collision and deterministic repeated batch output. Local verification log SHA-256: `d2f6b18a13f9ab9c81cf0f7c3d8f3cf9a4cfa0fc5f0343d70f715060755d2d67`.

## Skills used

Source review, native host testing and Linux cross-compilation.

## Still open

Both native macOS and AArch64 Linux checks now have results. The historical eight-checkpoint evidence and resource ledger retain their original recorded scope.

## Plain summary

A repeated test can no longer overwrite a retained receipt. The tests exercise the platform behavior the tool actually supports.

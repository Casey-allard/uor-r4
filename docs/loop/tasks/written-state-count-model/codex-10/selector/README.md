# Crossed geometric selection and read-back on Wick

TRAIN writes a six-root context record per supervised occurrence and eligible connector boundary. The ranker minimizes ordered root-coordinate distances with a fixed exclusion margin. Both arms share one frozen bank. A bank row is a typed array of six raw 32-bit words with a missing-root sentinel. Sorting and deduplicating equal prototypes preserve minimum distance exactly. The checkpoint and tokenizer classes are bound to the seed by digest. The blind command accepts bare token tapes, the old blind site scores, and the frozen banks. Only the separate oracle command takes grader rows. Query-only and fact-only constrain the old minimum to one known occurrence; all original alternatives on the other side are retained. The crossed WordStart proposal enters the frozen continuation operator. The value/wrong/no-read controls take effect at its outgoing source. Each realized handoff records both the input start and outgoing source.

The ahead-of-time CUDA kernel emits the actual fit and rank words. Its host instantiation independently executes the same typed operators and checks every word before release. A second implementation in Rust independently reconstructs the prototype fit, full-word pairs, root metric, contrastive cost, tie order and complete combined-arm selections. Context collision counts estimate the best possible binary classification on a **single local descriptor** with fixed annotations. Joint selection and the downstream writer also see additional information.

Build on Wick from the repository root. The first command builds a private executable with the changed downstream handoff entry. All other commands use their indicated independent manifests. The recorded binary digests describe the actual release environment.

```sh
export PATH="$HOME/.cargo/bin:/usr/local/cuda/bin:$PATH" CUDA_HOME=/usr/local/cuda UOR_NVCC_ARCH=compute_89
unit=docs/loop/tasks/written-state-count-model/codex-10/selector
cargo build --release --target x86_64-unknown-linux-gnu --manifest-path docs/labs/m1-d11-artifact-scorer/instrument/Cargo.toml --features entity-pair-chain --bin entity-pair-chain
cargo build --locked --release --target x86_64-unknown-linux-gnu --manifest-path "$unit/Cargo.toml"
cargo build --locked --release --target x86_64-unknown-linux-gnu --manifest-path "$unit/verify/Cargo.toml"
/usr/local/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_89 "$unit/cross.cu" -o bin/cross-device
```

Place the resulting Rust binaries under the private run's bin folder as chain, cross-input, and cross-verify. Run both finite checks with bin/chain fixture and bin/cross-input fixture bin/cross-device. [Wick run](run-wick.sh) binds each stage to the bank, tape, baseline and device lock. The [independent replay](replay-wick.sh) runs after blind predictions are sealed. The original four panel families and the separate unseen baseline are named in the runner. First-error positions come from the [actual-feedback writer](../writer/README.md).

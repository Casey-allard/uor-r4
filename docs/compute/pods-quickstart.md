# Running the GPU pod tooling from your own Runpod account

`scripts/pod/uor-pod` starts, bootstraps, leases and stops Runpod GPU pods for this project. Inside the project it uses the project's account. This page is for anyone else.

## What you need

- A Runpod account, and `runpodctl` configured with your own API key (`runpodctl config --apiKey ...`). `uor-pod` never reads or prints the key.
- A network volume in one datacenter (Runpod > Storage). Pods mount it at `/workspace`, which is the only durable place on a pod.
- An SSH key added in your Runpod account settings.
- `jq`, `gh` (only if you post to a GitHub board) and bash.
- Optionally a Hugging Face token in `~/.cache/huggingface/token`; `uor-pod up` copies it to the pod over SSH. The public data needs no token.

## Configure

```bash
mkdir -p ~/.config/uor-pod
cp scripts/pod/config.example ~/.config/uor-pod/config
```

Edit the volume id, datacenter, SSH key and caps. Every value can also be set as an environment variable, which wins over the file.

## First pod

```bash
scripts/pod/uor-pod status
scripts/pod/uor-pod up --lab me --session s1 --purpose "smoke test" --hours 1 --gpu 4090 --count 1
```

`up` checks every argument before calling the Runpod API, refuses to pass your caps, and then bootstraps the pod. The bootstrap:

1. clones the repository at `--ref` (default `main`) and installs the pinned Rust toolchain (1.97.1);
2. finds the CUDA 12.8 toolkit (installed from NVIDIA's apt repository if the image lacks it) and the GPU's compute capability;
3. builds `uor-r4-training` with `--features cuda`;
4. runs the GPU-against-CPU parity test (`cuda_stack_ops_parity`) and keeps the build in `/workspace/bin/<commit>-sm<capability>/` as a reusable cache only if that test passes.

Pods named `uor-test-*` (the `--test` flag) are not counted toward the pod cap, so the example leaves it out.

Then run a job on one GPU and stop the pod when you are done:

```bash
scripts/pod/uor-pod run POD --lab me --session s1 --gpu 0 -- nvidia-smi
scripts/pod/uor-pod release POD --lab me --session s1
scripts/pod/uor-pod down POD --lab me --session s1
```

A stopped pod still bills for its disk; `down` deletes it. Results you want to keep must be on `/workspace` or copied off first.

## What comes from NVIDIA and what is ours

- **From NVIDIA, through the image and NVIDIA's apt repository:** the GPU driver (`libcuda`), the runtime compiler NVRTC, and `nvcc` 12.8, which the `candle` library uses to build its own kernels. They are not in this repository and are not redistributed by it.
- **Ours, in this repository (MIT):** the training kernels, written in CUDA C as source strings in `crates/uor-r4-training/src/cuda_stack_kernels.rs` and `crates/uor-r4-training/src/native_geometric_cuda_kernels.rs`, and compiled for your GPU at startup by NVRTC (`cuda_stack_kernels.rs`, `compile_ptx_with_opts`). `crates/uor-r4-training/tests/cuda_stack_ops_parity.rs` checks each of them against the CPU implementation. See [cuda-training.md](cuda-training.md).

## Data

The public dataset [caseyallard/uor-r4-data](https://huggingface.co/datasets/caseyallard/uor-r4-data) has the tokenizer, the TinyStories training split tokenized for this project (`tinystories/`, CDLA-Sharing-1.0), the memory-dialogue sets, the dev split and the project's panels. The 214M checkpoints are at [caseyallard/uor-r4-geometric-214m](https://huggingface.co/caseyallard/uor-r4-geometric-214m). The project's private store also holds third-party-derived fine-tune text and held-out panels, which are not published.

The native prose learner (`train-native-prose`) is CPU-only and needs no pod; see the "Start here" section of the dataset card.

# Pod tooling and data usable from other accounts (9 October 2026, claude)

The owner asked what stops others from using the project's GPU setup. The answer was account details, not secrets or drivers, and this change removes them. Infrastructure, not a model result. References #2037.

## What was blocking
- `uor-pod` hard-coded the project's network volume (`lmd1pfah3y`), its datacenter, a private Runpod template (`h15vb984sw`, a wrapper around the public image `runpod/pytorch:1.0.2-cu1281-torch280-ubuntu2404`), the compute board #2037 and the repository.
- The bootstrap received no settings from `uor-pod`, so a different repository or data store could not reach the pod, and it fetched data only on non-canonical volumes.
- The training corpus of the native prose learner was only in the private Hugging Face store.
- No secrets were in the repository: keys stay in `runpodctl`'s own config, the HF token file and SSH keys.
- CUDA is not a blocker. The driver, NVRTC and nvcc come from NVIDIA through the image. The project's kernels are CUDA C source in `crates/uor-r4-training`, compiled at startup by NVRTC, and are already public under MIT.

## What changed
- `uor-pod` reads an optional `~/.config/uor-pod/config` (environment wins). New overrides: `UOR_POD_REPO_SLUG`, `UOR_POD_CANONICAL_DC`, `UOR_POD_VOLUME_ID`, `UOR_POD_NEW_VOLUME_GB`, `UOR_POD_IMAGE`; `UOR_POD_TEMPLATE=none` creates from the image; `UOR_POD_BOARD=none` posts no GitHub comments.
- The bootstrap takes `--repo-url` and `--hf-store` (characters restricted, checked by `--check-args`); with `--hf-store` it fetches on any volume.
- `scripts/pod/config.example` and [pods-quickstart.md](../../compute/pods-quickstart.md).
- With no config the behaviour is unchanged: the dry-run output of `up` is identical to main `c7fce45b1` (apart from the throwaway test repository's commit id), and the dry-run suite passes 240/240 (224 before, 16 new checks; 12 of them fail with the change set aside).
- Hugging Face: `caseyallard/uor-r4-data` now has `tinystories/tinystories_train.u16` (555,385,505 tokens, md5 `87dc182a…`, CDLA-Sharing-1.0 inherited from TinyStories) with its own licence note, and a "Start here" section on the dataset and model cards.

## Not done
- No real pod was started from an outside account; only dry runs. The first real `up` with `UOR_POD_TEMPLATE=none` is unverified.
- The bootstrap's HF file list is the project's fine-tune set; the public store does not hold those files, so an outside pod skips them with a warning.
- Prebuilt PTX per GPU type was considered and not done: NVRTC compiles in seconds and is needed anyway.

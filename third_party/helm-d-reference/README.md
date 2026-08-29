# HELM-D reference provenance

This directory pins the external reference for the next geometric-attention
direction. It does **not** vendor the HELM repository, Python packages,
datasets, tokenizer, or checkpoint bytes. `UPSTREAM.toml` is the
machine-readable record of the audited upstream revision, source hashes,
licenses, and separately hosted checkpoint.

## Implemented now versus permitted later

The current implementation copies the smallest source-faithful semantic core:
Lorentz projection, the pinned inner-product-derived attention logit, ordinary
causal softmax, and normalized Lorentz-centroid aggregation. UOR's first live
language result then uses a frozen ordinary decoder donor and changes only its
attention coordinate frames to R4/Spin. It does not claim to be a full HELM-D
decoder or to reproduce the HELM-D checkpoint.

The permitted later semantic-copy boundary is the dense HELM-D causal decoder:

- Lorentz token points and dense decoder blocks;
- manifold-valued Q/K/V projections;
- causal softmax over the upstream Lorentz-inner-product distance surrogate;
- Lorentz-centroid value aggregation;
- hyperbolic rotary position encoding, RMS normalization, residuals, and
  SwiGLU feed-forward blocks; and
- the final next-token vocabulary projection.

At the audited revision, the attention logit before masking and softmax is
`(2c + 2c * <q,k>_L) / scale + bias`. This is the implementation's
inner-product-derived extrinsic squared-distance surrogate at its exercised
curvature `c = 1`; it is not an `acosh` geodesic-distance-squared
implementation. An R4 port must preserve that distinction and must declare
any replacement score explicitly.

The boundary excludes MiCE, MoE routing, HMLA, focused-linear attention,
HyperCore graph and vision modules, the bundled evaluation harness and data,
the training corpus, and all model/tokenizer bytes. The initial UOR work may
translate the dense equations into a compact reference, but it must not claim
checkpoint parity until an explicit tensor-key audit and numerical fixture
comparison have passed.

## Why the full repository is not vendored

The upstream package uses eager wildcard imports that pull unrelated graph,
vision, optimizer, PyTorch Geometric, and CUDA dependencies into the HELM-D
import path. It also contains two slightly different HELM-D source trees: the
training-facing `helm/` tree and a private copy under the bundled
`lm-evaluation-harness/`. The evaluator loads with `strict=False`; its
embedding copy omits an unused parameter created by the training tree, and
several numerical clamp floors differ. Those differences are recorded here
instead of being silently inherited.

Accordingly, a future implementation should use direct, narrow imports or an
attributed compact translation. It should preserve the upstream parameter
names only when checkpoint comparison requires them, report the exact expected
missing and unexpected keys, and reject every other mismatch.

## Apple Silicon boundary

The checked-in upstream installation and training recipes are not portable to
this Mac as written. They pin CUDA 12.4 wheels and NVIDIA packages, install
CUDA-specific PyTorch Geometric builds, select four CUDA devices, and launch
four-process `bf16` training. The dense mathematical core is not itself a CUDA
kernel, but its complex-valued RoPE path (`torch.polar`, `view_as_complex`, and
complex multiplication) still needs explicit MPS qualification or a
real-valued sin/cos equivalent.

Use a small CPU reference first, or a compact real-valued Apple-Silicon port.
Do not install the upstream `requirements.txt` on this host merely to import
HELM-D, and do not launch the paper-scale 5-billion-token training recipe as a
mechanism check.

## License and artifact handling

The copied `LICENSE` is the upstream MIT notice and must accompany any copied
or substantially translated source. The paper and Zenodo checkpoint are
separately marked CC BY 4.0. The required Llama 3.1 tokenizer is gated and
covered by Meta's Llama 3.1 Community License; it is not included here.

The 1.382 GB checkpoint remains external. It must be downloaded only into an
ignored model cache after checking its published MD5, never committed to Git,
and never treated as a UOR result. Its published architecture is approximately
115 million parameters (`L6_W390_A6`, 128,256-token vocabulary, 2,048-token
context); the upstream filename calls it 120M.

### 6.3 Real-text comparison of time-mixing mechanisms (lead; WikiText-2 bytes)

**Setup.** A 3-layer byte-level language model:
- d = 128, a GLU channel mixer and RMSNorm;
- about 0.56M parameters (0.76M for the GRU);
- 1,000 steps × 4,096 bytes = 4.1M training bytes;
- AdamW with warmup and cosine decay;
- evaluated on the first 256 KiB of WikiText-2 validation in windows of 512 bytes.

The linear recurrences are trained with an associative scan. Every time-mixer uses four d×d maps; the GRU has six.

| Time-mixer | Bits/byte, seed 0 | Notes |
|---|---:|---|
| Diagonal real decay (Mamba/minGRU-like, commutative) | **1.869** | Fastest to train |
| Quaternion rotation + radial decay (continuous) | 1.881 | Snapped to 2I after training: **2.060** |
| Quaternion, trained with straight-through snapping to 2I in every lane | 1.984 | All lanes forced onto the 120-element group |
| Complex (2-D rotation + decay, commutative) | [pending] | |
| Quaternion snapped to the integer small-rotation codebook | [pending] | |
| Softmax attention + RoPE (transformer-style reference only) | [pending] | |
| GRU (nonlinear, sequential; +35% parameters) | [pending] | |
| Second seeds: quaternion, diagonal | [pending] | |

Rows marked [pending] were still running when this document was first committed. They will be filled in by a follow-up commit on the same pull request.

**Reading (preliminary; one seed).**
- Non-commutative rotation gives **no** language-modelling advantage over diagonal decay at this scale, as the literature predicts.
- Forcing *all* lanes onto the coarse 2I group costs about 0.10 bits/byte.

Both support the lane-mixture design in §8.4: exact 2I only for tracking lanes, finer rounded lanes for content.


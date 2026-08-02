# Probability metadata and message information

The observation compiler computes a full teacher softmax while the teacher
logits are available. The deployed transformerless runtime does not reproduce
that softmax: it consumes precompiled integer evidence and `ScoreQ` residuals.

## Recorded quantities

New observation shards retain the original 88-byte observation record and add
an aligned `shard-NN.bin.prob` sidecar. Each 16-byte row contains:

- `target_logprob_nats`: `ln P(target | context)` for the recorded target;
- `entropy_bits`: `-sum(p * log2(p))` over the complete teacher vocabulary;
- `top8_mass`: the original probability mass covered by the eight retained
  tokens, before their stored evidence weights are renormalized;
- `target_rank`: the target's rank when it is among those eight, otherwise
  `u16::MAX`.

The top-8 evidence weights remain a compact distillation surface for compiler
induction. They sum to 100 **after conditioning on the top eight**, and are
therefore not the original full-vocabulary probabilities. `top8_mass` is what
prevents that distinction from being lost.

## Message probability

For a token sequence `x[0..T]`, the teacher message probability is

`P(x) = product_t P(x[t] | x[..t])`.

The implementation never multiplies these probabilities. It reports the
equivalent additive information quantity:

`message_bits = sum_t -log2(P(x[t] | x[..t]))`

and `bits_per_token = message_bits / T`. This is the cross-entropy/NLL-style
measurement for a teacher-forced message and remains numerically stable for
long sequences.

## Runtime boundary

Entropy and log probabilities are compiler/certifier measurements only. Any
future uncertainty-aware serving policy must lower them to an integer artifact
field or a precompiled `ScoreQ` penalty/threshold. The deployed runtime must
not add floating-point logarithms, division, softmax, or probability
normalization.

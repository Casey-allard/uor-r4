// The GPU evaluator of a UORLUT01 SpiralCore/FBS artifact (claude-128): one
// thread per sequence, each from a fresh session, through the same integer
// core the host twin qualifies (src/fbs_core.h). Built ahead of time by nvcc
// into an admitted PTX module; no NVRTC.
#include "fbs_core.h"

// `status[s]` is fbs_sequence's code for sequence `s`.
extern "C" __global__ void fbs_eval_sequences(fbs_model m, const uint32_t *tokens, uint32_t sequences,
                                              uint32_t len, uint8_t *work, uint64_t work_bytes,
                                              uint32_t *out_tokens, uint32_t *out_kept,
                                              int32_t *out_query, int32_t *status) {
  uint32_t s = blockIdx.x * blockDim.x + threadIdx.x;
  if (s >= sequences) return;
  fbs_work w;
  fbs_work_carve(&m, work + (uint64_t)s * work_bytes, &w);
  uint64_t at = (uint64_t)s * len;
  status[s] = fbs_sequence(&m, &w, tokens + at, len, out_tokens + at, out_kept + at * m.kept,
                           out_query + at * m.width);
}

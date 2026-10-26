// Host build of the FBS core, for the finite parity tests against the Rust
// integer session only. Panels are evaluated by src/evaluator.cu on the GPU
// (CHARTER §3.6); the CLI never scores with this.
#include "fbs_core.h"

#include <vector>

extern "C" int64_t fbs_host_sizeof_model(void) { return (int64_t)sizeof(fbs_model); }
extern "C" int64_t fbs_host_sizeof_layer(void) { return (int64_t)sizeof(fbs_layer); }

// The bytes of one sequence's work, as the GPU evaluator allocates them.
extern "C" uint64_t fbs_host_work_bytes(const fbs_model *m) { return fbs_work_bytes(m); }

// `sequences` sequences of `len` tokens each, row-major; outputs per position.
// The work is carved exactly as the GPU kernel carves it.
extern "C" int fbs_host_sequences(const fbs_model *m, const uint32_t *tokens, uint32_t sequences,
                                  uint32_t len, uint32_t *out_tokens, uint32_t *out_kept,
                                  int32_t *out_query) {
  std::vector<uint64_t> buffer(fbs_work_bytes(m) / 8);
  fbs_work w;
  fbs_work_carve(m, reinterpret_cast<uint8_t *>(buffer.data()), &w);
  for (uint32_t s = 0; s < sequences; s++) {
    uint64_t at = (uint64_t)s * len;
    int err = fbs_sequence(m, &w, tokens + at, len, out_tokens + at, out_kept + at * m->kept,
                           out_query + at * m->width);
    if (err != FBS_OK) return err;
  }
  return FBS_OK;
}

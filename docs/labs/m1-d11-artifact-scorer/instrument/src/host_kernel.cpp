// Host build of the scorer's row body, for the finite parity tests against
// IntegerDiscreteModel only. The panel itself is scored by src/scorer.cu on the
// GPU (CHARTER §3.6); the CLI never calls this.
#define D11_HD
#define D11_CONST static const
#include "kernel_core.h"

extern "C" int64_t d11_sizeof_scratch(void) { return (int64_t)sizeof(D11Scratch); }

extern "C" int64_t d11_sizeof_row_out(void) { return (int64_t)sizeof(D11RowOut); }

extern "C" int64_t d11_sizeof_plan(void) { return (int64_t)sizeof(D11Plan); }

extern "C" void d11_host_score_rows(const D11Model* model, const uint32_t* prefixes,
                                    const int32_t* lengths, const uint32_t* targets, int32_t rows,
                                    int32_t stride, D11Scratch* scratch, D11RowOut* out) {
    for (int32_t row = 0; row < rows; ++row) {
        d11_evaluate_row(model, prefixes + (int64_t)row * stride, lengths[row], targets[row],
                         scratch, &out[row]);
    }
}

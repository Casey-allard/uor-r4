// The D11 artifact panel scorer: one thread scores one (prefix, target) row of
// a UORLUT01 lut-discrete artifact, exactly as IntegerDiscreteModel does on the
// host. Integer only; compiled ahead of time by nvcc and admitted as integer
// PTX in build.rs. The geometry tables sit in constant memory.
#define D11_HD __device__
#define D11_CONST static __constant__ const
#include "kernel_core.h"

// shape: vocabulary, blocks, context, kept_cells, stages, read_mode, read_kept,
// read_sweeps. plans holds one D11Plan per prefix length 1..=context. scratch
// and out hold one record per row of this launch.
extern "C" __global__ void d11_score_rows(const int32_t* table, const uint32_t* cells,
                                          const uint32_t* routes, const int32_t* gates,
                                          const int32_t* shape, const D11Plan* plans,
                                          const uint32_t* prefixes,
                                          const int32_t* lengths, int32_t rows,
                                          int32_t stride, D11Scratch* scratch,
                                          D11RowOut* out) {
    int32_t row = (int32_t)(blockIdx.x * blockDim.x + threadIdx.x);
    if (row >= rows) {
        return;
    }
    D11Model model;
    model.vocabulary = shape[0];
    model.blocks = shape[1];
    model.context = shape[2];
    model.kept_cells = shape[3];
    model.stages = shape[4];
    model.read_mode = shape[5];
    model.read_kept = shape[6];
    model.read_sweeps = shape[7];
    model.table = table;
    model.cells = cells;
    model.routes = routes;
    model.gates = gates;
    model.plans = plans;
    d11_evaluate_row(&model, prefixes + (int64_t)row * stride, lengths[row], &scratch[row],
                     &out[row]);
}

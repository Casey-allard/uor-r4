// Apply the registered final writer action to the frozen exact-copy record.
// The preceding scorer kernel owns the cursor, delimiter, cap and fence checks.
#include <stdint.h>
extern "C" __global__ void copy_completion(uint32_t* steps, const uint32_t* enabled,
                                           uint32_t eos, uint32_t n) {
    uint32_t i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n || !enabled[i]) return;
    uint32_t* s = steps + (uint64_t)i * 8;
    if (s[7] == 1 && s[2] > s[1]) s[4] = eos;
}

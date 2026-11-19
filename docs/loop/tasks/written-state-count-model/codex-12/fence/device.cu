// Compose the released ranking and continuation kernels with a delimiter bound.
#include <cuda_runtime.h>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>
#include "value.cu"
#define NONE CONT_NONE
#define WIDTH CONT_WIDTH
#define STRIDE CONT_STRIDE
#include "continuation.cu"
#undef STRIDE
#undef WIDTH
#undef NONE

constexpr u32 ROW = WIDTH + 2;

static void check(cudaError_t result) {
    if (result == cudaSuccess) return;
    std::fprintf(stderr, "%s\n", cudaGetErrorString(result));
    std::exit(2);
}
static void read(u32* values, size_t count) {
    if (std::fread(values, sizeof(u32), count, stdin) != count) std::exit(2);
}
static u32* reserve(size_t count) {
    u32* out;
    check(cudaMalloc(&out, sizeof(u32) * count));
    return out;
}
static u32* upload(const u32* values, size_t count) {
    u32* out = reserve(count);
    check(cudaMemcpy(out, values, sizeof(u32) * count, cudaMemcpyHostToDevice));
    return out;
}

__global__ void delimit(const u32* rows, const u32* stops, u32 n, u32* value, u32* fences) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const u32* row = rows + i * ROW;
    u32* dest = value + i * STRIDE;
    for (u32 j = 0; j < WIDTH; ++j) dest[j] = row[j];
    const u32 end = row[WIDTH], query = row[WIDTH + 1];
    u32 fence = NONE;
    if (end != NONE) {
        for (u32 p = end + 1; p < query && fence == NONE; ++p) {
            for (u32 k = 0; k < 4; ++k) {
                if (row[p] == stops[k]) fence = p;
            }
        }
    }
    fences[i] = fence;
    dest[WIDTH] = 0;
    dest[WIDTH + 1] = fence == NONE ? NONE : end;
    dest[WIDTH + 2] = fence;
}
__global__ void continue_from(const u32* values, const u32* choices, u32 n, u32* out) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const u32* row = values + i * STRIDE;
    const u32* v = choices + i * 7;
    u32* dest = out + i * CONT_STRIDE;
    for (u32 j = 0; j < WIDTH; ++j) dest[j] = row[j];
    dest[WIDTH] = row[WIDTH + 1];
    dest[WIDTH + 1] = row[WIDTH + 2];
    dest[WIDTH + 2] = v[1];
    dest[WIDTH + 3] = v[5];
    dest[WIDTH + 4] = v[6];
}
__global__ void finish(const u32* fences,const u32* values, const u32* continuation, u32 n, u32* out) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    u32* dest = out + i * 13;
    const u32 fence = fences[i], source = continuation[i * 4];
    dest[0] = fence;
    for (u32 j = 0; j < 7; ++j) dest[j + 1] = values[i * 7 + j];
    for (u32 j = 0; j < 4; ++j) dest[j + 8] = continuation[i * 4 + j];
    dest[12] = source < fence && fence - source < 8 ? source : NONE;
}

int main() {
    u32 h[3], stops[4], roots[4096], classes[4096];
    read(h, 3);
    const u32 n = h[0], nr = h[1], ne = h[2];
    if (!n || n > 8192 || !nr || nr > 4096 || !ne || ne > 4096) return 2;
    std::vector<u32> relations(nr * 5), exact(ne * 4), rows(n * ROW), out(n * 13);
    read(stops, 4); read(roots, 4096); read(classes, 4096);
    read(relations.data(), relations.size()); read(exact.data(), exact.size());
    read(rows.data(), rows.size());
    if (std::fgetc(stdin) != EOF) return 2;
    for (u32 i = 0; i < n; ++i) {
        const u32* row = rows.data() + i * ROW;
        const u32 end = row[WIDTH], query = row[WIDTH + 1];
        if (end != NONE && (end >= query || query >= WIDTH)) return 2;
        for (u32 j = 0; j < WIDTH; ++j) if (row[j] >= 4096) return 2;
    }
    for (u32 i = 0; i < nr; ++i) if (relations[i * 5] < 1 || relations[i * 5] > 3) return 2;
    for (u32 i = 0; i < ne; ++i) if (exact[i * 4] < 1 || exact[i * 4] > 3) return 2;
    auto d_stops = upload(stops, 4), d_roots = upload(roots, 4096), d_classes = upload(classes, 4096);
    auto d_rel = upload(relations.data(), relations.size()), d_exact = upload(exact.data(), exact.size());
    auto d_rows = upload(rows.data(), rows.size()), d_fence = reserve(n), d_values = reserve(n * 7);
    auto d_value_input = reserve(n * STRIDE), d_cont_input = reserve(n * CONT_STRIDE);
    auto d_cont = reserve(n * 4), d_out = reserve(out.size());
    const u32 block = 64, grid = (n + block - 1) / block;
    delimit<<<grid, block>>>(d_rows, d_stops, n, d_value_input, d_fence);
    value_predict<<<grid, block>>>(d_value_input, d_roots, d_classes, d_rel, nr, d_exact, ne, n, d_values);
    continue_from<<<grid, block>>>(d_value_input, d_values, n, d_cont_input);
    continuation_predict<<<grid, block>>>(d_cont_input, d_roots, d_classes, d_rel, nr, n, d_cont);
    finish<<<grid, block>>>(d_fence, d_values, d_cont, n, d_out);
    check(cudaGetLastError());
    check(cudaMemcpy(out.data(), d_out, out.size() * sizeof(u32), cudaMemcpyDeviceToHost));
    for (auto p : {d_stops, d_roots, d_classes, d_rel, d_exact, d_rows, d_fence, d_values,
                   d_value_input, d_cont_input, d_cont, d_out}) check(cudaFree(p));
    return std::fwrite(out.data(), sizeof(u32), out.size(), stdout) == out.size() ? 0 : 2;
}

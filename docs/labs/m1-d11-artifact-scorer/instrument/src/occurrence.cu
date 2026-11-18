// Fixed-width E8 occurrence selection. Only the TRAIN vote kernel takes labels.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
typedef unsigned int u32;

extern "C" __global__ void occurrence_positions(
    const u32 *ids, const u32 *lengths, const u32 *roots,
    u32 rows, u32 start_rule, u32 rule_count, u32 *positions) {
    const u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= rows * rule_count) return;
    const u32 row = i / rule_count;
    const u32 rule = start_rule + i % rule_count;
    const u32 anchor_back = rule / 6u;
    const u32 lookback = 1u + rule % 6u;
    const u32 length = lengths[row];
    u32 winner = 0xffffffffu;
    u32 best_distance = 0xffffffffu;
    if (anchor_back < length) {
        const u32 anchor = length - 1u - anchor_back;
        const u32 base = 192u * row;
        const u32 query = roots[ids[base + anchor]];
        for (u32 position = lookback; position < anchor; ++position) {
            const u32 code = roots[ids[base + position - lookback]];
            u32 distance = 0;
            for (u32 block = 0; block < 4; ++block) {
                const u32 x = (code >> (4 * block)) & 15u;
                const u32 y = (query >> (4 * block)) & 15u;
                for (u32 coord = 0; coord < 8; ++coord) {
                    const int diff = (int)D11_ROOTS[x][coord] - (int)D11_ROOTS[y][coord];
                    distance += (u32)(diff < 0 ? -diff : diff);
                }
            }
            if (distance < best_distance) {
                best_distance = distance;
                winner = position;
            }
        }
    }
    positions[i] = winner;
}

extern "C" __global__ void occurrence_fit(
    const u32 *positions, const u32 *labels, u32 rows, u32 *counts) {
    const u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= 2u * 192u) return;
    const u32 arm = i / 192u;
    const u32 rule = i % 192u;
    u32 hits = 0;
    for (u32 row = 0; row < rows; ++row) {
        hits += positions[row * 192u + rule] == labels[row * 2u + arm];
    }
    counts[i] = hits;
}

extern "C" __global__ void occurrence_winners(const u32 *counts, u32 *winners) {
    const u32 arm = threadIdx.x;
    if (arm >= 2) return;
    u32 best = 0;
    for (u32 rule = 1; rule < 192u; ++rule) {
        if (counts[arm * 192u + rule] > counts[arm * 192u + best]) best = rule;
    }
    winners[arm] = best;
}

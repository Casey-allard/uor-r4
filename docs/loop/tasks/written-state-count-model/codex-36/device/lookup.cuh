#pragma once

#include <cuda_runtime.h>
#include "kn_score_division.cuh"

namespace compact_count_cuda {
using U16 = unsigned short;
using U32 = unsigned int;
using U64 = unsigned long long;
using Wide = unsigned __int128;

constexpr U32 order_count = 11;
constexpr U64 one = U64(1) << 48;
constexpr U64 base = U64(1) << 36;

// All pointers refer to device memory. Host derives Order descriptors only
// from native section headers. Each probe writes its own disjoint output.
struct Order {
    U64 keys, rows;
    U32 contexts, followers;
    U64 discounts[3];
};
struct Probe {
    U16 len;
    U16 tokens[order_count];
};
struct OrderResult {
    U32 raw, adjusted;
    U64 raw_total, adjusted_total, q48;
};
static_assert(sizeof(Order) == 48);
static_assert(sizeof(Probe) == 24);
static_assert(sizeof(OrderResult) == 32);

__device__ __forceinline__ U32 read32(const unsigned char *p) {
    return U32(p[0]) | (U32(p[1]) << 8) | (U32(p[2]) << 16) | (U32(p[3]) << 24);
}

__device__ __forceinline__ Wide key_at(const unsigned char *p, U32 width) {
    Wide key = 0;
    for (U32 byte = 0; byte < width; ++byte)
        key = (key << 8) | Wide(p[byte]);
    return key;
}

__device__ __forceinline__ Wide step(
    Wide mass, const Order &descriptor, const OrderResult &r, U32 n1, U32 n2, U32 n3
) {
    const Wide d1 = descriptor.discounts[0], d2 = descriptor.discounts[1], d3 = descriptor.discounts[2];
    const Wide gamma = kn_score_division::divide(d1 * n1 + d2 * n2 + d3 * n3, r.adjusted_total);
    const U32 a = r.adjusted;
    const Wide removed = a == 0 ? 0 : (a == 1 ? d1 : (a == 2 ? d2 : d3));
    return kn_score_division::divide(Wide(a) * one - removed, r.adjusted_total) + ((gamma * mass) >> 48);
}

__global__ void lookup(
    const unsigned char *artifact, const Order *orders, const Probe *probes,
    U32 count, OrderResult *results, U32 *status
) {
    const U32 slot = blockIdx.x * blockDim.x + threadIdx.x;
    if (slot >= count) return;
    const Probe &p = probes[slot];
    OrderResult *out = results + U64(slot) * order_count;
    for (U32 k = 0; k < order_count; ++k) out[k] = {};
    status[slot] = 1;
    if (p.len == 0 || p.len > order_count) return;
    const U32 last = p.len - 1;
    if (p.tokens[last] == 0 || p.tokens[last] >= 4096) return;
    for (U32 j = 0; j < last; ++j) {
        if (p.tokens[j] >= 4096) return;
    }
    Wide mass = base;
    Wide key = 0;
    for (U32 k = 0; k < p.len; ++k) {
        if (k > 0) key = (key << 12) | p.tokens[last - k];
        const Order &d = orders[k];
        const U32 key_width = (12u * k + 7u) >> 3;
        const U32 key_stride = key_width + 4;
        const U32 row_stride = k == 10u ? 6u : 10u;
        const unsigned char *keys = artifact + d.keys;
        U32 left = 0, right = d.contexts;
        while (left < right) {
            const U32 middle = left + ((right - left) >> 1);
            if (key_at(keys + U64(middle) * key_stride, key_width) < key) left = middle + 1;
            else right = middle;
        }
        OrderResult &r = out[k];
        if (left < d.contexts && key_at(keys + U64(left) * key_stride, key_width) == key) {
            const U32 begin = read32(keys + U64(left) * key_stride + key_width);
            const U32 end = left + 1 == d.contexts ? d.followers
                : read32(keys + U64(left + 1) * key_stride + key_width);
            if (end <= begin || end > d.followers || end - begin >= 4096u) return;
            U32 n1 = 0, n2 = 0, n3 = 0;
            U16 prior = 0;
            for (U32 position = begin; position < end; ++position) {
                const unsigned char *row = artifact + d.rows + U64(position) * row_stride;
                const U16 next = U16(row[0]) | (U16(row[1]) << 8);
                const U32 raw = read32(row + 2);
                const U32 adjusted = k == 10u ? raw : read32(row + 6);
                if (next <= prior || next >= 4096 || raw == 0 || adjusted == 0) return;
                prior = next;
                r.raw_total += raw;
                r.adjusted_total += adjusted;
                if (next == p.tokens[last]) {
                    r.raw = raw;
                    r.adjusted = adjusted;
                }
                n1 += adjusted == 1;
                n2 += adjusted == 2;
                n3 += adjusted >= 3;
            }
            mass = step(mass, d, r, n1, n2, n3);
            if (mass >> 64) return;
        }
        r.q48 = mass == 0 ? 1 : U64(mass);
    }
    status[slot] = 0;
}
}  // namespace compact_count_cuda

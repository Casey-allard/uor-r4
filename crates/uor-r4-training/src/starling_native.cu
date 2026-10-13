// Native starling selection (rung 0b, kernel K1; #2032 card 6146209361, A1).
//
// Built ahead of time by nvcc (build.rs, `cuda` feature) into its own PTX
// module: no runtime compiler (owner, 13 October 2026: NVRTC is banned
// project-wide), and no other kernel shares the module.
//
// One thread per (batch row, head). Scanning positions in order: a document
// reset (BOS) first clears every ring of the row; key t then enters the ring
// (depth M) of its nearest icosian vertex, before query t reads, so a query
// sees sources j <= t of its own document only. Query t gathers the keys of
// its home vertex and that vertex's 12 neighbours and keeps the `slots`
// (K kept + E exploration) with the largest address dot, ties to the later
// position. Output per (row, t): the slot positions in rank order
// (STARLING_ABSENT when fewer exist), then the home cell and the key's cell.
// Exact f32 products and sums in a fixed order; no exponential, no
// normalisation, no host loop. Non-finite addresses raise error[0] = 1.

typedef unsigned int uint;
typedef unsigned long long u64;

#define STARLING_CELLS 120u
#define STARLING_NEIGHBOURS 12u
#define STARLING_MAX_SLOTS 16u
#define STARLING_ABSENT 0xffffffffu

__device__ __forceinline__ float starling_dot4(const float* a, const float* b) {
    float s = __fmul_rn(a[0], b[0]);
    s = __fadd_rn(s, __fmul_rn(a[1], b[1]));
    s = __fadd_rn(s, __fmul_rn(a[2], b[2]));
    return __fadd_rn(s, __fmul_rn(a[3], b[3]));
}

// Nearest icosian vertex by dot product; ties to the smallest vertex index.
__device__ __forceinline__ uint starling_nearest(const float* v, const float* roots) {
    uint best = 0;
    float best_dot = -3.402823466e+38f;
    for (uint c = 0; c < STARLING_CELLS; ++c) {
        float d = starling_dot4(v, roots + 4 * c);
        if (d > best_dot) {
            best_dot = d;
            best = c;
        }
    }
    return best;
}

// Rank order: larger address dot first, then the later position.
__device__ __forceinline__ bool starling_before(float s, uint j, float s2, uint j2) {
    return s > s2 || (s == s2 && j > j2);
}

extern "C" __global__ void starling_native_select(
    const float* query_address,   // [rows, time, 4], unit
    const float* key_address,     // [rows, time, 4], unit
    const uint* reset,            // [batch, time], 1 at a document start
    const float* roots,           // [120, 4]
    const uint* adjacency,        // [120, 12]
    uint* ring,                   // scratch [rows, 120, depth]
    uint* ring_len,               // scratch [rows, 120]
    uint* ring_head,              // scratch [rows, 120]
    uint* out,                    // [rows, time, slots + 2]
    uint* error,
    uint rows, uint time, uint heads, uint depth, uint slots
) {
    uint r = blockIdx.x * blockDim.x + threadIdx.x;
    if (r >= rows) return;
    if (slots == 0 || slots > STARLING_MAX_SLOTS || depth == 0 || heads == 0) {
        atomicOr(error, 2u);
        return;
    }
    uint b = r / heads;
    uint* my_ring = ring + (u64)r * STARLING_CELLS * depth;
    uint* my_len = ring_len + (u64)r * STARLING_CELLS;
    uint* my_head = ring_head + (u64)r * STARLING_CELLS;
    for (uint c = 0; c < STARLING_CELLS; ++c) {
        my_len[c] = 0;
        my_head[c] = 0;
    }
    uint width = slots + 2;
    float best_s[STARLING_MAX_SLOTS];
    uint best_j[STARLING_MAX_SLOTS];
    for (uint t = 0; t < time; ++t) {
        if (reset[(u64)b * time + t] != 0) {
            for (uint c = 0; c < STARLING_CELLS; ++c) {
                my_len[c] = 0;
                my_head[c] = 0;
            }
        }
        const float* key = key_address + ((u64)r * time + t) * 4;
        const float* query = query_address + ((u64)r * time + t) * 4;
        if (!isfinite(key[0]) || !isfinite(key[1]) || !isfinite(key[2]) || !isfinite(key[3])
            || !isfinite(query[0]) || !isfinite(query[1]) || !isfinite(query[2])
            || !isfinite(query[3])) {
            atomicOr(error, 1u);
            return;
        }
        uint cell = starling_nearest(key, roots);
        uint* cell_ring = my_ring + (u64)cell * depth;
        if (my_len[cell] == depth) {
            cell_ring[my_head[cell]] = t;
            my_head[cell] = (my_head[cell] + 1) % depth;
        } else {
            cell_ring[(my_head[cell] + my_len[cell]) % depth] = t;
            my_len[cell] += 1;
        }
        uint home = starling_nearest(query, roots);
        uint filled = 0;
        for (uint n = 0; n <= STARLING_NEIGHBOURS; ++n) {
            uint c = n == 0 ? home : adjacency[home * STARLING_NEIGHBOURS + n - 1];
            const uint* c_ring = my_ring + (u64)c * depth;
            for (uint i = 0; i < my_len[c]; ++i) {
                uint j = c_ring[(my_head[c] + i) % depth];
                float s = starling_dot4(query, key_address + ((u64)r * time + j) * 4);
                if (filled < slots || starling_before(s, j, best_s[slots - 1], best_j[slots - 1])) {
                    uint p = filled < slots ? filled : slots - 1;
                    if (filled < slots) filled += 1;
                    while (p > 0 && starling_before(s, j, best_s[p - 1], best_j[p - 1])) {
                        best_s[p] = best_s[p - 1];
                        best_j[p] = best_j[p - 1];
                        p -= 1;
                    }
                    best_s[p] = s;
                    best_j[p] = j;
                }
            }
        }
        uint* o = out + ((u64)r * time + t) * width;
        for (uint i = 0; i < slots; ++i) {
            o[i] = i < filled ? best_j[i] : STARLING_ABSENT;
        }
        o[slots] = home;
        o[slots + 1] = cell;
    }
}

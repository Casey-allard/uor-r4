// Integer interpolated modified Kneser-Ney scorer on the GPU (claude-421).
// Its own module, built ahead of time by nvcc into an admitted PTX and cubin.
// It ports KnTables::mass of kn_score.rs word for word: one thread per scored
// target, Q48 masses, add/subtract/multiply/floor-divide and binary search
// only. No atomics; every output word is written by exactly one thread.
//
// Tables are flat arrays with per-order offsets: order k's contexts are
// ctx_off[k]..ctx_off[k+1] and its triples tri_off[k]..tri_off[k+1], both in
// the counter's canonical order. A key is two u64 words, lo then hi, of the
// 128-bit PrimeRouteKey word. Arithmetic uses nvcc's unsigned __int128 for
// add, subtract, multiply and shift; division is div_u128_u64 below, never
// nvcc's own 128-bit division routine.

typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;
typedef unsigned __int128 u128;

#define KN_ONE (((u64)1) << 48)
#define KN_BASE (((u64)1) << 36)

// (ahi, alo) < (bhi, blo) as one 128-bit word.
__device__ __forceinline__ bool key_less(u64 ahi, u64 alo, u64 bhi, u64 blo) {
    return ahi < bhi || (ahi == bhi && alo < blo);
}

// floor(x / d) for d > 0, integer instructions only. nvcc's own 128-bit
// division calls a routine that the integer PTX admission refuses (it holds
// .f32), so: the high word by one native 64-bit divide, then the low word by
// 64 shift-and-subtract steps on a remainder below d.
__device__ __forceinline__ u128 div_u128_u64(u128 x, u64 d) {
    u64 hi = (u64)(x >> 64), lo = (u64)x;
    u64 q_hi = hi / d;
    u128 r = (u128)(hi - q_hi * d);
    u64 q_lo = 0;
    for (int b = 63; b >= 0; --b) {
        r = (r << 1) | (u128)((lo >> b) & 1ull);
        if (r >= (u128)d) {
            r -= (u128)d;
            q_lo |= 1ull << b;
        }
    }
    return ((u128)q_hi << 64) | (u128)q_lo;
}

// One thread per target j = positions[i]: masses[i] gets its Q48 mass and
// flags[i] is 1 when that mass is 0 or reaches 2^64.
extern "C" __global__ void kn_score_targets(const u16 *tokens, const u32 *positions, u32 count, u32 highest, const u32 *ctx_off, const u64 *ctx_keys, const u64 *ctx_total, const u32 *ctx_counts, const u32 *tri_off, const u64 *tri_keys, const u16 *tri_next, const u32 *tri_count, const u64 *discounts, u64 *masses, u32 *flags) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 j = positions[i];
    u16 target = tokens[j];
    const u128 m = (u128)KN_ONE;
    u128 mass = (u128)KN_BASE;
    u32 top = highest < j ? highest : j;
    for (u32 k = 0; k <= top; ++k) {
        // The key of tokens[j-k..j]: order in bits 120..128, the s-th latest
        // token at bits 120-12s..132-12s, as PrimeRouteKey::from_context.
        u64 lo = 0, hi = (u64)k << 56;
        for (u32 s = 1; s <= k; ++s) {
            u64 v = tokens[j - s];
            u32 bit = 108u - 12u * (s - 1u);
            if (bit >= 64u) {
                hi |= v << (bit - 64u);
            } else {
                lo |= v << bit;
                if (bit + 12u > 64u) hi |= v >> (64u - bit);
            }
        }
        // Lower bound over the contexts of order k.
        u32 a0 = ctx_off[k], a1 = ctx_off[k + 1u];
        while (a0 < a1) {
            u32 mid = a0 + (a1 - a0) / 2u;
            if (key_less(ctx_keys[2u * mid + 1u], ctx_keys[2u * mid], hi, lo)) a0 = mid + 1u;
            else a1 = mid;
        }
        if (a0 == ctx_off[k + 1u] || ctx_keys[2u * a0] != lo || ctx_keys[2u * a0 + 1u] != hi) continue;
        u64 total = ctx_total[a0];
        u128 n1 = ctx_counts[3u * a0], n2 = ctx_counts[3u * a0 + 1u], n3 = ctx_counts[3u * a0 + 2u];
        // Lower bound over the triples of order k by (key, next).
        u32 b0 = tri_off[k], b1 = tri_off[k + 1u];
        while (b0 < b1) {
            u32 mid = b0 + (b1 - b0) / 2u;
            u64 mhi = tri_keys[2u * mid + 1u], mlo = tri_keys[2u * mid];
            bool less = key_less(mhi, mlo, hi, lo) || (mhi == hi && mlo == lo && tri_next[mid] < target);
            if (less) b0 = mid + 1u;
            else b1 = mid;
        }
        u32 a = 0;
        if (b0 < tri_off[k + 1u] && tri_keys[2u * b0] == lo && tri_keys[2u * b0 + 1u] == hi && tri_next[b0] == target)
            a = tri_count[b0];
        u128 d1 = discounts[3u * k], d2 = discounts[3u * k + 1u], d3 = discounts[3u * k + 2u];
        u128 gamma = div_u128_u64(d1 * n1 + d2 * n2 + d3 * n3, total);
        u128 da = a == 0u ? (u128)0 : (a == 1u ? d1 : (a == 2u ? d2 : d3));
        u128 kept = (u128)a * m - da;
        // M = 2^48, so floor(gamma * mass / M) is a shift of a non-negative word.
        mass = div_u128_u64(kept, total) + ((gamma * mass) >> 48);
    }
    bool bad = mass == (u128)0 || (mass >> 64) != (u128)0;
    masses[i] = (u64)mass;
    flags[i] = bad ? 1u : 0u;
}

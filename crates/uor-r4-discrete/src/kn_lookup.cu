// Raw count lookups on the GPU (claude-433). Its own module, built ahead of
// time by nvcc into an admitted PTX and cubin. It ports RawTables::lookups of
// kn_ceiling.rs: one thread per target, the key derivation and the binary
// searches of kn_score.cu, writing the triple's count and the context's total
// for every order instead of a mass. No atomics; every output word is written
// by exactly one thread.
//
// Tables are flat arrays with per-order offsets: order k's contexts are
// ctx_off[k]..ctx_off[k+1] and its triples tri_off[k]..tri_off[k+1], both in
// the counter's canonical order. A key is two u64 words, lo then hi, of the
// 128-bit PrimeRouteKey word.

typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;

// (ahi, alo) < (bhi, blo) as one 128-bit word.
__device__ __forceinline__ bool key_less(u64 ahi, u64 alo, u64 bhi, u64 blo) {
    return ahi < bhi || (ahi == bhi && alo < blo);
}

// One thread per target j = positions[i]: counts and totals at
// i*(highest+1)+k get order k's (a, total), or (0, 0) for an unseen context
// or k > j. The caller zeroes both outputs.
extern "C" __global__ void kn_lookup_targets(const u16 *tokens, const u32 *positions, u32 count, u32 highest, const u32 *ctx_off, const u64 *ctx_keys, const u64 *ctx_total, const u32 *tri_off, const u64 *tri_keys, const u16 *tri_next, const u32 *tri_count, u32 *counts, u64 *totals) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 j = positions[i];
    u16 target = tokens[j];
    u64 base = (u64)i * (u64)(highest + 1u);
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
        totals[base + k] = ctx_total[a0];
        // Lower bound over the triples of order k by (key, next).
        u32 b0 = tri_off[k], b1 = tri_off[k + 1u];
        while (b0 < b1) {
            u32 mid = b0 + (b1 - b0) / 2u;
            u64 mhi = tri_keys[2u * mid + 1u], mlo = tri_keys[2u * mid];
            bool less = key_less(mhi, mlo, hi, lo) || (mhi == hi && mlo == lo && tri_next[mid] < target);
            if (less) b0 = mid + 1u;
            else b1 = mid;
        }
        if (b0 < tri_off[k + 1u] && tri_keys[2u * b0] == lo && tri_keys[2u * b0 + 1u] == hi && tri_next[b0] == target)
            counts[base + k] = tri_count[b0];
    }
}

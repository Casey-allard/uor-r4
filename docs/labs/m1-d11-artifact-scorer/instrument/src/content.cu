// GPU-authoritative native geometric prototype fit and occurrence routing.
#define D11_CONST __device__ __constant__
#include "d11_tables.h"
typedef unsigned int u32;
constexpr u32 NONE = 0xffffffffu, WIDTH = 192, STRIDE = 197, CAP = 288, WORDS = 16;
__device__ u32 metric(u32 a, u32 b) {
    if (a == b) return 0;
    if (a == NONE || b == NONE) return 64;
    u32 cost = 0;
    for (u32 k = 0; k < 4; ++k) {
        u32 x = (a >> (4*k)) & 15, y = (b >> (4*k)) & 15;
        for (u32 i = 0; i < 8; ++i) {
            int diff = D11_ROOTS[x][i] - D11_ROOTS[y][i];
            cost += (diff < 0) ? -diff : diff;
        }
    }
    return cost;
}
__device__ void site_context(const u32 *row, const u32 *roots, u32 at, u32 *out) {
    const int offsets[] = {-2,-1,1,2};
    for (u32 j = 0; j < 4; ++j) {
        int i = (int)at + offsets[j];
        out[j] = (i >= 0 && (u32)i < row[WIDTH]) ? roots[row[i]] : NONE;
    }
}
extern "C" __global__ void content_fit(const u32 *data, const u32 *roots,
        u32 rows, u32 *banks, u32 *counts) {
    const u32 kind = threadIdx.x;
    if (kind >= 4) return;
    u32 *bank = banks + kind*CAP*WORDS;
    const u32 words = kind == 3 ? WORDS : 4;
    u32 count = 0;
    for (u32 r = 0; r < rows; ++r) {
        const u32 *row = data + r*STRIDE;
        u32 sample[WORDS];
        if (kind == 3) {
            const u32 f = row[WIDTH+3], s = row[WIDTH+4], n = s-f-1;
            sample[0] = n;
            for (u32 j = 0; j < 15; ++j) sample[j+1] = j<n ? roots[row[f+1+j]] : NONE;
        } else {
            const u32 site = row[WIDTH+1+kind];
            if (site == NONE) continue;
            site_context(row,roots,site,sample);
        }
        bool found = false;
        for (u32 i = 0; i < count && !found; ++i) {
            bool same = true;
            for (u32 j = 0; j < words; ++j) same &= bank[i*WORDS+j] == sample[j];
            found = same;
        }
        if (!found) {
            for (u32 j = 0; j < words; ++j) bank[count*WORDS+j] = sample[j];
            ++count;
        }
    }
    counts[kind] = count;
}
__device__ u32 nearest(const u32 *site, const u32 *bank, u32 count) {
    u32 best = NONE;
    for (u32 i = 0; i < count && best; ++i) {
        u32 cost = 0;
        for (u32 j = 0; j < 4; ++j) cost += metric(site[j],bank[i*WORDS+j]);
        if (cost < best) best = cost;
    }
    return best;
}
extern "C" __global__ void content_sites(const u32 *data, const u32 *roots,
        u32 rows, const u32 *banks, const u32 *counts, u32 ablated, u32 *scores) {
    const u32 i = blockIdx.x*blockDim.x + threadIdx.x;
    if (i >= rows*WIDTH) return;
    const u32 *row = data + (i/WIDTH)*STRIDE;
    const u32 at = i%WIDTH;
    if (at >= row[WIDTH]) return;
    u32 site[4];
    site_context(row,roots,at,site);
    u32 cost = 0;
    if (!ablated) {
        const u32 excluded = nearest(site,banks+CAP*WORDS,counts[1]);
        cost = nearest(site,banks,counts[0]) + (excluded < 48 ? 48-excluded : 0);
    }
    scores[2*i] = cost;
    scores[2*i+1] = nearest(site,banks+2*CAP*WORDS,counts[2]);
}
extern "C" __global__ void content_bind(const u32 *data, const u32 *roots,
        u32 rows, const u32 *banks, const u32 *counts, const u32 *scores, u32 *out
#ifdef ENTITY_PAIR_CLASS_GATE
        , const u32 *classes, u32 gated
#endif
        ) {
    const u32 r = blockIdx.x*blockDim.x + threadIdx.x;
    if (r >= rows) return;
    const u32 *row = data+r*STRIDE;
    scores += 2*r*WIDTH;
    u32 best = NONE, query = NONE, fact = NONE, source = NONE;
    for (u32 q = 0; q < row[WIDTH]; ++q) {
        for (u32 f = 0; f+1 < q; ++f) {
            if (row[f] != row[q]) continue;
#ifdef ENTITY_PAIR_CLASS_GATE
            if (gated && !entity_pair_matches(row,row[WIDTH],classes,q,f)) continue;
#endif
            const u32 cost = scores[2*q] + scores[2*f+1];
            if (cost < best) { best=cost; query=q; fact=f; }
        }
    }
    if (query != NONE) {
        best = NONE;
        const u32 *relations = banks+3*CAP*WORDS;
        for (u32 s = fact+1; s < query && s <= fact+16; ++s) {
            const u32 gap = s-fact-1;
            for (u32 k = 0; k < counts[3]; ++k) {
                const u32 *rel = relations+k*WORDS;
                const u32 len = rel[0];
                u32 cost = 64*(gap>len ? gap-len : len-gap);
                for (u32 j = 0; j < gap && j < len; ++j) {
                    cost += metric(roots[row[fact+1+j]],rel[j+1]);
                }
                if (cost < best) { best=cost; source=s; }
            }
        }
    }
    out[3*r] = query; out[3*r+1] = fact; out[3*r+2] = source;
}

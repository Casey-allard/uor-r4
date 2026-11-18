// Exact n-gram counting over a resident u16 token stream (claude-412).
// Its own module: built ahead of time by nvcc into an admitted PTX and cubin,
// independent of discrete.cu. Integer only. The atomics are shared-memory
// integer adds in the planning histogram and, in the adjust mode (claude-429),
// global integer adds and mins on four violation words; none of their results
// depends on order, and every array that reaches the output is written by
// exactly one thread.
//
// Order k counts target j (k <= j < n) under context t[j-k..j]. Positions are
// sorted by their gram with a stable LSD radix sort (6 bits per pass) that
// reads each slot's keys from the stream once (claude-430), then run-length
// reduced by flags and scans.

typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;

#define RADIX_TILE 4096u
#define RADIX_WARPS 8u
#define BUCKET_TILE 1048576u
#define SCAN_THREADS 256u
#define SCAN_ITEMS 8u
#define SCAN_BLOCK (SCAN_THREADS * SCAN_ITEMS)
#define FULL_MASK 0xffffffffu
// A radix `back` with this bit set reads the left digit of order (back & 0xff).
#define LEFT_DIGIT 0x100u

// claude-429: the left digit of order-k target j. Order k+1 admits j as well
// when j >= k+1 and the slot t[j-k] that becomes interior is not BOS; then
// L(j) = t[j-k-1] + 1 (1..=4096, 13 bits), the distinct left token of the
// order-(k+1) gram ending at j. Otherwise L(j) = 0. Reads t[j-k] and
// t[j-k-1] only when j > k.
__device__ __forceinline__ u32 left_digit(const u16 *tokens, u32 j, u32 k) {
    return (j > k && tokens[j - k] != 0u) ? (u32)tokens[j - k - 1u] + 1u : 0u;
}

// The whole key word of one radix slot (claude-430): a token, or the left digit.
__device__ __forceinline__ u32 radix_key(const u16 *tokens, u32 j, u32 back) {
    if (back & LEFT_DIGIT) return left_digit(tokens, j, back & 0xffu);
    return (u32)tokens[j - back];
}

// Document boundaries (claude-415): order k admits target j only when t[j] is
// not BOS (0) and no BOS lies in t[j-k+1..j-1]. BOS may sit in the oldest
// slot t[j-k] alone, so a record's opening n-gram keeps its start marker.
// The host oracle's ngram_admitted is the same predicate.
__device__ __forceinline__ bool ngram_admitted(const u16 *tokens, u32 j, u32 k) {
    if (tokens[j] == 0u) return false;
    for (u32 s = 1u; s < k; ++s)
        if (tokens[j - s] == 0u) return false;
    return true;
}

// ---------------------------------------------------------------- planning

// Per-block histogram of the partition token t[j-off] over the admitted
// targets j in begin..end of order k; shared integer adds only.
extern "C" __global__ void ngram_token_hist(const u16 *tokens, u32 begin, u32 end, u32 k, u32 off,
                                            u32 *block_hists) {
    __shared__ u32 h[4096];
    for (u32 v = threadIdx.x; v < 4096u; v += blockDim.x) h[v] = 0;
    __syncthreads();
    for (u32 j = begin + blockIdx.x * blockDim.x + threadIdx.x; j < end; j += gridDim.x * blockDim.x)
        if (ngram_admitted(tokens, j, k)) atomicAdd(&h[tokens[j - off] & 4095u], 1u);
    __syncthreads();
    for (u32 v = threadIdx.x; v < 4096u; v += blockDim.x) block_hists[blockIdx.x * 4096u + v] = h[v];
}

// Column sums of the per-block histograms, in block order.
extern "C" __global__ void ngram_token_hist_reduce(const u32 *block_hists, u32 blocks, u64 *hist) {
    u32 v = blockIdx.x * blockDim.x + threadIdx.x;
    if (v >= 4096u) return;
    u64 sum = 0;
    for (u32 b = 0; b < blocks; ++b) sum += block_hists[b * 4096u + v];
    hist[v] = sum;
}

// ---------------------------------------------------------------- selection

// flags[i] = 1 when target j = begin + i of order k is admitted and has its
// partition token in lo..hi.
extern "C" __global__ void ngram_select_flags(const u16 *tokens, u32 begin, u32 count, u32 k, u32 off,
                                              u32 lo, u32 hi, u32 *flags) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 d = tokens[begin + i - off];
    flags[i] = (d >= lo && d < hi && ngram_admitted(tokens, begin + i, k)) ? 1u : 0u;
}

extern "C" __global__ void ngram_select_scatter(const u16 *tokens, u32 begin, u32 count, u32 k, u32 off,
                                                u32 lo, u32 hi, const u32 *scan, u32 out_base, u32 *pos) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 d = tokens[begin + i - off];
    if (d >= lo && d < hi && ngram_admitted(tokens, begin + i, k)) pos[out_base + scan[i]] = begin + i;
}

// ---------------------------------------------------------------- buckets

// claude-425: every position j in 1..n, stably counting-sorted once by its
// slot-1 token t[j-1] (the partition token of every order k >= 1). One warp
// per tile of BUCKET_TILE positions; hist is bin-major [4096][tiles], so its
// exclusive scan gives each (bin, tile) its output offset. Within a round the
// leader of each distinct bin adds its peer count: distinct addresses, no atomics.
extern "C" __global__ void ngram_bucket_hist(const u16 *tokens, u32 n, u32 tiles, u32 *hist) {
    __shared__ u32 h[4096];
    u32 lane = threadIdx.x % 32u, tile = blockIdx.x;
    for (u32 v = lane; v < 4096u; v += 32u) h[v] = 0;
    __syncwarp();
    u32 begin = 1u + tile * BUCKET_TILE;
    u32 end = min(begin + BUCKET_TILE, n);
    for (u32 base = begin; base < end; base += 32u) {
        u32 j = base + lane;
        bool valid = j < end;
        u32 d = valid ? ((u32)tokens[j - 1u] & 4095u) : 4096u;
        u32 peers = __match_any_sync(FULL_MASK, d);
        if (valid && lane == (u32)(__ffs(peers) - 1)) h[d] += __popc(peers);
        __syncwarp();
    }
    for (u32 v = lane; v < 4096u; v += 32u) hist[v * tiles + tile] = h[v];
}

// Stable scatter of the bucket sort: rank = peers with a lower lane; rounds
// and tiles keep ascending j, so each bucket holds its positions ascending.
extern "C" __global__ void ngram_bucket_scatter(const u16 *tokens, u32 n, u32 tiles, const u32 *offsets,
                                                u32 *bucketed) {
    __shared__ u32 base_of[4096];
    u32 lane = threadIdx.x % 32u, tile = blockIdx.x;
    for (u32 v = lane; v < 4096u; v += 32u) base_of[v] = offsets[v * tiles + tile];
    __syncwarp();
    u32 begin = 1u + tile * BUCKET_TILE;
    u32 end = min(begin + BUCKET_TILE, n);
    for (u32 base = begin; base < end; base += 32u) {
        u32 j = base + lane;
        bool valid = j < end;
        u32 d = valid ? ((u32)tokens[j - 1u] & 4095u) : 4096u;
        u32 peers = __match_any_sync(FULL_MASK, d);
        u32 below = peers & ((1u << lane) - 1u);
        u32 slot = valid ? base_of[d] + __popc(below) : 0u;
        __syncwarp();
        if (valid) {
            bucketed[slot] = j;
            if (below == 0u) base_of[d] += __popc(peers);
        }
        __syncwarp();
    }
}

// flags[i] = 1 when bucketed position j = bucketed[begin + i] is a target of
// order k (j >= k) and admitted. The bin range is the slice's, not re-checked,
// so a wrong slice shows up in the host's exact-count check.
extern "C" __global__ void ngram_slice_flags(const u16 *tokens, const u32 *bucketed, u32 begin, u32 count,
                                             u32 k, u32 *flags) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 j = bucketed[begin + i];
    flags[i] = (j >= k && ngram_admitted(tokens, j, k)) ? 1u : 0u;
}

extern "C" __global__ void ngram_slice_scatter(const u16 *tokens, const u32 *bucketed, u32 begin, u32 count,
                                               u32 k, const u32 *scan, u32 out_base, u32 *pos) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    u32 j = bucketed[begin + i];
    if (j >= k && ngram_admitted(tokens, j, k)) pos[out_base + scan[i]] = j;
}

// ---------------------------------------------------------------- radix sort

// claude-430: one gather per slot. Each position's key word is read from the
// stream once, into keys[i]; the slot's passes then read their digits from
// keys in order, and the scatter moves each key with its position.
extern "C" __global__ void ngram_radix_gather(const u16 *tokens, const u32 *pos, u32 n, u32 back, u16 *keys) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    keys[i] = (u16)radix_key(tokens, pos[i], back);
}

// One warp per tile of RADIX_TILE positions; hist is digit-major [64][tiles].
extern "C" __global__ void ngram_radix_hist(const u16 *keys, u32 n, u32 shift, u32 tiles, u32 *hist) {
    __shared__ u32 h[RADIX_WARPS][64];
    u32 warp = threadIdx.x / 32u, lane = threadIdx.x % 32u;
    u32 tile = blockIdx.x * RADIX_WARPS + warp;
    h[warp][lane] = 0;
    h[warp][lane + 32u] = 0;
    __syncwarp();
    if (tile >= tiles) return;
    u32 begin = tile * RADIX_TILE;
    u32 end = min(begin + RADIX_TILE, n);
    for (u32 base = begin; base < end; base += 32u) {
        u32 i = base + lane;
        bool valid = i < end;
        u32 d = valid ? ((u32)keys[i] >> shift) & 63u : 64u;
        u32 peers = __match_any_sync(FULL_MASK, d);
        if (valid && lane == (u32)(__ffs(peers) - 1)) h[warp][d] += __popc(peers);
        __syncwarp();
    }
    hist[lane * tiles + tile] = h[warp][lane];
    hist[(lane + 32u) * tiles + tile] = h[warp][lane + 32u];
}

// Stable scatter: within a round, rank = peers with a lower lane; rounds and
// tiles keep stream order, and offsets come from the digit-major scan.
extern "C" __global__ void ngram_radix_scatter(const u16 *keys_in, u16 *keys_out, const u32 *pos_in, u32 *pos_out,
                                               u32 n, u32 shift, u32 tiles, const u32 *offsets) {
    __shared__ u32 base_of[RADIX_WARPS][64];
    u32 warp = threadIdx.x / 32u, lane = threadIdx.x % 32u;
    u32 tile = blockIdx.x * RADIX_WARPS + warp;
    if (tile >= tiles) return;
    base_of[warp][lane] = offsets[lane * tiles + tile];
    base_of[warp][lane + 32u] = offsets[(lane + 32u) * tiles + tile];
    __syncwarp();
    u32 begin = tile * RADIX_TILE;
    u32 end = min(begin + RADIX_TILE, n);
    for (u32 base = begin; base < end; base += 32u) {
        u32 i = base + lane;
        bool valid = i < end;
        u32 p = valid ? pos_in[i] : 0u;
        u16 key = valid ? keys_in[i] : (u16)0;
        u32 d = valid ? ((u32)key >> shift) & 63u : 64u;
        u32 peers = __match_any_sync(FULL_MASK, d);
        u32 below = peers & ((1u << lane) - 1u);
        u32 slot = valid ? base_of[warp][d] + __popc(below) : 0u;
        __syncwarp();
        if (valid) {
            pos_out[slot] = p;
            keys_out[slot] = key;
            if (below == 0u) base_of[warp][d] += __popc(peers);
        }
        __syncwarp();
    }
}

// ---------------------------------------------------------------- scans

template <typename T> __device__ T block_exclusive_scan(T value, T *block_total) {
    __shared__ T warp_sums[SCAN_THREADS / 32u];
    u32 lane = threadIdx.x % 32u, warp = threadIdx.x / 32u;
    T x = value;
    for (u32 o = 1; o < 32u; o <<= 1) {
        T y = __shfl_up_sync(FULL_MASK, x, o);
        if (lane >= o) x += y;
    }
    if (lane == 31u) warp_sums[warp] = x;
    __syncthreads();
    if (warp == 0) {
        T w = lane < SCAN_THREADS / 32u ? warp_sums[lane] : (T)0;
        for (u32 o = 1; o < SCAN_THREADS / 32u; o <<= 1) {
            T y = __shfl_up_sync(FULL_MASK, w, o);
            if (lane >= o) w += y;
        }
        if (lane < SCAN_THREADS / 32u) warp_sums[lane] = w;
    }
    __syncthreads();
    T prefix = warp ? warp_sums[warp - 1u] : (T)0;
    *block_total = warp_sums[SCAN_THREADS / 32u - 1u];
    __syncthreads();
    return prefix + x - value;
}

template <typename T> __device__ void scan_reduce(const T *in, u32 n, T *sums) {
    u32 begin = blockIdx.x * SCAN_BLOCK + threadIdx.x * SCAN_ITEMS;
    T acc = 0;
    for (u32 k = 0; k < SCAN_ITEMS; ++k)
        if (begin + k < n) acc += in[begin + k];
    T total;
    block_exclusive_scan<T>(acc, &total);
    if (threadIdx.x == 0) sums[blockIdx.x] = total;
}

// One block: exclusive scan of `count` block sums in place; grand total out.
template <typename T> __device__ void scan_sums(T *sums, u32 count, T *total) {
    u32 per = (count + SCAN_THREADS - 1u) / SCAN_THREADS;
    u32 begin = threadIdx.x * per;
    T acc = 0;
    for (u32 k = 0; k < per; ++k)
        if (begin + k < count) acc += sums[begin + k];
    T all;
    T prefix = block_exclusive_scan<T>(acc, &all);
    for (u32 k = 0; k < per; ++k)
        if (begin + k < count) {
            T v = sums[begin + k];
            sums[begin + k] = prefix;
            prefix += v;
        }
    if (threadIdx.x == 0) *total = all;
}

// In place: data[i] = sum of data[0..i), data[n] = total. Each thread reads its words
// before it writes them, and no other thread reads them in this kernel.
template <typename T> __device__ void scan_apply(T *data, u32 n, const T *sums, const T *total) {
    u32 begin = blockIdx.x * SCAN_BLOCK + threadIdx.x * SCAN_ITEMS;
    T v[SCAN_ITEMS];
    T acc = 0;
    for (u32 k = 0; k < SCAN_ITEMS; ++k) {
        v[k] = begin + k < n ? data[begin + k] : (T)0;
        acc += v[k];
    }
    T ignored;
    T prefix = sums[blockIdx.x] + block_exclusive_scan<T>(acc, &ignored);
    for (u32 k = 0; k < SCAN_ITEMS; ++k)
        if (begin + k < n) {
            data[begin + k] = prefix;
            prefix += v[k];
        }
    if (blockIdx.x == 0 && threadIdx.x == 0) data[n] = *total;
}

extern "C" __global__ void ngram_scan32_reduce(const u32 *in, u32 n, u32 *sums) { scan_reduce<u32>(in, n, sums); }
extern "C" __global__ void ngram_scan32_sums(u32 *sums, u32 count, u32 *total) { scan_sums<u32>(sums, count, total); }
extern "C" __global__ void ngram_scan32_apply(u32 *data, u32 n, const u32 *sums, const u32 *total) {
    scan_apply<u32>(data, n, sums, total);
}
extern "C" __global__ void ngram_scan64_reduce(const u64 *in, u32 n, u64 *sums) { scan_reduce<u64>(in, n, sums); }
extern "C" __global__ void ngram_scan64_sums(u64 *sums, u32 count, u64 *total) { scan_sums<u64>(sums, count, total); }
extern "C" __global__ void ngram_scan64_apply(u64 *data, u32 n, const u64 *sums, const u64 *total) {
    scan_apply<u64>(data, n, sums, total);
}

// ---------------------------------------------------------------- reduce

// gram_flag[r]: the gram at sorted r differs from r-1; context_flag[r]: its context does.
// With `left` nonzero (claude-429), also left_flag[r] = 1 where r's left digit
// of `order` is nonzero and differs from r-1's in the same gram. The sort put
// the left digit least significant, so equal digits of one gram are adjacent
// and the flags of a gram's run count its distinct left tokens, N1+(. gram).
// With `left` zero, left_flag is not touched.
extern "C" __global__ void ngram_boundaries(const u16 *tokens, const u32 *sorted, u32 n, u32 order, u32 left,
                                            u32 *gram_flag, u32 *context_flag, u32 *left_flag) {
    u32 r = blockIdx.x * blockDim.x + threadIdx.x;
    if (r >= n) return;
    if (r == 0) {
        gram_flag[0] = 1u;
        context_flag[0] = 1u;
        if (left) left_flag[0] = left_digit(tokens, sorted[0], order) != 0u ? 1u : 0u;
        return;
    }
    u32 a = sorted[r], b = sorted[r - 1];
    u32 context_differs = 0;
    for (u32 s = 1; s <= order; ++s) context_differs |= (u32)(tokens[a - s] != tokens[b - s]);
    context_flag[r] = context_differs;
    u32 gram_differs = context_differs | (u32)(tokens[a] != tokens[b]);
    gram_flag[r] = gram_differs;
    if (left) {
        u32 l = left_digit(tokens, a, order);
        left_flag[r] = (l != 0u && (gram_differs || l != left_digit(tokens, b, order))) ? 1u : 0u;
    }
}

// starts[index[r]] = r wherever flag[r]; starts[total] = n.
extern "C" __global__ void ngram_run_starts(const u32 *flag, const u32 *index, u32 n, u32 *starts) {
    u32 r = blockIdx.x * blockDim.x + threadIdx.x;
    if (r < n && flag[r]) starts[index[r]] = r;
    if (r == 0) starts[index[n]] = n;
}

// One gram: its key (lo, hi words), next token, count, context-start flag and
// the packed indicators (count==1 | count==2 << 32), (count==3 | count==4 << 32).
// With `adjust` nonzero (claude-429, order 1 <= k < N), the count is the
// modified Kneser-Ney adjusted count: N1+(. gram) from left_scan (the
// exclusive scan of ngram_boundaries' left flags, n + 1 words), or the raw
// count when the gram's oldest slot t[j-k] is BOS. violations[0], [1] count
// the grams that break rule 4 (not a document start, N1+ = 0) and the
// document starts with N1+ != 0; violations[2], [3] hold the least such gram
// index of each (the host sets them to 0xffffffff). With `adjust` zero,
// left_scan and violations are not touched.
extern "C" __global__ void ngram_emit_grams(const u16 *tokens, const u32 *sorted, const u32 *gram_start,
                                            const u32 *context_flag, const u32 *left_scan, u32 grams,
                                            u32 order, u32 adjust, u64 *keys, u16 *next, u32 *counts,
                                            u32 *gram_context_flag, u64 *ind12, u64 *ind34,
                                            u32 *violations) {
    u32 g = blockIdx.x * blockDim.x + threadIdx.x;
    if (g >= grams) return;
    u32 r0 = gram_start[g], r1 = gram_start[g + 1u];
    u32 j = sorted[r0];
    u32 count = r1 - r0;
    if (adjust) {
        u32 types = left_scan[r1] - left_scan[r0];
        if (tokens[j - order] == 0u) {
            if (types != 0u) {
                atomicAdd(&violations[1], 1u);
                atomicMin(&violations[3], g);
            }
        } else {
            if (types == 0u) {
                atomicAdd(&violations[0], 1u);
                atomicMin(&violations[2], g);
            }
            count = types;
        }
    }
    u64 lo = 0, hi = (u64)order << 56;
    for (u32 s = 1; s <= order; ++s) {
        u64 v = tokens[j - s];
        u32 bit = 108u - 12u * (s - 1u);
        if (bit >= 64u) {
            hi |= v << (bit - 64u);
        } else {
            lo |= v << bit;
            if (bit + 12u > 64u) hi |= v >> (64u - bit);
        }
    }
    keys[2u * g] = lo;
    keys[2u * g + 1u] = hi;
    next[g] = tokens[j];
    counts[g] = count;
    gram_context_flag[g] = context_flag[r0];
    ind12[g] = (u64)(count == 1u) | ((u64)(count == 2u) << 32);
    ind34[g] = (u64)(count == 3u) | ((u64)(count == 4u) << 32);
}

// One context: [first gram, total count, N1, N2]. Prefix differences only.
extern "C" __global__ void ngram_emit_contexts(const u32 *context_start, const u32 *gram_start,
                                               const u64 *ind12_scan, u32 contexts, u32 *records) {
    u32 c = blockIdx.x * blockDim.x + threadIdx.x;
    if (c >= contexts) return;
    u32 g0 = context_start[c], g1 = context_start[c + 1u];
    u64 d = ind12_scan[g1] - ind12_scan[g0];
    records[4u * c] = g0;
    records[4u * c + 1u] = gram_start[g1] - gram_start[g0];
    records[4u * c + 2u] = (u32)(d & 0xffffffffull);
    records[4u * c + 3u] = (u32)(d >> 32);
}

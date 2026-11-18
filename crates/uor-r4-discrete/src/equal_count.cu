// Equal-byte count instrument. The expanded training histogram is discarded after
// freezing four successors per directly indexed row. Count and score use the
// same causal key generator. No position can see beyond its record's BOS.
typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;
static const u32 V = 4096;
static const u32 KEEP = 4;

__device__ __forceinline__ u64 scramble(u64 x) {
    x = (x ^ (x >> 30)) * 0xbf58476d1ce4e5b9ULL;
    x = (x ^ (x >> 27)) * 0x94d049bb133111ebULL;
    return x ^ (x >> 31);
}

// meta: packed two M1 token-root nibbles, the 64x64 composition table,
// and 2x16 resolved step frames. Both transport arms use the same layout.
__device__ u32 key_row(const u16 *tokens, u32 j, const u8 *meta,
                       u32 arm, u32 k, u32 rows) {
    u32 begin = j;
    while (begin && j - begin < k) {
        --begin;
        if (tokens[begin] == 0) break;
    }
    if (arm <= 1) {
        u32 left = 0, right = 0;
        for (u32 i = begin; i < j; ++i) {
            const u8 roots = meta[tokens[i]];
            const u32 a = meta[8192u + (roots & 15u)];
            const u32 b = meta[8208u + (roots >> 4)];
            left = meta[4096u + a * 64u + left];
            right = meta[4096u + b * 64u + right];
        }
        return scramble((left << 6) | right) % rows;
    }
    // Arm 2 stores a bucket of the ordered suffix (base-4099 rolling key).
    // Arm 3 scrambles the prefix at each step. Both include its actual length.
    u64 h = 1u;
    for (u32 i = begin; i < j; ++i) {
        if (arm == 2) h = h * 4099u + tokens[i] + 1u;
        else h = scramble(h + 0x9e3779b97f4a7c15ULL + tokens[i]);
    }
    return scramble(h ^ ((u64)(j-begin) << 56)) % rows;
}

extern "C" __global__ void equal_count_train(const u16 *tokens, u32 n, const u8 *meta,
        u32 arm, u32 k, u32 rows, u32 *hist, u32 *uni) {
    u32 j = blockIdx.x * blockDim.x + threadIdx.x;
    if (j >= n || tokens[j] == 0) return;
    u32 row = key_row(tokens, j, meta, arm, k, rows);
    atomicAdd(hist + row * V + tokens[j], 1u);
    atomicAdd(uni + tokens[j], 1u);
}

extern "C" __global__ void equal_count_freeze(const u32 *hist, u32 rows, u32 *table) {
    u32 row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    u32 ids[KEEP] = {}, counts[KEEP] = {};
    for (u32 token = 0; token < V; ++token) {
        u32 count = hist[row * V + token];
        if (count <= counts[KEEP-1]) continue;
        u32 at = KEEP-1;
        while (at && count > counts[at-1]) {
            ids[at] = ids[at-1]; counts[at] = counts[at-1]; --at;
        }
        ids[at] = token; counts[at] = count;
    }
    for (u32 i = 0; i < KEEP; ++i) {
        table[(row * KEEP + i)*2] = ids[i];
        table[(row * KEEP + i)*2+1] = counts[i];
    }
}

extern "C" __global__ void equal_count_score(const u16 *tokens, u32 n, const u8 *meta,
        u32 arm, u32 k, u32 rows, const u32 *table, const u32 *uni, u32 train_targets,
        u64 *masses) {
    u32 j = blockIdx.x * blockDim.x + threadIdx.x;
    if (j >= n || tokens[j] == 0) return;
    u32 t = tokens[j], row = key_row(tokens, j, meta, arm, k, rows);
    u32 hit = 0, total = 0;
    for (u32 i = 0; i < KEEP; ++i) {
        u32 index = (row * KEEP + i) * 2;
        u32 count = table[index+1];
        total += count;
        if (table[index] == t) hit += count;
    }
    u64 base = (((u64)uni[t]+1) << 32) / (train_targets + V);
    u64 mass = total ? (base + (((u64)hit << 32) / total)) / 2u : base;
    masses[j] = mass << 16; // Q32 -> Q48 for the project's PRLL bit instrument
}

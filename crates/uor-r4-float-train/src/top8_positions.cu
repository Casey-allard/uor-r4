// Exact bounded rival selection for the training objective. One warp owns a row; every lane
// scans a strided subset into a sorted top-k, and lane 0 merges the warp's candidates. The order
// is score descending and token id ascending on ties. Non-tied choices match repeated argmax;
// ties are deterministic here where the old CUDA reduction does not define their order.
//
// This module is separate from `cuda_stack.cu`, whose SHA-256 is part of checkpoint fingerprints.

typedef unsigned int uint;
typedef unsigned long long u64;

__device__ __forceinline__ void top8_insert(
    float* scores, uint* indices, uint& n, uint k, float score, uint index
) {
    if (n < k) {
        uint j = n++;
        while (j > 0 && (scores[j - 1] < score ||
                         (scores[j - 1] == score && indices[j - 1] > index))) {
            scores[j] = scores[j - 1];
            indices[j] = indices[j - 1];
            --j;
        }
        scores[j] = score;
        indices[j] = index;
    } else if (score > scores[k - 1] ||
               (score == scores[k - 1] && index < indices[k - 1])) {
        uint j = k - 1;
        while (j > 0 && (scores[j - 1] < score ||
                         (scores[j - 1] == score && indices[j - 1] > index))) {
            scores[j] = scores[j - 1];
            indices[j] = indices[j - 1];
            --j;
        }
        scores[j] = score;
        indices[j] = index;
    }
}

extern "C" __global__ void top8_positions(
    const float* scores, const unsigned char* allowed, uint* out, uint vocab, uint k
) {
    if (k == 0 || k > 8) return;
    __shared__ float shared_scores[256];
    __shared__ uint shared_indices[256];
    uint row = blockIdx.x;
    uint lane = threadIdx.x & 31;
    u64 offset = (u64)row * vocab;
    float local_scores[8];
    uint local_indices[8];
    uint n = 0;
    for (uint i = lane; i < vocab; i += 32) {
        if (allowed[offset + i] != 0) {
            top8_insert(local_scores, local_indices, n, k, scores[offset + i], i);
        }
    }
    uint base = lane * 8;
    #pragma unroll
    for (uint j = 0; j < 8; ++j) {
        shared_scores[base + j] = j < n ? local_scores[j] : -3.402823466e38f;
        shared_indices[base + j] = j < n ? local_indices[j] : 0xffffffffu;
    }
    __syncwarp();
    if (lane == 0) {
        for (uint s = 0; s < k; ++s) {
            float best = -3.402823466e38f;
            uint best_slot = 0xffffffffu;
            uint best_index = 0xffffffffu;
            for (uint j = 0; j < 256; ++j) {
                uint index = shared_indices[j];
                float score = shared_scores[j];
                if (score > best || (score == best && index < best_index)) {
                    best = score;
                    best_slot = j;
                    best_index = index;
                }
            }
            if (best_slot == 0xffffffffu || best_index == 0xffffffffu) {
                out[(u64)row * k + s] = 0;
            } else {
                out[(u64)row * k + s] = best_index;
                shared_indices[best_slot] = 0xffffffffu;
                shared_scores[best_slot] = -3.402823466e38f;
            }
        }
    }
}

// The nearest objective's fused score-and-select path. One block owns a row. Its eight warps
// each score one table candidate at a time, coalescing the candidate's width, and keep their
// local top-k. Thread 0 merges the eight lists. No (rows, vocabulary) score matrix is written.
extern "C" __global__ void nearest_top8_positions(
    const float* query, const float* table, const uint* targets, uint* out,
    uint vocab, uint width, uint k
) {
    if (k == 0 || k > 8) return;
    __shared__ float query_row[1024];
    __shared__ float lane_scores[64];
    __shared__ uint lane_indices[64];
    uint row = blockIdx.x;
    uint lane = threadIdx.x & 31;
    uint warp = threadIdx.x >> 5;
    uint warps = blockDim.x >> 5;
    for (uint x = threadIdx.x; x < width; x += blockDim.x) {
        query_row[x] = query[(u64)row * width + x];
    }
    __syncthreads();
    float local_scores[8];
    uint local_indices[8];
    uint n = 0;
    uint target = targets[row];
    for (uint candidate = warp; candidate < vocab; candidate += warps) {
        if (candidate == target) continue;
        float total = 0.0f;
        u64 table_offset = (u64)candidate * width;
        for (uint x = lane; x < width; x += 32) {
            total = __fadd_rn(total, fabsf(__fsub_rn(query_row[x], table[table_offset + x])));
        }
        for (int offset = 16; offset > 0; offset >>= 1) {
            total = __fadd_rn(total, __shfl_down_sync(0xffffffffu, total, offset));
        }
        if (lane == 0) {
            float score = __fdiv_rn(-total, (float)width);
            top8_insert(local_scores, local_indices, n, k, score, candidate);
        }
    }
    if (lane == 0) {
        uint base = warp * 8;
        #pragma unroll
        for (uint j = 0; j < 8; ++j) {
            lane_scores[base + j] =
                j < n ? local_scores[j] : -3.402823466e38f;
            lane_indices[base + j] = j < n ? local_indices[j] : 0xffffffffu;
        }
    }
    __syncthreads();
    if (threadIdx.x == 0) {
        uint total_candidates = warps * 8;
        for (uint s = 0; s < k; ++s) {
            float best = -3.402823466e38f;
            uint best_slot = 0xffffffffu;
            uint best_index = 0xffffffffu;
            for (uint j = 0; j < total_candidates; ++j) {
                uint index = lane_indices[j];
                float score = lane_scores[j];
                if (score > best || (score == best && index < best_index)) {
                    best = score;
                    best_slot = j;
                    best_index = index;
                }
            }
            if (best_slot == 0xffffffffu || best_index == 0xffffffffu) {
                out[(u64)row * k + s] = 0;
            } else {
                out[(u64)row * k + s] = best_index;
                lane_scores[best_slot] = -3.402823466e38f;
                lane_indices[best_slot] = 0xffffffffu;
            }
        }
    }
}

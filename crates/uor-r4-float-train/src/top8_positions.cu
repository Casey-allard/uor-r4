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

#include <cuda_runtime.h>
#include <stdint.h>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <vector>
#include "abi.h"

#define D11_HD __device__
#define D11_CONST __device__ __constant__
#include "kernel_core.h"

constexpr uint32_t NONE = UINT32_MAX;
constexpr uint32_t NO_RADIUS = 65535;
constexpr uint32_t MAX_COST = 1280;

__device__ bool stop(uint32_t id) {
    return id == 16 || id == 1 || id == 0 || id == 201;
}

__device__ uint32_t distance(uint32_t left, uint32_t right) {
    uint32_t total = 0;
    for (uint32_t block = 0; block < 4; ++block) {
        auto a = D11_ROOTS[(left >> (4 * block)) & 15];
        auto b = D11_ROOTS[(right >> (4 * block)) & 15];
        for (int axis = 0; axis < 8; ++axis) total += d11_abs_diff(a[axis], b[axis]);
    }
    return total;
}

__device__ uint32_t bank_root(const uint32_t* row, uint32_t slot) {
    return (row[3 + slot / 2] >> (16 * (slot % 2))) & 65535;
}

__device__ void canonical(uint32_t* packed, uint32_t signature, uint32_t depth, uint32_t slots) {
    for (uint32_t at = 0; at < slots; ++at) {
        uint32_t cls = (signature >> (2*at)) & 3;
        if (!cls) continue;
        uint32_t end = at < depth ? depth : slots;
        for (uint32_t next = at+1; next < end; ++next) {
            if (((signature >> (2*next)) & 3) == cls && packed[next] < packed[at]) {
                uint32_t temp = packed[at];
                packed[at] = packed[next];
                packed[next] = temp;
            }
        }
    }
}

__device__ bool valid_bank(const uint32_t* row, uint32_t used, bool unordered) {
    if (used > CAPACITY) return false;
    for (uint32_t i = 0; i < CAPACITY; ++i) {
        auto entry = row + i * RECORD_WORDS;
        if (i >= used) {
            for (uint32_t j = 0; j < RECORD_WORDS; ++j) if (entry[j]) return false;
            continue;
        }
        uint32_t left = entry[0] & 255, right = (entry[0] >> 8) & 255;
        if (left < 1 || left > 3 || right < 1 || right > 7 || (entry[0] >> 16) > 1
            || (entry[2] != NO_RADIUS && entry[2] > MAX_COST)
            || (entry[1] >> (2 * (left + right))) || ((entry[1] >> (2 * left)) & 3) != 1) return false;
        for (uint32_t k = 0; k < left + right; ++k) {
            uint32_t cls = (entry[1] >> (2*k)) & 3;
            if (cls > 2 || (k > left && cls != 2)) return false;
        }
        for (uint32_t k = left + right; k < MAX_SLOTS; ++k) if (bank_root(entry, k)) return false;
        if (unordered) {
            uint32_t canonical_geometry[MAX_SLOTS];
            for (uint32_t j=0; j<left+right; ++j) canonical_geometry[j]=bank_root(entry,j);
            canonical(canonical_geometry,entry[1],left,left+right);
            for (uint32_t j=0; j<left+right; ++j) if (canonical_geometry[j]!=bank_root(entry,j)) return false;
        }
        for (uint32_t j = 0; j < i; ++j) {
            auto previous = row + j * RECORD_WORDS;
            if ((entry[0]&65535) != (previous[0]&65535) || entry[1] != previous[1]) continue;
            if (entry[2] != previous[2]) return false;
            if ((entry[0] >> 16) == (previous[0] >> 16)) continue;
            bool same = true;
            for (uint32_t k = 3; k < RECORD_WORDS; ++k) same &= entry[k] == previous[k];
            if (same) return false;
        }
    }
    return true;
}

__device__ int compare(int n, int d, int m, int e) {
    return n * e - m * d;
}

__global__ void score_rows(const uint32_t* input, uint32_t* traces, uint32_t nrows) {
    uint32_t at = blockIdx.x * blockDim.x + threadIdx.x;
    if (at >= nrows) return;
    auto row = input + at * ROW_WORDS;
    auto out = traces + at * TRACE_WORDS;
    for (int i = 0; i < TRACE_WORDS; ++i) out[i] = NONE;
    out[0] = 5;
    out[7] = 0;
    uint32_t len = row[0], fact = row[1], question = row[2], fence = row[3];
    uint32_t mask = row[4] & row[5];
    out[1] = mask;
    if (len > MAX_TAPE || !fact || !(fact < fence && fence < question && question < len)
        || (row[4] | row[5]) >> 3 || row[7] > 1 || row[TOKENS] != 0) return;
    for (uint32_t i = 0; i < MAX_TAPE; ++i) {
        if (i < len) {
            if (row[TOKENS+i] >= 4096 || row[CLASSES+i] > 2 || row[ROOTS+i] > 65535) return;
        } else if (row[TOKENS+i] || row[CLASSES+i] || row[ROOTS+i]) return;
    }
    uint32_t first_stop = fact + 1;
    while (first_stop < question && !stop(row[TOKENS+first_stop])) ++first_stop;
    if (first_stop != fence || !valid_bank(row+BANK,row[6],row[7]==1)) return;
    for (uint32_t k = 0; k < 3; ++k) {
        if (!(mask & (1u << k))) continue;
        uint32_t source = fact + 2 + k;
        if (source >= fence || row[CLASSES+source] != 1) return;
    }
    int best_num = 0, best_den = 0, next_num = 0, next_den = 0;
    uint32_t best_source = NONE;
    bool unseen = false;
    for (uint32_t k = 0; k < 3; ++k) {
        if (!(mask & (1u << k))) continue;
        auto c = out + TRACE_HEAD + k*CAND_WORDS;
        uint32_t source = fact + 2 + k, end = source + 1;
        while (end < fence && row[CLASSES+end] == 2) ++end;
        c[0] = source; c[1] = end; c[2] = 1;
        if (fence - source >= 8) continue;
        uint32_t depth = k+1, word = end-source, slots = depth+word;
        uint32_t shape = depth | (word<<8), sig = 0;
        c[2] = 2;
        c[3] = shape;
        for (uint32_t j = 0; j < slots; ++j) {
            sig |= row[CLASSES+fact+1+j] << (2*j);
            c[10+j] = row[ROOTS+fact+1+j];
        }
        c[4] = sig;
        if (row[7]==1) canonical(c+10,sig,depth,slots);
        uint32_t minima[] = {NONE,NONE}, radius = NO_RADIUS;
        for (uint32_t j = 0; j < row[6]; ++j) {
            auto e = row+BANK+j*RECORD_WORDS;
            if ((e[0]&65535) != shape || e[1] != sig) continue;
            uint32_t total = 0;
            for (uint32_t s = 0; s < slots; ++s) total += distance(c[10+s], bank_root(e,s));
            uint32_t cls = e[0]>>16;
            if (total < minima[cls]) minima[cls] = total;
            radius = e[2];
        }
        c[5] = minima[0]; c[6] = minima[1]; c[7] = radius;
        if (minima[0] == NONE || minima[1] == NONE || radius == NO_RADIUS || minima[0] > radius) {
            unseen = true;
            continue;
        }
        int num = int(minima[1]) - int(minima[0]), den = int(slots);
        c[2] = 3; c[8] = uint32_t(num); c[9] = uint32_t(den);
        if (!best_den || compare(num,den,best_num,best_den) > 0) {
            next_num = best_num; next_den = best_den;
            best_num = num; best_den = den; best_source = source;
        } else if (!next_den || compare(num,den,next_num,next_den) > 0) {
            next_num = num; next_den = den;
        }
    }
    if (best_den) { out[3] = uint32_t(best_num); out[4] = uint32_t(best_den); }
    if (next_den) { out[5] = uint32_t(next_num); out[6] = uint32_t(next_den); }
    out[0] = unseen ? 2 : !best_den ? 1 : best_num <= 0 ? 4 :
             (next_den && !compare(best_num,best_den,next_num,next_den)) ? 3 : 0;
    if (out[0]) return;
    out[2] = best_source;
    uint32_t state[] = {1,best_source,best_source,len,16,1,0,201};
    for (uint32_t step = 0; step < 8; ++step) {
        state[4] = 16; state[5] = 1; state[6] = 0; state[7] = 201;
        d11_exact_copy_write(row+TOKENS,NONE,state);
        // The unchanged condition from completion.cu, on this single writer step.
        if (state[7] == 1 && state[2] > state[1]) state[4] = 1;
        for (uint32_t j = 0; j < 8; ++j) out[WRITER+8*step+j] = state[j];
        out[7] = step+1;
        if (state[7]) return;
    }
}

static void checked(cudaError_t result) {
    if (result == cudaSuccess) return;
    std::fprintf(stderr,"%s\n",cudaGetErrorString(result));
    std::exit(2);
}

int main(int argc, char** argv) {
    if (argc != 3) return 2;
    std::ifstream in(argv[1],std::ios::binary);
    uint32_t head[2] = {};
    if (!in.read(reinterpret_cast<char*>(head),sizeof(head)) || head[0] != ABI_MAGIC || !head[1] || head[1] > 16) return 3;
    std::vector<uint32_t> rows(head[1]*ROW_WORDS), out(head[1]*TRACE_WORDS);
    if (!in.read(reinterpret_cast<char*>(rows.data()),rows.size()*sizeof(uint32_t)) || in.peek()!=EOF) return 3;
    uint32_t *device_in = nullptr, *device_out = nullptr;
    checked(cudaMalloc(&device_in,rows.size()*sizeof(uint32_t)));
    checked(cudaMalloc(&device_out,out.size()*sizeof(uint32_t)));
    checked(cudaMemcpy(device_in,rows.data(),rows.size()*sizeof(uint32_t),cudaMemcpyHostToDevice));
    score_rows<<<1,32>>>(device_in,device_out,head[1]);
    checked(cudaGetLastError());
    checked(cudaMemcpy(out.data(),device_out,out.size()*sizeof(uint32_t),cudaMemcpyDeviceToHost));
    checked(cudaFree(device_in));
    checked(cudaFree(device_out));
    std::ofstream file(argv[2],std::ios::binary);
    head[0] = TRACE_MAGIC;
    file.write(reinterpret_cast<char*>(head),sizeof(head));
    file.write(reinterpret_cast<char*>(out.data()),out.size()*sizeof(uint32_t));
    return file ? 0 : 4;
}

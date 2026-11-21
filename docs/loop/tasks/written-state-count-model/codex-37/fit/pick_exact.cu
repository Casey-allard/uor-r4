// Exact pair Pick over the original causal UORT prefix.
#include <cuda_runtime.h>
#include <array>
#include <cstdint>
#include <cstdio>
#include <stdexcept>
#include <string>
#include <vector>

constexpr uint32_t missing = UINT32_MAX;
constexpr uint32_t context = 192;
constexpr uint64_t id_limit = 32ULL * 1024 * 1024;

struct Prefix {
    uint32_t start;
    uint32_t end; // exclusive, so the target at end is never read by the kernel
};

#define CUDA(operation) do { \
    cudaError_t code = (operation); \
    if (code != cudaSuccess) throw std::runtime_error(cudaGetErrorString(code)); \
} while (0)

__global__ void exact_pairs(const uint16_t *ids, const Prefix *prefixes, uint32_t *picks,
                            uint32_t rows) {
    const uint32_t row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= rows) return;
    const Prefix w = prefixes[row];
    uint32_t picked = missing;
    if (w.end - w.start >= 3) {
        const uint32_t t = w.end - 1;
        const uint16_t left = ids[t - 1];
        const uint16_t right = ids[t];
        for (uint32_t p = t - 1; p > w.start; --p) {
            if (ids[p - 1] == left && ids[p] == right) {
                picked = p - w.start;
                break;
            }
        }
    }
    picks[row] = picked;
}

int main(int argc, const char **argv) {
    try {
        if (argc != 3) throw std::runtime_error("usage: d37-c2-device UORT NEW-PICK-TAPE");
        FILE *input = fopen(argv[1], "rb");
        if (!input) throw std::runtime_error("cannot open UORT input");
        std::array<unsigned char, 64> header;
        if (fread(header.data(), header.size(), 1, input) != 1 ||
            std::string(reinterpret_cast<char *>(header.data()), 8) != std::string("UORT\1\0\0\0", 8) ||
            header[16] != 0 || header[17] != 16 || header[18] != 0 || header[19] != 0) {
            throw std::runtime_error("wrong UORT v1 header");
        }
        uint64_t number = 0;
        for (uint32_t i = 0; i < 8; ++i) number |= uint64_t(header[8 + i]) << (8 * i);
        if (number == 0 || number > id_limit) throw std::runtime_error("invalid UORT count");
        std::vector<uint16_t> ids(number);
        if (fread(ids.data(), sizeof(uint16_t), ids.size(), input) != ids.size() ||
            fgetc(input) != EOF || ferror(input) || fclose(input) != 0) {
            throw std::runtime_error("invalid UORT payload length");
        }
        std::vector<Prefix> prefixes;
        prefixes.reserve(ids.size());
        uint32_t bos = 0;
        uint16_t previous = 1;
        for (uint32_t at = 0; at < ids.size(); ++at) {
            const uint16_t id = ids[at];
            if (id >= 4096 || (id == 0) != (previous == 1))
                throw std::runtime_error("invalid UORT frame at index " + std::to_string(at));
            if (id == 0) {
                bos = at;
            } else {
                const uint32_t oldest = at > context ? at - context : 0;
                prefixes.push_back({oldest > bos ? oldest : bos, at});
            }
            previous = id;
        }
        if (previous != 1 || prefixes.empty())
            throw std::runtime_error("UORT must end at EOS and include targets");
        const uint64_t rows = prefixes.size();
        uint16_t *on_ids;
        Prefix *on_prefixes;
        uint32_t *on_picks;
        std::vector<uint32_t> picks(rows);
        CUDA(cudaMalloc(&on_ids, ids.size() * sizeof(uint16_t)));
        CUDA(cudaMalloc(&on_prefixes, rows * sizeof(Prefix)));
        CUDA(cudaMalloc(&on_picks, rows * sizeof(uint32_t)));
        CUDA(cudaMemcpy(on_ids, ids.data(), ids.size() * sizeof(uint16_t), cudaMemcpyHostToDevice));
        CUDA(cudaMemcpy(on_prefixes, prefixes.data(), rows * sizeof(Prefix), cudaMemcpyHostToDevice));
        exact_pairs<<<(rows + 255) / 256, 256>>>(on_ids, on_prefixes, on_picks, uint32_t(rows));
        CUDA(cudaGetLastError());
        CUDA(cudaMemcpy(picks.data(), on_picks, rows * sizeof(uint32_t), cudaMemcpyDeviceToHost));
        CUDA(cudaFree(on_picks));
        CUDA(cudaFree(on_prefixes));
        CUDA(cudaFree(on_ids));
        FILE *out = fopen(argv[2], "wbx");
        if (!out || fwrite("R37C2010", 8, 1, out) != 1 || fwrite(&rows, 8, 1, out) != 1 ||
            fwrite(picks.data(), sizeof(uint32_t), picks.size(), out) != picks.size() || fclose(out) != 0) {
            throw std::runtime_error("could not create C2 result tape");
        }
        return 0;
    } catch (const std::exception &error) {
        fprintf(stderr, "%s\n", error.what());
        return 1;
    }
}

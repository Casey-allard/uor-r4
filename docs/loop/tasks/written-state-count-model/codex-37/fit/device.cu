// D31 integer CeilingTable EM with the D37 three- and five-class shapes.
// Each row is bucket plus the fourteen Q48 masses. Updates and score sums
// are exact, deterministic, and remain on the device for all 96 passes.
#include <cuda_runtime.h>
#include <stdint.h>
#include <cstdio>
#include <stdexcept>
#include <string>
#include <vector>

using Word = unsigned long long;
using Wide = unsigned __int128;
constexpr int experts = 14, base_buckets = 176;
constexpr Word one = 1ULL << 32, mass_one = 1ULL << 48;
constexpr Word row_limit = 1ULL << 24;
constexpr unsigned passes = 96;
struct Row { Word bucket, mass[experts]; };

#define CUDA(operation) do { \
    cudaError_t code = (operation); \
    if (code != cudaSuccess) throw std::runtime_error(cudaGetErrorString(code)); \
} while (0)

__device__ bool live(unsigned bucket, unsigned expert) {
    if (expert == 13) {
#ifdef D37_MASK_PICK
        return false;
#else
        return bucket >= base_buckets;
#endif
    }
    return expert == 0 || expert == 12 || expert <= 1 + (bucket % base_buckets) / 16;
}

// floor(x*2^32/d) for 0 <= x <= d <= 2^80. Retain the top 31 denominator
// bits to get an integer upper estimate, then correct against the full
// width. For a shifted denominator the estimate is at most four too high.
// Restoring division keeps both the source and loaded backend integer:
// ptxas may lower CUDA's / operator through a float reciprocal.
__device__ Word fraction(Wide x, Wide d) {
    if (x == 0) return 0;
    if (x == d) return one;
    Word top = Word(d >> 64);
    unsigned width = top ? 128 - __clzll(top) : 64 - __clzll(Word(d));
    unsigned shift = width > 31 ? width - 31 : 0;
    Wide numerator = x << 32;
    Word remainder = Word(numerator >> shift);
    Word trial = Word(d >> shift) << 32;
    Word guess = 0;
    #pragma unroll 1
    for (unsigned i = 0; i <= 32; ++i) {
        const bool take = remainder >= trial;
        guess = (guess << 1) | Word(take);
        remainder -= take ? trial : 0;
        trial >>= 1;
    }
    while (Wide(guess) * d > numerator) --guess;
    return guess;
}

__device__ Word bits_q12(Word mass) {
    unsigned exponent = 63 - __clzll(mass);
    Word normalized = mass << (63 - exponent);
    Word fraction = 0;
    for (unsigned i = 0; i < 12; ++i) {
        Word high = __umul64hi(normalized, normalized);
        fraction = (fraction << 1) | (high >> 63);
        normalized = high >> 63 ? high : ((high << 1) | (normalized * normalized >> 63));
    }
    return ((48 - exponent) << 12) - fraction;
}

__global__ void initialize(Word *weights, unsigned buckets) {
    auto b = blockIdx.x * blockDim.x + threadIdx.x;
    if (b >= buckets) return;
    Word rest = one;
    for (unsigned e = 1; e < experts; ++e) {
        bool present = live(b, e);
        weights[b * experts + e] = present;
        rest -= present;
    }
    weights[b * experts] = rest;
}

// Reduce bit totals per block before the global add.
// All accumulators fit u64 because at most 2^24 rows contribute <= 2^32 each.
__global__ void accumulate(const Row *rows, Word size, const Word *weights,
                           Word *responsibility, Word *bits, bool update) {
    constexpr unsigned tile = 32;
    __shared__ Word scores[tile];
    Word score = 0;
    for (Word i = blockIdx.x * blockDim.x + threadIdx.x; i < size;
         i += Word(gridDim.x) * blockDim.x) {
        const Row &row = rows[i];
        const Word *w = weights + row.bucket * experts;
        Wide sum = 0;
        Wide component[experts];
        for (unsigned e = 0; e < experts; ++e) {
            component[e] = Wide(w[e]) * row.mass[e];
            sum += component[e];
        }
        Word mass = Word(sum >> 32);
        score += bits_q12(mass ? mass : 1);
        if (update && sum != 0) {
            Word *r = responsibility + row.bucket * experts;
            for (unsigned e = 0; e < experts; ++e) {
                if (component[e] != 0) atomicAdd(r + e, fraction(component[e], sum));
            }
        }
    }
    unsigned lane = threadIdx.x % tile;
    for (unsigned step = tile / 2; step > 0; step /= 2) {
        score += __shfl_down_sync(0xffffffff, score, step);
    }
    if (lane == 0) scores[threadIdx.x / tile] = score;
    __syncthreads();
    if (threadIdx.x == 0) {
        Word total = 0;
        for (unsigned warp = 0; warp < blockDim.x / tile; ++warp) total += scores[warp];
        atomicAdd(bits, total);
    }
}

__device__ unsigned largest(const Word *row) {
    unsigned best = 0;
    for (unsigned e = 1; e < experts; ++e) if (row[e] > row[best]) best = e;
    return best;
}

__global__ void maximize(Word *weights, const Word *responsibility, unsigned buckets) {
    auto b = blockIdx.x * blockDim.x + threadIdx.x;
    if (b >= buckets) return;
    Word *w = weights + b * experts;
    const Word *r = responsibility + b * experts;
    Word total = 0;
    for (unsigned e = 0; e < experts; ++e) total += r[e];
    if (total == 0) return;
    Word assigned = 0;
    for (unsigned e = 0; e < experts; ++e) {
        w[e] = fraction(r[e], total);
        assigned += w[e];
    }
    w[largest(w)] += one - assigned;
    for (unsigned e = 0; e < experts; ++e) {
        if (w[e] == 0 && live(b, e)) {
            --w[largest(w)];
            w[e] = 1;
        }
    }
}

__global__ void probe_mass(const Row *rows, const Word *weights, const Word *ordinals,
                           Word *masses, unsigned count) {
    unsigned index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) return;
    const Row &row = rows[ordinals[index]];
    const Word *w = weights + row.bucket * experts;
    Wide sum = 0;
    for (unsigned e = 0; e < experts; ++e) sum += Wide(w[e]) * row.mass[e];
    Word mass = Word(sum >> 32);
    masses[2 * index] = row.bucket;
    masses[2 * index + 1] = mass ? mass : 1;
}

void probes(const Row *rows, const Word *weights, Word row_count,
            const char *ordinals_path, const char *output_path) {
    FILE *input = fopen(ordinals_path, "rb");
    if (!input) throw std::runtime_error("could not open probe ordinals");
    std::vector<Word> ordinals;
    Word index;
    size_t length;
    while ((length = fread(&index, 1, sizeof(index), input)) == sizeof(index)) {
        if (ordinals.size() >= 1024 || index >= row_count ||
            (!ordinals.empty() && ordinals.back() >= index)) {
            throw std::runtime_error("invalid probe order or bounds");
        }
        ordinals.push_back(index);
    }
    if (length != 0 || ferror(input) || fclose(input) != 0 || ordinals.empty()) {
        throw std::runtime_error("invalid probe ordinal tape length");
    }
    Word *on_device, *masses;
    const size_t bytes = ordinals.size() * sizeof(Word);
    CUDA(cudaMalloc(&on_device, bytes));
    CUDA(cudaMalloc(&masses, bytes * 2));
    CUDA(cudaMemcpy(on_device, ordinals.data(), bytes, cudaMemcpyHostToDevice));
    probe_mass<<<(ordinals.size() + 63) / 64, 64>>>(rows, weights, on_device, masses,
                                                  unsigned(ordinals.size()));
    CUDA(cudaGetLastError());
    std::vector<Word> output(ordinals.size() * 2);
    CUDA(cudaMemcpy(output.data(), masses, bytes * 2, cudaMemcpyDeviceToHost));
    CUDA(cudaFree(masses));
    CUDA(cudaFree(on_device));
    FILE *file = fopen(output_path, "wbx");
    if (!file || fwrite(output.data(), sizeof(Word), output.size(), file) != output.size() ||
        fclose(file) != 0) throw std::runtime_error("could not seal probe masses");
}

int main(int argc, char **argv) {
    try {
        if (argc != 3 && argc != 5) {
            throw std::runtime_error("usage: d37-r-device ROWS NEW-TRACE [ORDINALS NEW-BUCKETS-MASSES]");
        }
        FILE *input = fopen(argv[1], "rb");
        if (!input) throw std::runtime_error("could not open row tape");
        Word count = 0, buckets_word = 0;
        char magic[8];
        if (fread(magic, 8, 1, input) != 1 ||
            std::string(magic, 8) != "R37EM001" ||
            fread(&count, 8, 1, input) != 1 || count == 0 || count > row_limit ||
            fread(&buckets_word, 8, 1, input) != 1 ||
            (buckets_word != 528 && buckets_word != 880)) {
            throw std::runtime_error("invalid row tape header");
        }
        const unsigned buckets = unsigned(buckets_word);
        const unsigned words = experts * buckets;
        const unsigned bucket_blocks = (buckets + 63) / 64;
        std::vector<Row> host(count);
        if (fread(host.data(), sizeof(Row), count, input) != count || fgetc(input) != EOF ||
            ferror(input) || fclose(input) != 0) throw std::runtime_error("invalid row tape length");
        for (Word ordinal = 0; ordinal < count; ++ordinal) {
            const Row &row = host[ordinal];
            if (row.bucket >= buckets || row.mass[0] == 0 || row.mass[12] != (1ULL << 36))
                throw std::runtime_error("invalid bucket or mandatory mass at ordinal " + std::to_string(ordinal));
            for (unsigned e = 0; e < experts; ++e) {
                Word mass = row.mass[e];
                bool inactive = e == 13 ? row.bucket < base_buckets :
                    e > 1 + (row.bucket % base_buckets) / 16 && e < 12;
                if (mass > mass_one || (inactive && mass != 0) ||
                    (e == 13 && mass != 0 && mass != mass_one)) {
                    throw std::runtime_error("invalid expert mass at ordinal " + std::to_string(ordinal));
                }
            }
        }
        Row *rows;
        Word *weights, *response, *bits;
        CUDA(cudaMalloc(&rows, count * sizeof(Row)));
        CUDA(cudaMalloc(&weights, words * sizeof(Word)));
        CUDA(cudaMalloc(&response, words * sizeof(Word)));
        CUDA(cudaMalloc(&bits, sizeof(Word)));
        CUDA(cudaMemcpy(rows, host.data(), count * sizeof(Row), cudaMemcpyHostToDevice));
        std::vector<Row>().swap(host);
        initialize<<<bucket_blocks, 64>>>(weights, buckets);
        CUDA(cudaGetLastError());
        std::vector<Word> trace((passes + 1) * (words + 1));
        const unsigned blocks = count < 262144 ? unsigned((count + 255) / 256) : 1024;
        for (unsigned pass = 0; pass <= passes; ++pass) {
            Word *saved = trace.data() + pass * (words + 1);
            CUDA(cudaMemset(response, 0, words * sizeof(Word)));
            CUDA(cudaMemset(bits, 0, sizeof(Word)));
            accumulate<<<blocks, 256>>>(rows, count, weights, response, bits, pass != passes);
            CUDA(cudaGetLastError());
            CUDA(cudaMemcpy(saved, bits, sizeof(Word), cudaMemcpyDeviceToHost));
            CUDA(cudaMemcpy(saved + 1, weights, words * sizeof(Word), cudaMemcpyDeviceToHost));
            if (pass != passes) maximize<<<bucket_blocks, 64>>>(weights, response, buckets);
            CUDA(cudaGetLastError());
            if (pass % 8 == 0) fprintf(stderr, "pass %u bits %llu\n", pass, saved[0]);
        }
        if (argc == 5) probes(rows, weights, count, argv[3], argv[4]);
        CUDA(cudaFree(bits));
        CUDA(cudaFree(response));
        CUDA(cudaFree(weights));
        CUDA(cudaFree(rows));
        FILE *output = fopen(argv[2], "wbx");
        if (!output || fwrite("R37TR001", 8, 1, output) != 1 ||
            fwrite(&count, 8, 1, output) != 1 ||
            fwrite(&buckets_word, 8, 1, output) != 1 ||
            fwrite(trace.data(), sizeof(Word), trace.size(), output) != trace.size() ||
            fclose(output) != 0) throw std::runtime_error("could not seal new trace");
        return 0;
    } catch (const std::exception &error) {
        fprintf(stderr, "%s\n", error.what());
        return 1;
    }
}

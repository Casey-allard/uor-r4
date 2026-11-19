// Integer-only worker for the P1(c) tangent certificate. The Rust driver
// independently checks the sealed payload before passing staged rows here.
#include <cuda_runtime.h>

#include <algorithm>
#include <array>
#include <cstdint>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

using Word = unsigned long long;
constexpr unsigned Lanes = 13;
constexpr unsigned Buckets = 176;
constexpr unsigned Threads = 128;
constexpr unsigned Fields = Lanes + 2;
constexpr unsigned WeightCount = Buckets * Lanes;
constexpr Word MassOne = 1ULL << 48;

static_assert(sizeof(Word) == 8 && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__);

struct Wide {
    Word lo;
    Word hi;
};

static_assert(sizeof(Wide) == 16);

void check_cuda(cudaError_t status, const char *operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(status));
    }
}
#define CUDA(operation) check_cuda((operation), #operation)

template <typename T> class DeviceBuffer {
  public:
    explicit DeviceBuffer(size_t length) : length(length) {
        CUDA(cudaMalloc(&pointer, length * sizeof(T)));
    }
    ~DeviceBuffer() { cudaFree(pointer); }
    DeviceBuffer(const DeviceBuffer &) = delete;
    DeviceBuffer &operator=(const DeviceBuffer &) = delete;

    void upload(const T *input) {
        CUDA(cudaMemcpy(pointer, input, length * sizeof(T), cudaMemcpyHostToDevice));
    }

    void download(T *output) const {
        CUDA(cudaMemcpy(output, pointer, length * sizeof(T), cudaMemcpyDeviceToHost));
    }

    T *pointer = nullptr;

  private:
    size_t length;
};

__device__ Wide add(Wide a, Wide b, unsigned *error) {
    const Word lo = a.lo + b.lo;
    const Word high = a.hi + b.hi;
    const bool carry = lo < a.lo;
    const Word hi = high + carry;
    if (high < a.hi || hi < high) {
        atomicExch(error, 1);
    }
    return {lo, hi};
}

// The input bounds are 1 <= numerator <= 2^48+1 and 1 <= divisor
// <= 2^48. Start with the integer quotient and append 32 fractional
// bits by exact restoring division. Each doubled remainder is < 2^49.
__device__ Wide directed_ceil_q32(Word numerator, Word divisor, unsigned *error) {
    Word whole = 0;
    Word remainder = numerator;
    if (numerator >= divisor) {
        const int shift = __clzll(divisor) - __clzll(numerator);
        Word aligned = divisor << shift;
        #pragma unroll 1
        for (int bit = shift; bit >= 0; --bit) {
            whole <<= 1;
            if (remainder >= aligned) {
                remainder -= aligned;
                whole |= 1;
            }
            aligned >>= 1;
        }
    }
    Word fraction = 0;
    #pragma unroll 1
    for (unsigned bit = 0; bit < 32; ++bit) {
        remainder <<= 1;
        fraction <<= 1;
        if (remainder >= divisor) {
            remainder -= divisor;
            fraction |= 1;
        }
    }
    const Wide floor = {(whole << 32) | fraction, whole >> 32};
#ifdef KN_CERT_FAULT_FLOOR
    return floor;
#else
    return add(floor, {Word(remainder != 0), 0}, error);
#endif
}

// Frozen lawful_likelihood::bits_q12 and fitter device implementation.
__device__ Word bits_q12(Word mass) {
    const unsigned leading = __clzll(mass);
    Word normalized = mass << leading;
    unsigned fraction = 0;
    #pragma unroll
    for (unsigned round = 0; round < 12; ++round) {
        const Word high = __umul64hi(normalized, normalized);
        const bool upper = (high >> 63) != 0;
        fraction = (fraction << 1) | upper;
        normalized = upper ? high : (high << 1) | ((normalized * normalized) >> 63);
    }
    return (leading - 15) * 4096ULL - fraction;
}

__device__ Wide block_sum(Wide value, Wide *shared, unsigned *error) {
    const unsigned lane = threadIdx.x % 32;
    const unsigned warp = threadIdx.x / 32;
    for (unsigned offset = 16; offset != 0; offset /= 2) {
        const Wide next = {__shfl_down_sync(0xffffffffU, value.lo, offset),
                           __shfl_down_sync(0xffffffffU, value.hi, offset)};
        if (lane < offset) {
            value = add(value, next, error);
        }
    }
    if (lane == 0) {
        shared[warp] = value;
    }
    __syncthreads();
    if (threadIdx.x == 0) {
        for (unsigned w = 1; w < Threads / 32; ++w) {
            value = add(value, shared[w], error);
        }
    }
    __syncthreads();
    return value;
}

struct Chunk {
    unsigned start;
    unsigned length;
    unsigned bucket;
};

__global__ void reduce_chunks(
    const Word *masses, const Word *weights, const Chunk *chunks, Wide *partial,
    unsigned *error, bool include_rowmax
) {
    __shared__ Wide shared[Threads / 32];
    const Chunk chunk = chunks[blockIdx.x];
    const unsigned mask = ((1U << (chunk.bucket / 16 + 2)) - 1) | (1U << (Lanes - 1));
    const Word *anchor = weights + chunk.bucket * Lanes;
    const bool valid = threadIdx.x < chunk.length;
    Word values[Lanes] = {};
    Word rowmax = 1;
    Wide sum{};
    if (valid) {
        const Word *row = masses + size_t(chunk.start + threadIdx.x) * Lanes;
        #pragma unroll
        for (unsigned e = 0; e < Lanes; ++e) {
            values[e] = row[e];
            const Wide product = {anchor[e] * values[e], __umul64hi(anchor[e], values[e])};
            sum = add(sum, product, error);
            if ((mask & (1U << e)) && values[e] > rowmax) {
                rowmax = values[e];
            }
        }
    }
    if (sum.hi > (MassOne >> 32)) {
        atomicExch(error, 1);
    }
    Word mass = (sum.hi << 32) | (sum.lo >> 32);
    if (mass > MassOne) {
        atomicExch(error, 1);
    }
    mass = mass == 0 || mass > MassOne ? 1 : mass;

    Wide *out = partial + blockIdx.x * Fields;
    for (unsigned e = 0; e < Lanes; ++e) {
        const Wide term = valid && (mask & (1U << e))
            ? directed_ceil_q32(values[e] + 1, mass, error) : Wide{};
        const Wide total = block_sum(term, shared, error);
        if (threadIdx.x == 0) {
            out[e] = total;
        }
    }
    const Wide score = block_sum({valid ? bits_q12(mass) : 0, 0}, shared, error);
    const Wide row_bound = block_sum(
        {valid && include_rowmax ? bits_q12(rowmax) : 0, 0}, shared, error
    );
    if (threadIdx.x == 0) {
        out[Lanes] = score;
        out[Lanes + 1] = row_bound;
    }
}

__global__ void reduce_buckets(
    const Wide *partial, const unsigned *offsets, Wide *totals, unsigned *error
) {
    __shared__ Wide shared[Threads / 32];
    const unsigned b = blockIdx.x;
    const unsigned field = blockIdx.y;
    Wide sum{};
    for (unsigned i = offsets[b] + threadIdx.x; i < offsets[b + 1]; i += Threads) {
        sum = add(sum, partial[i * Fields + field], error);
    }
    const Wide total = block_sum(sum, shared, error);
    if (threadIdx.x == 0) {
        totals[b * Fields + field] = total;
    }
}

template <typename T> void read_exact(T *data, size_t count) {
    if (!std::cin.read(reinterpret_cast<char *>(data), count * sizeof(T))) {
        throw std::runtime_error("incomplete worker input");
    }
}

int main() {
    try {
        std::ios::sync_with_stdio(false);
        std::cin.tie(nullptr);
        std::array<unsigned, 2> header{};
        std::array<unsigned, Buckets> counts{};
        read_exact(header.data(), header.size());
        read_exact(counts.data(), counts.size());
        const auto [targets, anchors] = header;
        if (targets == 0 || targets >= (1U << 24) || anchors == 0 || anchors > 3) {
            throw std::runtime_error("worker target or anchor count outside bound");
        }
        std::vector<std::array<Word, WeightCount>> host_weights(anchors);
        for (auto &weights : host_weights) {
            read_exact(weights.data(), weights.size());
        }
        std::vector<Chunk> schedule;
        std::array<unsigned, Buckets + 1> offsets{};
        unsigned start = 0;
        for (unsigned b = 0; b < Buckets; ++b) {
            if (counts[b] > targets - start) {
                throw std::runtime_error("worker bucket count exceeds target census");
            }
            const unsigned end = start + counts[b];
            while (start < end) {
                const unsigned n = std::min(Threads, end - start);
                schedule.push_back({start, n, b});
                start += n;
            }
            offsets[b + 1] = schedule.size();
        }
        if (start != targets) {
            throw std::runtime_error("worker bucket census does not cover all targets");
        }
        DeviceBuffer<Word> masses(size_t(targets) * Lanes);
        {
            std::vector<Word> input(size_t(targets) * Lanes);
            read_exact(input.data(), input.size());
            if (std::cin.peek() != std::char_traits<char>::eof()) {
                throw std::runtime_error("extra bytes after worker mass rows");
            }
            masses.upload(input.data());
        }
        DeviceBuffer<Chunk> chunks(schedule.size());
        chunks.upload(schedule.data());
        DeviceBuffer<unsigned> bucket_offsets(offsets.size());
        bucket_offsets.upload(offsets.data());
        DeviceBuffer<unsigned> error(1);
        DeviceBuffer<Word> weights(WeightCount);
        DeviceBuffer<Wide> partial(schedule.size() * Fields);
        DeviceBuffer<Wide> totals(Buckets * Fields);
        std::array<Wide, Buckets * Fields> output{};
        const unsigned zero = 0;
        for (unsigned a = 0; a < anchors; ++a) {
            error.upload(&zero);
            weights.upload(host_weights[a].data());
            reduce_chunks<<<schedule.size(), Threads>>>(
                masses.pointer, weights.pointer, chunks.pointer, partial.pointer, error.pointer,
                a == 0
            );
            CUDA(cudaGetLastError());
            reduce_buckets<<<dim3(Buckets, Fields), Threads>>>(
                partial.pointer, bucket_offsets.pointer, totals.pointer, error.pointer
            );
            CUDA(cudaGetLastError());
            unsigned overflow = 0;
            error.download(&overflow);
            if (overflow) {
                throw std::runtime_error("device arithmetic range or carry overflow");
            }
            totals.download(output.data());
            if (!std::cout.write(
                    reinterpret_cast<const char *>(output.data()), output.size() * sizeof(Wide))) {
                throw std::runtime_error("cannot write worker output");
            }
        }
        std::cout.flush();
        if (!std::cout) {
            throw std::runtime_error("cannot flush worker output");
        }
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "kn-ceiling-certificate-device: " << error.what() << '\n';
        return 1;
    }
}

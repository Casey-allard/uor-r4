// Standalone GPU evaluator for kn-ceiling-fit-frame/1, order 10.
// Reference: crates/uor-r4-discrete/src/kn_ceiling.rs and
// crates/uor-r4-integer/src/lawful_likelihood.rs.
#include <cuda_runtime.h>
#include <openssl/evp.h>

#include <algorithm>

#include <array>
#include <cerrno>
#include <chrono>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>

using Word = unsigned long long;
using Clock = std::chrono::steady_clock;
namespace fs = std::filesystem;
constexpr unsigned Lanes = 13;
constexpr unsigned Buckets = 176;
constexpr unsigned RowBytes = 109;
constexpr unsigned Threads = 256;
constexpr unsigned Warps = Threads / 32;
constexpr unsigned Entries = Buckets * Lanes;
constexpr unsigned MetricWords = 6;
constexpr unsigned Stripes = 128;
constexpr unsigned AllMask = (1 << Lanes) - 1;
constexpr unsigned RestrictedMask = 1 | (1 << (Lanes - 1));
constexpr Word WeightOne = 1ULL << 32;
constexpr Word MassOne = 1ULL << 48;
constexpr Word UniformMass = MassOne / 4096;
constexpr unsigned MaxTargets = (1U << 24) - 1;

void check_cuda(cudaError_t status, const char *operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(status));
    }
}
#define CUDA(operation) check_cuda((operation), #operation)

template <typename T> class DeviceBuffer {
  public:
    explicit DeviceBuffer(size_t n) : length(n) { CUDA(cudaMalloc(&pointer, n * sizeof(T))); }
    ~DeviceBuffer() { cudaFree(pointer); }
    DeviceBuffer(const DeviceBuffer &) = delete;
    DeviceBuffer &operator=(const DeviceBuffer &) = delete;

    void copy_from(const T *data) {
        CUDA(cudaMemcpy(pointer, data, length * sizeof(T), cudaMemcpyHostToDevice));
    }
    void copy_to(T *data) const {
        CUDA(cudaMemcpy(data, pointer, length * sizeof(T), cudaMemcpyDeviceToHost));
    }
    void zero() { CUDA(cudaMemset(pointer, 0, length * sizeof(T))); }

    T *pointer = nullptr;
    size_t length;
};

__host__ __device__ unsigned live_mask(unsigned bucket, unsigned enabled) {
    return enabled & (((1U << (bucket / 16 + 2)) - 1) | (1U << (Lanes - 1)));
}

struct Wide {
    Word lo;
    Word hi;
};

__device__ Wide mul(Word a, Word b) { return {a * b, __umul64hi(a, b)}; }

__device__ Wide add(Wide a, Wide b) {
    Word lo = a.lo + b.lo;
    return {lo, a.hi + b.hi + (lo < a.lo)};
}

__device__ Word shr(Wide value, unsigned shift) {
    return shift == 0 ? value.lo : (value.lo >> shift) | (value.hi << (64 - shift));
}

__device__ bool greater(Wide a, Wide b) {
    return a.hi > b.hi || (a.hi == b.hi && a.lo > b.lo);
}

// Exact floor(numerator * 2^32 / denominator) for 0 <= numerator <= denominator
// <= 2^80. Approximating the denominator by its highest 32 bits gives an upper
// estimate; the dropped tail can change the real quotient by less than 2.
// The quotient and normalized divisor both fit in 64 bits. Compare against the
// original denominator to correct the estimate, preserving all Q48 product bits.
struct Fraction {
    Wide denominator;
    int shift;
    unsigned divisor;
    Word reciprocal;

    __device__ explicit Fraction(Wide value) : denominator(value) {
        const unsigned bits = value.hi ? 128 - __clzll(value.hi) : 64 - __clzll(value.lo);
        shift = int(bits) - 32;
        divisor = normalize(value);
        // floor((2^64 - 1)/divisor), using a shared restoring division
        // per row. The normalized divisor has bit 31 set, so the high
        // 32-bit half of the all-ones dividend contributes one quotient bit.
        unsigned remainder = 0xffffffffU - divisor;
        reciprocal = 1;
        #pragma unroll 1
        for (unsigned bit = 0; bit < 32; ++bit) {
            const bool carry = (remainder >> 31) != 0;
            remainder = (remainder << 1) | 1;
            const bool take = carry || remainder >= divisor;
            remainder -= take ? divisor : 0;
            reciprocal = (reciprocal << 1) | take;
        }
    }

    __device__ Word normalize(Wide value) const {
        return shift >= 0 ? shr(value, shift) : value.lo << -shift;
    }

    __device__ Word quotient(Wide numerator) const {
        if ((numerator.lo | numerator.hi) == 0) {
            return 0;
        }
        if (numerator.lo == denominator.lo && numerator.hi == denominator.hi) {
            return WeightOne;
        }
        // Since numerator < denominator, (numerator << 32) >> shift < 2^64.
        const Wide scaled = {
            numerator.lo << 32, (numerator.hi << 32) | (numerator.lo >> 32)
        };
        const Word normalized = normalize(scaled);
        // High-half multiplication estimates floor(normalized/divisor) from
        // below, with at most one missing unit. Correct this using the remainder.
        Word estimate = __umul64hi(normalized, reciprocal);
        estimate += normalized - estimate * divisor >= divisor;
        Wide measured = mul(estimate, denominator.lo);
        measured.hi += estimate * denominator.hi;
        if (greater(measured, scaled)) {
            --estimate;
            const Word lo = measured.lo - denominator.lo;
            measured.hi -= denominator.hi + (lo > measured.lo);
            measured.lo = lo;
            estimate -= greater(measured, scaled);
        }
        return estimate;
    }
};

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

__global__ void initialize(Word *weights, unsigned enabled, bool near_kn) {
    const unsigned b = blockIdx.x * blockDim.x + threadIdx.x;
    if (b >= Buckets) {
        return;
    }
    const unsigned mask = live_mask(b, enabled);
    const unsigned count = __popc(mask);
    const Word share = near_kn ? 1 : Fraction({count, 0}).quotient({1, 0});
    for (unsigned e = 0; e < Lanes; ++e) {
        weights[b * Lanes + e] = (mask & (1U << e)) ? share : 0;
    }
    weights[b * Lanes] += WeightOne - share * count;
}

__device__ unsigned largest(const Word *weights) {
    unsigned best = 0;
    for (unsigned e = 1; e < Lanes; ++e) {
        if (weights[e] > weights[best]) {
            best = e;
        }
    }
    return best;
}

__global__ void update(
    Word *weights, const Word *responsibilities, unsigned enabled, unsigned *dwell, int *error
) {
    const unsigned b = blockIdx.x * blockDim.x + threadIdx.x;
    if (b >= Buckets) {
        return;
    }
    const unsigned mask = live_mask(b, enabled);
    const Word *response = responsibilities + b * Lanes;
    Word *row = weights + b * Lanes;
    Word total = 0;
    for (unsigned e = 0; e < Lanes; ++e) {
        total += response[e];
    }
    if (total != 0) {
        const Fraction fraction({total, 0});
        Word assigned = 0;
        for (unsigned e = 0; e < Lanes; ++e) {
            row[e] = fraction.quotient({response[e], 0});
            assigned += row[e];
        }
#ifndef KN_FIT_FAULT_REMAINDER
        row[largest(row)] += WeightOne - assigned;
#endif
        for (unsigned e = 0; e < Lanes; ++e) {
            if ((mask & (1U << e)) && row[e] == 0) {
                --row[largest(row)];
                row[e] = 1;
            }
        }
    }
    Word sum = 0;
    for (unsigned e = 0; e < Lanes; ++e) {
        const bool live = (mask & (1U << e)) != 0;
        const Word weight = row[e];
        if (live != (weight > 0) || weight > WeightOne) {
            atomicExch(error, 1);
        }
        if (live && weight == 1) {
            ++dwell[b * Lanes + e];
        }
        sum += weight;
    }
    if (sum != WeightOne) {
        atomicExch(error, 1);
    }
}

struct Chunk {
    unsigned offset;
    unsigned length;
    unsigned bucket;
};

// Each CTA takes targets in a single bucket and parity. Masses are lane-major
// on device so a warp reads adjacent targets for every active expert.
template <bool CollectResponse, unsigned Enabled>
__global__ void evaluate(
    const Word *masses, const uint16_t *metadata, unsigned stride, const Chunk *chunks,
    unsigned first_chunk, const Word *weights, Word *resp, Word *metrics, Word *fixture_scores
) {
    constexpr unsigned Outputs = MetricWords + (CollectResponse ? Lanes : 0);
    __shared__ Word partial[Outputs][Warps];
    __shared__ Word shared_weights[Lanes];
    const Chunk chunk = chunks[first_chunk + blockIdx.x];
    if constexpr (Enabled != 1) {
        if (threadIdx.x < Lanes) {
            shared_weights[threadIdx.x] = weights[chunk.bucket * Lanes + threadIdx.x];
        }
        __syncthreads();
    }
    const unsigned index = chunk.offset + threadIdx.x;
    Word responses[Lanes] = {};
    Word n = 0;
    Word bytes = 0;
    Word bit_cost = 0;
    Word novel = 0;
    if (threadIdx.x < chunk.length) {
        const uint16_t fields = metadata[index];
        n = 1;
        bytes = fields & 255;
        novel = fields >> 8;
        if constexpr (Enabled == 1) {
            bit_cost = bits_q12(masses[index]);
            if (fixture_scores) {
                fixture_scores[index] = bit_cost;
            }
        } else {
            Wide sum{0, 0};
            Wide terms[Lanes] = {};
            #pragma unroll
            for (unsigned e = 0; e < Lanes; ++e) {
                if ((Enabled & (1U << e)) && shared_weights[e]) {
                    terms[e] = mul(shared_weights[e], masses[e * size_t(stride) + index]);
                    sum = add(sum, terms[e]);
                }
            }
            const Word mass = shr(sum, 32);
            bit_cost = bits_q12(mass ? mass : 1);
            if constexpr (CollectResponse) {
                const Fraction fraction(sum);
                #pragma unroll
                for (unsigned e = 0; e < Lanes; ++e) {
                    if (Enabled & (1U << e)) {
                        responses[e] = fraction.quotient(terms[e]);
                    }
                }
            }
        }
    }
    const Word tallies[MetricWords] = {n, bytes, bit_cost, novel, novel * bytes, novel * bit_cost};
    const unsigned lane = threadIdx.x % 32;
    const unsigned warp = threadIdx.x / 32;
    #pragma unroll
    for (unsigned metric = 0; metric < Outputs; ++metric) {
        if (metric >= MetricWords && !(Enabled & (1U << (metric - MetricWords)))) {
            continue;
        }
        Word value = metric < MetricWords ? tallies[metric] : responses[metric - MetricWords];
        for (unsigned offset = 16; offset > 0; offset /= 2) {
            value += __shfl_down_sync(0xffffffff, value, offset);
        }
        if (lane == 0) {
            partial[metric][warp] = value;
        }
    }
    __syncthreads();
    const unsigned field = threadIdx.x;
    if (field < Outputs &&
        (field < MetricWords || (Enabled & (1U << (field - MetricWords))))) {
        Word sum = 0;
        #pragma unroll
        for (unsigned w = 0; w < Warps; ++w) {
            sum += partial[field][w];
        }
        if (sum != 0) {
            if (field < MetricWords) {
                // Independent stripes avoid contention on the objective/tally words.
                atomicAdd(metrics + (blockIdx.x % Stripes) * MetricWords + field, sum);
            } else {
                atomicAdd(resp + chunk.bucket * Lanes + field - MetricWords, sum);
            }
        }
    }
}

// Accumulation ceilings at MaxTargets are below 2^56 for responsibilities,
// 2^42 for objective words, and 2^32 for credited bytes.
__global__ void reduce_tally(const Word *partials, Word *total) {
    const unsigned field = threadIdx.x;
    if (field < MetricWords) {
        Word sum = 0;
        for (unsigned stripe = 0; stripe < Stripes; ++stripe) {
            sum += partials[stripe * MetricWords + field];
        }
        total[field] = sum;
    }
}

__global__ void combine_fold_scores(Word *last_tally, const Word *first_fold) {
    const unsigned field = threadIdx.x;
    if (field < MetricWords) {
        last_tally[field] += first_fold[field];
    }
}

class Sha256 {
  public:
    Sha256() {
        if (!context || EVP_DigestInit_ex(context.get(), EVP_sha256(), nullptr) != 1) {
            throw std::runtime_error("cannot initialize SHA-256");
        }
    }

    void add(const void *bytes, size_t length) {
        if (EVP_DigestUpdate(context.get(), bytes, length) != 1) {
            throw std::runtime_error("cannot update SHA-256");
        }
    }

    std::string finish() {
        unsigned char digest[32];
        unsigned length;
        if (EVP_DigestFinal_ex(context.get(), digest, &length) != 1 || length != sizeof(digest)) {
            throw std::runtime_error("cannot finish SHA-256");
        }
        constexpr char digits[] = "0123456789abcdef";
        std::string hex;
        for (unsigned char byte : digest) {
            hex += digits[byte >> 4];
            hex += digits[byte & 15];
        }
        return hex;
    }

  private:
    std::unique_ptr<EVP_MD_CTX, decltype(&EVP_MD_CTX_free)>
        context{EVP_MD_CTX_new(), EVP_MD_CTX_free};
};

std::string hash_file(const fs::path &path) {
    std::ifstream in(path, std::ios::binary);
    Sha256 hash;
    std::array<char, 65536> buffer;
    while (in.read(buffer.data(), buffer.size()) || in.gcount() > 0) {
        hash.add(buffer.data(), in.gcount());
    }
    if (in.bad() || !in.eof()) {
        throw std::runtime_error("cannot hash " + path.string());
    }
    return hash.finish();
}

// Linux private read-only mapping. Exact EOF is checked before accessing rows.
class Frame {
  public:
    Frame(const fs::path &path, unsigned n) : length(size_t(n) * RowBytes) {
        const int fd = open(path.c_str(), O_RDONLY | O_CLOEXEC | O_NOFOLLOW);
        if (fd < 0) {
            throw std::runtime_error("cannot open frame: " + std::string(std::strerror(errno)));
        }
        struct stat info {};
        if (fstat(fd, &info) != 0 || !S_ISREG(info.st_mode) || info.st_size < 0 ||
            uint64_t(info.st_size) != length) {
            close(fd);
            throw std::runtime_error("frame size differs from exact target count");
        }
        auto mapping = mmap(nullptr, length, PROT_READ, MAP_PRIVATE, fd, 0);
        close(fd);
        if (mapping == MAP_FAILED) {
            throw std::runtime_error("cannot map frame: " + std::string(std::strerror(errno)));
        }
        data = static_cast<const uint8_t *>(mapping);
    }

    ~Frame() { munmap(const_cast<uint8_t *>(data), length); }
    Frame(const Frame &) = delete;
    Frame &operator=(const Frame &) = delete;

    const uint8_t *operator[](unsigned t) const { return data + size_t(t) * RowBytes; }

  private:
    size_t length;
    const uint8_t *data;
};

Word load_le(const uint8_t *p) {
    Word value;
    std::memcpy(&value, p, sizeof(value));
    static_assert(__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__, "requires little-endian host");
    return value;
}

struct StagedFrame {
    std::vector<Word> masses;
    std::vector<uint16_t> metadata;
    std::vector<Chunk> chunks;
    std::array<unsigned, Buckets * 2> bucket_counts{};
    unsigned split_chunk = 0;
    std::string digest;

    StagedFrame(const Frame &frame, unsigned n) : masses(size_t(n) * Lanes), metadata(n) {
        auto group = [](const uint8_t *row, unsigned t) {
            const unsigned b = row[0] | (unsigned(row[1]) << 8);
            if (b >= Buckets || row[2] > 1 || row[4] > 1) {
                throw std::runtime_error("frame metadata outside domain at target " + std::to_string(t));
            }
            return b + row[4] * Buckets;
        };
        for (unsigned t = 0; t < n; ++t) {
            ++bucket_counts[group(frame[t], t)];
        }
        std::array<unsigned, Buckets * 2> next{};
        std::array<unsigned, Buckets * 2> limits{};
        unsigned start = 0;
        for (unsigned g = 0; g < Buckets * 2; ++g) {
            if (g == Buckets) {
                split_chunk = chunks.size();
            }
            next[g] = start;
            const unsigned end = start + bucket_counts[g];
            limits[g] = end;
            for (; start < end; start += Threads) {
                chunks.push_back({start, std::min(Threads, end - start), g % Buckets});
            }
            start = end;
        }
        Sha256 hash;
        for (unsigned t = 0; t < n; ++t) {
            uint8_t row[RowBytes];
            std::memcpy(row, frame[t], sizeof(row));
            hash.add(row, sizeof(row));
            const unsigned g = group(row, t);
            if (next[g] >= limits[g]) {
                throw std::runtime_error("frame bucket census changed while staging");
            }
            const unsigned index = next[g]++;
            metadata[index] = row[3] | (uint16_t(row[2]) << 8);
            const unsigned present = live_mask(g % Buckets, AllMask);
            for (unsigned e = 0; e < Lanes; ++e) {
                const Word value = load_le(row + 5 + e * 8);
                if (value > MassOne || (e == 0 && value == 0) ||
                    (e == Lanes - 1 && value != UniformMass) ||
                    (!(present & (1U << e)) && value != 0)) {
                    throw std::runtime_error("frame mass outside domain at target " + std::to_string(t));
                }
                masses[e * size_t(n) + index] = value;
            }
        }
        digest = hash.finish();
    }
};

using Tallies = std::array<Word, MetricWords>;
using Weights = std::array<Word, Entries>;
using Dwell = std::array<unsigned, Entries>;

struct Fit {
    Weights start{};
    Weights end{};
    Dwell dwell{};
    std::vector<Word> trace;
    bool converged = false;
    unsigned best_pass = 0;
    unsigned best_checkpoint_pass = 24;
    Tallies score{};
};

bool plateau(const std::vector<Word> &trace, Word tolerance) {
    auto close = [&](Word a, Word b) { return a >= b ? a - b <= tolerance : b - a <= tolerance; };
    const size_t end = trace.size() - 1;
    if (!close(trace[end], trace[end - 8])) {
        return false;
    }
    for (size_t pass = end - 7; pass <= end; ++pass) {
        if (!close(trace[pass], trace[pass - 1])) {
            return false;
        }
    }
    return true;
}

class Runner {
  public:
    Runner(const StagedFrame &staged, unsigned n, std::ostream *fixture)
        : targets(n), chunks(staged.chunks.size()), split(staged.split_chunk),
          masses(size_t(n) * Lanes), metadata(n), descriptors(chunks), fixture_scores(fixture ? n : 1),
          fixture(fixture) {
        masses.copy_from(staged.masses.data());
        metadata.copy_from(staged.metadata.data());
        descriptors.copy_from(staged.chunks.data());
    }

    template <unsigned Enabled>
    Tallies score(int parity) {
        const Tallies tally = run_pass<false, Enabled>(parity);
        if (fixture && Enabled == 1) {
            std::vector<Word> per_row(targets);
            fixture_scores.copy_to(per_row.data());
            write_words(per_row);
            write_words(tally);
        }
        return tally;
    }

    template <unsigned Enabled>
    Fit fit(const char *name, bool near_kn, int parity, bool primary) {
        const auto began = Clock::now();
        Fit result;
        initialize<<<1, Threads>>>(weights.pointer, Enabled, near_kn);
        CUDA(cudaGetLastError());
        weights.copy_to(result.start.data());
        if (fixture) {
            write_words(result.start);
        }
        dwell.zero();
        error.zero();
        const unsigned max_pass = primary ? 64 : 24;
        for (unsigned pass = 0; pass <= max_pass; ++pass) {
            const bool checkpoint = pass >= 24 && (pass - 24) % 8 == 0;
            // All primary checkpoints first score without advancing the state.
            // Intermediate traces come from the pre-update E step.
            const bool score_only = pass == max_pass || (primary && checkpoint);
            Tallies evaluation{};
            if (score_only) {
                evaluation = run_pass<false, Enabled>(parity);
            } else {
                evaluation = run_pass<true, Enabled>(parity);
            }
            result.trace.push_back(evaluation[2]);
            if (fixture) {
                write_words(evaluation);
            }
            if (evaluation[2] < result.trace[result.best_pass]) {
                result.best_pass = pass;
            }
            if (checkpoint) {
                if (evaluation[2] < result.trace[result.best_checkpoint_pass]) {
                    result.best_checkpoint_pass = pass;
                }
                result.converged = plateau(result.trace, tolerance(evaluation[0]));
                std::cerr << name << " pass=" << pass << " objective_q12=" << evaluation[2]
                          << " elapsed_ms=" << elapsed_ms(began) << '\n';
                if ((primary && result.converged) || pass == max_pass) {
                    result.score = parity < 0 ? evaluation : run_pass<false, Enabled>(1 - parity);
                    if (fixture) {
                        write_words(result.score);
                    }
                    break;
                }
            }
            if (score_only) {
                run_pass<true, Enabled>(parity);
            }
            if (fixture) {
                Weights responses{};
                resp.copy_to(responses.data());
                write_words(responses);
            }
            update<<<1, Threads>>>(weights.pointer, resp.pointer, Enabled, dwell.pointer, error.pointer);
            CUDA(cudaGetLastError());
            int invalid = 0;
            error.copy_to(&invalid);
            if (invalid) {
                throw std::runtime_error(std::string(name) + " CUDA weight invariant failed");
            }
            if (fixture) {
                Weights current{};
                weights.copy_to(current.data());
                write_words(current);
            }
        }
        weights.copy_to(result.end.data());
        dwell.copy_to(result.dwell.data());
        if (fixture) {
            const Weights dwell_words = [&] {
                Weights words{};
                std::copy(result.dwell.begin(), result.dwell.end(), words.begin());
                return words;
            }();
            write_words(dwell_words);
        }
        return result;
    }

    void save_first_fold() {
        CUDA(cudaMemcpy(first_fold.pointer, tally.pointer, sizeof(Tallies), cudaMemcpyDeviceToDevice));
    }

    Tallies split_score() {
        combine_fold_scores<<<1, 32>>>(tally.pointer, first_fold.pointer);
        CUDA(cudaGetLastError());
        Tallies result{};
        tally.copy_to(result.data());
        if (fixture) {
            write_words(result);
        }
        return result;
    }

    static Word tolerance(Word targets) { return 4096 * targets / 10000; }

    static long long elapsed_ms(Clock::time_point began) {
        return std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - began).count();
    }

  private:
    template <bool Response, unsigned Enabled>
    Tallies run_pass(int parity) {
        const unsigned begin = parity == 1 ? split : 0;
        const unsigned end = parity == 0 ? split : chunks;
        if constexpr (Response) {
            resp.zero();
        }
        metrics.zero();
        if (begin < end) {
            evaluate<Response, Enabled><<<end - begin, Threads>>>(
                masses.pointer, metadata.pointer, targets, descriptors.pointer, begin,
                weights.pointer, resp.pointer, metrics.pointer,
                fixture && Enabled == 1 ? fixture_scores.pointer : nullptr
            );
            CUDA(cudaGetLastError());
        }
        reduce_tally<<<1, 32>>>(metrics.pointer, tally.pointer);
        CUDA(cudaGetLastError());
        Tallies result{};
        tally.copy_to(result.data());
        return result;
    }

    template <typename Words> void write_words(const Words &words) {
        fixture->write(reinterpret_cast<const char *>(words.data()), words.size() * sizeof(Word));
        if (!*fixture) {
            throw std::runtime_error("cannot write witness");
        }
    }

    unsigned targets;
    unsigned chunks;
    unsigned split;
    DeviceBuffer<Word> masses;
    DeviceBuffer<uint16_t> metadata;
    DeviceBuffer<Chunk> descriptors;
    DeviceBuffer<Word> fixture_scores;
    DeviceBuffer<Word> weights{Entries};
    DeviceBuffer<Word> resp{Entries};
    DeviceBuffer<Word> metrics{MetricWords * Stripes};
    DeviceBuffer<Word> tally{MetricWords};
    DeviceBuffer<Word> first_fold{MetricWords};
    DeviceBuffer<unsigned> dwell{Entries};
    DeviceBuffer<int> error{1};
    std::ostream *fixture;
};

void emit_score(std::ostream &out, const Tallies &score) {
    out << "{\"all\":{\"targets\":" << score[0] << ",\"bytes\":" << score[1]
        << ",\"bits_q12\":" << score[2] << "},\"novel\":{\"targets\":" << score[3]
        << ",\"bytes\":" << score[4] << ",\"bits_q12\":" << score[5] << "}}";
}

std::string score_json(const Tallies &score) {
    std::ostringstream out;
    out << "{\"scored\":";
    emit_score(out, score);
    out << '}';
    return out.str();
}

template <typename T> void emit_row(std::ostream &out, const T *row, size_t length) {
    out << '[';
    for (size_t e = 0; e < length; ++e) {
        if (e) {
            out << ',';
        }
        out << row[e];
    }
    out << ']';
}

std::string fit_json(
    const Fit &fit, unsigned enabled, const std::array<unsigned, Buckets * 2> &counts, int parity
) {
    std::ostringstream out;
    out << "{\"fit_bits_q12\":";
    emit_row(out, fit.trace.data(), fit.trace.size());
    out << ",\"endpoint\":" << fit.trace.size() - 1 << ",\"best_pass\":" << fit.best_pass
        << ",\"best_checkpoint_pass\":" << fit.best_checkpoint_pass
        << ",\"converged\":" << (fit.converged ? "true" : "false") << ",\"rise_passes\":[";
    bool first = true;
    for (size_t pass = 1; pass < fit.trace.size(); ++pass) {
        if (fit.trace[pass] > fit.trace[pass - 1]) {
            if (!first) {
                out << ',';
            }
            out << pass;
            first = false;
        }
    }
    out << "],\"scored\":";
    emit_score(out, fit.score);
    out << ",\"buckets\":[";
    for (unsigned b = 0; b < Buckets; ++b) {
        if (b) {
            out << ',';
        }
        const unsigned count = (parity != 1 ? counts[b] : 0) +
                               (parity != 0 ? counts[Buckets + b] : 0);
        out << "{\"bucket\":" << b << ",\"targets\":" << count << ",\"mask\":"
            << live_mask(b, enabled) << ",\"start_q32\":";
        emit_row(out, fit.start.data() + b * Lanes, Lanes);
        out << ",\"weights_q32\":";
        emit_row(out, fit.end.data() + b * Lanes, Lanes);
        out << ",\"floor_dwell\":";
        emit_row(out, fit.dwell.data() + b * Lanes, Lanes);
        out << '}';
    }
    out << "]}";
    return out.str();
}

void save(const fs::path &path, const std::string &contents) {
    const auto staged = path.string() + ".part";
    std::ofstream out(staged, std::ios::binary);
    out << contents << '\n';
    out.close();
    if (!out) {
        throw std::runtime_error("cannot write " + staged);
    }
    fs::rename(staged, path);
}

int main(int argc, char **argv) {
    try {
        if (argc != 5 && argc != 7) {
            throw std::runtime_error(
                "usage: kn-ceiling-fit FRAME TARGETS OUT --identity MANIFEST_SHA256 FRAME_SHA256"
                " | FRAME TARGETS OUT --fixture"
            );
        }
        const bool fixture = argc == 5 && std::string(argv[4]) == "--fixture";
        if (!fixture && (argc != 7 || std::string(argv[4]) != "--identity")) {
            throw std::runtime_error("provide the verified manifest and frame hashes");
        }
        size_t parsed = 0;
        const std::string count = argv[2];
        const auto targets = std::stoull(count, &parsed);
        if (parsed != count.size() || count[0] < '1' || count[0] > '9' ||
            targets == 0 || targets > MaxTargets || (fixture && targets > 65536)) {
            throw std::runtime_error("target count outside supported domain");
        }
        const fs::path out = argv[3];
        if (!fs::create_directory(out)) {
            throw std::runtime_error("fit output directory must be new");
        }
        const auto load_started = Clock::now();
        std::ofstream fixture_output;
        if (fixture) {
            fixture_output.open(out / "fixture.bin.part", std::ios::binary);
            if (!fixture_output) {
                throw std::runtime_error("cannot open finite witness");
            }
        }
        std::array<unsigned, Buckets * 2> counts{};
        std::unique_ptr<Runner> runner;
        std::string frame_hash;
        const auto manifest_hash = fixture ? "" : hash_file(fs::path(argv[1]).parent_path() / "manifest.json");
        {
            const Frame frame(argv[1], targets);
            const StagedFrame staged(frame, targets);
            frame_hash = staged.digest;
            if (!fixture && (manifest_hash != argv[5] || frame_hash != argv[6])) {
                throw std::runtime_error("manifest or frame SHA-256 mismatch");
            }
            counts = staged.bucket_counts;
            CUDA(cudaSetDevice(0));
            runner = std::make_unique<Runner>(staged, targets, fixture ? &fixture_output : nullptr);
        }
        size_t free_mem = 0;
        size_t total_mem = 0;
        CUDA(cudaMemGetInfo(&free_mem, &total_mem));
        std::cerr << "gpu_stage_ms=" << Runner::elapsed_ms(load_started)
                  << " allocated_frame_bytes=" << targets * (sizeof(Word) * Lanes + sizeof(uint16_t))
                  << " device_free_bytes_after_stage=" << free_mem
                  << " device_total_bytes=" << total_mem << '\n';
        const std::string identity = "\"manifest_sha256\":" +
            (fixture ? std::string("null") : "\"" + manifest_hash + "\"") +
            ",\"frame_sha256\":\"" + frame_hash +
            "\",\"fitter_sha256\":\"" + hash_file("/proc/self/exe") + "\"";
        std::ostringstream report;
        report << "{\"schema\":\"kn-ceiling-fit-result/1\",\"targets\":" << targets
               << ",\"tolerance_q12\":" << Runner::tolerance(targets) << ',' << identity;
        auto record = [&](const char *name, const std::string &json) {
            const std::string bound = "{" + identity + "," + json.substr(1);
            save(out / (std::string(name) + ".json"), bound);
            report << ",\"" << name << "\":" << bound;
        };
        record("kn_only", score_json(runner->score<1>(-1)));
        auto all = [&](const char *name, bool near_kn, int parity, bool primary) {
            auto fit = runner->fit<AllMask>(name, near_kn, parity, primary);
            record(name, fit_json(fit, AllMask, counts, parity));
        };
        auto restricted = [&](const char *name, bool near_kn) {
            auto fit = runner->fit<RestrictedMask>(name, near_kn, -1, false);
            record(name, fit_json(fit, RestrictedMask, counts, -1));
        };
        all("u_count", true, -1, true);
        all("u_count_uniform", false, -1, false);
        restricted("kn_u", true);
        restricted("kn_u_uniform", false);
        all("split_even_fit", false, 0, false);
        runner->save_first_fold();
        all("split_odd_fit", false, 1, false);
        record("u_split", score_json(runner->split_score()));
        if (fixture) {
            fixture_output.close();
            if (!fixture_output) {
                throw std::runtime_error("cannot close finite witness");
            }
            fs::rename(out / "fixture.bin.part", out / "fixture.bin");
        }
        report << '}';
        save(out / "result.json", report.str());
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "kn-ceiling-fit: " << error.what() << '\n';
        return 1;
    }
}

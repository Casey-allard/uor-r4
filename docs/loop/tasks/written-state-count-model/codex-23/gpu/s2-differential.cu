#include <cuda_runtime.h>

#include <charconv>
#include <cinttypes>
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {
constexpr unsigned vocabulary = 4096;
constexpr unsigned legal = vocabulary - 1;
constexpr unsigned max_order = 10;
constexpr unsigned row_bytes = 20;
constexpr unsigned max_reads = 4;
constexpr unsigned top_size = 8;
constexpr uint64_t q32 = uint64_t{1} << 32;
constexpr uint64_t floor_budget = q32 / 32;

struct Key {
    uint64_t low;
    uint64_t high;
};

__host__ __device__ int compare(Key a, Key b) {
    if (a.high != b.high) return a.high < b.high ? -1 : 1;
    if (a.low != b.low) return a.low < b.low ? -1 : 1;
    return 0;
}

__host__ __device__ uint64_t load64(const unsigned char* bytes) {
    uint64_t value = 0;
    for (unsigned i = 0; i < 8; ++i) {
        value |= static_cast<uint64_t>(bytes[i]) << (8 * i);
    }
    return value;
}

__host__ __device__ Key load_key(const unsigned char* row) {
    return {load64(row), load64(row + 8)};
}

__host__ __device__ unsigned load_token(const unsigned char* row) {
    return row[16] | (static_cast<unsigned>(row[17]) << 8);
}

struct Prefix {
    size_t start;
    size_t end;
};

struct Read {
    unsigned token;
    uint64_t count;
    uint64_t extra;
};

struct Ranked {
    unsigned token;
    uint64_t mass;
};

struct Trace {
    bool hit;
    Key key;
    unsigned order;
    unsigned read_len;
    Read reads[max_reads];
    Ranked top[top_size];
    uint64_t mass[vocabulary];
};

__device__ Key suffix_key(const uint16_t* end, unsigned order) {
    Key key{0, static_cast<uint64_t>(order) << 56};
    for (unsigned slot = 0; slot < order; ++slot) {
        const uint64_t token = end[-1 - static_cast<int>(slot)];
        const unsigned shift = 108 - 12 * slot;
        if (shift >= 64) {
            key.high |= token << (shift - 64);
        } else {
            key.low |= token << shift;
            // Include a slot's upper bits if it spans the two words.
            if (shift > 52) key.high |= token >> (64 - shift);
        }
    }
    return key;
}

__device__ size_t lower_bound(const unsigned char* rows, size_t count, Key key) {
    size_t low = 0;
    size_t high = count;
    while (low < high) {
        const size_t mid = low + (high - low) / 2;
        if (compare(load_key(rows + mid * row_bytes), key) < 0) {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    return low;
}

__device__ uint64_t cumulative_mass(uint64_t count, uint64_t total) {
    // At most four ranks of 31 make count <= 2^33. Multiplying by half
    // the count budget stays below 2^64. Recover the doubled product's
    // exact quotient by also doubling the remainder.
    uint64_t remainder = ((q32 - floor_budget) / 2) * count;
    uint64_t quotient = 0;
    // Use integer restoring division. CUDA lowering of variable device
    // division can introduce floating-point reciprocal instructions.
    for (int bit = 30; bit >= 0; --bit) {
        const uint64_t shifted = total << bit;
        if (remainder >= shifted) {
            remainder -= shifted;
            quotient |= uint64_t{1} << bit;
        }
    }
    return 2 * quotient + (2 * remainder >= total);
}

__device__ unsigned read_index(const Trace& trace, unsigned token) {
    for (unsigned read = 0; read < trace.read_len; ++read) {
        if (trace.reads[read].token == token) return read;
    }
    return trace.read_len;
}

__device__ uint64_t uniform_mass(unsigned token, uint64_t budget) {
    return budget / legal + (token <= budget % legal);
}

__device__ void insert(Ranked* top, Ranked candidate) {
    for (unsigned position = 0; position < top_size; ++position) {
        const Ranked previous = top[position];
        if (candidate.mass > previous.mass ||
            (candidate.mass == previous.mass && candidate.token < previous.token)) {
            top[position] = candidate;
            candidate = previous;
        }
    }
}

__global__ void replay(const unsigned char* rows, size_t row_count, const uint16_t* tokens,
                       const Prefix* prefixes, size_t prefix_count, Trace* traces) {
    const size_t query = static_cast<size_t>(blockIdx.x) * blockDim.x + threadIdx.x;
    if (query >= prefix_count) return;
    Trace& trace = traces[query];
    const Prefix prefix = prefixes[query];
    unsigned available = 0;
    while (available < max_order && available < prefix.end - prefix.start) {
        const unsigned token = tokens[prefix.end - available - 1];
        if (token == 1) break;
        ++available;
        if (token == 0) break;
    }

    for (int order = static_cast<int>(available); order >= 0; --order) {
        const Key key = suffix_key(tokens + prefix.end, order);
        const size_t first = lower_bound(rows, row_count, key);
        uint64_t total = 0;
        for (size_t i = first; i < row_count; ++i) {
            const unsigned char* row = rows + i * row_bytes;
            if (compare(load_key(row), key) != 0) break;
            const uint64_t count = uint64_t{1} << row[18];
            trace.reads[trace.read_len++] = {load_token(row), count, 0};
            total += count;
        }
        if (total == 0) continue;

        trace.hit = true;
        trace.key = key;
        trace.order = order;
        uint64_t cumulative = 0;
        uint64_t previous = 0;
        for (unsigned read = 0; read < trace.read_len; ++read) {
            cumulative += trace.reads[read].count;
            const uint64_t boundary = cumulative_mass(cumulative, total);
            trace.reads[read].extra = boundary - previous;
            previous = boundary;
        }
        break;
    }

    const uint64_t budget = trace.hit ? floor_budget : q32;
    for (unsigned read = 0; read < trace.read_len; ++read) {
        const Read entry = trace.reads[read];
        insert(trace.top, {entry.token, uniform_mass(entry.token, budget) + entry.extra});
    }
    unsigned uniform_candidates = 0;
    for (unsigned token = 1; uniform_candidates < top_size; ++token) {
        if (read_index(trace, token) == trace.read_len) {
            insert(trace.top, {token, uniform_mass(token, budget)});
            ++uniform_candidates;
        }
    }

    // Differential output only: compare the complete support independently
    // of candidate selection. The sparse top-eight above never reads it.
    for (unsigned token = 1; token < vocabulary; ++token) {
        const unsigned read = read_index(trace, token);
        const uint64_t extra = read < trace.read_len ? trace.reads[read].extra : 0;
        trace.mass[token] = uniform_mass(token, budget) + extra;
    }
}

void check(cudaError_t status, const char* operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(status));
    }
}

template <typename T>
class Buffer {
public:
    explicit Buffer(size_t len) : bytes_(sizeof(T) * len) {
        check(cudaMalloc(&ptr_, bytes_), "cudaMalloc");
    }
    ~Buffer() { cudaFree(ptr_); }
    Buffer(const Buffer&) = delete;
    Buffer& operator=(const Buffer&) = delete;

    T* get() const { return ptr_; }
    void upload(const T* source) {
        check(cudaMemcpy(ptr_, source, bytes_, cudaMemcpyHostToDevice), "upload");
    }
    void download(T* destination) const {
        check(cudaMemcpy(destination, ptr_, bytes_, cudaMemcpyDeviceToHost), "download");
    }
    void zero() { check(cudaMemset(ptr_, 0, bytes_), "cudaMemset"); }

private:
    T* ptr_ = nullptr;
    size_t bytes_;
};

std::vector<unsigned char> read_rows(const char* path) {
    std::ifstream input(path, std::ios::binary);
    if (!input) throw std::runtime_error("cannot open packed S2 rows");
    std::vector<unsigned char> rows;
    input.seekg(0, std::ios::end);
    const auto len = input.tellg();
    if (len < 0 || len % row_bytes != 0) throw std::runtime_error("partial S2 row");
    rows.resize(static_cast<size_t>(len));
    input.seekg(0);
    if (!input.read(reinterpret_cast<char*>(rows.data()), len)) {
        throw std::runtime_error("cannot read packed S2 rows");
    }

    Key previous{};
    unsigned previous_token = 0;
    unsigned reads = 0;
    for (size_t offset = 0; offset < rows.size(); offset += row_bytes) {
        const auto* row = rows.data() + offset;
        const Key key = load_key(row);
        const unsigned order = key.high >> 56;
        const unsigned token = load_token(row);
        const int ordering = compare(previous, key);
        if (order > max_order || token == 0 || token >= vocabulary ||
            row[18] < 1 || row[18] > 31 || row[19] != 0 ||
            (offset != 0 && (ordering > 0 || (ordering == 0 && token <= previous_token)))) {
            throw std::runtime_error("invalid or unordered packed S2 row");
        }
        const unsigned unused = 120 - 12 * order;
        const uint64_t low_mask =
            unused >= 64 ? UINT64_MAX : (uint64_t{1} << unused) - 1;
        const uint64_t high_mask = unused <= 64 ? 0 : (uint64_t{1} << (unused - 64)) - 1;
        if ((key.low & low_mask) != 0 || (key.high & high_mask) != 0) {
            throw std::runtime_error("nonzero unused S2 key slots");
        }
        reads = ordering == 0 ? reads + 1 : 1;
        if (reads > max_reads) throw std::runtime_error("too many S2 successors");
        previous = key;
        previous_token = token;
    }
    return rows;
}

std::vector<Prefix> read_queries(const char* path, std::vector<uint16_t>& tokens) {
    std::ifstream input(path);
    if (!input) throw std::runtime_error("cannot open query file");
    std::vector<Prefix> prefixes;
    std::string line;
    while (std::getline(input, line)) {
        const size_t start = tokens.size();
        std::istringstream words(line);
        std::string word;
        while (words >> word) {
            unsigned token = 0;
            const auto* end = word.data() + word.size();
            const auto parsed = std::from_chars(word.data(), end, token);
            if (parsed.ec != std::errc{} || parsed.ptr != end || token >= vocabulary) {
                throw std::runtime_error("invalid query token");
            }
            tokens.push_back(token);
        }
        prefixes.push_back({start, tokens.size()});
    }
    if (input.bad() || prefixes.empty()) throw std::runtime_error("missing finite queries");
    if (prefixes.size() > std::numeric_limits<int>::max()) {
        throw std::runtime_error("too many finite queries");
    }
    return prefixes;
}

void emit(const std::vector<Trace>& traces) {
    std::puts("[");
    for (size_t i = 0; i < traces.size(); ++i) {
        const Trace& trace = traces[i];
        std::printf("%s{\"order\":%u,\"key\":", i ? ",\n" : "", trace.order);
        if (trace.hit) {
            std::printf("\"%016" PRIx64 "%016" PRIx64 "\"", trace.key.high, trace.key.low);
        } else {
            std::printf("null");
        }
        std::printf(",\"reads\":[");
        for (unsigned read = 0; read < trace.read_len; ++read) {
            std::printf("%s[%u,%" PRIu64 "]", read ? "," : "", trace.reads[read].token,
                        trace.reads[read].count);
        }
        std::printf("],\"top1\":%u,\"top8\":[", trace.top[0].token);
        for (unsigned rank = 0; rank < top_size; ++rank) {
            std::printf("%s[%u,%" PRIu64 "]", rank ? "," : "", trace.top[rank].token,
                        trace.top[rank].mass);
        }
        std::printf("],\"mass\":[");
        for (unsigned token = 0; token < vocabulary; ++token) {
            std::printf("%s%" PRIu64, token ? "," : "", trace.mass[token]);
        }
        std::printf("]}");
    }
    std::puts("\n]");
    if (std::fflush(stdout) != 0) throw std::runtime_error("cannot write trace");
}
}  // namespace

int main(int argc, char** argv) {
    if (argc != 3) {
        std::fprintf(stderr, "usage: %s packed-s2.rows queries.txt\n", argv[0]);
        return EXIT_FAILURE;
    }
    try {
        auto rows = read_rows(argv[1]);
        const size_t row_count = rows.size() / row_bytes;
        if (rows.empty()) rows.resize(1);
        std::vector<uint16_t> tokens;
        const auto prefixes = read_queries(argv[2], tokens);
        if (tokens.empty()) tokens.resize(1);
        Buffer<unsigned char> device_rows(rows.size());
        Buffer<uint16_t> device_tokens(tokens.size());
        Buffer<Prefix> device_prefixes(prefixes.size());
        Buffer<Trace> device_traces(prefixes.size());
        device_rows.upload(rows.data());
        device_tokens.upload(tokens.data());
        device_prefixes.upload(prefixes.data());
        device_traces.zero();

        constexpr unsigned threads = 32;
        replay<<<(prefixes.size() + threads - 1) / threads, threads>>>(
            device_rows.get(), row_count, device_tokens.get(), device_prefixes.get(),
            prefixes.size(), device_traces.get());
        check(cudaGetLastError(), "launch replay");
        std::vector<Trace> traces(prefixes.size());
        device_traces.download(traces.data());
        emit(traces);
        return EXIT_SUCCESS;
    } catch (const std::exception& error) {
        std::fprintf(stderr, "%s\n", error.what());
        return EXIT_FAILURE;
    }
}

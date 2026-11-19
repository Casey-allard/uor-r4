#include <cuda_runtime.h>
#include <array>
#include <fstream>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <vector>

#include "discrete_read.cuh"

namespace {

struct Counts {
    const U *rows;
    U size;
};

struct Trunk {
    const U *steps;
    const U *normalization;
    const U *unigram;
    Counts counts[6];
};

struct Observation {
    U length, query, read, recurrence, successor, winner, winner_roots, output_roots;
    W score;
    int before[32];
};

__device__ U count_at(Counts table, U key, U token) {
    U low = 0, high = table.size;
    while (low < high) {
        const U mid = low + (high - low) / 2;
        const U *record = table.rows + 3 * mid;
        if (record[0] < key || (record[0] == key && record[1] < token)) low = mid + 1;
        else high = mid;
    }
    const U *record = table.rows + 3 * low;
    return low < table.size && record[0] == key && record[1] == token ? record[2] : 0;
}

__device__ U recurrence(const U *ids, U length, const U *roots, const U *g, const Trunk &trunk) {
    const U last = length - 1;
    const U prev = last ? last - 1 : 0;
    const U mid = last / 2;
    U packed = 0;
    for (U block = 0; block < 4; ++block) {
        const U shift = 4 * block;
        U accumulated = g[IDENTITY], prev_frame = g[IDENTITY], mid_frame = g[IDENTITY];
        for (U pos = 0; pos < length; ++pos) {
            const U root = (roots[ids[pos]] >> shift) & 15;
            accumulated = compose(g, trunk.steps[16 * block + root], accumulated);
            if (pos == prev) prev_frame = accumulated;
            if (pos == mid) mid_frame = accumulated;
        }
        const U previous = compose(g, accumulated, g[INVERSE + prev_frame]);
        const U middle = compose(g, accumulated, g[INVERSE + mid_frame]);
        const U first_root = (roots[ids[0]] >> shift) & 15;
        const U last_root = (roots[ids[last]] >> shift) & 15;
        const U prev_root = (roots[ids[prev]] >> shift) & 15;
        const U mid_root = (roots[ids[mid]] >> shift) & 15;
        int coordinates[8], first[8], prior[8], center[8], normalized[8];
        for (U d = 0; d < 8; ++d) coordinates[d] = static_cast<int>(g[8 * first_root + d]);
        action(g, accumulated, coordinates, first);
        for (U d = 0; d < 8; ++d) coordinates[d] = static_cast<int>(g[8 * prev_root + d]);
        action(g, previous, coordinates, prior);
        for (U d = 0; d < 8; ++d) coordinates[d] = static_cast<int>(g[8 * mid_root + d]);
        action(g, middle, coordinates, center);
        for (U d = 0; d < 8; ++d) {
            coordinates[d] = static_cast<int>(g[8 * last_root + d]) + first[d] + prior[d] + center[d];
        }
        const U snapped = snap(coordinates, 4, g);
        for (U d = 0; d < 8; ++d) coordinates[d] = static_cast<int>(g[8 * snapped + d]);
        action(g, trunk.normalization[block], coordinates, normalized);
        const U root = root_id(normalized, g);
        if (root == ABSENT) return ABSENT;
        packed |= root << shift;
    }
    return packed;
}

__global__ void pack_roots(const int *table, const U *g, U vocabulary, U *roots, U *status) {
    const U token = blockIdx.x * blockDim.x + threadIdx.x;
    if (token >= vocabulary) return;
    U packed = 0;
    for (U block = 0; block < 4; ++block) {
        const U root = root_id(table + token * 32 + block * 8, g);
        if (root == ABSENT) { atomicExch(status, 1u); return; }
        packed |= root << (4 * block);
    }
    roots[token] = packed;
}

__global__ void encode_prefixes(const int *table, const U *ids, U words, int *out, U *status) {
    const U i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= words) return;
    const U token = ids[i / 32];
    if (token >= 4096) { atomicExch(status, 1u); return; }
    out[i] = table[token * 32 + i % 32];
}

__global__ void context_vectors(const U *routes, const int *gates, const U *ids,
                                const int *input, const U *plans, const U *g, const U *spec,
                                const U *roots, Trunk trunk, int *out,
                                Observation *observations, U *status) {
    const U row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= spec[4] || *status != 0) return;
    int vector[MAX_WIDTH];
    if (!read_vector(input, ids, routes, gates, plans, g, spec, nullptr, row, 2, 0, 0, vector, nullptr)) {
        atomicExch(status, 2u);
        return;
    }
    const U length = plans[row * (3 + 4 * spec[6] + 16)];
    const U *prefix = ids + row * spec[2];
    U read_code = 0, read_roots[4];
    for (U block = 0; block < 4; ++block) {
        const U root = root_id(vector + 8 * block, g);
        if (root == ABSENT) { atomicExch(status, 3u); return; }
        read_roots[block] = root;
        read_code |= root << (4 * block);
    }
    const U rec_code = recurrence(prefix, length, roots, g, trunk);
    if (rec_code == ABSENT) { atomicExch(status, 4u); return; }
    const U query = prefix[length - 1];
    U successor = ABSENT;
    for (U pos = 0; pos + 1 < length; ++pos) {
        if (roots[prefix[pos]] == roots[query]) successor = prefix[pos + 1];
    }

    U best = trunk.unigram[0], best_unigram = 0;
    W best_score = 0;
    for (U token = 0; token < spec[0]; ++token) {
        const U q = count_at(trunk.counts[0], query, token);
        const U r = count_at(trunk.counts[1], read_code, token);
        if (token != successor && q == 0 && r == 0 &&
            count_at(trunk.counts[2], rec_code, token) == 0) continue;
        const W qr = count_at(trunk.counts[3], (query << 16) | read_code, token);
        const W qc = count_at(trunk.counts[4], (query << 16) | rec_code, token);
        const W rr = count_at(trunk.counts[5], (read_code << 16) | rec_code, token);
        const W score = (qr << 6) + (qc << 5) + (rr << 3) + (W(r) << 1) + q +
                        (token == successor ? 256u : 0u);
        const U unigram = trunk.unigram[token + 1];
        if (score > best_score || (score == best_score && score > 0 && unigram > best_unigram)) {
            best = token;
            best_score = score;
            best_unigram = unigram;
        }
    }
    U output_code = 0;
    for (U block = 0; block < 4; ++block) {
        const U destination = (roots[best] >> (4 * block)) & 15;
        const U frame = g[BFS + 16 * read_roots[block] + destination];
        int rotated[8];
        action(g, frame, vector + 8 * block, rotated);
        const U output_root = snap(rotated, 1, g);
        output_code |= output_root << (4 * block);
        for (U d = 0; d < 8; ++d) out[row * 32 + block * 8 + d] = static_cast<int>(g[8 * output_root + d]);
    }
    if (observations) {
        Observation &trace = observations[row];
        trace.length = length;
        trace.query = query;
        trace.read = read_code;
        trace.recurrence = rec_code;
        trace.successor = successor;
        trace.winner = best;
        trace.winner_roots = roots[best];
        trace.output_roots = output_code;
        trace.score = best_score;
        for (U d = 0; d < 32; ++d) trace.before[d] = vector[d];
    }
}

void require(bool condition, const char *message) {
    if (!condition) throw std::runtime_error(message);
}

void check(cudaError_t status) {
    if (status != cudaSuccess) throw std::runtime_error(cudaGetErrorString(status));
}

template <typename T> class Buffer {
public:
    T *ptr = nullptr;
    explicit Buffer(size_t size) { check(cudaMalloc(&ptr, size * sizeof(T))); }
    explicit Buffer(const std::vector<T> &words) : Buffer(words.size()) {
        check(cudaMemcpy(ptr, words.data(), words.size() * sizeof(T), cudaMemcpyHostToDevice));
    }
    ~Buffer() { cudaFree(ptr); }
    Buffer(const Buffer &) = delete;
    Buffer &operator=(const Buffer &) = delete;
    std::vector<T> read(size_t n) {
        std::vector<T> words(n);
        check(cudaMemcpy(words.data(), ptr, n * sizeof(T), cudaMemcpyDeviceToHost));
        return words;
    }
};

template <typename T = U> std::vector<T> words(std::istream &input, size_t size) {
    std::vector<T> result(size);
    require(static_cast<bool>(input.read(reinterpret_cast<char *>(result.data()),
                                         size * sizeof(T))), "truncated context packet");
    return result;
}

void run(const char *packet, const char *output, const char *trace) {
    static_assert(sizeof(U) == 4 && sizeof(int) == 4 && __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__);
    std::ifstream input(packet, std::ios::binary);
    char magic[8];
    require(static_cast<bool>(input.read(magic, 8)) && std::string(magic, 8) == "D25CTX01",
            "invalid context packet magic");
    const auto spec = words(input, 16);
    const U vocab = spec[0], context = spec[2], rows = spec[4];
    require(vocab == 4096 && spec[1] == 4 && context == 192 && rows > 0 && rows <= 4096 &&
            spec[5] == 5 && spec[6] == 286 && spec[8] == 2 && spec[9] == 8 && spec[10] == 1 &&
            spec[11] == 0 && spec[12] == 0 && spec[13] == 0 && spec[14] == 0 && spec[15] == 0,
            "unexpected frozen D25 context shape");
    const auto g = words(input, 5120);
    const auto table = words<int>(input, vocab * 32);
    const auto routes = words(input, 20);
    const auto gates = words<int>(input, 45);
    const auto steps = words(input, 64);
    const auto normalization = words(input, 4);
    const auto unigram = words(input, vocab + 1);
    std::array<std::vector<U>, 6> count_tables;
    for (auto &counts : count_tables) {
        const U count = words(input, 1).front();
        require(count > 0 && count <= 2000000, "unexpected D25 count table size");
        counts = words(input, 3 * count);
    }
    const size_t stride = 3 + 4 * spec[6] + 16;
    std::vector<U> row_ids, plans, ids;
    for (U row = 0; row < rows; ++row) {
        row_ids.push_back(words(input, 1).front());
        const auto plan = words(input, stride);
        const auto tokens = words(input, context);
        plans.insert(plans.end(), plan.begin(), plan.end());
        ids.insert(ids.end(), tokens.begin(), tokens.end());
    }
    require(input.peek() == std::char_traits<char>::eof() && !input.bad(), "trailing context packet");
    std::ifstream existing(output);
    require(!existing, "output must be fresh");
    if (trace) {
        std::ifstream existing_trace(trace);
        require(!existing_trace && std::string(output) != trace, "trace must be a fresh distinct path");
    }

    Buffer<U> gpu_spec(spec), gpu_g(g), gpu_routes(routes), gpu_plans(plans), gpu_ids(ids);
    Buffer<U> gpu_steps(steps), gpu_normalization(normalization), gpu_unigram(unigram);
    Buffer<int> gpu_gates(gates), gpu_table(table), gpu_input(ids.size() * 32), gpu_out(rows * 32);
    Buffer<U> gpu_roots(vocab), gpu_status(std::vector<U>{0});
    std::unique_ptr<Buffer<Observation>> gpu_trace;
    if (trace) gpu_trace = std::make_unique<Buffer<Observation>>(rows);
    Trunk trunk{gpu_steps.ptr, gpu_normalization.ptr, gpu_unigram.ptr, {}};
    std::array<std::unique_ptr<Buffer<U>>, 6> storage;
    for (size_t i = 0; i < count_tables.size(); ++i) {
        storage[i] = std::make_unique<Buffer<U>>(count_tables[i]);
        trunk.counts[i] = {storage[i]->ptr, static_cast<U>(count_tables[i].size() / 3)};
    }
    constexpr U threads = 64;
    pack_roots<<<(vocab + threads - 1) / threads, threads>>>(
        gpu_table.ptr, gpu_g.ptr, vocab, gpu_roots.ptr, gpu_status.ptr);
    check(cudaGetLastError());
    encode_prefixes<<<(ids.size() * 32 + threads - 1) / threads, threads>>>(
        gpu_table.ptr, gpu_ids.ptr, ids.size() * 32, gpu_input.ptr, gpu_status.ptr);
    check(cudaGetLastError());
    context_vectors<<<(rows + threads - 1) / threads, threads>>>(
        gpu_routes.ptr, gpu_gates.ptr, gpu_ids.ptr, gpu_input.ptr, gpu_plans.ptr,
        gpu_g.ptr, gpu_spec.ptr, gpu_roots.ptr, trunk, gpu_out.ptr,
        gpu_trace ? gpu_trace->ptr : nullptr, gpu_status.ptr);
    check(cudaGetLastError());
    require(gpu_status.read(1).front() == 0, "context CUDA rejected packet or read");
    const auto vectors = gpu_out.read(rows * 32);
    std::ofstream out(output);
    for (U row = 0; row < rows; ++row) {
        out << row_ids[row];
        for (U d = 0; d < 32; ++d) out << '\t' << vectors[row * 32 + d];
        out << '\n';
    }
    out.close();
    require(out.good(), "could not write context vectors");
    if (trace) {
        const auto observations = gpu_trace->read(rows);
        std::ofstream out_trace(trace);
        for (U row = 0; row < rows; ++row) {
            const auto &entry = observations[row];
            out_trace << row_ids[row] << '\t' << entry.length << '\t' << entry.query << '\t'
                      << entry.read << '\t' << entry.recurrence << '\t';
            if (entry.successor == ABSENT) out_trace << "NA";
            else out_trace << entry.successor;
            out_trace << '\t' << entry.winner << '\t' << entry.score << '\t'
                      << entry.winner_roots << '\t' << entry.output_roots;
            for (const int coordinate : entry.before) out_trace << '\t' << coordinate;
            out_trace << '\n';
        }
        out_trace.close();
        require(out_trace.good(), "could not write context observations");
    }
}

} // namespace

int main(int argc, char **argv) {
    if (argc != 3 && argc != 4) {
        std::cerr << "usage: d25-context PACKET OUT_TSV [TRACE_TSV]\n";
        return 1;
    }
    try {
        run(argv[1], argv[2], argc == 4 ? argv[3] : nullptr);
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}

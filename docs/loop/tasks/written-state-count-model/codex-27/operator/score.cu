// Independent AOT SM89 differential for the D27 scoring and release policy.
#include <cuda_runtime.h>

#include <array>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <memory>
#include <fstream>
#include <stdexcept>

namespace {
constexpr unsigned width = 192;
constexpr unsigned batch_size = 256;
constexpr unsigned no_score = UINT32_MAX;
constexpr unsigned radius = 16;
enum Outcome : unsigned { empty, selected, tie, collision };

struct Span { unsigned start, end; };
struct Slot { unsigned id, packed_root; };
struct Words { Span cue, query; };
struct Input {
    unsigned size, candidates, queries;
    Slot tape[width];
    Words words[width];
};
struct Evidence {
    unsigned literal, collision, distance, exact_selected, geometry_selected;
};
struct Output {
    unsigned exact, geometry;
    Evidence evidence[width];
};
static_assert(sizeof(Input) == (3 + width * 6) * sizeof(uint32_t));
static_assert(sizeof(Output) == (2 + width * 5) * sizeof(uint32_t));
__constant__ int coordinates[16][8];

__device__ unsigned size(Span span) {
    return span.end - span.start;
}

__device__ bool same_word(const Input& row, Span left, Span right) {
    if (size(left) != size(right)) return false;
    for (unsigned i = 0; i < size(left); ++i) {
        if (row.tape[left.start + i].id != row.tape[right.start + i].id) return false;
    }
    return true;
}

__device__ unsigned packed_distance(unsigned left, unsigned right) {
    unsigned distance = 0;
    for (unsigned block = 0; block < 4; ++block) {
        const auto* a = coordinates[left & 15];
        const auto* b = coordinates[right & 15];
        for (unsigned axis = 0; axis < 8; ++axis) {
            const int delta = a[axis] - b[axis];
            distance += delta * delta;
        }
        left >>= 4;
        right >>= 4;
    }
    return distance;
}

__device__ Evidence score(const Input& row, unsigned candidate) {
    Evidence evidence{0, 0, no_score, 0, 0};
    if (candidate >= row.candidates) return evidence;
    const Span cue = row.words[candidate].cue;
    for (unsigned i = 0; i < row.queries; ++i) {
        const Span query = row.words[i].query;
        if (size(cue) != size(query)) continue;
        if (same_word(row, cue, query)) {
            evidence.literal = 1;
            continue;
        }
        unsigned distance = 0;
        for (unsigned piece = 0; piece < size(cue); ++piece) {
            distance += packed_distance(row.tape[cue.start + piece].packed_root,
                                        row.tape[query.start + piece].packed_root);
        }
        if (distance == 0) {
            evidence.collision = 1;
        } else if (distance <= radius * size(cue) && distance < evidence.distance) {
            evidence.distance = distance;
        }
    }
    return evidence;
}

__device__ Outcome decide(const Input& row, Evidence* evidence, bool native) {
    bool any_literal = false;
    bool any_collision = false;
    for (unsigned i = 0; i < row.candidates; ++i) {
        any_literal |= evidence[i].literal != 0;
        any_collision |= evidence[i].collision != 0;
    }
    if (!any_literal && native && any_collision) return collision;
    if (!any_literal && !native) return empty;

    unsigned winner = width;
    unsigned best_distance = no_score;
    bool tied = false;
    for (unsigned i = 0; i < row.candidates; ++i) {
        const unsigned distance = any_literal
            ? (evidence[i].literal ? 0 : no_score)
            : evidence[i].distance;
        if (distance == no_score) continue;
        const Span cue = row.words[i].cue;
        if (winner == width ||
            distance * size(row.words[winner].cue) < best_distance * size(cue)) {
            winner = i;
            best_distance = distance;
            tied = false;
        } else if (distance * size(row.words[winner].cue) == best_distance * size(cue) &&
                   !same_word(row, cue, row.words[winner].cue)) {
            tied = true;
        }
    }
    if (winner == width) return empty;
    if (tied) return tie;
    for (unsigned i = 0; i < row.candidates; ++i) {
        if (same_word(row, row.words[i].cue, row.words[winner].cue)) {
            if (native) evidence[i].geometry_selected = 1;
            else evidence[i].exact_selected = 1;
        }
    }
    return selected;
}

__global__ void score_rows(const Input* input, Output* output) {
    const auto& row = input[blockIdx.x];
    auto& result = output[blockIdx.x];
    const auto candidate = threadIdx.x;
    result.evidence[candidate] = score(row, candidate);
    __syncthreads();
    if (candidate == 0) {
        result.exact = decide(row, result.evidence, false);
        result.geometry = decide(row, result.evidence, true);
    }
}

void check(cudaError_t status, const char* operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(status));
    }
}

template <typename T>
class DeviceBatch {
public:
    DeviceBatch() {
        check(cudaMalloc(reinterpret_cast<void**>(&data_), batch_size * sizeof(T)), "cudaMalloc");
    }
    ~DeviceBatch() { cudaFree(data_); }
    DeviceBatch(const DeviceBatch&) = delete;
    DeviceBatch& operator=(const DeviceBatch&) = delete;
    T* data() { return data_; }
private:
    T* data_ = nullptr;
};

bool valid(Span span, unsigned size) {
    return span.start > 0 && span.start < span.end && span.end <= size;
}

bool valid(const Input& row) {
    if (row.size == 0 || row.size > width || row.candidates > width || row.queries > width ||
        row.tape[0].id != 0) return false;
    for (unsigned i = 0; i < row.size; ++i) {
        if (row.tape[i].id >= 4096 || (i > 0 && row.tape[i].id < 2) ||
            row.tape[i].packed_root > UINT16_MAX) return false;
    }
    for (unsigned i = 0; i < row.candidates; ++i) {
        if (!valid(row.words[i].cue, row.size)) return false;
    }
    for (unsigned i = 0; i < row.queries; ++i) {
        if (!valid(row.words[i].query, row.size)) return false;
    }
    return true;
}

void run(const char* path) {
    const uint32_t endian = 1;
    if (*reinterpret_cast<const unsigned char*>(&endian) != 1)
        throw std::runtime_error("UORREL01 requires a little-endian host");
    std::ifstream source(path, std::ios::binary);
    char magic[8];
    int root_table[16][8];
    if (!source.read(magic, sizeof magic) || std::memcmp(magic, "UORREL01", 8) ||
        !source.read(reinterpret_cast<char*>(&root_table), sizeof root_table))
        throw std::runtime_error("invalid UORREL01 header");
    for (const auto& root : root_table) {
        unsigned populated = 0;
        for (int coordinate : root) {
            if (coordinate != -2 && coordinate != 0 && coordinate != 2)
                throw std::runtime_error("invalid D11 coordinate");
            populated += coordinate != 0;
        }
        if (populated != 2) throw std::runtime_error("invalid D11 root");
    }

    int device = 0;
    cudaDeviceProp properties{};
    check(cudaGetDevice(&device), "cudaGetDevice");
    check(cudaGetDeviceProperties(&properties, device), "cudaGetDeviceProperties");
    if (properties.major != 8 || properties.minor != 9)
        throw std::runtime_error("expected a native SM89 device");
    check(cudaMemcpyToSymbol(coordinates, root_table, sizeof root_table), "copy D11 roots");
    DeviceBatch<Input> device_inputs;
    DeviceBatch<Output> device_outputs;
    auto inputs = std::make_unique<std::array<Input, batch_size>>();
    auto outputs = std::make_unique<std::array<Output, batch_size>>();
    std::size_t rows = 0;
    while (true) {
        unsigned count = 0;
        for (; count < batch_size; ++count) {
            auto& row = (*inputs)[count];
            if (!source.read(reinterpret_cast<char*>(&row), sizeof row)) {
                if (source.eof() && source.gcount() == 0) break;
                throw std::runtime_error("could not read complete UORREL01 row");
            }
            if (!valid(row)) throw std::runtime_error("invalid UORREL01 row");
        }
        if (count == 0) break;
        check(cudaMemcpy(device_inputs.data(), inputs->data(), count * sizeof(Input),
                         cudaMemcpyHostToDevice), "copy input batch");
        score_rows<<<count, width>>>(device_inputs.data(), device_outputs.data());
        check(cudaGetLastError(), "launch relation score");
        check(cudaMemcpy(outputs->data(), device_outputs.data(), count * sizeof(Output),
                         cudaMemcpyDeviceToHost), "copy output batch");
        if (!std::cout.write(reinterpret_cast<const char*>(outputs->data()), count * sizeof(Output)))
            throw std::runtime_error("could not write output batch");
        rows += count;
    }
    if (rows == 0) throw std::runtime_error("input contains no UORREL01 rows");
    if (!std::cout.flush()) throw std::runtime_error("could not flush output batches");
    std::cerr << "SM89 " << properties.name << ", scored " << rows << " rows\n";
}
}  // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::cerr << "usage: d27-score INPUT > OUTPUT\n";
        return 1;
    }
    try {
        run(argv[1]);
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}

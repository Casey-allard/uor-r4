#include <cuda_runtime.h>

#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

using Word = uint32_t;
using Distance = uint64_t;
constexpr Word PacketMagic = 0x31353344;
constexpr Word Train = 1;
constexpr Word Score = 2;
constexpr Word Nil = UINT32_MAX;
constexpr Word Width = 192;
constexpr Word Dimensions = 32;
constexpr Word MaxRows = 4096;
constexpr Word MaxPrototypes = 8192;
constexpr Word MaxPairs = 1000000;
constexpr char BankMagic[] = "D35PAIR1";
constexpr char ScoreMagic[] = "D35RANK1";

struct Candidate {
    Word start, end;
    int32_t local[Dimensions];
    int32_t contextual[Dimensions];
};
struct Row {
    Word first, count, singleton, pair_first;
};
struct TrainingPair {
    Word gold, rival;
};
struct Geometry {
    int32_t local[Dimensions];
    int64_t contextual[Dimensions];
};
struct Distances {
    Distance forward, reverse;
};
struct Winners {
    Word forward, reverse;
};

static_assert(__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__);
static_assert(sizeof(Candidate) == 264);
static_assert(sizeof(Geometry) == 384);
static_assert(sizeof(Distances) == 16);

__device__ Geometry difference(const Candidate& left, const Candidate& right) {
    Geometry result;
    for (Word d = 0; d < Dimensions; ++d) {
        result.local[d] = left.local[d] - right.local[d];
        result.contextual[d] = int64_t(left.contextual[d]) - right.contextual[d];
    }
    return result;
}

__global__ void fit_bank(const Candidate* candidates, const TrainingPair* input,
                         Word count, Geometry* bank) {
    const Word i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const TrainingPair pair = input[i];
    bank[i] = difference(candidates[pair.gold], candidates[pair.rival]);
}

__device__ Distance magnitude(int64_t value) {
    return value < 0 ? Distance(-value) : Distance(value);
}

__device__ Distances nearest(const Geometry& query, const Geometry* bank, Word count) {
    Distances best{UINT64_MAX, UINT64_MAX};
    for (Word p = 0; p < count; ++p) {
        Distances next{0, 0};
        for (Word d = 0; d < Dimensions; ++d) {
            next.forward += magnitude(int64_t(query.local[d]) - bank[p].local[d]);
            next.forward += magnitude(query.contextual[d] - bank[p].contextual[d]);
            next.reverse += magnitude(int64_t(query.local[d]) + bank[p].local[d]);
            next.reverse += magnitude(query.contextual[d] + bank[p].contextual[d]);
        }
        if (next.forward < best.forward) best.forward = next.forward;
        if (next.reverse < best.reverse) best.reverse = next.reverse;
    }
    return best;
}

__global__ void score_bank(const Row* rows, const Candidate* candidates, const Geometry* bank,
                           Word prototypes, Distances* pairs, Winners* winners) {
    const Row row = rows[blockIdx.x];
    const Word i = threadIdx.x;
    if (row.count <= 1) {
        if (i == 0) winners[blockIdx.x] = {row.singleton, row.singleton};
        return;
    }

    __shared__ Winners picked[Width];
    if (i < row.count) {
        const Candidate& candidate = candidates[row.first + i];
        bool forward = true;
        bool reverse = true;
        Word out = row.pair_first + i * (row.count - 1);
        for (Word j = 0; j < row.count; ++j) {
            if (i == j) continue;
            const Geometry query = difference(candidate, candidates[row.first + j]);
            const Distances rank = nearest(query, bank, prototypes);
            pairs[out++] = rank;
            forward &= rank.forward < rank.reverse;
            reverse &= rank.reverse < rank.forward;
        }
        picked[i] = {forward ? candidate.start : Nil, reverse ? candidate.start : Nil};
    }
    __syncthreads();
    if (i == 0) {
        Winners winner{Nil, Nil};
        for (Word j = 0; j < row.count; ++j) {
            if (picked[j].forward != Nil) winner.forward = picked[j].forward;
            if (picked[j].reverse != Nil) winner.reverse = picked[j].reverse;
        }
        winners[blockIdx.x] = winner;
    }
}

[[noreturn]] static void fail(const char* message) {
    std::fprintf(stderr, "%s\n", message);
    std::exit(2);
}
static void check(cudaError_t status) {
    if (status != cudaSuccess) fail(cudaGetErrorString(status));
}
template<class T> static void read(FILE* input, T* records, size_t n) {
    if (std::fread(records, sizeof(T), n, input) != n) fail("truncated input");
}
static void require_end(FILE* input) {
    if (std::fgetc(input) != EOF || std::ferror(input)) fail("extra or unreadable input");
}
template<class T> static void write(const T* records, size_t n) {
    if (std::fwrite(records, sizeof(T), n, stdout) != n) fail("output write failed");
}
template<class T> static T* allocate(size_t count) {
    T* device;
    check(cudaMalloc(&device, (count == 0 ? 1 : count) * sizeof(T)));
    return device;
}
template<class T> static T* upload(const std::vector<T>& records) {
    T* device = allocate<T>(records.size());
    if (!records.empty()) {
        check(cudaMemcpy(device, records.data(), records.size() * sizeof(T), cudaMemcpyHostToDevice));
    }
    return device;
}
template<class T> static void download(std::vector<T>& records, T* device) {
    if (!records.empty()) {
        check(cudaMemcpy(records.data(), device, records.size() * sizeof(T), cudaMemcpyDeviceToHost));
    }
    check(cudaFree(device));
}

static std::vector<Geometry> load_bank(const char* path) {
    FILE* input = std::fopen(path, "rb");
    if (input == nullptr) fail("could not open bank");
    char magic[8];
    Word count;
    read(input, magic, sizeof(magic));
    read(input, &count, 1);
    if (std::memcmp(magic, BankMagic, sizeof(magic)) || count == 0 || count > MaxPrototypes) {
        fail("invalid bank header");
    }
    std::vector<Geometry> bank(count);
    read(input, bank.data(), bank.size());
    require_end(input);
    std::fclose(input);
    for (const Geometry& prototype : bank) {
        for (Word d = 0; d < Dimensions; ++d) {
            if (prototype.local[d] < -65535 || prototype.local[d] > 65535 ||
                prototype.contextual[d] < -4294967295LL ||
                prototype.contextual[d] > 4294967295LL) {
                fail("prototype outside native coordinate difference ranges");
            }
        }
    }
    return bank;
}

int main(int argc, char** argv) {
    Word header[3];
    read(stdin, header, 3);
    const Word mode = header[1], n = header[2];
    if (header[0] != PacketMagic || n == 0 || n > MaxRows ||
        ((mode != Train || argc != 1) && (mode != Score || argc != 2))) {
        fail("invalid packet header or bank argument");
    }

    std::vector<Row> rows(n);
    std::vector<Candidate> candidates;
    std::vector<TrainingPair> train;
    Word pairs = 0;
    for (Row& row : rows) {
        Word fields[2];
        read(stdin, fields, 2);
        const Word count = fields[0], marker = fields[1];
        if (count > Width || (mode == Train && count < 2)) fail("invalid candidate count");
        row = {Word(candidates.size()), count, Nil, pairs};
        if (count >= 2) pairs += count * (count - 1);
        if (mode == Score && pairs > MaxPairs) fail("too many scored pairs");
        Word gold = Nil;
        for (Word i = 0; i < count; ++i) {
            Candidate candidate;
            read(stdin, &candidate, 1);
            if (candidate.start == 0 || candidate.start >= candidate.end || candidate.end > Width ||
                (i > 0 && candidate.start <= candidates.back().start)) {
                fail("invalid or unordered original candidate span");
            }
            for (int32_t coordinate : candidate.local) {
                if (coordinate < INT16_MIN || coordinate > INT16_MAX) {
                    fail("local coordinate outside native i16 range");
                }
            }
            if (candidate.start == marker) gold = row.first + i;
            candidates.push_back(candidate);
        }
        if (mode == Train) {
            if (gold == Nil || train.size() + count - 1 > MaxPrototypes) {
                fail("missing training target or too many prototypes");
            }
            for (Word i = row.first; i < row.first + count; ++i) {
                if (i != gold) train.push_back({gold, i});
            }
        } else if (marker != Nil) {
            if (count != 1 || gold == Nil) fail("invalid frozen singleton choice");
            row.singleton = marker;
        }
    }
    require_end(stdin);

    if (mode == Train) {
        Candidate* device_candidates = upload(candidates);
        TrainingPair* device_train = upload(train);
        Geometry* device_bank = allocate<Geometry>(train.size());
        const Word count = Word(train.size());
        fit_bank<<<(count + 63) / 64, 64>>>(device_candidates, device_train, count, device_bank);
        check(cudaGetLastError());
        std::vector<Geometry> bank(count);
        download(bank, device_bank);
        check(cudaFree(device_train));
        check(cudaFree(device_candidates));
        write(BankMagic, 8);
        write(&count, 1);
        write(bank.data(), bank.size());
    } else {
        const std::vector<Geometry> bank = load_bank(argv[1]);
        Candidate* device_candidates = upload(candidates);
        Row* device_rows = upload(rows);
        Geometry* device_bank = upload(bank);
        Distances* device_distances = allocate<Distances>(pairs);
        Winners* device_winners = allocate<Winners>(n);
        score_bank<<<n, Width>>>(device_rows, device_candidates, device_bank, Word(bank.size()),
                                 device_distances, device_winners);
        check(cudaGetLastError());
        std::vector<Distances> distances(pairs);
        std::vector<Winners> winners(n);
        download(winners, device_winners);
        download(distances, device_distances);
        check(cudaFree(device_bank));
        check(cudaFree(device_rows));
        check(cudaFree(device_candidates));

        write(ScoreMagic, 8);
        write(&n, 1);
        for (Word row_index = 0; row_index < n; ++row_index) {
            const Row& row = rows[row_index];
            const Winners winner = winners[row_index];
            const Word fields[] = {row.count, winner.forward, winner.reverse};
            write(fields, 3);
            for (Word i = 0; i < row.count; ++i) {
                const Candidate& candidate = candidates[row.first + i];
                const Word span[] = {candidate.start, candidate.end};
                write(span, 2);
                if (row.count >= 2) {
                    write(distances.data() + row.pair_first + i * (row.count - 1), row.count - 1);
                }
            }
        }
    }
    return std::fflush(stdout) == 0 ? 0 : 2;
}

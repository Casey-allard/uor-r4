// Ahead-of-time SM89 diagnostic backend for the ordered role scalar operator.
#include <cuda_runtime.h>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

using Word = uint32_t;
constexpr Word Width = 192, Vocab = 4096, Coords = 32, Classes = 3, Context = 12;
constexpr int Limit = 48;
constexpr Word Nil = UINT32_MAX, Abi = 0x324c524f;
enum Operation : Word { Signature, Train, Public, Bounded, ClampFixture };
static bool is_fit(Operation op) { return op == Train || op == ClampFixture; }
struct Span { Word start, end; };
struct Site { Word root, cls; };
struct Row {
    Word ids[Width], size;
    Span content, region, requested, excluded;
};
struct Tables {
    Site sites[Vocab];
    int8_t roots[16][8];
    uint8_t axes[64][8];
};
struct Bank { int16_t coord[Classes][Coords]; };
struct Feature { int16_t coord[Coords] = {}; };
struct Candidate {
    Span span;
    Word admission;
    Word context[2 * Context]; // Original IDs; missing/structural uses Nil.
    int32_t feature[Coords];
    Word distances[Classes], winner, evidence;
    Word gold, rival, applied, clamp_low, clamp_high;
    int32_t margin; // Rival distance minus gold distance, TRAIN only.
    int32_t updated_gold[Coords], updated_rival[Coords];
};
struct Trace {
    Word decision[8];
    Word labels_seen;
    Candidate slots[Width];
};
static_assert(sizeof(Tables) == 33408 && sizeof(Row) == 804);
static_assert(sizeof(Bank) == 192 && sizeof(Candidate) == 536 && sizeof(Trace) == 102948);

__host__ __device__ bool same(Span a, Span b) {
    return a.start == b.start && a.end == b.end;
}
__host__ __device__ bool includes(Span region, Span span) {
    return span.start >= region.start && span.end <= region.end;
}

__device__ Feature feature(const Tables& tables, const Row& row, Span span, Word mode,
                        Candidate& out) {
    Feature point;
    for (Word side = 0; side < 2; ++side) {
        for (Word distance = 0; distance < Context; ++distance) {
            Word offset = side == 0 ? span.start - 1 - distance : span.end + distance;
            Word state = mode == 0 ? 0 : (side == 0 ? 1 : 33) + distance;
            Word slot = side * Context + distance;
            if (offset >= row.size || !includes(row.region, {offset, offset + 1}) ||
                !includes(row.content, {offset, offset + 1})) {
                out.context[slot] = Nil;
                continue;
            }
            Word id = row.ids[offset];
            out.context[slot] = id;
            Word packed = tables.sites[id].root;
            for (Word block = 0; block < 4; ++block) {
                const auto& root = tables.roots[(packed >> (4 * block)) & 15];
                for (Word axis = 0; axis < 8; ++axis) {
                    Word code = tables.axes[state][axis];
                    int coordinate = root[code & 7];
                    point.coord[8 * block + axis] += (code & 8) ? -coordinate : coordinate;
                }
            }
        }
    }
    for (Word axis = 0; axis < Coords; ++axis) out.feature[axis] = point.coord[axis];
    return point;
}

__device__ void score(const Bank& bank, const Feature& point, Candidate& out) {
    Word first = UINT32_MAX, second = UINT32_MAX, nearest = Nil;
    for (Word cls = 0; cls < Classes; ++cls) {
        Word distance = 0;
        for (Word axis = 0; axis < Coords; ++axis) {
            int delta = point.coord[axis] - bank.coord[cls][axis];
            distance += delta < 0 ? -delta : delta;
        }
        out.distances[cls] = distance;
        if (distance < first) {
            second = first;
            first = distance;
            nearest = cls;
        } else if (distance < second) {
            second = distance;
        }
    }
    out.winner = first == second ? Nil : nearest;
    out.evidence = second - first;
}

__device__ int unit(int delta) { return (delta > 0) - (delta < 0); }

__device__ int16_t clamp(int coordinate, Candidate& trace) {
    if (coordinate < -Limit) {
        ++trace.clamp_low;
        return -Limit;
    }
    if (coordinate > Limit) {
        ++trace.clamp_high;
        return Limit;
    }
    return static_cast<int16_t>(coordinate);
}

__device__ void update(Bank& bank, const Feature& point, Candidate& trace) {
    Word gold = trace.gold;
    Word rival = Nil, distance = UINT32_MAX;
    for (Word cls = 0; cls < Classes; ++cls) {
        if (cls != gold && trace.distances[cls] < distance) {
            rival = cls;
            distance = trace.distances[cls];
        }
    }
    trace.rival = rival;
    trace.margin = static_cast<int>(distance) - static_cast<int>(trace.distances[gold]);
    if (trace.margin >= 1) return;
    trace.applied = 1;
    for (Word axis = 0; axis < Coords; ++axis) {
        int target = bank.coord[gold][axis];
        int other = bank.coord[rival][axis];
        bank.coord[gold][axis] = clamp(target + unit(point.coord[axis] - target), trace);
        bank.coord[rival][axis] = clamp(other - unit(point.coord[axis] - other), trace);
        trace.updated_gold[axis] = bank.coord[gold][axis];
        trace.updated_rival[axis] = bank.coord[rival][axis];
    }
}

struct Rank {
    Span span = {Nil, Nil};
    int best = INT_MIN, next = INT_MIN;

    __device__ void consider(Span candidate, int evidence) {
        if (evidence > best) {
            next = best;
            best = evidence;
            span = candidate;
        } else if (evidence > next) {
            next = evidence;
        }
    }

    __device__ Span winner() const {
        return best > next ? span : Span{Nil, Nil};
    }
};

__device__ void emit_decision(const Rank (&ranks)[2], Word* out) {
    for (Word role = 0; role < 2; ++role) {
        Span winner = ranks[role].winner();
        out[2 * role] = winner.start;
        out[2 * role + 1] = winner.end;
        out[4 + role] = static_cast<Word>(ranks[role].best);
        out[6 + role] = static_cast<Word>(ranks[role].next);
    }
}

__device__ void transition(const Tables& tables, const Row& row, Word mode, const Bank& bank,
                           Bank* training, Trace& trace) {
    Rank ranks[2];
    for (Word start = row.content.start; start < row.content.end; ++start) {
        if (tables.sites[row.ids[start]].cls != 1) continue;
        Word end = start + 1;
        while (end < row.content.end && tables.sites[row.ids[end]].cls == 2) ++end;
        Span span{start, end};
        Candidate& candidate = trace.slots[start];
        candidate.span = span;
        candidate.admission = includes(row.region, span) ? 1 : 0;
        if (!candidate.admission) continue;
        const Feature point = feature(tables, row, span, mode, candidate);
        score(bank, point, candidate);
        if (candidate.winner < 2) ranks[candidate.winner].consider(span, candidate.evidence);
        if (training != nullptr) {
            candidate.gold = 2;
            if (same(span, row.requested)) {
                candidate.gold = 0;
                trace.labels_seen |= 1;
            } else if (same(span, row.excluded)) {
                candidate.gold = 1;
                trace.labels_seen |= 2;
            }
            update(*training, point, candidate);
        }
    }

    emit_decision(ranks, trace.decision);
}

__global__ void infer(const Tables* tables, const Row* rows, const Bank* bank, Word mode,
                      Word count, Trace* trace) {
    Word row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row < count) transition(*tables, rows[row], mode, *bank, nullptr, trace[row]);
}

__global__ void train(const Tables* tables, const Row* rows, Bank* bank, Word mode,
                      Word count, Trace* trace) {
    // One serial transition walks the presealed row/start order on every pass.
    for (Word pass = 0; pass < 2; ++pass) {
        for (Word row = 0; row < count; ++row) {
            transition(*tables, rows[row], mode, *bank, bank, trace[pass * count + row]);
        }
    }
}

struct RankedCandidate {
    Span span;
    Word winner, evidence;
};
struct FixedPair { RankedCandidate candidates[2]; };
static_assert(sizeof(FixedPair) == 32);

__global__ void fixed_rank(const FixedPair* pairs, Word count, Word* result) {
    Word row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= count) return;
    Rank ranks[2];
    for (const auto& candidate : pairs[row].candidates) {
        if (candidate.winner < 2) {
            ranks[candidate.winner].consider(candidate.span, candidate.evidence);
        }
    }
    emit_decision(ranks, result + 8 * row);
}

[[noreturn]] static void reject(const char* reason) {
    std::fprintf(stderr, "%s\n", reason);
    std::exit(2);
}

static void check(cudaError_t status) {
    if (status != cudaSuccess) reject(cudaGetErrorString(status));
}

template<class T> static void read(T* target, size_t count = 1) {
    if (std::fread(target, sizeof(T), count, stdin) != count) reject("short CUDA request");
}

template<class T> static void write(const T* source, size_t count = 1) {
    if (std::fwrite(source, sizeof(T), count, stdout) != count) reject("short CUDA response write");
}

template<class T> static T* allocate(size_t count) {
    T* result;
    check(cudaMalloc(&result, count * sizeof(T)));
    return result;
}

template<class T> static T* upload(const T* source, size_t count = 1) {
    T* result = allocate<T>(count);
    check(cudaMemcpy(result, source, count * sizeof(T), cudaMemcpyHostToDevice));
    return result;
}

static bool valid(Span span, Word size) {
    return span.start <= span.end && span.end <= size;
}

static bool present(Span span) { return !same(span, {Nil, Nil}); }

static void validate(const Tables& tables, const Bank& bank, const std::vector<Row>& rows,
                     Operation op) {
    for (auto site : tables.sites) {
        if (site.root >= 65536 || site.cls > 2) reject("invalid packed root or word class");
    }
    for (const auto& root : tables.roots) {
        for (auto coordinate : root) {
            if (coordinate != -2 && coordinate != 0 && coordinate != 2)
                reject("invalid orbit root coordinate");
        }
    }
    for (Word state = 0; state < 64; ++state) {
        Word sources = 0;
        for (Word axis = 0; axis < 8; ++axis) {
            Word code = tables.axes[state][axis];
            if (code > 15 || (state == 0 && code != axis)) reject("invalid signed-axis code");
            sources |= 1 << (code & 7);
        }
        if (sources != 255) reject("signed-axis row is not bijective");
    }
    for (const auto& point : bank.coord) {
        for (auto coordinate : point) {
            if (coordinate < -Limit || coordinate > Limit) reject("prototype coordinate out of range");
            if ((op == Train || op == Signature) && coordinate != 0)
                reject("fit and signature operations require zero initial prototypes");
        }
    }
    for (const auto& row : rows) {
        if (row.size < 1 || row.size > Width || row.ids[0] != 0 ||
            !valid(row.content, row.size) || row.content.start == 0 ||
            !valid(row.region, row.size)) reject("invalid tape, framing or eligibility interval");
        for (Word i = 1; i < row.size; ++i) {
            if (row.ids[i] < 2 || row.ids[i] >= Vocab) reject("invalid original token ID");
        }
        for (Word i = row.size; i < Width; ++i) {
            if (row.ids[i] != 0) reject("nonzero tape padding");
        }
        if ((is_fit(op) || op == Public) && !same(row.region, {0, row.size}))
            reject("fit and public operations require full tape eligibility");
        if (!is_fit(op) && (present(row.requested) || present(row.excluded)))
            reject("role labels are only accepted in fit");
        const Span labels[] = {row.requested, row.excluded};
        for (auto span : labels) {
            if (present(span) && (!valid(span, row.size) || span.start == span.end ||
                                  !includes(row.content, span))) reject("invalid TRAIN label");
        }
        if (present(row.requested) && same(row.requested, row.excluded))
            reject("duplicate TRAIN label");
    }
}

static int rerank_fixed() {
    Word header[2];
    read(header, 2);
    Word count = header[1];
    if (header[0] != Abi || count == 0 || count > 8192) reject("invalid fixed-rank header");
    std::vector<FixedPair> pairs(count);
    read(pairs.data(), count);
    if (std::fgetc(stdin) != EOF || std::ferror(stdin)) reject("unexpected fixed-rank tail");
    for (const auto& pair : pairs) {
        if (same(pair.candidates[0].span, pair.candidates[1].span))
            reject("fixed-rank pair repeats a span");
        for (const auto& candidate : pair.candidates) {
            if (!valid(candidate.span, Width) || candidate.span.start == candidate.span.end ||
                (candidate.winner >= Classes && candidate.winner != Nil) ||
                candidate.evidence > 3072 || ((candidate.winner == Nil) != (candidate.evidence == 0)))
                reject("invalid fixed-rank candidate");
        }
    }
    auto* device_pairs = upload(pairs.data(), count);
    auto* device_result = allocate<Word>(8 * count);
    fixed_rank<<<(count + 63) / 64, 64>>>(device_pairs, count, device_result);
    check(cudaGetLastError());
    std::vector<Word> result(8 * count);
    check(cudaMemcpy(result.data(), device_result, result.size() * sizeof(Word), cudaMemcpyDeviceToHost));
    check(cudaFree(device_result));
    check(cudaFree(device_pairs));
    write(result.data(), result.size());
    return 0;
}

int main(int argc, char** argv) {
    if (argc == 2 && std::strcmp(argv[1], "--fixed") == 0) return rerank_fixed();
    if (argc != 1) reject("invalid CUDA operation arguments");
    Word header[4];
    read(header, 4);
    Word count = header[3], mode = header[2];
    if (header[0] != Abi || header[1] > ClampFixture || mode > 1 || count < 1 || count > 8192)
        reject("invalid CUDA request header");
    auto op = static_cast<Operation>(header[1]);
    if (op == ClampFixture && count != 1) reject("clamp fixture requires one row");
    Tables tables;
    Bank bank;
    read(&tables);
    read(&bank);
    std::vector<Row> rows(count);
    read(rows.data(), count);
    if (std::fgetc(stdin) != EOF || std::ferror(stdin)) reject("unexpected CUDA request tail");
    validate(tables, bank, rows, op);

    auto* device_tables = upload(&tables);
    auto* device_rows = upload(rows.data(), count);
    auto* device_bank = upload(&bank);
    size_t total = (is_fit(op) ? 2 : 1) * count;
    auto* device_trace = allocate<Trace>(total);
    check(cudaMemset(device_trace, 0, total * sizeof(Trace)));

    cudaEvent_t start, end;
    check(cudaEventCreate(&start));
    check(cudaEventCreate(&end));
    check(cudaEventRecord(start));
    if (is_fit(op)) {
        train<<<1, 1>>>(device_tables, device_rows, device_bank, mode, count, device_trace);
    } else {
        infer<<<(count + 63) / 64, 64>>>(device_tables, device_rows, device_bank, mode, count,
                                        device_trace);
    }
    check(cudaGetLastError());
    check(cudaEventRecord(end));
    check(cudaEventSynchronize(end));
    float elapsed_ms;
    check(cudaEventElapsedTime(&elapsed_ms, start, end));

    std::vector<Trace> traces(total);
    check(cudaMemcpy(traces.data(), device_trace, total * sizeof(Trace), cudaMemcpyDeviceToHost));
    check(cudaMemcpy(&bank, device_bank, sizeof(Bank), cudaMemcpyDeviceToHost));
    check(cudaFree(device_trace));
    check(cudaFree(device_bank));
    check(cudaFree(device_rows));
    check(cudaFree(device_tables));
    check(cudaEventDestroy(start));
    check(cudaEventDestroy(end));

    if (is_fit(op)) {
        for (size_t i = 0; i < total; ++i) {
            const Row& row = rows[i % count];
            Word required = present(row.requested) | (present(row.excluded) << 1);
            if (traces[i].labels_seen != required) reject("TRAIN label did not enumerate");
        }
    }
    std::fprintf(stderr, "device_ms=%.6f bank_bytes=%zu trace_bytes=%zu allocated_bytes=%zu\n",
                 elapsed_ms, sizeof(Bank), total * sizeof(Trace),
                 sizeof(Tables) + sizeof(Bank) + count * sizeof(Row) + total * sizeof(Trace));
    write(&bank);
    for (const auto& trace : traces) {
        write(trace.decision, 8);
        Word candidates = 0;
        for (const auto& candidate : trace.slots) candidates += candidate.span.end != 0;
        write(&candidates);
        for (const auto& candidate : trace.slots) {
            if (candidate.span.end != 0) write(&candidate);
        }
    }
    return 0;
}

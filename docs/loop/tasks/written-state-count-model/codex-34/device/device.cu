// Frozen D26 role evidence over original prefix IDs, derived from the D19 operator.
#include <cuda_runtime.h>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>

using Word = uint32_t;
constexpr Word Vocab = 4096, Width = 192, Offsets = 6, Roots = 65537;
constexpr Word Nil = UINT32_MAX, Abi = 0x31443452, ResultWidth = 8 + Width * 5;
constexpr Word SentenceFence = 1;
struct Site { Word root, cls, assertion_cls, fences, lexical; };
struct Cell { Word key, counts[3]; };
struct Span { Word start, end; };
struct Row { Word ids[Width], size; Span content; };
struct Rank {
    Span span{Nil, Nil};
    int best = INT_MIN, next = INT_MIN;
    __device__ void add(Span candidate, int score) {
        if (score > best) {
            next = best;
            best = score;
            span = candidate;
        } else if (score > next) {
            next = score;
        }
    }
};
static_assert(sizeof(Row) == (Width + 3) * sizeof(Word));

__device__ const Cell* lookup(const Cell* cells, Word count, Word key) {
    Word first = 0, end = count;
    while (first < end) {
        Word middle = (first + end) / 2;
        if (cells[middle].key < key) first = middle + 1;
        else end = middle;
    }
    return first < count && cells[first].key == key ? cells + first : nullptr;
}

// Signed cell evidence has magnitude at most 13*total. Its quotient by
// total+2 therefore fits four bits. Match Rust's truncation toward zero.
__device__ int cell_quotient(int evidence, int denominator) {
    int magnitude = evidence < 0 ? -evidence : evidence;
    int quotient = 0;
    for (int bit = 3; bit >= 0; --bit) {
        int threshold = denominator << bit;
        if (magnitude >= threshold) {
            magnitude -= threshold;
            quotient |= 1 << bit;
        }
    }
    return evidence < 0 ? -quotient : quotient;
}

__device__ void add_root(const Cell* cells, Word count, Word offset, Word root, int (&result)[2]) {
    const int weights[Offsets] = {1, 2, 4, 4, 2, 1};
    const Cell* cell = lookup(cells, count, offset * Roots + root);
    if (cell == nullptr) return;
    const int total = cell->counts[0] + cell->counts[1] + cell->counts[2];
    for (Word role = 0; role < 2; ++role) {
        const int signed_evidence = 16 * static_cast<int>(cell->counts[role + 1]) - 3 * total;
        result[role] += weights[offset] * cell_quotient(signed_evidence, total + 2);
    }
}

__device__ void scores(const Site* sites, const Cell* cells, Word count,
                       const Row& row, Span span, Span context, int (&result)[2]) {
    const Word at[Offsets] = {span.start - 3, span.start - 2, span.start - 1,
                             span.end, span.end + 1, span.end + 2};
    for (Word i = 0; i < Offsets; ++i) {
        const Word root = at[i] >= context.start && at[i] < context.end
                              ? sites[row.ids[at[i]]].root : Roots - 1;
        add_root(cells, count, i, root, result);
    }
}


__device__ Word native_end(const Site* sites, const Row& row, Word start, bool assertion) {
    Word end = start + 1;
    while (end < row.size) {
        const Site site = sites[row.ids[end]];
        if ((assertion ? site.assertion_cls : site.cls) != 2) break;
        ++end;
    }
    return end;
}

__device__ bool lexical_continuation(Site site) {
    return site.lexical == 2 && site.assertion_cls == 2;
}

__device__ Word detached_end(const Site* sites, const Row& row, Word start) {
    if (sites[row.ids[start]].lexical != 1 || start + 1 >= row.content.end ||
        !lexical_continuation(sites[row.ids[start + 1]])) return Nil;
    Word end = start + 2;
    while (end < row.size && lexical_continuation(sites[row.ids[end]])) ++end;
    return end <= row.content.end ? end : Nil;
}

__device__ bool matches(const Row& row, Span query, Span fact) {
    if (query.end - query.start != fact.end - fact.start) return false;
    for (Word i = 0; i < query.end - query.start; ++i)
        if (row.ids[query.start + i] != row.ids[fact.start + i]) return false;
    return true;
}

__device__ Span earlier_head(const Site* sites, const Row& row, const Word* detached,
                             Span query) {
    if (query.start < row.content.start || query.end > row.content.end) return {Nil, Nil};
    const Word first = row.content.start > 1 ? row.content.start : 1;
    for (Word start = first; start < query.start; ++start) {
        if (!(sites[row.ids[start - 1]].fences & SentenceFence)) continue;
        if (sites[row.ids[start]].assertion_cls == 1) {
            const Word end = native_end(sites, row, start, true);
            if (end <= query.start && matches(row, query, {start, end})) return {start, end};
            if (end <= row.content.end) continue;
        }
        if (detached == nullptr || start <= row.content.start) continue;
        const Word end = detached[start];
        if (end <= query.start && matches(row, query, {start, end})) return {start, end};
    }
    return {Nil, Nil};
}

__device__ Word rank(const Site* sites, const Cell* cells, Word count, const Row& row,
                      Span span, Span witness, Rank (&ranks)[2]) {
    int evidence[2]{};
    scores(sites, cells, count, row, span, {0, row.size}, evidence);
    Word flags = 0;
    if (witness.start != Nil) {
        ranks[0].add(span, evidence[0] - evidence[1]);
        flags |= 1;
    }
    if (evidence[1] > evidence[0]) {
        ranks[1].add(span, evidence[1]);
        flags |= 2;
    }
    return flags;
}

__device__ void detached_record(Word* record, Span query, Span witness, Word flags) {
    record[0] = query.end;
    record[1] = witness.start;
    record[2] = witness.end;
    record[4] = flags;
}

__global__ void predict(const Site* sites, const Cell* cells, Word count, const Row* rows,
                        Word n, Word mode, Word* result) {
    const Word i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const Row& row = rows[i];
    Word* out = result + ResultWidth * i;
    Word detached[Width];
    for (Word start = 0; start < Width; ++start) {
        const Word end = mode == 8 && start >= row.content.start && start < row.content.end
                             ? detached_end(sites, row, start) : Nil;
        detached[start] = end;
        Word* record = out + 8 + 5 * start;
        record[0] = Nil;
        record[1] = Nil;
        record[2] = Nil;
        record[3] = end != Nil && start > row.content.start &&
                            (sites[row.ids[start - 1]].fences & SentenceFence)
                        ? end : Nil;
        record[4] = 0;
    }

    Rank ranks[2];
    // Preserve the ordering of the inherited iterator before adding endpoints.
    for (Word start = 0; start < row.size; ++start) {
        if (sites[row.ids[start]].cls != 1) continue;
        const Span span{start, native_end(sites, row, start, false)};
        const Span witness = earlier_head(sites, row, detached, span);
        const Word flags = rank(sites, cells, count, row, span, witness, ranks);
        if (detached[start] == span.end)
            detached_record(out + 8 + 5 * start, span, witness, flags);
    }
    for (Word start = row.content.start; start < row.content.end; ++start) {
        const Site site = sites[row.ids[start]];
        const Word inherited = site.cls == 1 ? native_end(sites, row, start, false) : Nil;
        Word extra[2] = {detached[start], Nil};
        if (site.assertion_cls == 1) {
            const Span repeat{start, native_end(sites, row, start, true)};
            if (repeat.end <= row.content.end &&
                earlier_head(sites, row, nullptr, repeat).start != Nil)
                extra[1] = repeat.end;
        }
        if (extra[1] < extra[0]) {
            const Word swap = extra[0];
            extra[0] = extra[1];
            extra[1] = swap;
        }
        for (Word slot = 0; slot < 2; ++slot) {
            const Word end = extra[slot];
            if (end == Nil || end == inherited || (slot == 1 && end == extra[0])) continue;
            const Span span{start, end};
            const Span witness = earlier_head(sites, row, detached, span);
            const Word flags = rank(sites, cells, count, row, span, witness, ranks);
            if (end == detached[start])
                detached_record(out + 8 + 5 * start, span, witness, flags);
        }
    }

    for (Word role = 0; role < 2; ++role) {
        const Rank rank = ranks[role];
        const int floor = role == 0 ? INT_MIN : 12;
        const bool allowed = rank.best > rank.next && rank.best >= floor;
        out[2 * role] = allowed ? rank.span.start : Nil;
        out[2 * role + 1] = allowed ? rank.span.end : Nil;
        out[4 + role] = static_cast<Word>(rank.best);
        out[6 + role] = static_cast<Word>(rank.next);
    }
}
static void check(cudaError_t status) {
    if (status == cudaSuccess) return;
    std::fprintf(stderr, "%s\n", cudaGetErrorString(status));
    std::exit(2);
}
template<class T> static void read(T* records, size_t n) {
    if (std::fread(records, sizeof(T), n, stdin) != n) std::exit(2);
}
template<class T> static T* upload(const std::vector<T>& records) {
    T* device;
    check(cudaMalloc(&device, records.size() * sizeof(T)));
    check(cudaMemcpy(device, records.data(), records.size() * sizeof(T), cudaMemcpyHostToDevice));
    return device;
}

int main() {
    Word header[4];
    read(header, 4);
    const Word mode = header[1], n = header[2], count = header[3];
    if (header[0] != Abi || (mode != 7 && mode != 8) || n == 0 || n > 8192 || count == 0 ||
        count > Offsets * Roots) return 2;
    std::vector<Site> sites(Vocab);
    read(sites.data(), sites.size());
    for (Site site : sites) {
        if (site.root >= Roots - 1 || site.cls > 2 || site.assertion_cls > 2 ||
            site.fences > SentenceFence || site.lexical > 2) return 2;
    }
    std::vector<Cell> cells(count);
    read(cells.data(), cells.size());
    for (Word i = 0; i < count; ++i) {
        const Cell cell = cells[i];
        if (cell.key >= Offsets * Roots || (i > 0 && cell.key <= cells[i - 1].key) ||
            (cell.counts[0] | cell.counts[1] | cell.counts[2]) == 0) return 2;
        for (Word value : cell.counts) if (value > 8192 * Width) return 2;
    }
    std::vector<Row> rows(n);
    read(rows.data(), rows.size());
    if (std::fgetc(stdin) != EOF) return 2;
    for (const Row& row : rows) {
        if (row.size == 0 || row.size > Width || row.ids[0] != 0 ||
            row.content.start > row.content.end || row.content.end > row.size) return 2;
        for (Word p = 1; p < row.size; ++p) if (row.ids[p] < 2 || row.ids[p] >= Vocab) return 2;
    }
    Site* device_sites = upload(sites);
    Cell* device_cells = upload(cells);
    Row* device_rows = upload(rows);
    std::vector<Word> output(ResultWidth * n);
    Word* device_output;
    check(cudaMalloc(&device_output, output.size() * sizeof(Word)));
    predict<<<(n + 63) / 64, 64>>>(device_sites, device_cells, count, device_rows, n, mode,
                                   device_output);
    check(cudaGetLastError());
    check(cudaMemcpy(output.data(), device_output, output.size() * sizeof(Word), cudaMemcpyDeviceToHost));
    check(cudaFree(device_sites));
    check(cudaFree(device_cells));
    check(cudaFree(device_rows));
    check(cudaFree(device_output));
    return std::fwrite(output.data(), sizeof(Word), output.size(), stdout) == output.size() ? 0 : 2;
}

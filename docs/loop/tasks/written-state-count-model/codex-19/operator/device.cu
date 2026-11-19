// D11 sparse role cells and assertion agreement over the original token tape.
#include <cuda_runtime.h>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <vector>

using Word = uint32_t;
constexpr Word Vocab = 4096, Width = 192, Offsets = 6, Roots = 65537;
constexpr Word Nil = UINT32_MAX, Abi = 0x3141524f;
struct Site { Word root, cls, assertion_cls, delimiter; };
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

__device__ void scores(const Site* sites, const Cell* cells, Word count,
                       const Row& row, Span span, int (&result)[2]) {
    const Word at[Offsets] = {span.start - 3, span.start - 2, span.start - 1,
                             span.end, span.end + 1, span.end + 2};
    const int weights[Offsets] = {1, 2, 4, 4, 2, 1};
    for (Word i = 0; i < Offsets; ++i) {
        const Word root = at[i] < row.size ? sites[row.ids[at[i]]].root : Roots - 1;
        const Cell* cell = lookup(cells, count, i * Roots + root);
        if (cell == nullptr) continue;
        const int total = cell->counts[0] + cell->counts[1] + cell->counts[2];
        for (Word role = 0; role < 2; ++role) {
            const int signed_evidence = 16 * static_cast<int>(cell->counts[role + 1]) - 3 * total;
            result[role] += weights[i] * cell_quotient(signed_evidence, total + 2);
        }
    }
}

__device__ Word boundary(const Site* sites, const Row& row) {
    bool word_after = false;
    for (Word p = row.content.end; p > row.content.start;) {
        const Site site = sites[row.ids[--p]];
        if (word_after && site.delimiter != 0) return p;
        word_after |= site.cls == 1;
    }
    return Nil;
}

__device__ bool supported(const Site* sites, const Row& row, Span span, Word split) {
    if (split == Nil || span.start <= split || span.end > row.content.end) return false;
    for (Word fact = row.content.start > 1 ? row.content.start : 1; fact < split; ++fact) {
        if (sites[row.ids[fact]].assertion_cls != 1 || sites[row.ids[fact - 1]].delimiter == 0) {
            continue;
        }
        Word end = fact + 1;
        while (end < row.size && sites[row.ids[end]].assertion_cls == 2) ++end;
        if (end > split || end - fact != span.end - span.start) continue;
        Word equal = 0;
        while (fact + equal < end && row.ids[fact + equal] == row.ids[span.start + equal]) ++equal;
        if (fact + equal == end) return true;
    }
    return false;
}

__global__ void predict(const Site* sites, const Cell* cells, Word count, const Row* rows,
                        Word n, bool treatment, Word* result) {
    Word i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const Row& row = rows[i];
    const Word split = treatment ? boundary(sites, row) : Nil;
    Rank ranks[2];
    for (Word start = 0; start < row.size; ++start) {
        if (sites[row.ids[start]].cls != 1) continue;
        Word end = start + 1;
        while (end < row.size && sites[row.ids[end]].cls == 2) ++end;
        const Span span{start, end};
        int evidence[2]{};
        scores(sites, cells, count, row, span, evidence);
        if (evidence[1] > evidence[0]) ranks[1].add(span, evidence[1]);
        if (treatment) {
            if (supported(sites, row, span, split)) ranks[0].add(span, evidence[0] - evidence[1]);
        } else if (evidence[0] > evidence[1]) {
            ranks[0].add(span, evidence[0]);
        }
    }
    Word* out = result + 8 * i;
    for (Word role = 0; role < 2; ++role) {
        const Rank rank = ranks[role];
        const bool allowed = rank.best > rank.next && ((treatment && role == 0) || rank.best >= 12);
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
    if (header[0] != Abi || mode > 1 || n == 0 || n > 8192 || count == 0 ||
        count > Offsets * Roots) return 2;
    std::vector<Site> sites(Vocab);
    read(sites.data(), sites.size());
    for (Site site : sites) {
        if (site.root >= Roots - 1 || site.cls > 2 || site.assertion_cls > 2 ||
            site.delimiter > 1) return 2;
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
    std::vector<Word> output(8 * n);
    Word* device_output;
    check(cudaMalloc(&device_output, output.size() * sizeof(Word)));
    predict<<<(n + 63) / 64, 64>>>(device_sites, device_cells, count, device_rows, n, mode == 1,
                                   device_output);
    check(cudaGetLastError());
    check(cudaMemcpy(output.data(), device_output, output.size() * sizeof(Word), cudaMemcpyDeviceToHost));
    check(cudaFree(device_sites));
    check(cudaFree(device_cells));
    check(cudaFree(device_rows));
    check(cudaFree(device_output));
    return std::fwrite(output.data(), sizeof(Word), output.size(), stdout) == output.size() ? 0 : 2;
}

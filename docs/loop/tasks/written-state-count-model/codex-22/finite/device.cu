// A finite probe calling the frozen production-candidate CUDA scoring function.
// Keep the D19 source and its independently sealed standalone entry point intact.
#define main d19_standalone_main
#include "../../codex-19/operator/device.cu"
#undef main

struct Candidate {
    Row tape;
    Span entity;
};

__global__ void project(const Site* sites, const Cell* cells, Word count,
                        const Candidate* candidates, Word n, Word* out) {
    const Word i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) return;
    const Candidate& c = candidates[i];
    const Span span = c.entity, clause = c.tape.content;
    int evidence[2]{};
    scores(sites, cells, count, c.tape, span, clause, evidence);
    const Word at[Offsets] = {span.start-3, span.start-2, span.start-1,
                              span.end, span.end+1, span.end+2};
    for (Word j = 0; j < Offsets; ++j) {
        const bool inside = clause.start <= at[j] && at[j] < clause.end;
        const Word root = inside ? sites[c.tape.ids[at[j]]].root : Roots-1;
        out[i * 8 + j] = j * Roots + root;
    }
    out[i * 8 + 6] = static_cast<Word>(evidence[0]);
    out[i * 8 + 7] = static_cast<Word>(evidence[1]);
}

int main() {
    static_assert(sizeof(Candidate) == (Width + 5) * sizeof(Word));
    Word header[3];
    read(header, 3);
    const Word n = header[1], count = header[2];
    if (header[0] != 0x31435043 || !n || n > 64 || !count || count > Offsets*Roots) return 2;
    std::vector<Site> sites(Vocab);
    std::vector<Cell> cells(count);
    std::vector<Candidate> rows(n);
    read(sites.data(), sites.size());
    read(cells.data(), count);
    read(rows.data(), n);
    if (std::fgetc(stdin) != EOF) return 2;
    for (auto s : sites) if (s.root >= Roots-1 || s.cls > 2) return 2;
    for (Word i = 0; i < count; ++i) {
        const auto& c = cells[i];
        if (c.key >= Offsets*Roots || (i && cells[i-1].key >= c.key)) return 2;
        for (auto value : c.counts) if (value > 1000000) return 2;
    }
    for (const auto& c : rows) {
        const auto& t = c.tape;
        if (t.size > Width || t.content.start > c.entity.start ||
            c.entity.start >= c.entity.end || c.entity.end > t.content.end ||
            t.content.end > t.size) return 2;
        for (Word i = 0; i < t.size; ++i) if (t.ids[i] >= Vocab) return 2;
    }
    Site* ds = upload(sites);
    Cell* dc = upload(cells);
    Candidate* dr = upload(rows);
    std::vector<Word> output(8*n);
    Word* result;
    check(cudaMalloc(&result, output.size()*sizeof(Word)));
    project<<<1,64>>>(ds, dc, count, dr, n, result);
    check(cudaGetLastError());
    check(cudaMemcpy(output.data(), result, output.size()*sizeof(Word), cudaMemcpyDeviceToHost));
    check(cudaFree(ds)); check(cudaFree(dc)); check(cudaFree(dr)); check(cudaFree(result));
    return std::fwrite(output.data(), sizeof(Word), output.size(), stdout) == output.size() ? 0 : 2;
}

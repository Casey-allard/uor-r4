// Exhaust the existing 16-bit carry code at the input of the frozen D26 read.
#define main d25_source_cli
#include "../../codex-25/device/source.cu"
#undef main

namespace {
constexpr unsigned codes = 65536;
constexpr unsigned max_candidates = 8;  // Seven copyable pieces, plus empty sentinel.
constexpr unsigned address_mask = 0x00ffffff;
__constant__ int roots[16][8];

__global__ void exhaust(const Candidate *rows, const Group *groups, const Exemplar *bank,
                        unsigned bank_size, unsigned *out) {
    const unsigned code = blockIdx.x * blockDim.x + threadIdx.x;
    const Group group = groups[blockIdx.y];
    Distance distances[2 * max_candidates];
    for (unsigned j = 0; j < group.end - group.start; ++j) {
        const Candidate &row = rows[group.start + j];
        if (!row.wide) continue;
        Exemplar feature = row.feature;
        for (unsigned k = 0; k < dimensions; ++k) {
            feature.context[k] = roots[(code >> (4 * (k / 8))) & 15][k % 8];
        }
        distances[2 * j + 1] = nearest(feature, bank, bank_size, true);
    }
    const Selection answer = select(rows + group.start, {0, group.end - group.start}, distances, 3);
    out[blockIdx.y * codes + code] = (unsigned(answer.reason) << 24) | answer.address;
}

using Roots = std::array<std::array<int, 8>, 16>;
std::vector<unsigned> run(const std::vector<Candidate> &rows, const std::vector<Exemplar> &bank,
                          const Roots &orbit) {
    const auto groups = group_rows(rows);
    require(!groups.empty() && groups.size() <= 256, "invalid group count");
    for (auto group : groups) {
        require(group.end - group.start <= max_candidates, "group exceeds original copy bound");
        for (auto j = group.start; j < group.end; ++j)
            require(rows[j].address <= address_mask, "original address overflow");
    }
    validate_bank(bank);
    DeviceBuffer<Candidate> device_rows(rows.size());
    DeviceBuffer<Group> device_groups(groups.size());
    DeviceBuffer<Exemplar> device_bank(bank.size());
    DeviceBuffer<unsigned> output(codes * groups.size());
    device_rows.upload(rows);
    device_groups.upload(groups);
    device_bank.upload(bank);
    cuda_check(cudaMemcpyToSymbol(roots, orbit.data(), sizeof(orbit)));
    exhaust<<<dim3(codes / 128, groups.size()), 128>>>(device_rows.data(), device_groups.data(),
                                                       device_bank.data(), bank.size(), output.data());
    cuda_check(cudaGetLastError());
    return output.download(codes * groups.size());
}

void store(const char *path, const std::vector<Candidate> &rows, const std::vector<unsigned> &map) {
    std::ostringstream bytes;
    bytes.write("D30FIB01", 8);
    const auto groups = group_rows(rows);
    put_le(bytes, groups.size(), 4);
    put_le(bytes, codes, 4);
    for (size_t i = 0; i < groups.size(); ++i) {
        put_le(bytes, rows[groups[i].start].row, 4);
        for (unsigned k = 0; k < codes; ++k) put_le(bytes, map[i * codes + k], 4);
    }
    save_fresh(path, bytes.str());
}

void finite_capacity(const Roots &orbit, const char *out) {
    Candidate first{}, second{}, sentinel{};
    first.row = second.row = 1;
    first.wide = second.wide = 1;
    first.address = 8;
    second.address = 9;
    second.feature.local[0] = 16;
    sentinel.row = 2;
    std::vector<Exemplar> bank;
    for (int root : {0, 15}) {
        for (Candidate row : {first, second}) {
            row.feature.label = ((root == 0) == (row.address == 8)) ? 1 : -1;
            for (int i = 0; i < dimensions; ++i) row.feature.context[i] = orbit[root][i % 8];
            bank.push_back(row.feature);
        }
    }
    const std::vector<Candidate> rows{first, second, sentinel};
    const auto map = run(rows, bank, orbit);
    unsigned count[3]{};
    for (unsigned code = 0; code < codes; ++code) {
        int delta = 0;
        for (int i = 0; i < dimensions; ++i) {
            const int x = orbit[(code >> (4 * (i / 8))) & 15][i % 8];
            delta += std::abs(x - orbit[15][i % 8]) - std::abs(x - orbit[0][i % 8]);
        }
        const unsigned expected = delta > 0 ? 8 : delta < 0 ? 9 : unsigned(nonpositive) << 24;
        require(map[code] == expected && map[codes + code] == (unsigned(empty) << 24),
                "finite signed source differs");
        ++count[delta > 0 ? 0 : delta < 0 ? 1 : 2];
    }
    require(count[0] && count[1] && count[2], "finite control lacks both wins and abstention");
    store(out, rows, map);
}

} // namespace

int main(int argc, char **argv) {
    try {
        const bool planted = argc == 4 && std::string(argv[1]) == "finite";
        require(planted || (argc == 6 && std::string(argv[1]) == "run"),
                "expected finite ROOTS OUT or run ROOTS NOMINAL.rows BANK OUT");
        std::ifstream input(argv[2]);
        Roots orbit{};
        for (auto &root : orbit) for (int &x : root) x = read_integer<int>(input);
        std::string extra;
        require(!(input >> extra), "trailing root words");
        if (planted) finite_capacity(orbit, argv[3]);
        else {
            const auto rows = load_rows(argv[3], false);
            std::ifstream file(argv[4], std::ios::binary);
            const auto bank = load_bank(file);
            store(argv[5], rows, run(rows, bank, orbit));
        }
        return 0;
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}

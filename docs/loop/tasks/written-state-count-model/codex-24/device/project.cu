// Use the frozen CUDA projector and table layout from D15.
#define main d15_standalone_main
#include "../../codex-15/device/device.cu"
#undef main

#include <algorithm>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>

struct ProjectionRow {
    Row row{};
    Span mention{};
};

__global__ void project_rows(const Tables* tables, const ProjectionRow* rows,
                             Word size, Word mode, Feature* features) {
    const Word i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < size) {
        Candidate trace;
        features[i] = feature(*tables, rows[i].row, rows[i].mention, mode, trace);
    }
}

static void require(bool success, const char* message) {
    if (!success) {
        throw std::runtime_error(message);
    }
}

static Tables load_tables(const char* path) {
    static_assert(__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__);
    std::ifstream input(path, std::ios::binary);
    Tables tables{};
    require(static_cast<bool>(input.read(reinterpret_cast<char*>(&tables), sizeof(tables))),
            "Missing or truncated D15 tables");
    require(input.peek() == EOF && !input.bad(), "Unexpected D15 table tail");
    validate(tables, Bank{}, {}, Signature);
    return tables;
}

static std::vector<ProjectionRow> load_projection_rows(const char* path) {
    constexpr size_t maximum_rows = 1'000'000;
    std::ifstream input(path);
    require(static_cast<bool>(input), "Cannot open candidates");
    std::vector<ProjectionRow> rows;
    for (std::string line; std::getline(input, line);) {
        ProjectionRow candidate;
        auto& row = candidate.row;
        std::istringstream fields(line);
        require(static_cast<bool>(fields >> candidate.mention.start >> candidate.mention.end
                                         >> row.region.start >> row.region.end),
                "Candidate requires mention and clause bounds");
        while (!(fields >> std::ws).eof()) {
            require(row.size < Width && static_cast<bool>(fields >> row.ids[row.size]),
                    "Malformed or overlong token tape");
            ++row.size;
        }
        require(row.size > 1 && row.ids[0] == 0, "Candidate tape requires leading BOS and content");
        for (Word i = 1; i < row.size; ++i) {
            require(row.ids[i] >= 2 && row.ids[i] < Vocab, "Invalid original token ID");
        }
        row.content = {1, row.size};
        require(row.region.start >= 1 && row.region.start <= candidate.mention.start &&
                candidate.mention.start < candidate.mention.end &&
                candidate.mention.end <= row.region.end && row.region.end <= row.size,
                "Invalid half-open mention and clause bounds");
        require(rows.size() < maximum_rows, "Too many candidate tapes");
        rows.push_back(candidate);
    }
    require(input.eof() && !input.bad() && !rows.empty(), "Missing or unreadable candidate tapes");
    return rows;
}

static void project(const Tables& tables, const std::vector<ProjectionRow>& rows,
                    Word mode, const char* path) {
    constexpr Word batch_size = 1024;
    constexpr Word block_size = 128;
    auto* device_tables = upload(&tables, 1);
    auto* device_rows = allocate<ProjectionRow>(batch_size);
    auto* device_features = allocate<Feature>(batch_size);
    std::vector<Feature> features(batch_size);
    std::ostringstream output;
    for (size_t start = 0; start < rows.size(); start += batch_size) {
        const auto count = static_cast<Word>(std::min<size_t>(batch_size, rows.size() - start));
        check(cudaMemcpy(device_rows, rows.data() + start, count * sizeof(ProjectionRow),
                         cudaMemcpyHostToDevice));
        project_rows<<<(count + block_size - 1) / block_size, block_size>>>(
            device_tables, device_rows, count, mode, device_features);
        check(cudaGetLastError());
        check(cudaMemcpy(features.data(), device_features, count * sizeof(Feature),
                         cudaMemcpyDeviceToHost));
        for (Word i = 0; i < count; ++i) {
            output << features[i].coord[0];
            for (Word axis = 1; axis < Coords; ++axis) {
                output << ' ' << features[i].coord[axis];
            }
            output << '\n';
        }
    }
    check(cudaFree(device_features));
    check(cudaFree(device_rows));
    check(cudaFree(device_tables));

    const auto bytes = output.str();
    auto* file = std::fopen(path, "wbx");
    require(file != nullptr, "Cannot create output. Choose a new output path.");
    size_t size = std::fwrite(bytes.data(), 1, bytes.size(), file);
    int closed = std::fclose(file);
    require(size == bytes.size() && closed == 0, "Incomplete feature output");
}

int main(int argc, char** argv) {
    try {
        require(argc == 5, "Usage: d24-project identity|positional TABLES CANDIDATES FEATURES");
        const std::string arm(argv[1]);
        require(arm == "identity" || arm == "positional", "Unknown projector arm");
        const auto tables = load_tables(argv[2]);
        const auto rows = load_projection_rows(argv[3]);
        project(tables, rows, arm == "positional", argv[4]);
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}

// Reuse the D17 native distance, bank ABI, collision validation and radius kernels.
// D28 replaces only the legacy three-candidate input with explicit original sites.
#define main d17_legacy_dispatch
#include "../../codex-17/native/device.cu"
#undef main

constexpr uint32_t FIT_MAGIC = 0x38324644;
constexpr size_t FIT_WIDTH = 13;

__global__ void copy_original_examples(const uint32_t* data, uint32_t count,
                                       uint32_t* bank, uint32_t* families,
                                       uint32_t* invalid) {
    uint32_t index = threadIdx.x;
    if (index >= count) return;
    const auto* row = data + FIT_WIDTH * index;
    uint32_t depth = row[5] & 255;
    uint32_t word = row[5] >> 8;
    if (row[0] < 28000 || row[0] >= 28068 || row[4] != (row[0] - 28000) / 4
        || !(row[1] < row[2] && row[2] <= row[3] && row[3] < CONTEXT)
        || row[3] - row[1] >= 8 || depth < 1 || depth > 3
        || word < 1 || word > 7 || word != row[2] - row[1] || row[7] > 1) {
        atomicOr(invalid, 1u);
        return;
    }
    uint32_t* record = bank + RECORD * index;
    record[0] = row[5] | (row[7] << 16);
    record[1] = row[6];
    record[2] = NO_RADIUS;
    for (int part = 0; part < 5; ++part) record[3 + part] = row[8 + part];
    families[index] = row[4];
}

int main(int argc, char** argv) {
    if (argc != 3) return 3;
    auto input = read(argv[1], 4 + CAP * FIT_WIDTH);
    if (input.size() < 4 || input[0] != FIT_MAGIC || !input[1] || input[1] > CAP
        || input[2] != FIT_WIDTH || input[3] || input.size() != 4 + input[1] * FIT_WIDTH) {
        return 3;
    }
    const uint32_t n = input[1];
    auto rows = copy(input.data() + 4, n * FIT_WIDTH);
    auto bank = copy<uint32_t>(nullptr, CAP * RECORD);
    auto families = copy<uint32_t>(nullptr, CAP);
    auto invalid = copy<uint32_t>(nullptr, 1);
    auto minima = copy<uint32_t>(nullptr, CAP);
    copy_original_examples<<<1, CAP>>>(rows, n, bank, families, invalid);
    check(cudaGetLastError());
    uint32_t status = 0;
    check(cudaMemcpy(&status, invalid, sizeof(status), cudaMemcpyDeviceToHost));
    if (status) return 4;
    nearest_other_family<<<1, CAP>>>(bank, families, n, minima);
    check(cudaGetLastError());
    assign_radius<<<1, CAP>>>(bank, n, minima);
    check(cudaGetLastError());
    std::vector<uint32_t> output(4 + CAP * RECORD);
    output[0] = RAW_MAGIC;
    output[1] = 0;
    output[2] = n;
    output[3] = CAP;
    check(cudaMemcpy(output.data() + 4, bank, CAP * RECORD * sizeof(uint32_t), cudaMemcpyDeviceToHost));
    if (!bank_valid(output.data() + 4, n, 0)) return 4;
    std::ifstream existing(argv[2]);
    if (existing) return 5;
    return write(argv[2], output) ? 0 : 6;
}

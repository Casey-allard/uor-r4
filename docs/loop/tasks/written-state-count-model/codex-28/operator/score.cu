// Reuse the fitted ordered bank's exact root table, geometry and L1 operator.
#define main legacy_onset_main
#include "../../codex-17/native/device.cu"
#undef main

constexpr uint32_t d28_input = 0x49383244;  // D28I
constexpr uint32_t d28_output = 0x4f383244; // D28O
constexpr size_t input_header = 6, candidate_words = 10, candidate_trace = 10;
constexpr size_t row_words = input_header + CONTEXT * candidate_words;
constexpr size_t result_words = 2 + CONTEXT * candidate_trace;

__device__ bool score_original(const uint32_t* row, const uint32_t* bank,
                               uint32_t used, uint32_t* output) {
    const uint32_t length = row[0], fact = row[1], query = row[2];
    const uint32_t fence = row[3], mask = row[4], count = row[5];
    if (length > CONTEXT || !fact || !(fact < query && query < length) ||
        (fence != NONE && !(fact < fence && fence < query)) ||
        (fence == NONE && (count || mask)) || mask > 7 || count > CONTEXT) return false;

    for (size_t i = 0; i < result_words; ++i) output[i] = NONE;
    uint32_t best_source = NONE, previous_word_end = fact + 2, seen_mask = 0;
    int best_num = 0, best_den = 0;
    bool tied = false, unseen = false;
    for (size_t i = 0; i < CONTEXT; ++i) {
        const auto c = row + input_header + candidate_words * i;
        if (i >= count) {
            for (size_t j = 0; j < candidate_words; ++j) if (c[j]) return false;
            continue;
        }
        const uint32_t source = c[0], end = c[1], connector = c[2], copied = c[3];
        if (!(previous_word_end <= source && source < end && end <= fence) ||
            connector != source - fact - 1 || copied != fence - source) return false;
        if (connector <= 3) {
            const uint32_t bit = 1u << (connector - 1);
            if (!(mask & bit)) return false;
            seen_mask |= bit;
        }
        previous_word_end = end;
        auto trace = output + 2 + i * candidate_trace;
        trace[0] = source;
        trace[1] = end;
        trace[2] = connector;
        trace[3] = copied;
        trace[4] = 1;
        if (copied >= 8) {
            for (size_t j = 4; j < candidate_words; ++j) if (c[j]) return false;
            continue;
        }
        const uint32_t depth = connector < 3 ? connector : 3;
        const uint32_t slots = depth + end - source, signature = c[4];
        if ((signature >> (2 * slots)) || ((signature >> (2 * depth)) & 3) != 1) return false;
        Geometry g = {depth | ((end - source) << 8), signature, {}};
        for (uint32_t j = 0; j < 10; ++j) {
            const uint32_t cls = (signature >> (2 * j)) & 3;
            const uint32_t root = (c[5 + j / 2] >> (16 * (j % 2))) & 65535;
            if ((j < slots && (cls > 2 || (j > depth && cls != 2))) ||
                (j >= slots && root)) return false;
            g.roots[j] = root;
        }
        uint32_t closest[2] = {NONE, NONE}, radius = NO_RADIUS;
        for (uint32_t j = 0; j < used; ++j) {
            const auto record = bank + j * RECORD;
            if (!match(g, record)) continue;
            const uint32_t label = record[0] >> 16, d = distance(g, record);
            if (d < closest[label]) closest[label] = d;
            radius = record[2];
        }
        trace[4] = 2;
        trace[5] = closest[0];
        trace[6] = closest[1];
        trace[7] = radius;
        if (closest[0] == NONE || closest[1] == NONE || radius == NO_RADIUS ||
            closest[0] > radius) {
            unseen = true;
            continue;
        }
        const int num = int(closest[1]) - int(closest[0]), den = int(slots);
        trace[4] = 3;
        trace[8] = uint32_t(num);
        trace[9] = uint32_t(den);
        const int order = best_den ? cmp(num, den, best_num, best_den) : 1;
        if (order > 0) {
            best_num = num;
            best_den = den;
            best_source = source;
            tied = false;
        } else if (!order) tied = true;
    }
    if (seen_mask != mask) return false;
    output[0] = unseen ? 2 : !best_den ? 1 : best_num <= 0 ? 4 : tied ? 3 : 0;
    if (!output[0]) output[1] = best_source;
    return true;
}

__global__ void run_original(const uint32_t* rows, uint32_t count, const uint32_t* bank,
                             uint32_t used, uint32_t* results, uint32_t* error) {
    const size_t i = size_t(blockIdx.x) * blockDim.x + threadIdx.x;
    if (i < count && !score_original(rows + i * row_words, bank, used,
                                    results + i * result_words)) atomicExch(error, 1u);
}

int main(int argc, char** argv) {
    // A concurrent batch is bounded independently of the scored model.
    constexpr size_t max_batch = 8192;
    if (argc != 4) return 3;
    const auto input = read(argv[1], 4 + max_batch * row_words);
    const auto raw = read(argv[2], 4 + CAP * RECORD);
    if (input.size() < 4 || input[0] != d28_input || !input[1] || input[1] > max_batch ||
        input[2] != row_words || input[3] || input.size() != 4 + input[1] * row_words ||
        raw.size() != 4 + CAP * RECORD || raw[0] != RAW_MAGIC || raw[1] ||
        raw[3] != CAP || !bank_valid(raw.data() + 4, raw[2], raw[1])) return 3;

    int device = 0;
    cudaDeviceProp props{};
    check(cudaGetDevice(&device));
    check(cudaGetDeviceProperties(&props, device));
    if (props.major != 8 || props.minor != 9) return 5;
    const uint32_t n = input[1], zero = 0;
    auto rows = copy(input.data() + 4, n * row_words);
    auto bank = copy(raw.data() + 4, CAP * RECORD);
    auto results = copy<uint32_t>(nullptr, n * result_words);
    auto error = copy(&zero, 1);
    run_original<<<(n + 63) / 64, 64>>>(rows, n, bank, raw[2], results, error);
    check(cudaGetLastError());

    uint32_t bad = 0;
    check(cudaMemcpy(&bad, error, sizeof(bad), cudaMemcpyDeviceToHost));
    if (bad) return 4;
    std::vector<uint32_t> output(4 + n * result_words);
    output[0] = d28_output;
    output[1] = n;
    output[2] = result_words;
    output[3] = 0;
    check(cudaMemcpy(output.data() + 4, results, n * result_words * sizeof(uint32_t),
                     cudaMemcpyDeviceToHost));
    check(cudaFree(error));
    check(cudaFree(results));
    check(cudaFree(bank));
    check(cudaFree(rows));
    std::fprintf(stderr, "device=%s sm=%d%d rows=%u\n", props.name, props.major, props.minor, n);
    return write(argv[3], output) ? 0 : 6;
}

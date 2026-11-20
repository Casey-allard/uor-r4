// Reuse the fitted ordered bank's exact root table, geometry and L1 operator.
#define main legacy_onset_main
#include "../../codex-17/native/device.cu"
#undef main

constexpr uint32_t d28_input = 0x49383244;  // D28I
constexpr uint32_t d28_output = 0x4f383244; // D28O
constexpr uint32_t fit_magic = 0x38324644, calibration_magic = 0x39324e52;
constexpr size_t fit_width = 13;
constexpr size_t input_header = 6, candidate_words = 10, candidate_trace = 10;
constexpr size_t row_words = input_header + CONTEXT * candidate_words;
constexpr size_t result_words = 2 + CONTEXT * candidate_trace;

__device__ bool score_original(const uint32_t* row, const uint32_t* bank, const uint32_t* calibration,
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
        uint32_t closest[2] = {NONE, NONE}, radius = NO_RADIUS, negative_radius = NO_RADIUS;
        for (uint32_t j = 0; j < used; ++j) {
            const auto record = bank + j * RECORD;
            if (!match(g, record)) continue;
            const uint32_t label = record[0] >> 16, d = distance(g, record);
            if (d < closest[label]) closest[label] = d;
            radius = record[2];
            negative_radius = calibration[j];
        }
        trace[4] = 2;
        trace[5] = closest[0];
        trace[6] = closest[1];
        trace[7] = radius;
        if (closest[0] == NONE || closest[1] == NONE || radius == NO_RADIUS ||
            closest[0] > radius) {
            trace[8] = negative_radius;
            if (negative_radius != NO_RADIUS && closest[1] <= negative_radius &&
                closest[1] < closest[0]) {
                trace[4] = 4;
                continue;
            }
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
                             const uint32_t* calibration, uint32_t used,
                             uint32_t* results, uint32_t* error) {
    const size_t i = size_t(blockIdx.x) * blockDim.x + threadIdx.x;
    if (i < count && !score_original(rows + i * row_words, bank, calibration, used,
                                    results + i * result_words)) atomicExch(error, 1u);
}

__global__ void nearest_negative(const uint32_t* training, const uint32_t* bank,
                                 uint32_t used, uint32_t* minima, uint32_t* error) {
    const uint32_t i = threadIdx.x;
    if (i >= used) return;
    const auto row = training + i * fit_width;
    const auto record = bank + i * RECORD;
    if (row[0] < 28000 || row[0] >= 28068 || row[4] != (row[0] - 28000) / 4 ||
        !(row[1] < row[2] && row[2] <= row[3] && row[3] < CONTEXT) ||
        row[3] - row[1] >= 8 || row[7] > 1 || row[2] - row[1] != row[5] >> 8 ||
        record[0] != (row[5] | (row[7] << 16)) || record[1] != row[6]) {
        atomicExch(error, 1u);
        return;
    }
    for (uint32_t slot = 0; slot < 5; ++slot) {
        if (record[3 + slot] != row[8 + slot]) {
            atomicExch(error, 1u);
            return;
        }
    }
    uint32_t minimum = NONE;
    if (row[7]) {
        const Geometry source = geometry(record);
        for (uint32_t j = 0; j < used; ++j) {
            const auto target = bank + j * RECORD;
            if (!(target[0] >> 16) || !match(source, target) ||
                row[4] == training[j * fit_width + 4]) continue;
            const uint32_t d = distance(source, target);
            if (d < minimum) minimum = d;
        }
    }
    minima[i] = minimum;
}

__global__ void negative_radii(const uint32_t* bank, uint32_t used,
                              const uint32_t* minima, uint32_t* radii) {
    const uint32_t i = threadIdx.x;
    if (i >= used) return;
    const Geometry source = geometry(bank + i * RECORD);
    uint32_t radius = 0;
    bool seen = false;
    for (uint32_t j = 0; j < used; ++j) {
        const auto negative = bank + j * RECORD;
        if (!(negative[0] >> 16) || !match(source, negative)) continue;
        if (minima[j] == NONE) {
            radii[i] = NO_RADIUS;
            return;
        }
        seen = true;
        if (minima[j] > radius) radius = minima[j];
    }
    radii[i] = seen ? radius : NO_RADIUS;
}

static bool calibration_valid(const std::vector<uint32_t>& sidecar,
                              const std::vector<uint32_t>& bank) {
    const uint32_t used = bank[2];
    if (sidecar.size() != 4 + CAP || sidecar[0] != calibration_magic ||
        sidecar[1] != used || sidecar[2] != CAP || sidecar[3]) return false;
    for (uint32_t i = 0; i < CAP; ++i) {
        const uint32_t radius = sidecar[4 + i];
        if (i >= used) {
            if (radius) return false;
            continue;
        }
        if (radius > MAX_COST && radius != NO_RADIUS) return false;
        const auto record = bank.data() + 4 + i * RECORD;
        uint32_t negatives = 0;
        for (uint32_t j = 0; j < used; ++j) {
            const auto other = bank.data() + 4 + j * RECORD;
            if ((record[0] & 65535) != (other[0] & 65535) || record[1] != other[1]) continue;
            if (radius != sidecar[4 + j]) return false;
            negatives += other[0] >> 16;
        }
        if (radius != NO_RADIUS && negatives < 2) return false;
    }
    return true;
}

int main(int argc, char** argv) {
    constexpr size_t max_batch = 8192;
    const bool fit = argc == 5 && !std::strcmp(argv[1], "fit");
    if (!fit && !(argc == 6 && !std::strcmp(argv[1], "score"))) return 3;
    const auto raw = read(argv[3], 4 + CAP * RECORD);
    if (raw.size() != 4 + CAP * RECORD || raw[0] != RAW_MAGIC || raw[1] ||
        raw[3] != CAP || !bank_valid(raw.data() + 4, raw[2], raw[1])) return 3;
    const auto input = read(argv[2], 4 + (fit ? CAP * fit_width : max_batch * row_words));
    if (input.size() < 4 || !input[1] || input[3]) return 3;
    if (fit) {
        if (input[0] != fit_magic || input[1] != raw[2] || input[2] != fit_width ||
            input.size() != 4 + input[1] * fit_width) return 3;
    } else if (input[0] != d28_input || input[1] > max_batch || input[2] != row_words ||
               input.size() != 4 + input[1] * row_words) return 3;
    const auto output_path = argv[fit ? 4 : 5];
    if (std::ifstream(output_path)) return 3;
    const auto calibration = fit ? std::vector<uint32_t>{} : read(argv[4], 4 + CAP);
    if (!fit && !calibration_valid(calibration, raw)) return 3;

    int device = 0;
    cudaDeviceProp props{};
    check(cudaGetDevice(&device));
    check(cudaGetDeviceProperties(&props, device));
    if (props.major != 8 || props.minor != 9) return 5;
    const uint32_t n = input[1], zero = 0;
    std::fprintf(stderr, "device=%s sm=%d%d rows=%u\n", props.name, props.major, props.minor, n);
    auto bank = copy(raw.data() + 4, CAP * RECORD);
    auto error = copy(&zero, 1);
    if (fit) {
        auto training = copy(input.data() + 4, n * fit_width);
        auto minima = copy<uint32_t>(nullptr, CAP);
        auto radii = copy<uint32_t>(nullptr, CAP);
        nearest_negative<<<1, CAP>>>(training, bank, n, minima, error);
        check(cudaGetLastError());
        uint32_t bad = 0;
        check(cudaMemcpy(&bad, error, sizeof(bad), cudaMemcpyDeviceToHost));
        if (bad) return 4;
        negative_radii<<<1, CAP>>>(bank, n, minima, radii);
        check(cudaGetLastError());
        std::vector<uint32_t> output(4 + CAP);
        output[0] = calibration_magic;
        output[1] = n;
        output[2] = CAP;
        check(cudaMemcpy(output.data() + 4, radii, CAP * sizeof(uint32_t), cudaMemcpyDeviceToHost));
        check(cudaDeviceReset());
        if (!calibration_valid(output, raw)) return 4;
        return write(output_path, output) ? 0 : 6;
    }
    auto rows = copy(input.data() + 4, n * row_words);
    auto radii = copy(calibration.data() + 4, CAP);
    auto results = copy<uint32_t>(nullptr, n * result_words);
    run_original<<<(n + 63) / 64, 64>>>(rows, n, bank, radii, raw[2], results, error);
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
    check(cudaDeviceReset());
    return write(output_path, output) ? 0 : 6;
}

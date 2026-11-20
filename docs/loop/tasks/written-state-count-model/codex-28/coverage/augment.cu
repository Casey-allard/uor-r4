// Compile with the original D17 native directory on the include path.
#define main d17_device_entry
#include "device.cu"
#undef main

// Keep the original learned records and radii byte-for-byte. Examine only
// actual TRAIN-negative geometries for an originally positive-only signature.
__global__ void append_missing_negatives(uint32_t* frozen, const uint32_t* fitted) {
    const unsigned original = frozen[2];
    unsigned used = original;
    for (unsigned i = 0; i < fitted[2] && used < CAP; ++i) {
        const auto* negative = fitted + 4 + i * RECORD;
        if ((negative[0] >> 16) != 1) continue;
        unsigned radius = NO_RADIUS;
        bool ineligible = false;
        for (unsigned j = 0; j < used; ++j) {
            const auto* existing = frozen + 4 + j * RECORD;
            if ((existing[0] & 65535) != (negative[0] & 65535) ||
                existing[1] != negative[1]) continue;
            if (j < original && (existing[0] >> 16) == 1) ineligible = true;
            if ((existing[0] >> 16) == 0) radius = existing[2];
            bool identical = true;
            for (unsigned w = 3; w < RECORD; ++w)
                identical &= existing[w] == negative[w];
            if (identical) ineligible = true;
        }
        if (ineligible || radius == NO_RADIUS) continue;
        auto* added = frozen + 4 + used++ * RECORD;
        for (unsigned w = 0; w < RECORD; ++w) added[w] = negative[w];
        added[2] = radius;
    }
    frozen[2] = used;
}

int main(int argc, char** argv) {
    if (argc != 4) return 2;
    const unsigned size = 4 + CAP * RECORD;
    auto old = read(argv[1], size);
    auto fitted = read(argv[2], size);
    for (const auto* input : {&old, &fitted}) {
        if (input->size() != size || (*input)[0] != RAW_MAGIC ||
            (*input)[1] != 0 || (*input)[3] != CAP ||
            !bank_valid(input->data() + 4, (*input)[2], 0)) return 3;
    }
    auto* result = copy(old.data(), size);
    auto* source = copy(fitted.data(), size);
    append_missing_negatives<<<1,1>>>(result, source);
    check(cudaGetLastError());
    std::vector<uint32_t> out(size);
    check(cudaMemcpy(out.data(), result, size * sizeof(uint32_t), cudaMemcpyDeviceToHost));
    check(cudaDeviceReset());
    if (out[2] < old[2] || !bank_valid(out.data() + 4, out[2], 0) ||
        std::memcmp(old.data() + 4, out.data() + 4, old[2] * RECORD * 4)) return 4;
    return write(argv[3], out) ? 0 : 5;
}

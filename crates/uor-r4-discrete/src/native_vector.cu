// Integer-only Q31 Hamilton transport for the native vector trainer.
// No float, matrix, transcendental or runtime-compilation path exists here.
typedef long long s64;
typedef unsigned long long u64;

__device__ __forceinline__ u64 product(s64 a, s64 b) {
    return (u64)(a * b);
}

__device__ __forceinline__ s64 q31_word(u64 word) {
    s64 shifted = (s64)(word + (1ull << 30)) >> 31;
    if (shifted < -2147483648ll) return -2147483648ll;
    if (shifted > 2147483648ll) return 2147483648ll;
    return shifted;
}

__device__ __forceinline__ void hamilton_rotate_words(const s64* a, const s64* b, s64* out) {
    u64 w =
        product(a[0], b[0]) -
        product(a[1], b[1]) -
        product(a[2], b[2]) -
        product(a[3], b[3]);
    u64 x =
        product(a[0], b[1]) +
        product(a[1], b[0]) +
        product(a[2], b[3]) -
        product(a[3], b[2]);
    u64 y =
        product(a[0], b[2]) -
        product(a[1], b[3]) +
        product(a[2], b[0]) +
        product(a[3], b[1]);
    u64 z =
        product(a[0], b[3]) +
        product(a[1], b[2]) -
        product(a[2], b[1]) +
        product(a[3], b[0]);
    out[0] = q31_word(w);
    out[1] = q31_word(x);
    out[2] = q31_word(y);
    out[3] = q31_word(z);
}

__device__ __forceinline__ s64 q31_saturate(s64 word) {
    if (word < -2147483648ll) return -2147483648ll;
    if (word > 2147483648ll) return 2147483648ll;
    return word;
}

__device__ __forceinline__ u64 abs_word(s64 a, s64 b) {
    return a >= b ? (u64)(a - b) : (u64)(b - a);
}

extern "C" __global__ void native_q31_hamilton_rotate(
    const s64* gates,
    const s64* values,
    s64* output,
    unsigned count
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= count) return;
    u64 at = (u64)item * 4ull;
    const s64* a = gates + at;
    const s64* b = values + at;
    s64 p00 = a[0], p01 = a[0], p02 = a[0], p03 = a[0];
    s64 p10 = a[1], p11 = a[1], p12 = a[1], p13 = a[1];
    s64 p20 = a[2], p21 = a[2], p22 = a[2], p23 = a[2];
    s64 p30 = a[3], p31 = a[3], p32 = a[3], p33 = a[3];
    u64 w =
        product(p00, b[0]) -
        product(p11, b[1]) -
        product(p22, b[2]) -
        product(p33, b[3]);
    u64 x =
        product(p01, b[1]) +
        product(p10, b[0]) +
        product(p23, b[3]) -
        product(p32, b[2]);
    u64 y =
        product(p02, b[2]) -
        product(p13, b[3]) +
        product(p20, b[0]) +
        product(p31, b[1]);
    u64 z =
        product(p03, b[3]) +
        product(p12, b[2]) -
        product(p21, b[1]) +
        product(p30, b[0]);
    output[at] = q31_word(w);
    output[at + 1] = q31_word(x);
    output[at + 2] = q31_word(y);
    output[at + 3] = q31_word(z);
}

extern "C" __global__ void native_q31_hamilton_train_step(
    const s64* gates,
    const s64* values,
    const s64* targets,
    unsigned count,
    unsigned step_shift,
    s64* predictions,
    s64* candidate_gates,
    s64* accepted_gates,
    u64* loss_before,
    u64* loss_after
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= count) return;
    u64 at = (u64)item * 4ull;
    const s64* gate = gates + at;
    const s64* value = values + at;
    const s64* target = targets + at;
    s64* prediction = predictions + at;

    hamilton_rotate_words(gate, value, prediction);
    u64 before = 0;
    s64 residual[4];
    for (unsigned j = 0; j < 4; ++j) {
        before += abs_word(prediction[j], target[j]);
        residual[j] = q31_saturate(prediction[j] - target[j]);
    }

    s64 conjugate[4] = {value[0], -value[1], -value[2], -value[3]};
    s64 credit[4];
    hamilton_rotate_words(residual, conjugate, credit);
    s64* candidate = candidate_gates + at;
    for (unsigned j = 0; j < 4; ++j) {
        candidate[j] = q31_saturate(gate[j] - (credit[j] >> step_shift));
    }

    s64 candidate_prediction[4];
    hamilton_rotate_words(candidate, value, candidate_prediction);
    u64 after = 0;
    for (unsigned j = 0; j < 4; ++j) {
        after += abs_word(candidate_prediction[j], target[j]);
    }
    s64* accepted = accepted_gates + at;
    const s64* selected = after < before ? candidate : gate;
    for (unsigned j = 0; j < 4; ++j) {
        accepted[j] = selected[j];
    }
    loss_before[item] = before;
    loss_after[item] = after;
}

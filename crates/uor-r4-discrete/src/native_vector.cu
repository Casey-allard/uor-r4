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

// First-pick read, twin of `Q31FirstPickRead::apply`. One thread per row;
// every thread writes only its own row's words, so no atomics or reductions.
// Kernel A: query and key addresses, 16 exact s64 sums per side per row.
extern "C" __global__ void native_q31_read_address(
    const s64* stream,
    const s64* query_gates,
    const s64* key_gates,
    unsigned rows,
    unsigned width,
    s64* queries,
    s64* keys
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows) return;
    const s64* row = stream + (u64)item * (u64)width;
    s64 query[16];
    s64 key[16];
    for (unsigned c = 0; c < 16; ++c) {
        query[c] = 0;
        key[c] = 0;
    }
    s64 rotated[4];
    for (unsigned j = 0; j < width / 4; ++j) {
        unsigned base = (4 * j) % 16;
        hamilton_rotate_words(query_gates + 4ull * j, row + 4ull * j, rotated);
        for (unsigned k = 0; k < 4; ++k) query[base + k] += rotated[k];
        hamilton_rotate_words(key_gates + 4ull * j, row + 4ull * j, rotated);
        for (unsigned k = 0; k < 4; ++k) key[base + k] += rotated[k];
    }
    u64 at = (u64)item * 16ull;
    for (unsigned c = 0; c < 16; ++c) {
        queries[at + c] = query[c];
        keys[at + c] = key[c];
    }
}

// Kernel B: row `t` of each window picks the earlier row `s < t` whose key
// is nearest its query in u64 L1 distance, a tie going to the latest `s`,
// then adds the weighted table row of the token after the pick. A picked id
// outside the vocab leaves the row as the stream; the host refuses it.
extern "C" __global__ void native_q31_first_pick_read(
    const s64* queries,
    const s64* keys,
    const s64* stream,
    const unsigned* ids,
    const s64* table,
    const s64* weights,
    unsigned rows,
    unsigned time,
    unsigned width,
    unsigned long long vocab,
    unsigned shift,
    unsigned* picks,
    s64* out
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows) return;
    unsigned t = item % time;
    unsigned first = item - t;
    u64 at = (u64)item * (u64)width;
    for (unsigned l = 0; l < width; ++l) out[at + l] = stream[at + l];
    if (t == 0) {
        picks[item] = 0;
        return;
    }
    const s64* query = queries + (u64)item * 16ull;
    unsigned best = 0;
    u64 best_distance = ~0ull;
    for (unsigned s = 0; s < t; ++s) {
        const s64* key = keys + (u64)(first + s) * 16ull;
        u64 distance = 0;
        for (unsigned c = 0; c < 16; ++c) distance += abs_word(query[c], key[c]);
        if (distance <= best_distance) {
            best_distance = distance;
            best = s;
        }
    }
    picks[item] = best + 1;
    u64 id = (u64)ids[first + best + 1];
    if (id >= vocab) return;
    const s64* payload = table + id * (u64)width;
    s64 round = shift > 0 ? (1ll << (shift - 1)) : 0;
    for (unsigned l = 0; l < width; ++l) {
        s64 scaled = q31_saturate((weights[l] * payload[l] + round) >> shift);
        out[at + l] = q31_saturate(out[at + l] + scaled);
    }
}

// Gate update, twin of `Q31FirstPickRead::train_pick_step`. Every sum is
// exact in s64 or __int128, so the order of addition cannot change a word.
typedef __int128 s128;
typedef unsigned __int128 u128;

__device__ __forceinline__ u64 address_distance(const s64* query, const s64* key) {
    u64 distance = 0;
    for (unsigned c = 0; c < 16; ++c) distance += abs_word(query[c], key[c]);
    return distance;
}

__device__ __forceinline__ s64 sign_word(s64 value) {
    return (value > 0) - (value < 0);
}

// Kernel C: per row with a target, the rival (nearest other earlier key, a
// tie to the latest row), the hinge `max(0, d(q, k*) + m - d(q, k_r))` with
// `m >= 1` when the rival is later, as two u64 words, and the three sign
// residuals (query, target key, rival key), zero when the hinge is zero.
extern "C" __global__ void native_q31_read_pick_loss(
    const s64* queries,
    const s64* keys,
    const unsigned* targets,
    unsigned rows,
    unsigned time,
    u64 margin,
    unsigned* rivals,
    u64* loss,
    s64* residuals
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= rows) return;
    unsigned t = item % time;
    unsigned first = item - t;
    s64* residual = residuals + (u64)item * 48ull;
    for (unsigned c = 0; c < 48; ++c) residual[c] = 0;
    rivals[item] = 0;
    loss[2ull * item] = 0;
    loss[2ull * item + 1] = 0;
    unsigned target = targets[item];
    if (t == 0 || target == 0) return;
    unsigned a = target - 1;
    const s64* query = queries + (u64)item * 16ull;
    unsigned best = 0;
    bool found = false;
    u64 best_distance = ~0ull;
    for (unsigned s = 0; s < t; ++s) {
        if (s == a) continue;
        u64 distance = address_distance(query, keys + (u64)(first + s) * 16ull);
        if (distance <= best_distance) {
            best_distance = distance;
            best = s;
            found = true;
        }
    }
    if (!found) return;
    rivals[item] = best + 1;
    u64 m = best > a && margin == 0 ? 1 : margin;
    const s64* target_key = keys + (u64)(first + a) * 16ull;
    const s64* rival_key = keys + (u64)(first + best) * 16ull;
    s128 gap = (s128)address_distance(query, target_key) + (s128)m - (s128)best_distance;
    if (gap <= 0) return;
    u128 positive = (u128)gap;
    loss[2ull * item] = (u64)positive;
    loss[2ull * item + 1] = (u64)(positive >> 64);
    for (unsigned l = 0; l < 16; ++l) {
        s64 to_target = sign_word(query[l] - target_key[l]);
        s64 to_rival = sign_word(query[l] - rival_key[l]);
        residual[l] = to_target - to_rival;
        residual[16 + l] = -to_target;
        residual[32 + l] = to_rival;
    }
}

// Word `k` of `residual (x) conj(block * 2^shift)`, exact; the residual is in
// sign units. Multiplying by `2^shift` avoids shifting a negative __int128.
__device__ __forceinline__ s128 credit_word(
    const s64* residual,
    const s64* block,
    unsigned shift,
    unsigned k
) {
    s128 lift = (s128)1 << shift;
    s128 a0 = residual[0], a1 = residual[1], a2 = residual[2], a3 = residual[3];
    s128 c0 = (s128)block[0] * lift;
    s128 c1 = -((s128)block[1] * lift);
    s128 c2 = -((s128)block[2] * lift);
    s128 c3 = -((s128)block[3] * lift);
    if (k == 0) return a0 * c0 - a1 * c1 - a2 * c2 - a3 * c3;
    if (k == 1) return a0 * c1 + a1 * c0 + a2 * c3 - a3 * c2;
    if (k == 2) return a0 * c2 - a1 * c3 + a2 * c0 + a3 * c1;
    return a0 * c3 + a1 * c2 - a2 * c1 + a3 * c0;
}

// Kernel D: one thread per gate word, query gates first, then key gates. It
// runs over the rows in order and writes the candidate word
// `sat(gate - clamp(credit >> (step_shift + 1), +-2^32))`.
extern "C" __global__ void native_q31_read_gate_credit(
    const s64* stream,
    const s64* residuals,
    const unsigned* targets,
    const unsigned* rivals,
    const s64* query_gates,
    const s64* key_gates,
    unsigned rows,
    unsigned time,
    unsigned width,
    unsigned credit_shift,
    unsigned step_shift,
    s64* candidate_query,
    s64* candidate_key
) {
    unsigned item = blockIdx.x * blockDim.x + threadIdx.x;
    if (item >= 2 * width) return;
    bool on_key = item >= width;
    unsigned word = on_key ? item - width : item;
    unsigned j = word / 4;
    unsigned k = word % 4;
    unsigned base = (4 * j) % 16;
    s128 credit = 0;
    for (unsigned row = 0; row < rows; ++row) {
        if (rivals[row] == 0) continue;
        const s64* residual = residuals + (u64)row * 48ull;
        if (!on_key) {
            const s64* block = stream + (u64)row * (u64)width + 4ull * j;
            credit += credit_word(residual + base, block, credit_shift, k);
            continue;
        }
        unsigned first = row - row % time;
        u64 a = (u64)(first + targets[row] - 1);
        u64 b = (u64)(first + rivals[row] - 1);
        credit += credit_word(residual + 16 + base, stream + a * width + 4ull * j, credit_shift, k);
        credit += credit_word(residual + 32 + base, stream + b * width + 4ull * j, credit_shift, k);
    }
    s128 step = credit >> (step_shift + 1);
    s128 clamp = (s128)1 << 32;
    if (step > clamp) step = clamp;
    if (step < -clamp) step = -clamp;
    const s64* gates = on_key ? key_gates : query_gates;
    s64* candidate = on_key ? candidate_key : candidate_query;
    candidate[word] = q31_saturate(gates[word] - (s64)step);
}

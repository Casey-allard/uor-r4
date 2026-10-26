// The SpiralCore/FBS step of a UORLUT01 stack artifact, integer only
// (d11-exporter-float-trunk, claude-128).
//
// A port of `uor_r4_integer::stack`'s `stack_step_fbs`, its kernels and the
// native vector writer (crates/uor-r4-integer/src/stack/session.rs and
// kernels.rs), with every floor, round-half-up, saturation and wrapping
// convention of the Rust session. It is compiled twice: by nvcc into the
// evaluator's PTX module, and by the host C++ compiler into a twin that only
// qualifies the port against the Rust session (it never scores a panel).
//
// An FBS shape always has geometric gates, the icosian-snapped transport,
// learned rotations and the Hamilton gate, so only those branches are ported.
// Products are formed modulo 2^64 or 2^128 exactly as the Rust kernels' table
// products (equal to wrapping multiplication). No floating point anywhere.

#ifndef FBS_CORE_H
#define FBS_CORE_H

#include <stdint.h>

#if defined(__CUDACC__)
#define FBS_FN static __host__ __device__ inline
#else
#define FBS_FN static inline
#endif

typedef __int128 fbs_i128;
typedef unsigned __int128 fbs_u128;

#define FBS_RESIDUAL_EXP (-16)
#define FBS_PRODUCT_EXP (-32)
#define FBS_STATE_EXP (-32)
#define FBS_NORM_FINE_EXP (-24)
#define FBS_Q12_EXP (-12)
#define FBS_GRID_EXP_BIAS 64
#define FBS_CELLS 256
// PHI_Q32 - 2^32 (`uor_r4_integer::stack::PHI_Q32` = 6_949_403_065).
#define FBS_PHI_LO 2654435769ull

// One FBS projection: per output 8-block, the input atom and eight codes.
typedef struct {
  const uint16_t *atoms;
  const uint8_t *codes; // blocks * 8
  uint32_t blocks;
} fbs_map;

typedef struct {
  const int16_t *rec_gain;
  const int16_t *mlp_gain;
  fbs_map rec_in, rec_gate, rec_out, mlp_gate, mlp_up, mlp_down;
  const int16_t *taps; // 4 * width, [tap][channel]
  const int32_t *conv_bias;    // width
  const int32_t *gate_bias;    // width / 4 + width
  const int32_t *decay_offset; // width / 4
} fbs_layer;

typedef struct {
  uint32_t vocab, width, mlp, context, kept, layer_count;
  int64_t eps_mantissa;
  int32_t eps_exp;
  const fbs_layer *layers;
  const int16_t *final_gain;   // width
  const int16_t *writer_table; // vocab * width, Q12
  const int32_t *roots_q12;    // 16 * 8
  const int8_t *icosian;       // 120 * 4 * 2: H4_ROOT_COEFFICIENTS
  const uint64_t *keep_q31;    // 16: GEOMETRIC_KEEP_Q31
} fbs_model;

// One sequence's buffers and state; `fbs_work_len` gives each length.
typedef struct {
  int32_t *x;        // width
  int64_t *scratch;  // max(width, mlp)
  int32_t *branches; // 2 width
  int32_t *gate_out; // width / 4 + width
  int64_t *wide;     // width
  int16_t *act;      // max(width, mlp)
  int32_t *proj;     // width
  int32_t *gate;     // mlp
  int32_t *up;       // mlp
  int32_t *query;    // width
  int64_t *cells;    // 256
  int64_t *state;    // layers * width
  int32_t *history;  // layers * 3 width
} fbs_work;

// The bytes of one sequence's `fbs_work`, each buffer 8-byte aligned.
FBS_FN uint64_t fbs_align8(uint64_t n) {
  return (n + 7) & ~7ull;
}

#define FBS_WORK_LAYOUT(X)                                                                     \
  X(x, int32_t, d) X(scratch, int64_t, wide) X(branches, int32_t, 2 * d) X(gate_out, int32_t, g) \
  X(wide, int64_t, d) X(act, int16_t, wide) X(proj, int32_t, d) X(gate, int32_t, mlp)           \
  X(up, int32_t, mlp) X(query, int32_t, d) X(cells, int64_t, FBS_CELLS)                         \
  X(state, int64_t, (uint64_t)layers * d) X(history, int32_t, (uint64_t)layers * 3 * d)

#define FBS_WORK_DIMS(m)                                                                       \
  uint64_t d = (m)->width, mlp = (m)->mlp, wide = d > mlp ? d : mlp, g = d / 4 + d,             \
           layers = (m)->layer_count;                                                           \
  (void)d; (void)mlp; (void)wide; (void)g; (void)layers

FBS_FN uint64_t fbs_work_bytes(const fbs_model *m) {
  FBS_WORK_DIMS(m);
  uint64_t bytes = 0;
#define FBS_SIZE(name, type, len) bytes += fbs_align8((uint64_t)(len) * sizeof(type));
  FBS_WORK_LAYOUT(FBS_SIZE)
#undef FBS_SIZE
  return bytes;
}

// Carve one sequence's buffers from `base` (fbs_work_bytes(m) bytes, 8-aligned).
FBS_FN void fbs_work_carve(const fbs_model *m, uint8_t *base, fbs_work *w) {
  FBS_WORK_DIMS(m);
  uint64_t at = 0;
#define FBS_CARVE(name, type, len)                                                             \
  w->name = (type *)(base + at);                                                                \
  at += fbs_align8((uint64_t)(len) * sizeof(type));
  FBS_WORK_LAYOUT(FBS_CARVE)
#undef FBS_CARVE
}

// ---------------------------------------------------------------------------
// Scalar conventions.

FBS_FN int64_t fbs_sat_add64(int64_t a, int64_t b) {
  if (b > 0 && a > INT64_MAX - b) return INT64_MAX;
  if (b < 0 && a < INT64_MIN - b) return INT64_MIN;
  return a + b;
}

FBS_FN int64_t fbs_sat_neg64(int64_t v) { return v == INT64_MIN ? INT64_MAX : -v; }

FBS_FN int32_t fbs_sat_add32(int32_t a, int32_t b) {
  int64_t s = (int64_t)a + (int64_t)b;
  if (s > INT32_MAX) return INT32_MAX;
  if (s < INT32_MIN) return INT32_MIN;
  return (int32_t)s;
}

FBS_FN int64_t fbs_wrap_add64(int64_t a, int64_t b) { return (int64_t)((uint64_t)a + (uint64_t)b); }
FBS_FN int64_t fbs_wrap_sub64(int64_t a, int64_t b) { return (int64_t)((uint64_t)a - (uint64_t)b); }
FBS_FN int64_t fbs_wrap_neg64(int64_t a) { return (int64_t)(0ull - (uint64_t)a); }
FBS_FN int64_t fbs_shl64(int64_t a, uint32_t k) { return (int64_t)((uint64_t)a << k); }
FBS_FN fbs_i128 fbs_wrap_add128(fbs_i128 a, fbs_i128 b) { return (fbs_i128)((fbs_u128)a + (fbs_u128)b); }
FBS_FN fbs_i128 fbs_wrap_sub128(fbs_i128 a, fbs_i128 b) { return (fbs_i128)((fbs_u128)a - (fbs_u128)b); }
FBS_FN fbs_i128 fbs_wrap_neg128(fbs_i128 a) { return (fbs_i128)((fbs_u128)0 - (fbs_u128)a); }
FBS_FN uint64_t fbs_uabs64(int64_t v) { return v < 0 ? 0ull - (uint64_t)v : (uint64_t)v; }
FBS_FN uint32_t fbs_uabs32(int32_t v) { return v < 0 ? 0u - (uint32_t)v : (uint32_t)v; }

FBS_FN int32_t fbs_clz64(uint64_t v) {
  if (v == 0) return 64;
#if defined(__CUDA_ARCH__)
  return __clzll((long long)v);
#else
  return __builtin_clzll(v);
#endif
}

FBS_FN int32_t fbs_clz128(fbs_u128 v) {
  uint64_t hi = (uint64_t)(v >> 64);
  return hi != 0 ? fbs_clz64(hi) : 64 + fbs_clz64((uint64_t)v);
}

FBS_FN int32_t fbs_bit_length(uint64_t v) { return 64 - fbs_clz64(v); }

// Round-half-up arithmetic shift right by `s` (left if negative), saturating
// at the i64 range (kernels.rs `shift`).
FBS_FN int64_t fbs_shift(int64_t v, int32_t s) {
  if (s > 0) {
    if (s >= 63) return v < 0 ? -1 : 0;
    int64_t half = (int64_t)1 << (s - 1);
    return fbs_sat_add64(v, half) >> s;
  } else if (s < 0) {
    uint32_t k = 0u - (uint32_t)s;
    if (k > 63) k = 63;
    int64_t limit = INT64_MAX >> k;
    if (v > limit) return INT64_MAX;
    if (v < -limit) return INT64_MIN;
    return fbs_shl64(v, k);
  }
  return v;
}

// Round-half-up shift of an i128 right by `s`, saturated to i64.
FBS_FN int64_t fbs_shift_wide(fbs_i128 v, uint32_t s) {
  fbs_i128 r = s == 0 ? v : fbs_wrap_add128(v, (fbs_i128)1 << (s - 1)) >> s;
  if (r > (fbs_i128)INT64_MAX) return INT64_MAX;
  if (r < (fbs_i128)INT64_MIN) return INT64_MIN;
  return (int64_t)r;
}

FBS_FN int32_t fbs_to_exp_i32(int64_t v, int32_t from, int32_t to) {
  int64_t s = fbs_shift(v, to - from);
  if (s > INT32_MAX) return INT32_MAX;
  if (s < INT32_MIN) return INT32_MIN;
  return (int32_t)s;
}

// `(16 + m) a` by shifts and additions, wrapping (kernels.rs `scale_16_plus`).
FBS_FN int64_t fbs_scale_16_plus(int64_t a, uint8_t m) {
  int64_t t = fbs_wrap_add64(fbs_shl64(a, 1), ((m >> 3) & 1) ? a : 0);
  t = fbs_wrap_add64(fbs_shl64(t, 1), ((m >> 2) & 1) ? a : 0);
  t = fbs_wrap_add64(fbs_shl64(t, 1), ((m >> 1) & 1) ? a : 0);
  return fbs_wrap_add64(fbs_shl64(t, 1), (m & 1) ? a : 0);
}

// `scalar(code) value`, rounded half up (kernels.rs `grid_apply`).
FBS_FN int64_t fbs_grid_apply(int64_t v, int16_t code) {
  if (code == 0) return 0;
  uint16_t c = code < 0 ? (uint16_t)(0u - (uint32_t)(uint16_t)code) : (uint16_t)code;
  int32_t e = (int32_t)(c >> 4) - FBS_GRID_EXP_BIAS;
  int64_t scaled = fbs_shift(fbs_scale_16_plus(v, (uint8_t)(c & 15)), 4 - e);
  return code < 0 ? fbs_sat_neg64(scaled) : scaled;
}

FBS_FN fbs_i128 fbs_mul_i128(fbs_i128 a, fbs_i128 b) { return (fbs_i128)((fbs_u128)a * (fbs_u128)b); }

// Floor division by restoring long division (kernels.rs `stack_div_u64`).
FBS_FN uint64_t fbs_div_u64(uint64_t n, uint64_t d) {
  if (d == 0 || n < d) return 0;
  uint32_t sh = (uint32_t)(fbs_clz64(d) - fbs_clz64(n));
  uint64_t divisor = d << sh, place = 1ull << sh, rem = n, q = 0;
  for (;;) {
    if (rem >= divisor) {
      rem -= divisor;
      q |= place;
    }
    if (place == 1) return q;
    place >>= 1;
    divisor >>= 1;
  }
}

FBS_FN fbs_u128 fbs_div_u128(fbs_u128 n, fbs_u128 d) {
  if (d == 0 || n < d) return 0;
  uint32_t sh = (uint32_t)(fbs_clz128(d) - fbs_clz128(n));
  fbs_u128 divisor = d << sh, place = (fbs_u128)1 << sh, rem = n, q = 0;
  for (;;) {
    if (rem >= divisor) {
      rem -= divisor;
      q |= place;
    }
    if (place == 1) return q;
    place >>= 1;
    divisor >>= 1;
  }
}

// Floor square root, digit by digit (kernels.rs `stack_isqrt`).
FBS_FN fbs_u128 fbs_isqrt(fbs_u128 value) {
  fbs_u128 rem = value, root = 0, bit = (fbs_u128)1 << 126;
  while (bit > value) bit >>= 2;
  while (bit != 0) {
    if (rem >= root + bit) {
      rem -= root + bit;
      root = (root >> 1) + bit;
    } else {
      root >>= 1;
    }
    bit >>= 2;
  }
  return root;
}

// ---------------------------------------------------------------------------
// Vector kernels.

// Requantize `v * 2^exp` to 16 bits; returns the new exponent.
FBS_FN int32_t fbs_quantize16(const int64_t *v, uint32_t n, int32_t exp, int16_t *out) {
  uint64_t max = 0;
  for (uint32_t i = 0; i < n; i++) {
    uint64_t a = fbs_uabs64(v[i]);
    if (a > max) max = a;
  }
  int32_t s = fbs_bit_length(max) - 15;
  if (s < 0) s = 0;
  for (uint32_t i = 0; i < n; i++) {
    int64_t y = fbs_shift(v[i], s);
    if (y > 32767) y = 32767;
    if (y < -32767) y = -32767;
    out[i] = (int16_t)y;
  }
  return exp + s;
}

// RMSNorm without its gain, `x / rms(x)` at 2^-24, rounded to nearest
// (kernels.rs `stack_rms_norm_fine`).
FBS_FN void fbs_rms_norm_fine(const int32_t *x, uint32_t n, int64_t eps_mantissa, int32_t eps_exp,
                              int64_t *out) {
  uint32_t max = 0;
  for (uint32_t i = 0; i < n; i++) {
    uint32_t a = fbs_uabs32(x[i]);
    if (a > max) max = a;
  }
  int32_t sh = fbs_bit_length((uint64_t)max) - 24;
  if (sh < 0) sh = 0;
  uint64_t sum = 0;
  for (uint32_t i = 0; i < n; i++) {
    int64_t y = fbs_shift((int64_t)x[i], sh);
    out[i] = y;
    uint64_t u = fbs_uabs64(y);
    sum += u * u;
  }
  int64_t mean = (int64_t)fbs_div_u64(sum, n > 0 ? n : 1);
  int32_t square_exp = (FBS_RESIDUAL_EXP + sh) * 2;
  int64_t eps_int = fbs_shift(eps_mantissa, square_exp - eps_exp);
  if (eps_int < 0) eps_int = 0;
  int64_t ms_signed = fbs_wrap_add64(mean, eps_int);
  if (ms_signed < 1) ms_signed = 1;
  fbs_u128 ms = (fbs_u128)(uint64_t)ms_signed;
  fbs_u128 s = fbs_isqrt(ms << 68);
  if (s < 1) s = 1;
  fbs_u128 r = fbs_div_u128((fbs_u128)1 << 96, s);
  for (uint32_t i = 0; i < n; i++) {
    // |v| <= 2^24, so `r u` is exact (the Rust kernel's seven radix-16 digits).
    uint64_t u = fbs_uabs64(out[i]);
    fbs_u128 magnitude = r * (fbs_u128)u;
    fbs_i128 product = out[i] < 0 ? fbs_wrap_neg128((fbs_i128)magnitude) : (fbs_i128)magnitude;
    fbs_i128 y = fbs_wrap_add128(product, (fbs_i128)1 << 37) >> 38;
    if (y > (fbs_i128)INT64_MAX) y = INT64_MAX;
    if (y < (fbs_i128)INT64_MIN) y = INT64_MIN;
    out[i] = (int64_t)y;
  }
}

FBS_FN void fbs_norm_gain(int64_t *v, const int16_t *codes, uint32_t n) {
  for (uint32_t i = 0; i < n; i++) v[i] = fbs_grid_apply(v[i], codes[i]);
}

// One FBS projection from an i64 input (session.rs `stack_fbs_gather`).
FBS_FN void fbs_gather64(const fbs_map *map, const int64_t *x, int32_t x_exp, int32_t *out) {
  for (uint32_t o = 0; o < map->blocks; o++) {
    const int64_t *values = x + ((uint32_t)map->atoms[o] << 3);
    for (uint32_t j = 0; j < 8; j++) {
      uint8_t code = map->codes[o * 8 + j];
      int64_t v = values[code & 7];
      if (code & 8) v = fbs_sat_neg64(v);
      out[o * 8 + j] = fbs_to_exp_i32(v, x_exp, FBS_RESIDUAL_EXP);
    }
  }
}

// One FBS projection from 16-bit activations.
FBS_FN void fbs_gather16(const fbs_map *map, const int16_t *x, int32_t x_exp, int32_t *out) {
  for (uint32_t o = 0; o < map->blocks; o++) {
    const int16_t *values = x + ((uint32_t)map->atoms[o] << 3);
    for (uint32_t j = 0; j < 8; j++) {
      uint8_t code = map->codes[o * 8 + j];
      int64_t v = (int64_t)values[code & 7];
      if (code & 8) v = fbs_sat_neg64(v);
      out[o * 8 + j] = fbs_to_exp_i32(v, x_exp, FBS_RESIDUAL_EXP);
    }
  }
}

// Hamilton product a (x) b, exactly modulo 2^128.
FBS_FN void fbs_hamilton(const int64_t a[4], const int64_t b[4], fbs_i128 out[4]) {
#define FBS_P(i, j) fbs_mul_i128((fbs_i128)a[i], (fbs_i128)b[j])
  out[0] = fbs_wrap_sub128(fbs_wrap_sub128(fbs_wrap_sub128(FBS_P(0, 0), FBS_P(1, 1)), FBS_P(2, 2)), FBS_P(3, 3));
  out[1] = fbs_wrap_sub128(fbs_wrap_add128(fbs_wrap_add128(FBS_P(0, 1), FBS_P(1, 0)), FBS_P(2, 3)), FBS_P(3, 2));
  out[2] = fbs_wrap_add128(fbs_wrap_add128(fbs_wrap_sub128(FBS_P(0, 2), FBS_P(1, 3)), FBS_P(2, 0)), FBS_P(3, 1));
  out[3] = fbs_wrap_add128(fbs_wrap_sub128(fbs_wrap_add128(FBS_P(0, 3), FBS_P(1, 2)), FBS_P(2, 1)), FBS_P(3, 0));
#undef FBS_P
}

// `v k` for an icosian coefficient k in -2..=2 (kernels.rs `small_mul`).
FBS_FN int64_t fbs_small_mul(int64_t v, int8_t k) {
  switch (k) {
  case -2: return fbs_wrap_neg64(fbs_shl64(v, 1));
  case -1: return fbs_wrap_neg64(v);
  case 0: return 0;
  case 1: return v;
  case 2: return fbs_shl64(v, 1);
  default: {
    uint8_t remaining = k < 0 ? (uint8_t)(0u - (uint32_t)(uint8_t)k) : (uint8_t)k;
    int64_t addend = v, term = 0;
    while (remaining != 0) {
      if (remaining & 1) term = fbs_wrap_add64(term, addend);
      remaining >>= 1;
      if (remaining != 0) addend = fbs_shl64(addend, 1);
    }
    return k < 0 ? fbs_wrap_neg64(term) : term;
  }
  }
}

// Whether `da + db phi > 0`, exactly (kernels.rs `snap_difference_positive`).
FBS_FN int fbs_snap_difference_positive(int64_t da, int64_t db) {
  int64_t p = fbs_wrap_add64(fbs_shl64(da, 1), db);
  if (db == 0) return p > 0;
  if (db > 0) {
    if (p >= 0) return 1;
    fbs_u128 b2 = (fbs_u128)(uint64_t)db * (fbs_u128)(uint64_t)db;
    fbs_u128 pa = (fbs_u128)fbs_uabs64(p);
    return (b2 << 2) + b2 > pa * pa;
  }
  if (p <= 0) return 0;
  fbs_u128 ba = (fbs_u128)fbs_uabs64(db);
  fbs_u128 b2 = ba * ba;
  fbs_u128 pp = (fbs_u128)(uint64_t)p;
  return pp * pp > (b2 << 2) + b2;
}

// The icosian root with the largest exact dot product with `raw`, the lowest
// index on a tie (kernels.rs `stack_snap_select`).
FBS_FN uint32_t fbs_snap_select(const int8_t *icosian, const int32_t raw[4]) {
  uint32_t best = 0;
  int64_t best_a = 0, best_b = 0;
  for (uint32_t index = 0; index < 120; index++) {
    const int8_t *root = icosian + index * 8;
    int64_t a = 0, b = 0;
    for (uint32_t c = 0; c < 4; c++) {
      int64_t v = (int64_t)raw[c];
      a = fbs_wrap_add64(a, fbs_small_mul(v, root[c * 2]));
      b = fbs_wrap_add64(b, fbs_small_mul(v, root[c * 2 + 1]));
    }
    if (index == 0 || fbs_snap_difference_positive(fbs_wrap_sub64(a, best_a), fbs_wrap_sub64(b, best_b))) {
      best = index;
      best_a = a;
      best_b = b;
    }
  }
  return best;
}

// The snapped transition: the chosen root scaled by `lambda` (Q31), at Q31
// (kernels.rs `stack_snap_rotation`).
FBS_FN void fbs_snap_rotation(const int8_t *icosian, const int32_t raw[4], uint64_t lambda, int64_t out[4]) {
  const int8_t *root = icosian + fbs_snap_select(icosian, raw) * 8;
  uint64_t lambda_phi = lambda + ((lambda * FBS_PHI_LO + (1ull << 31)) >> 32);
  for (uint32_t c = 0; c < 4; c++) {
    int64_t sum = fbs_wrap_add64(fbs_small_mul((int64_t)lambda, root[c * 2]),
                                 fbs_small_mul((int64_t)lambda_phi, root[c * 2 + 1]));
    out[c] = fbs_shift(sum, 1);
  }
}

// ---------------------------------------------------------------------------
// The recurrence core, geometric with snapped rotations (session.rs
// `stack_recurrence_core` and `stack_lanes`). Returns the exponent of
// `w->act[..width]`.
FBS_FN int32_t fbs_recurrence(const fbs_model *m, const fbs_layer *layer, int64_t *state,
                              int32_t *history, fbs_work *w) {
  uint32_t d = m->width, lanes = d / 4, g = lanes + d;
  for (uint32_t i = 0; i < g; i++) w->gate_out[i] = fbs_sat_add32(w->gate_out[i], layer->gate_bias[i]);
  const int32_t *drive = w->branches, *gate = w->branches + d;
  const int16_t *tap0 = layer->taps, *tap1 = tap0 + d, *tap2 = tap1 + d, *tap3 = tap2 + d;
  const int32_t *back1 = history, *back2 = history + d, *back3 = history + 2 * d;
  for (uint32_t i = 0; i < d; i++) {
    int64_t c = (int64_t)layer->conv_bias[i];
    c = fbs_wrap_add64(c, fbs_grid_apply((int64_t)drive[i], tap0[i]));
    c = fbs_wrap_add64(c, fbs_grid_apply((int64_t)back1[i], tap1[i]));
    c = fbs_wrap_add64(c, fbs_grid_apply((int64_t)back2[i], tap2[i]));
    c = fbs_wrap_add64(c, fbs_grid_apply((int64_t)back3[i], tap3[i]));
    w->wide[i] = c;
  }
  // Age the history: rows 0 and 1 move to rows 1 and 2, the drive into row 0.
  for (uint32_t i = 3 * d; i-- > d;) history[i] = history[i - d];
  for (uint32_t i = 0; i < d; i++) history[i] = drive[i];
  const int32_t *decays = w->gate_out, *rotations = w->gate_out + lanes;
  for (uint32_t lane = 0; lane < lanes; lane++) {
    int32_t s_q16 = fbs_sat_add32(decays[lane], layer->decay_offset[lane]);
    if (s_q16 < 0) s_q16 = 0;
    if (s_q16 > (1 << 16)) s_q16 = 1 << 16;
    uint32_t idx = (uint32_t)(s_q16 >> 12);
    if (idx > 15) idx = 15;
    uint64_t lambda = 1ull << (30 - idx), keep = m->keep_q31[idx];
    int32_t raw[4] = {rotations[lane * 4], rotations[lane * 4 + 1], rotations[lane * 4 + 2], rotations[lane * 4 + 3]};
    int64_t transition[4];
    fbs_snap_rotation(m->icosian, raw, lambda, transition);
    int64_t *held = state + lane * 4;
    int64_t h[4] = {held[0], held[1], held[2], held[3]};
    fbs_i128 moved[4];
    fbs_hamilton(transition, h, moved);
    for (uint32_t k = 0; k < 4; k++) {
      fbs_i128 drive_term = fbs_mul_i128((fbs_i128)keep, (fbs_i128)w->wide[lane * 4 + k]);
      fbs_i128 pushed = (fbs_i128)((fbs_u128)drive_term << 16);
      held[k] = fbs_shift_wide(fbs_wrap_add128(moved[k], pushed), 31);
    }
  }
  // The icosian-snapped Hamilton output gate on the state.
  for (uint32_t lane = 0; lane < lanes; lane++) {
    int32_t raw[4] = {gate[lane * 4], gate[lane * 4 + 1], gate[lane * 4 + 2], gate[lane * 4 + 3]};
    int64_t q[4];
    fbs_snap_rotation(m->icosian, raw, 1ull << 31, q);
    int64_t rhs[4];
    for (uint32_t k = 0; k < 4; k++) rhs[k] = fbs_shift(state[lane * 4 + k], FBS_RESIDUAL_EXP - FBS_STATE_EXP);
    fbs_i128 prod[4];
    fbs_hamilton(q, rhs, prod);
    for (uint32_t k = 0; k < 4; k++) w->scratch[lane * 4 + k] = fbs_shift_wide(prod[k], 15);
  }
  return fbs_quantize16(w->scratch, d, FBS_PRODUCT_EXP, w->act);
}

// The Hamilton gate of the feed-forward block (session.rs `stack_swiglu`).
FBS_FN void fbs_hamilton_gate(const fbs_model *m, const int32_t *gate, const int32_t *up, int64_t *out) {
  for (uint32_t j = 0; j < m->mlp / 4; j++) {
    int32_t raw[4] = {gate[j * 4], gate[j * 4 + 1], gate[j * 4 + 2], gate[j * 4 + 3]};
    int64_t q[4];
    fbs_snap_rotation(m->icosian, raw, 1ull << 31, q);
    int64_t rhs[4] = {(int64_t)up[j * 4], (int64_t)up[j * 4 + 1], (int64_t)up[j * 4 + 2], (int64_t)up[j * 4 + 3]};
    fbs_i128 prod[4];
    fbs_hamilton(q, rhs, prod);
    for (uint32_t k = 0; k < 4; k++) out[j * 4 + k] = fbs_shift_wide(prod[k], 15);
  }
}

FBS_FN int64_t fbs_neg_l1_8(const int32_t *x, const int32_t *y) {
  int64_t sum = 0;
  for (uint32_t i = 0; i < 8; i++) {
    int64_t diff = (int64_t)x[i] - (int64_t)y[i];
    sum = fbs_wrap_add64(sum, diff < 0 ? -diff : diff);
  }
  return fbs_wrap_neg64(sum);
}

// The native vector writer (session.rs `stack_native_writer`).
FBS_FN uint32_t fbs_writer(const fbs_model *m, const int32_t *q, int64_t *cells, uint32_t *kept) {
  uint32_t d = m->width;
  int64_t port_a[16], port_b[16];
  for (uint32_t r = 0; r < 16; r++) {
    port_a[r] = fbs_neg_l1_8(q, m->roots_q12 + r * 8);
    port_b[r] = fbs_neg_l1_8(q + d - 8, m->roots_q12 + r * 8);
  }
  for (uint32_t c = 0; c < FBS_CELLS; c++) cells[c] = fbs_wrap_add64(port_a[c >> 4], port_b[c & 15]);
  for (uint32_t k = 0; k < m->kept; k++) {
    uint32_t best = 0;
    for (uint32_t c = 1; c < FBS_CELLS; c++)
      if (cells[c] > cells[best]) best = c;
    kept[k] = best;
    cells[best] = INT64_MIN;
  }
  uint32_t token = UINT32_MAX;
  int64_t token_score = INT64_MIN;
  for (uint32_t k = 0; k < m->kept; k++) {
    for (uint32_t candidate = kept[k]; candidate < m->vocab; candidate += FBS_CELLS) {
      const int16_t *row = m->writer_table + (uint64_t)candidate * d;
      int64_t sum = 0;
      for (uint32_t i = 0; i < d; i++) {
        int64_t diff = (int64_t)q[i] - (int64_t)row[i];
        sum = fbs_wrap_add64(sum, diff < 0 ? -diff : diff);
      }
      int64_t score = fbs_wrap_neg64(sum);
      if (score > token_score || (score == token_score && candidate < token)) {
        token = candidate;
        token_score = score;
      }
    }
  }
  return token;
}

// Errors of `fbs_step`.
#define FBS_OK 0
#define FBS_ERR_TOKEN 1
#define FBS_ERR_CONTEXT 2

// One step of the FBS session (session.rs `stack_step_fbs`): the token's
// writer row in, the layers, the final norm and the writer. Writes the chosen
// token, the kept cells (in selection order) and the Q12 query.
FBS_FN int fbs_step(const fbs_model *m, fbs_work *w, uint32_t token, uint32_t *position,
                    uint32_t *out_token, uint32_t *out_kept, int32_t *out_query) {
  uint32_t d = m->width, mlp = m->mlp;
  if (token >= m->vocab) return FBS_ERR_TOKEN;
  if (*position >= m->context) return FBS_ERR_CONTEXT;
  const int16_t *row = m->writer_table + (uint64_t)token * d;
  for (uint32_t i = 0; i < d; i++) w->x[i] = fbs_to_exp_i32((int64_t)row[i], FBS_Q12_EXP, FBS_RESIDUAL_EXP);
  for (uint32_t l = 0; l < m->layer_count; l++) {
    const fbs_layer *layer = m->layers + l;
    int64_t *state = w->state + (uint64_t)l * d;
    int32_t *history = w->history + (uint64_t)l * 3 * d;
    fbs_rms_norm_fine(w->x, d, m->eps_mantissa, m->eps_exp, w->scratch);
    fbs_norm_gain(w->scratch, layer->rec_gain, d);
    fbs_gather64(&layer->rec_in, w->scratch, FBS_NORM_FINE_EXP, w->branches);
    fbs_gather64(&layer->rec_gate, w->scratch, FBS_NORM_FINE_EXP, w->gate_out);
    int32_t act_exp = fbs_recurrence(m, layer, state, history, w);
    fbs_gather16(&layer->rec_out, w->act, act_exp, w->proj);
    for (uint32_t i = 0; i < d; i++) w->x[i] = fbs_sat_add32(w->x[i], w->proj[i]);
    fbs_rms_norm_fine(w->x, d, m->eps_mantissa, m->eps_exp, w->scratch);
    fbs_norm_gain(w->scratch, layer->mlp_gain, d);
    fbs_gather64(&layer->mlp_gate, w->scratch, FBS_NORM_FINE_EXP, w->gate);
    fbs_gather64(&layer->mlp_up, w->scratch, FBS_NORM_FINE_EXP, w->up);
    fbs_hamilton_gate(m, w->gate, w->up, w->scratch);
    act_exp = fbs_quantize16(w->scratch, mlp, FBS_PRODUCT_EXP, w->act);
    fbs_gather16(&layer->mlp_down, w->act, act_exp, w->proj);
    for (uint32_t i = 0; i < d; i++) w->x[i] = fbs_sat_add32(w->x[i], w->proj[i]);
  }
  fbs_rms_norm_fine(w->x, d, m->eps_mantissa, m->eps_exp, w->scratch);
  fbs_norm_gain(w->scratch, m->final_gain, d);
  for (uint32_t i = 0; i < d; i++) {
    int32_t q = fbs_to_exp_i32(w->scratch[i], FBS_NORM_FINE_EXP, FBS_Q12_EXP);
    if (q > 32767) q = 32767;
    if (q < -32768) q = -32768;
    w->query[i] = q;
    out_query[i] = q;
  }
  *out_token = fbs_writer(m, w->query, w->cells, out_kept);
  *position += 1;
  return FBS_OK;
}

// A whole sequence from a fresh session: `len` tokens, one output per
// position. `w`'s state and history are zeroed first.
FBS_FN int fbs_sequence(const fbs_model *m, fbs_work *w, const uint32_t *tokens, uint32_t len,
                        uint32_t *out_tokens, uint32_t *out_kept, int32_t *out_query) {
  uint32_t d = m->width;
  for (uint64_t i = 0; i < (uint64_t)m->layer_count * d; i++) w->state[i] = 0;
  for (uint64_t i = 0; i < (uint64_t)m->layer_count * 3 * d; i++) w->history[i] = 0;
  uint32_t position = 0;
  for (uint32_t t = 0; t < len; t++) {
    int err = fbs_step(m, w, tokens[t], &position, out_tokens + t, out_kept + (uint64_t)t * m->kept,
                       out_query + (uint64_t)t * d);
    if (err != FBS_OK) return err;
  }
  return FBS_OK;
}

#endif // FBS_CORE_H

// Prelude for the SpiralCore/FBS sheet-arm module (spiralcore-fbs-offline-float-sheet,
// claude-119). This module is built on its own (build.rs) from this prelude, the pinned
// src/spiralcore_fbs.cu (included unchanged, for its route helpers) and
// src/spiralcore_fbs_arms.cu. F32 storage only: qualified execution is F32 CUDA.
typedef unsigned int uint;
typedef unsigned long long u64;
typedef float act_t;
__device__ __forceinline__ float act_to_f(act_t v) { return v; }
__device__ __forceinline__ act_t act_from_f(float v) { return v; }

#define _USE_MATH_DEFINES
#include<math.h>
#include<stdint.h>
#include "cuda_utils.cuh"

#define UNARY_OP(TYPENAME, FN_NAME, FUNC) \
extern "C" __global__ void FN_NAME( \
    const size_t numel, \
    const size_t num_dims, \
    const size_t *info, \
    const TYPENAME *inp, \
    TYPENAME *out \
) { \
    const size_t *dims = info; \
    const size_t *strides = info + num_dims; \
    if (info == nullptr || is_contiguous(num_dims, dims, strides)) { \
        for (unsigned int i = blockIdx.x * blockDim.x + threadIdx.x; i < numel; i += blockDim.x * gridDim.x) { \
            TYPENAME x = inp ? inp[i] : out[i]; \
            out[i] = FUNC; \
        } \
    } \
    else { \
        for (unsigned int i = blockIdx.x * blockDim.x + threadIdx.x; i < numel; i += blockDim.x * gridDim.x) { \
            unsigned strided_i = get_strided_index(i, num_dims, dims, strides); \
            TYPENAME x = inp ? inp[strided_i] : out[i]; \
            out[i] = FUNC; \
        } \
    } \
} \







template<typename T>
__device__ __forceinline__ T relu_fwd(T x) {
    T zero = 0.;
    return maxg(x, zero);
}





#define UNARY_OP1(TYPENAME, FN_NAME, FUNC) \
extern "C" __global__ void FN_NAME( \
    const size_t numel, \
    const size_t num_dims, \
    const size_t *info, \
    const TYPENAME param, \
    const TYPENAME *inp, \
    TYPENAME *out \
) { \
    const size_t *dims = info; \
    const size_t *strides = info + num_dims; \
    if (info == nullptr || is_contiguous(num_dims, dims, strides)) { \
        for (unsigned int i = blockIdx.x * blockDim.x + threadIdx.x; i < numel; i += blockDim.x * gridDim.x) { \
            TYPENAME x = inp ? inp[i] : out[i]; \
            out[i] = FUNC; \
        } \
    } \
    else { \
        for (unsigned int i = blockIdx.x * blockDim.x + threadIdx.x; i < numel; i += blockDim.x * gridDim.x) { \
            unsigned strided_i = get_strided_index(i, num_dims, dims, strides); \
            TYPENAME x = inp ? inp[strided_i] : out[i]; \
            out[i] = FUNC; \
        } \
    } \
} \

template<typename T>
__device__ T sign_(T t) {
  return static_cast<T>(t > static_cast<T>(0)) - static_cast<T>(t < static_cast<T>(0));
}


#if __CUDA_ARCH__ >= 800
UNARY_OP(__nv_bfloat16, ucopy_bf16, x)
UNARY_OP(__nv_bfloat16, uneg_bf16, -x)
UNARY_OP(__nv_bfloat16, urecip_bf16, recipg(x))
UNARY_OP(__nv_bfloat16, usin_bf16, sing(x))
UNARY_OP(__nv_bfloat16, ucos_bf16, cosg(x))
UNARY_OP(__nv_bfloat16, uceil_bf16, ceilg(x))
UNARY_OP(__nv_bfloat16, ufloor_bf16, floorg(x))
UNARY_OP(__nv_bfloat16, uround_bf16, roundg(x))
UNARY_OP(__nv_bfloat16, uabs_bf16, absg(x))
UNARY_OP(__nv_bfloat16, usqr_bf16, x*x)
UNARY_OP(__nv_bfloat16, usqrt_bf16, sqrtg(x))
UNARY_OP(__nv_bfloat16, urelu_bf16, relu_fwd(x))
UNARY_OP(__nv_bfloat16, usign_bf16, sign_(x))
#endif

#if __CUDA_ARCH__ >= 890
#define F8E4M3_TO_FLOAT(x) __half2float(__nv_cvt_fp8_to_halfraw(x.__x, __NV_E4M3))

UNARY_OP(__nv_fp8_e4m3, ucopy_f8_e4m3, x)
UNARY_OP(__nv_fp8_e4m3, uneg_fp8_e4m3, __nv_fp8_e4m3(-F8E4M3_TO_FLOAT(x)))
UNARY_OP(__nv_fp8_e4m3, urecip_fp8_e4m3, recipg(x))
UNARY_OP(__nv_fp8_e4m3, usin_fp8_e4m3, sing(x))
UNARY_OP(__nv_fp8_e4m3, ucos_fp8_e4m3, cosg(x))
UNARY_OP(__nv_fp8_e4m3, uceil_fp8_e4m3, ceilg(x))
UNARY_OP(__nv_fp8_e4m3, ufloor_fp8_e4m3, floorg(x))
UNARY_OP(__nv_fp8_e4m3, uround_fp8_e4m3, roundg(x))
UNARY_OP(__nv_fp8_e4m3, uabs_fp8_e4m3, absg(x))
UNARY_OP(__nv_fp8_e4m3, usqr_fp8_e4m3, __nv_fp8_e4m3(F8E4M3_TO_FLOAT(x)*F8E4M3_TO_FLOAT(x)))
UNARY_OP(__nv_fp8_e4m3, usqrt_fp8_e4m3, sqrtg(x))
UNARY_OP(__nv_fp8_e4m3, urelu_fp8_e4m3, __nv_fp8_e4m3(relu_fwd(F8E4M3_TO_FLOAT(x))))
UNARY_OP(__nv_fp8_e4m3, usign_fp8_e4m3, __nv_fp8_e4m3(sign_(F8E4M3_TO_FLOAT(x))))
#endif

#if __CUDA_ARCH__ >= 530
UNARY_OP(__half, ucopy_f16, x)
UNARY_OP(__half, uneg_f16, -x)
UNARY_OP(__half, urecip_f16, recipg(x))
UNARY_OP(__half, usin_f16, sing(x))
UNARY_OP(__half, ucos_f16, cosg(x))
UNARY_OP(__half, uceil_f16, ceilg(x))
UNARY_OP(__half, ufloor_f16, floorg(x))
UNARY_OP(__half, uround_f16, roundg(x))
UNARY_OP(__half, uabs_f16, absg(x))
UNARY_OP(__half, usqr_f16, x*x)
UNARY_OP(__half, usqrt_f16, sqrtg(x))
UNARY_OP(__half, urelu_f16, relu_fwd(x))
UNARY_OP(__half, usign_f16, sign_(x))
#endif

UNARY_OP(uint8_t, ucopy_u8, x)
UNARY_OP(uint32_t, ucopy_u32, x)
UNARY_OP(int64_t, ucopy_i64, x)
UNARY_OP(float, ucopy_f32, x)
UNARY_OP(double, ucopy_f64, x)
UNARY_OP(float, uneg_f32, -x)
UNARY_OP(double, uneg_f64, -x)
UNARY_OP(float, urecip_f32, recipg(x))
UNARY_OP(double, urecip_f64, recipg(x))
UNARY_OP(float, usin_f32, sing(x))
UNARY_OP(double, usin_f64, sing(x))
UNARY_OP(float, ucos_f32, cosg(x))
UNARY_OP(double, ucos_f64, cosg(x))
UNARY_OP(float, uceil_f32, ceilg(x))
UNARY_OP(double, uceil_f64, ceilg(x))
UNARY_OP(float, ufloor_f32, floorg(x))
UNARY_OP(double, ufloor_f64, floorg(x))
UNARY_OP(float, uround_f32, roundg(x))
UNARY_OP(double, uround_f64, roundg(x))
UNARY_OP(float, uabs_f32, absg(x))
UNARY_OP(double, uabs_f64, absg(x))
UNARY_OP(float, usqr_f32, x*x)
UNARY_OP(double, usqr_f64, x*x)
UNARY_OP(float, usqrt_f32, sqrtg(x))
UNARY_OP(double, usqrt_f64, sqrtg(x))
UNARY_OP(float, urelu_f32, relu_fwd(x))
UNARY_OP(double, urelu_f64, relu_fwd(x))
UNARY_OP(float, usign_f32, sign_(x))
UNARY_OP(double, usign_f64, sign_(x))

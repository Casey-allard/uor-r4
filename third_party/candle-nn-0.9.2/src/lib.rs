//! candle-nn
//!
//! ## Other Crates
//!
//! Candle consists of a number of crates. This crate holds structs and functions
//! that allow you to build and train neural nets. You may wish
//! to look at the docs for the other crates which can be found here:
//!
//! - [candle-core](https://docs.rs/candle-core/). Core Datastructures and DataTypes.
//! - [candle-nn](https://docs.rs/candle-nn/). Building blocks for Neural Nets.
//! - [candle-datasets](https://docs.rs/candle-datasets/). Rust access to commonly used Datasets like MNIST.
//! - [candle-examples](https://docs.rs/candle-examples/). Examples of Candle in Use.
//! - [candle-onnx](https://docs.rs/candle-onnx/). Loading and using ONNX models.
//! - [candle-pyo3](https://docs.rs/candle-pyo3/). Access to Candle from Python.
//! - [candle-transformers](https://docs.rs/candle-transformers/). Candle implementation of many published transformer models.
//!

#[cfg(any(
    feature = "metal",
    feature = "accelerate",
    feature = "mkl",
    feature = "cudnn"
))]
compile_error!("D29: Candle NN metal/accelerate/mkl/cudnn backends are inactive; use the reviewed native CUDA boundary. Compatibility feature names activate no vendor dependencies.");

pub mod activation;
pub mod batch_norm;
pub mod conv;
pub mod cpu_flash_attention;
pub mod embedding;
pub mod encoding;
pub mod func;
pub mod group_norm;
pub mod init;
pub mod kv_cache;
pub mod layer_norm;
pub mod linear;
pub mod loss;
pub mod moe;
pub mod ops;
pub mod optim;
pub mod rnn;
#[cfg(any())] // Quarantined; historical/src/rotary_emb.rs preserves the upstream source.
pub mod rotary_emb;
pub mod sampling;
pub mod sequential;
pub mod var_builder;
pub mod var_map;

pub use activation::{prelu, Activation, PReLU};
pub use batch_norm::{batch_norm, BatchNorm, BatchNormConfig};
pub use conv::{
    conv1d, conv1d_no_bias, conv2d, conv2d_no_bias, conv_transpose1d, conv_transpose1d_no_bias,
    conv_transpose2d, conv_transpose2d_no_bias, Conv1d, Conv1dConfig, Conv2d, Conv2dConfig,
    ConvTranspose1d, ConvTranspose1dConfig, ConvTranspose2d, ConvTranspose2dConfig,
};
pub use embedding::{embedding, Embedding};
pub use func::{func, func_t, Func, FuncT};
pub use group_norm::{group_norm, GroupNorm};
pub use init::Init;
pub use layer_norm::{
    layer_norm, layer_norm_no_bias, rms_norm, LayerNorm, LayerNormConfig, RmsNorm,
};
pub use linear::{linear, linear_b, linear_no_bias, Linear};
pub use ops::Dropout;
pub use optim::{AdamW, Optimizer, ParamsAdamW, SGD};
pub use rnn::{gru, lstm, GRUConfig, LSTMConfig, GRU, LSTM, RNN};
pub use sequential::{seq, Sequential};
pub use var_builder::VarBuilder;
pub use var_map::VarMap;

pub use candle::{Module, ModuleT};

#[cfg(test)]
mod uor_boundary_tests {
    use super::*;

    /// Fixed host storage is an operand for admission checks only. Every call
    /// below must return before any model arithmetic or device launch.
    #[test]
    fn prohibited_entries_and_cpu_norm_refuse_without_model_compute() -> candle::Result<()> {
        let fixture = candle::Tensor::from_slice(&[0f32], (1, 1), &candle::Device::Cpu)?;
        for result in [
            ops::softmax(&fixture, 1),
            ops::log_softmax(&fixture, 1),
            ops::softmax_last_dim(&fixture),
            ops::sigmoid(&fixture),
            ops::hard_sigmoid(&fixture),
            ops::silu(&fixture),
            ops::swiglu(&fixture),
            ops::mish(&fixture),
            ops::selu(&fixture, 1., 1.),
            ops::sdpa(&fixture, &fixture, &fixture, None, false, 1., 1.),
            loss::nll(&fixture, &fixture),
            loss::cross_entropy(&fixture, &fixture),
            loss::binary_cross_entropy_with_logit(&fixture, &fixture),
            sampling::gumbel_softmax(&fixture, 1., 1),
            cpu_flash_attention::run_flash_attn_cpu::<f32>(
                &fixture, &fixture, &fixture, None, 1., None, None,
            ),
            moe::moe_gemm(&fixture, &fixture, &None, &fixture, &fixture, 1, false),
            Linear::new(fixture.clone(), None).forward(&fixture),
            ops::rms_norm(&fixture, &fixture, 1e-5),
            ops::layer_norm(&fixture, &fixture, &fixture, 1e-5),
            Activation::Gelu.forward(&fixture),
        ] {
            let error = result.err().ok_or_else(|| {
                candle::Error::Msg("prohibited entry unexpectedly returned a tensor".into())
            })?;
            let text = error.to_string();
            assert!(
                text.contains("D24") || text.contains("D29"),
                "wrong refusal: {text}"
            );
        }
        Ok(())
    }
}

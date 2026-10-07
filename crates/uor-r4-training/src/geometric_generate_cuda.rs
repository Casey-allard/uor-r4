//! Offline CUDA execution of authenticated native Generate integer factors.
//! No trainable floats participate in the hard forward. Portable serving stays
//! in core; the compatibility learner still downloads scores for its host pool.
use crate::{
    invalid,
    native_geometric_cuda_kernels::{launch, Arg},
    Result,
};
use candle_core::{op::BackpropOp, CudaStorage, DType, Device, Storage, Tensor};
use uor_r4_core::native_geometric::learner::geometric_generate::{
    NativeGeometricGenerate, SCORE_SHIFT,
};
use uor_r4_integer::h4_tables::{H4Code, ROOT_COUNT};

pub(crate) struct NativeGenerateCuda {
    payload: String,
    vocab: usize,
    lanes: usize,
    pairs: usize,
    relative: Tensor,
    prototypes: Tensor,
    edges: Tensor,
    factors: Tensor,
    pub(crate) staged_bytes: usize,
}
fn cuda_storage(storage: &Storage) -> Result<&CudaStorage> {
    match storage {
        Storage::Cuda(storage) => Ok(storage),
        _ => Err(invalid("native Generate expected CUDA storage")),
    }
}
impl NativeGenerateCuda {
    pub(crate) fn new(native: &NativeGeometricGenerate, relative: &Tensor) -> Result<Self> {
        if !relative.device().is_cuda()
            || relative.dtype() != DType::U32
            || relative.dims() != [ROOT_COUNT, ROOT_COUNT]
        {
            return Err(invalid(
                "native Generate CUDA relative table/device differs",
            ));
        }
        let device = relative.device();
        let lanes = native.lanes();
        let vocab = native.vocab_size();
        let pairs = native.energy().edges().len();
        // Native construction has already authenticated nibbles, geometry,
        // tokenizer and ordered edges. Decode this frozen artifact, not Vars.
        let mut factors =
            Vec::with_capacity(vocab + lanes * ROOT_COUNT + pairs * ROOT_COUNT * ROOT_COUNT);
        for token in 0..vocab {
            factors.push(
                i64::from(
                    native
                        .token_bias(token)
                        .map_err(|e| invalid(e.to_string()))?,
                ) << SCORE_SHIFT,
            );
        }
        for lane in 0..lanes {
            for relative in 0..ROOT_COUNT {
                factors.push(
                    i64::from(
                        native
                            .energy()
                            .get_unary(lane as u8, relative as u8)
                            .map_err(|e| invalid(e.to_string()))?,
                    ) << SCORE_SHIFT,
                );
            }
        }
        for pair in 0..pairs {
            for left in 0..ROOT_COUNT {
                for right in 0..ROOT_COUNT {
                    factors.push(
                        i64::from(
                            native
                                .energy()
                                .get_pair(pair, left as u8, right as u8)
                                .map_err(|e| invalid(e.to_string()))?,
                        ) << SCORE_SHIFT,
                    );
                }
            }
        }
        let prototypes = native
            .prototypes()
            .iter()
            .map(|&v| u32::from(v))
            .collect::<Vec<_>>();
        let edges = native
            .energy()
            .edges()
            .iter()
            .flat_map(|e| [u32::from(e.left), u32::from(e.right)])
            .collect::<Vec<_>>();
        // An empty edge buffer has one unused element, so the launcher never
        // depends on zero-sized CUDA allocation behavior.
        let edges = if edges.is_empty() { vec![0] } else { edges };
        let staged_bytes = factors.len() * 8 + (prototypes.len() + edges.len()) * 4;
        Ok(Self {
            payload: native.metadata().payload_sha256.clone(),
            vocab,
            lanes,
            pairs,
            relative: relative.clone(),
            prototypes: Tensor::from_vec(prototypes, vocab * lanes, device)?,
            edges: Tensor::from_vec(edges.clone(), edges.len(), device)?,
            factors: Tensor::from_vec(factors.clone(), factors.len(), device)?,
            staged_bytes,
        })
    }
    /// One GPU thread per vocabulary ID. All scores stay exact i64 Q24 until
    /// the independently detached offline f32 anchor is formed on device.
    pub(crate) fn score(
        &self,
        native: &NativeGeometricGenerate,
        state: &[H4Code],
    ) -> Result<(Tensor, Tensor)> {
        if state.len() != self.lanes
            || native.lanes() != self.lanes
            || native.vocab_size() != self.vocab
            || native.energy().edges().len() != self.pairs
            || native.metadata().payload_sha256 != self.payload
        {
            return Err(invalid(
                "native Generate CUDA snapshot/state identity differs",
            ));
        }
        let Device::Cuda(device) = self.relative.device() else {
            return Err(invalid("native Generate CUDA device missing"));
        };
        let states = Tensor::from_vec(
            state
                .iter()
                .map(|v| u32::from(v.index()))
                .collect::<Vec<_>>(),
            self.lanes,
            self.relative.device(),
        )?;
        let scores = device.alloc_zeros::<i64>(self.vocab)?;
        let anchor = device.alloc_zeros::<f32>(self.vocab)?;
        let (r, _) = self.relative.storage_and_layout();
        let (p, _) = self.prototypes.storage_and_layout();
        let (e, _) = self.edges.storage_and_layout();
        let (f, _) = self.factors.storage_and_layout();
        let (s, _) = states.storage_and_layout();
        launch(
            device,
            "native_generate_q24",
            self.vocab,
            &[
                Arg::U(cuda_storage(&r)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::U(cuda_storage(&p)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::U(cuda_storage(&e)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(cuda_storage(&f)?.as_cuda_slice::<i64>()?.as_view()),
                Arg::U(cuda_storage(&s)?.as_cuda_slice::<u32>()?.as_view()),
                Arg::I(scores.as_view()),
                Arg::F(anchor.as_view()),
                Arg::N(self.vocab as u32),
                Arg::N(self.lanes as u32),
                Arg::N(self.pairs as u32),
            ],
        )?;
        Ok((
            Tensor::from_storage(
                Storage::Cuda(CudaStorage::wrap_cuda_slice(scores, device.clone())),
                self.vocab,
                BackpropOp::none(),
                false,
            ),
            Tensor::from_storage(
                Storage::Cuda(CudaStorage::wrap_cuda_slice(anchor, device.clone())),
                self.vocab,
                BackpropOp::none(),
                false,
            ),
        ))
    }
}

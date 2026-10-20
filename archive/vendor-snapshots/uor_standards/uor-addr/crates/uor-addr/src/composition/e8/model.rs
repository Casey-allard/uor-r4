//! CS-E8 `PrismModel*` declarations, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e8::value::E8Carrier;
#[allow(unused_imports)]
use crate::composition::e8::verbs::{
    compose_e8_inference, compose_e8_inference_blake3, compose_e8_inference_keccak256,
    compose_e8_inference_sha3_256, compose_e8_inference_sha512, VERB_TERMS_COMPOSE_E8_INFERENCE,
    VERB_TERMS_COMPOSE_E8_INFERENCE_BLAKE3, VERB_TERMS_COMPOSE_E8_INFERENCE_KECCAK256,
    VERB_TERMS_COMPOSE_E8_INFERENCE_SHA3_256, VERB_TERMS_COMPOSE_E8_INFERENCE_SHA512,
};
use crate::label::{
    CompositionLabelE8Blake3, CompositionLabelE8Keccak256, CompositionLabelE8Sha256,
    CompositionLabelE8Sha3_256, CompositionLabelE8Sha512,
};

addr_models! {
    input: E8Carrier<'a>,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE8Sha256,
        model: CompositionModelE8Sha256,
        route: CompositionRouteE8Sha256,
        verb: compose_e8_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE8Blake3,
        model: CompositionModelE8Blake3,
        route: CompositionRouteE8Blake3,
        verb: compose_e8_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE8Sha3_256,
        model: CompositionModelE8Sha3_256,
        route: CompositionRouteE8Sha3_256,
        verb: compose_e8_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE8Keccak256,
        model: CompositionModelE8Keccak256,
        route: CompositionRouteE8Keccak256,
        verb: compose_e8_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: CompositionLabelE8Sha512,
        model: CompositionModelE8Sha512,
        route: CompositionRouteE8Sha512,
        verb: compose_e8_inference_sha512
    },
}

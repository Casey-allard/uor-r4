//! CS-E7 `PrismModel*` declarations, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e7::value::E7Carrier;
#[allow(unused_imports)]
use crate::composition::e7::verbs::{
    compose_e7_inference, compose_e7_inference_blake3, compose_e7_inference_keccak256,
    compose_e7_inference_sha3_256, compose_e7_inference_sha512, VERB_TERMS_COMPOSE_E7_INFERENCE,
    VERB_TERMS_COMPOSE_E7_INFERENCE_BLAKE3, VERB_TERMS_COMPOSE_E7_INFERENCE_KECCAK256,
    VERB_TERMS_COMPOSE_E7_INFERENCE_SHA3_256, VERB_TERMS_COMPOSE_E7_INFERENCE_SHA512,
};
use crate::label::{
    CompositionLabelE7Blake3, CompositionLabelE7Keccak256, CompositionLabelE7Sha256,
    CompositionLabelE7Sha3_256, CompositionLabelE7Sha512,
};

addr_models! {
    input: E7Carrier<'a>,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE7Sha256,
        model: CompositionModelE7Sha256,
        route: CompositionRouteE7Sha256,
        verb: compose_e7_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE7Blake3,
        model: CompositionModelE7Blake3,
        route: CompositionRouteE7Blake3,
        verb: compose_e7_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE7Sha3_256,
        model: CompositionModelE7Sha3_256,
        route: CompositionRouteE7Sha3_256,
        verb: compose_e7_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE7Keccak256,
        model: CompositionModelE7Keccak256,
        route: CompositionRouteE7Keccak256,
        verb: compose_e7_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: CompositionLabelE7Sha512,
        model: CompositionModelE7Sha512,
        route: CompositionRouteE7Sha512,
        verb: compose_e7_inference_sha512
    },
}

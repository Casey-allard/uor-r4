//! CS-E6 `PrismModel*` declarations, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::e6::value::E6Carrier;
#[allow(unused_imports)]
use crate::composition::e6::verbs::{
    compose_e6_inference, compose_e6_inference_blake3, compose_e6_inference_keccak256,
    compose_e6_inference_sha3_256, compose_e6_inference_sha512, VERB_TERMS_COMPOSE_E6_INFERENCE,
    VERB_TERMS_COMPOSE_E6_INFERENCE_BLAKE3, VERB_TERMS_COMPOSE_E6_INFERENCE_KECCAK256,
    VERB_TERMS_COMPOSE_E6_INFERENCE_SHA3_256, VERB_TERMS_COMPOSE_E6_INFERENCE_SHA512,
};
use crate::label::{
    CompositionLabelE6Blake3, CompositionLabelE6Keccak256, CompositionLabelE6Sha256,
    CompositionLabelE6Sha3_256, CompositionLabelE6Sha512,
};

addr_models! {
    input: E6Carrier<'a>,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE6Sha256,
        model: CompositionModelE6Sha256,
        route: CompositionRouteE6Sha256,
        verb: compose_e6_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE6Blake3,
        model: CompositionModelE6Blake3,
        route: CompositionRouteE6Blake3,
        verb: compose_e6_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE6Sha3_256,
        model: CompositionModelE6Sha3_256,
        route: CompositionRouteE6Sha3_256,
        verb: compose_e6_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelE6Keccak256,
        model: CompositionModelE6Keccak256,
        route: CompositionRouteE6Keccak256,
        verb: compose_e6_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: CompositionLabelE6Sha512,
        model: CompositionModelE6Sha512,
        route: CompositionRouteE6Sha512,
        verb: compose_e6_inference_sha512
    },
}

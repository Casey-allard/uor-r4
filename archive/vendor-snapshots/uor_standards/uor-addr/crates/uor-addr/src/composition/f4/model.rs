//! CS-F4 `PrismModel*` declarations, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::f4::value::F4Carrier;
#[allow(unused_imports)]
use crate::composition::f4::verbs::{
    compose_f4_inference, compose_f4_inference_blake3, compose_f4_inference_keccak256,
    compose_f4_inference_sha3_256, compose_f4_inference_sha512, VERB_TERMS_COMPOSE_F4_INFERENCE,
    VERB_TERMS_COMPOSE_F4_INFERENCE_BLAKE3, VERB_TERMS_COMPOSE_F4_INFERENCE_KECCAK256,
    VERB_TERMS_COMPOSE_F4_INFERENCE_SHA3_256, VERB_TERMS_COMPOSE_F4_INFERENCE_SHA512,
};
use crate::label::{
    CompositionLabelF4Blake3, CompositionLabelF4Keccak256, CompositionLabelF4Sha256,
    CompositionLabelF4Sha3_256, CompositionLabelF4Sha512,
};

addr_models! {
    input: F4Carrier<'a>,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelF4Sha256,
        model: CompositionModelF4Sha256,
        route: CompositionRouteF4Sha256,
        verb: compose_f4_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelF4Blake3,
        model: CompositionModelF4Blake3,
        route: CompositionRouteF4Blake3,
        verb: compose_f4_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelF4Sha3_256,
        model: CompositionModelF4Sha3_256,
        route: CompositionRouteF4Sha3_256,
        verb: compose_f4_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelF4Keccak256,
        model: CompositionModelF4Keccak256,
        route: CompositionRouteF4Keccak256,
        verb: compose_f4_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: CompositionLabelF4Sha512,
        model: CompositionModelF4Sha512,
        route: CompositionRouteF4Sha512,
        verb: compose_f4_inference_sha512
    },
}

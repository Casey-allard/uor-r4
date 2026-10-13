//! CS-G2 `PrismModel*` declarations, one per σ-axis.

#![cfg(feature = "alloc")]

use crate::composition::g2::value::G2Carrier;
#[allow(unused_imports)]
use crate::composition::g2::verbs::{
    compose_g2_inference, compose_g2_inference_blake3, compose_g2_inference_keccak256,
    compose_g2_inference_sha3_256, compose_g2_inference_sha512, VERB_TERMS_COMPOSE_G2_INFERENCE,
    VERB_TERMS_COMPOSE_G2_INFERENCE_BLAKE3, VERB_TERMS_COMPOSE_G2_INFERENCE_KECCAK256,
    VERB_TERMS_COMPOSE_G2_INFERENCE_SHA3_256, VERB_TERMS_COMPOSE_G2_INFERENCE_SHA512,
};
use crate::label::{
    CompositionLabelG2Blake3, CompositionLabelG2Keccak256, CompositionLabelG2Sha256,
    CompositionLabelG2Sha3_256, CompositionLabelG2Sha512,
};

addr_models! {
    input: G2Carrier<'a>,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelG2Sha256,
        model: CompositionModelG2Sha256,
        route: CompositionRouteG2Sha256,
        verb: compose_g2_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelG2Blake3,
        model: CompositionModelG2Blake3,
        route: CompositionRouteG2Blake3,
        verb: compose_g2_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelG2Sha3_256,
        model: CompositionModelG2Sha3_256,
        route: CompositionRouteG2Sha3_256,
        verb: compose_g2_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: CompositionLabelG2Keccak256,
        model: CompositionModelG2Keccak256,
        route: CompositionRouteG2Keccak256,
        verb: compose_g2_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: CompositionLabelG2Sha512,
        model: CompositionModelG2Sha512,
        route: CompositionRouteG2Sha512,
        verb: compose_g2_inference_sha512
    },
}

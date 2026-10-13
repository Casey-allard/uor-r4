//! `ring::AddressModel*` — the ring realization's `PrismModel` declarations,
//! one per admissible σ-axis ([`crate::hash`]). Each binds the shared
//! [`AddressResolverTuple`](crate::resolvers) ψ-tower and the axis's
//! capacity profile (`AddrBounds` for the 32-byte axes, `AddrBounds64` for
//! sha512). `AddressModel` (sha256) is the default.

use crate::label::{
    AddressLabelBlake3, AddressLabelKeccak256, AddressLabelSha256, AddressLabelSha3_256,
    AddressLabelSha512,
};
use crate::ring::value::RingElement;
#[allow(unused_imports)]
use crate::ring::verbs::{
    address_inference, address_inference_blake3, address_inference_keccak256,
    address_inference_sha3_256, address_inference_sha512, VERB_TERMS_ADDRESS_INFERENCE,
    VERB_TERMS_ADDRESS_INFERENCE_BLAKE3, VERB_TERMS_ADDRESS_INFERENCE_KECCAK256,
    VERB_TERMS_ADDRESS_INFERENCE_SHA3_256, VERB_TERMS_ADDRESS_INFERENCE_SHA512,
};

addr_models! {
    input: RingElement,
    {
        hasher: prism::crypto::Sha256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: AddressLabelSha256,
        model: AddressModel,
        route: AddressRoute,
        verb: address_inference
    },
    {
        hasher: prism::crypto::Blake3Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: AddressLabelBlake3,
        model: AddressModelBlake3,
        route: AddressRouteBlake3,
        verb: address_inference_blake3
    },
    {
        hasher: prism::crypto::Sha3_256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: AddressLabelSha3_256,
        model: AddressModelSha3_256,
        route: AddressRouteSha3_256,
        verb: address_inference_sha3_256
    },
    {
        hasher: prism::crypto::Keccak256Hasher,
        bounds: crate::bounds::AddrBounds,
        shape: AddressLabelKeccak256,
        model: AddressModelKeccak256,
        route: AddressRouteKeccak256,
        verb: address_inference_keccak256
    },
    {
        hasher: prism::crypto::Sha512Hasher,
        bounds: crate::bounds::AddrBounds64,
        shape: AddressLabelSha512,
        model: AddressModelSha512,
        route: AddressRouteSha512,
        verb: address_inference_sha512
    },
}

//! Phase B: assertions about the `HostTypes` trait shape.
//!
//! `HostTypes` is the sole host-environment carrier. It exposes exactly three
//! associated types: `Decimal`, `HostString`, `WitnessBytes`. The v0.2.1
//! `DateTime` slot is removed per target §4.1 — the foundation has no
//! wall-clock source (see §1.6); downstream associates timestamps out-of-band.
//! `DefaultHostTypes` selects `f64`/`str`/`[u8]`.

use uor_foundation::{DefaultHostTypes, HostTypes};

fn require_host_types<H: HostTypes>() {
    let _ = core::marker::PhantomData::<H>;
}

#[test]
fn default_host_types_implements_host_types() {
    require_host_types::<DefaultHostTypes>();
}

#[test]
fn default_host_types_decimal_is_f64() {
    fn assert_eq_type<A: 'static, B: 'static>() -> bool {
        core::any::TypeId::of::<A>() == core::any::TypeId::of::<B>()
    }
    assert!(assert_eq_type::<
        <DefaultHostTypes as HostTypes>::Decimal,
        f64,
    >());
}

#[test]
fn host_types_trait_is_publicly_implementable() {
    /// Downstream-style override: a marker that swaps `Decimal` to `f32`
    /// while keeping the other defaults.
    struct EmbeddedHost;
    impl HostTypes for EmbeddedHost {
        type Decimal = f32;
        type HostString = str;
        type WitnessBytes = [u8];
        const EMPTY_DECIMAL: f32 = 0.0;
        const EMPTY_HOST_STRING: &'static str = "";
        const EMPTY_WITNESS_BYTES: &'static [u8] = &[];
    }
    require_host_types::<EmbeddedHost>();
}

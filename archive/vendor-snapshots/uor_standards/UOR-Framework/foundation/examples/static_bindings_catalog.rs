//! v0.2.2 Phase Q.3 example: build a compile-time `BindingsTable` from static
//! `Binding` declarations using Phase P.3's `to_binding_entry` helper.
//!
//! Downstream declares a `'static [Binding]` array (as produced by the
//! `uor!` macro or manual compile-time construction), then lifts each
//! `Binding` to a `BindingEntry` via `to_binding_entry()`. The result is a
//! `'static [BindingEntry]` array consumable by `BindingsTable::try_new`.
//!
//! The admission is entirely compile-time:
//! - `ContentAddress::from_u64_fingerprint` is const-fn
//! - `Binding::to_binding_entry` is const-fn
//! - `BindingsTable::try_new` is const-fn (validates sort order)
//!
//! Run with: `cargo run --example static_bindings_catalog -p uor-foundation`

use uor_foundation::enforcement::{Binding, BindingEntry, BindingsTable};

static BINDINGS: &[Binding] = &[
    Binding {
        name_index: 0,
        type_index: 0,
        value_index: 0,
        surface: "x",
        content_address: 0x0000_0000_0000_0001,
    },
    Binding {
        name_index: 1,
        type_index: 0,
        value_index: 1,
        surface: "y",
        content_address: 0x0000_0000_0000_00ff,
    },
    Binding {
        name_index: 2,
        type_index: 0,
        value_index: 2,
        surface: "z",
        content_address: 0x0000_0000_0000_ffff,
    },
];

// Lift each Binding to a BindingEntry at const-time. `to_binding_entry` maps
// `content_address: u64` → `ContentAddress::from_u64_fingerprint` (left-shifted
// into high bits) and re-uses `surface.as_bytes()` as `&'static [u8]`.
static ENTRIES: &[BindingEntry] = &[
    BINDINGS[0].to_binding_entry(),
    BINDINGS[1].to_binding_entry(),
    BINDINGS[2].to_binding_entry(),
];

// Validate sort order at const-time. try_new checks strict-ascending ContentAddress.
const TABLE: BindingsTable = match BindingsTable::try_new(ENTRIES) {
    Ok(t) => t,
    Err(_) => panic!("bindings are not in strict-ascending ContentAddress order"),
};

fn main() {
    println!("Static BindingsTable with {} entries:", TABLE.entries.len());
    for entry in TABLE.entries {
        println!(
            "  address={:?} bytes={:?}",
            entry.address,
            core::str::from_utf8(entry.bytes).expect("surface is utf8")
        );
    }

    // Lookup: binary search by address (BindingsTable entries are sorted).
    let lookup_addr = BINDINGS[1].to_binding_entry().address;
    let found = TABLE
        .entries
        .binary_search_by_key(&lookup_addr.as_u128(), |e| e.address.as_u128())
        .map(|idx| &TABLE.entries[idx]);

    match found {
        Ok(entry) => println!(
            "Lookup found: {:?}",
            core::str::from_utf8(entry.bytes).unwrap_or("?")
        ),
        Err(_) => println!("Lookup miss"),
    }
}

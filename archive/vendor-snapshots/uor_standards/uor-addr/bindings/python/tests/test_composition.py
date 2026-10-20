"""Smoke tests for the κ-label composition (ADR-061) Python bindings.

Pinned invariants (mirror CL-C-FFI-* on the C side):

- **PY-COMP-01**: every operation composes a well-formed κ-label.
- **PY-COMP-02**: CS-G2 is commutative — `compose_g2(a, b) == compose_g2(b, a)`.
- **PY-COMP-03**: a witness round-trips (TC-05) to the label its
  flat entry point yields.
- **PY-COMP-04**: a malformed operand raises `AddressError`.
- **PY-COMP-05**: composition works on a non-default σ-axis (blake3).

Run with `python -m pytest bindings/python/tests/` (the bundled
`libuor_addr_c.so` must be current — see the release workflow / the
local `cargo build -p uor-addr-c --release` + copy step).
"""

import os
import sys

import pytest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from uor_addr import AddressError, HASH_BLAKE3, kappa  # noqa: E402


def _operands():
    a = kappa.json_address(b'{"role":"left"}')
    b = kappa.json_address(b'{"role":"right"}')
    return a, b


def test_py_comp_01_every_op_composes_well_formed_label():
    a, b = _operands()
    labels = [
        kappa.compose_g2(a, b),
        kappa.compose_f4(a),
        kappa.compose_e6(a),
        kappa.compose_e7(a),
        kappa.compose_e8(a),
    ]
    for label in labels:
        assert label.startswith("sha256:")
        assert len(label) == 71


def test_py_comp_02_g2_is_commutative():
    a, b = _operands()
    assert kappa.compose_g2(a, b) == kappa.compose_g2(b, a)


def test_py_comp_03_witness_round_trips_for_every_op():
    a, b = _operands()
    cases = [
        kappa.compose_g2_with_witness(a, b),
        kappa.compose_f4_with_witness(a),
        kappa.compose_e6_with_witness(a),
        kappa.compose_e7_with_witness(a),
        kappa.compose_e8_with_witness(a),
    ]
    for g in cases:
        with g:
            assert g.verify() == g.kappa_label()


def test_py_comp_03b_witness_matches_flat_label():
    a = kappa.json_address(b'{"role":"left"}')
    flat = kappa.compose_e8(a)
    with kappa.compose_e8_with_witness(a) as g:
        assert g.kappa_label() == flat


def test_py_comp_04_malformed_operand_raises():
    with pytest.raises(AddressError):
        kappa.compose_e8(b"not-a-kappa-label")


def test_py_comp_05_blake3_axis():
    a = kappa.json_address_with_hash(b'{"role":"left"}', HASH_BLAKE3)
    b = kappa.json_address_with_hash(b'{"role":"right"}', HASH_BLAKE3)
    ab = kappa.compose_g2(a, b, HASH_BLAKE3)
    assert ab.startswith("blake3:")
    assert kappa.compose_g2(b, a, HASH_BLAKE3) == ab

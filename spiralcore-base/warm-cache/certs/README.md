# Test certificates

These are test-only certificates for the local QUIC loopback and emulated WAN
binaries. They are not committed and must not be used for production.

Generate a CA and a localhost server certificate:

    bash certs/gen_test_certs.sh

Then run QUIC binaries with:

    WARM_CERT_DIR=certs target/release/quic_wan server 127.0.0.1:39412

The certificate uses SAN localhost/127.0.0.1. For a remote WAN host, regenerate
the server certificate for the name the client uses and keep the CA private.

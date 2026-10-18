#!/usr/bin/env bash
# Generate a test CA, a localhost server cert, and a second (wrong) CA.
# Extensions are explicit because macOS LibreSSL x509 verification requires
# keyUsage/SKI/AKI on both the CA and the leaf.
set -euo pipefail
DIR=${1:?usage: make_tls_certs.sh DIR}
mkdir -p "$DIR"
cd "$DIR"
openssl req -newkey rsa:2048 -nodes -keyout ca.key -out ca.csr \
  -subj '/CN=uor-demo-ca' >/dev/null 2>&1
printf 'basicConstraints=critical,CA:TRUE\nkeyUsage=critical,keyCertSign,cRLSign\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid:always\n' > ca_ext.txt
openssl x509 -req -in ca.csr -signkey ca.key -out ca.pem -days 2 -extfile ca_ext.txt \
  >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes -keyout server.key -out server.csr \
  -subj '/CN=localhost' >/dev/null 2>&1
printf 'subjectAltName=DNS:localhost,IP:127.0.0.1\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n' > san.txt
openssl x509 -req -in server.csr -CA ca.pem -CAkey ca.key -CAcreateserial \
  -out server.pem -days 2 -extfile san.txt >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes -keyout ca2.key -out ca2.csr \
  -subj '/CN=wrong-ca' >/dev/null 2>&1
openssl x509 -req -in ca2.csr -signkey ca2.key -out ca2.pem -days 2 -extfile ca_ext.txt \
  >/dev/null 2>&1
echo CERTS_OK

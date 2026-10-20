#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
openssl req -x509 -newkey rsa:2048 -nodes -keyout ca-key.pem -out ca-cert.pem \
  -days 2 -subj "/CN=WarmCache Test CA"
openssl req -newkey rsa:2048 -nodes -keyout server-key.pem -out server.csr \
  -subj "/CN=localhost"
cat > ext.cnf <<'EOF'
subjectAltName=DNS:localhost,IP:127.0.0.1
basicConstraints=critical,CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
EOF
openssl x509 -req -in server.csr -CA ca-cert.pem -CAkey ca-key.pem \
  -CAcreateserial -out server-cert.pem -days 2 -extfile ext.cnf
rm -f server.csr ext.cnf
echo "test certs generated in $(pwd)"

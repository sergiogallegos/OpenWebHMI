#!/usr/bin/env sh
set -eu

out_dir="${1:-.dev-certs}"
mkdir -p "$out_dir"
chmod 700 "$out_dir"

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$out_dir/openwebhmi.key" \
  -out "$out_dir/openwebhmi.crt" \
  -days 30 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

chmod 600 "$out_dir/openwebhmi.key"
chmod 644 "$out_dir/openwebhmi.crt"
printf '%s\n' "cert=$out_dir/openwebhmi.crt"
printf '%s\n' "key=$out_dir/openwebhmi.key"

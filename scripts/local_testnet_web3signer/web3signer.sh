#!/usr/bin/env bash

#
# Starts a web3signer.
#
set -Eeuo pipefail

source ./vars.env

mkdir temp && cd temp
eval VERSION=$(curl -X GET "https://api.github.com/repos/ConsenSys/web3signer/releases/latest" | jq ".tag_name") &&
wget -c "https://artifacts.consensys.net/public/web3signer/raw/names/web3signer.zip/versions/$VERSION/web3signer-$VERSION.zip" -O web3signer.zip &&
unzip web3signer.zip && mv web3signer-* web3signer &&
mv web3signer/ $WEB3SIGNER_DIR/web3signer &&
cd .. &&
rm -r temp &&

exec $WEB3SIGNER_DIR/web3signer/bin/web3signer \
--http-listen-port=9099 \
--tls-known-clients-file=../../testing/web3signer_tests/tls/web3signer/known_clients.txt \
--tls-keystore-file=../../testing/web3signer_tests/tls/web3signer/key.p12 \
--tls-keystore-password-file=../../testing/web3signer_tests/tls/web3signer/password.txt \
eth2 \
--network=mainnet \
--slashing-protection-enabled=false \
--keystores-path=$WEB3SIGNER_DIR/keys/ \
--keystores-passwords-path=$WEB3SIGNER_DIR/secrets/


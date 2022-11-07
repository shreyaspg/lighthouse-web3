#!/usr/bin/env bash

#
# Starts a web3signer.
#


set -x
set -Eeuo pipefail

source ./vars.env

mkdir -p $WEB3SIGNER_DIR/web3signer/bin/
cp -r /home/ccm-user/web3signer/build/distributions/web3signer-develop/* $WEB3SIGNER_DIR/web3signer/

exec $WEB3SIGNER_DIR/web3signer/bin/web3signer \
	--tls-allow-any-client=true \
	--tls-keystore-file=../../testing/web3signer_tests/tls/web3signer/key.p12 \
	--tls-keystore-password-file=../../testing/web3signer_tests/tls/web3signer/password.txt \
--logging=all \
--http-listen-port=9099 \
eth2 \
--fortanix-dsm-enabled=true \
--server="https://apps.sdkms.fortanix.com" \
--api-key="OTA5NzMxZjAtYzliNy00NTg5LWI0MTEtYjhiZjlhZjExNmQ2OmN0NEM0bVExQjFTZUlfYlcyNVk4X3FnaURnd0JMN2lVUkROOFowUGVzX1BQN3BFSVVjX1lKZ3RJTGMwcWZtdUxLNTFSdlVMVUNKeGhCR1ZSdjN4ek13" \
--secret-name=$1 \
--slashing-protection-enabled=false \
--network=mainnet


#!/usr/bin/env bash

#
# Deploys the deposit contract and makes deposits for $VALIDATOR_COUNT insecure deterministic validators.
# Produces a testnet specification and a genesis state where the genesis time
# is now + $GENESIS_DELAY.
#
# Generates datadirs for multiple validator keys according to the
# $VALIDATOR_COUNT and $BN_COUNT variables.
#

set -o nounset -o errexit -o pipefail
source ./vars.env

lcli \
	deploy-deposit-contract \
	--eth1-http http://localhost:8545 \
	--confirmations 1 \
	--validator-count $VALIDATOR_COUNT

NOW=`date +%s`
GENESIS_TIME=`expr $NOW + $GENESIS_DELAY`

lcli \
	new-testnet \
	--spec $SPEC_PRESET \
	--deposit-contract-address $DEPOSIT_CONTRACT_ADDRESS \
	--testnet-dir $TESTNET_DIR \
	--min-genesis-active-validator-count $GENESIS_VALIDATOR_COUNT \
	--min-genesis-time $GENESIS_TIME \
	--genesis-delay $GENESIS_DELAY \
	--genesis-fork-version $GENESIS_FORK_VERSION \
	--altair-fork-epoch $ALTAIR_FORK_EPOCH \
	--eth1-id $CHAIN_ID \
	--eth1-follow-distance 1 \
	--seconds-per-slot $SECONDS_PER_SLOT \
	--seconds-per-eth1-block $SECONDS_PER_ETH1_BLOCK \
	--force


echo Specification generated at $TESTNET_DIR.
echo "Generating $VALIDATOR_COUNT validators concurrently... (this may take a while)"

lcli \
	insecure-web3signer-validators \
	--count $VALIDATOR_COUNT \
	--base-dir $DATADIR \
	--web3signer-dir $WEB3SIGNER_DIR \
	--node-count $BN_COUNT \
	--root-certificate-path $ROOT_CERTIFICATE_PATH \
  --client-identity-path $CLIENT_IDENTITY_PATH \
  --client-identity-password $CLIENT_IDENTITY_PASSWORD \
  --web3signer-url $WEB3SIGNER_URL

lcli \
   dsm-import \
   --ks-path $WEB3SIGNER_DIR/keys/*.json \
   --ks-pass $WEB3SIGNER_DIR/secrets/*.txt
   
echo "Importing Key inside DSM"

echo Validators generated with keystore passwords at $DATADIR.
echo "Building genesis state... (this might take a while)"
lcli \
	interop-genesis \
	--spec $SPEC_PRESET \
	--genesis-time $GENESIS_TIME \
	--testnet-dir $TESTNET_DIR \
	$GENESIS_VALIDATOR_COUNT

echo Created genesis state in $TESTNET_DIR



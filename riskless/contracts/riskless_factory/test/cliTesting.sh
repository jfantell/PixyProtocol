#!/bin/bash

# Makes use of terrad
resetKey=0
if [ $1 -eq "resetKey" ]; then
    resetKey=1
fi

echo "Current working directory: $(pwd)"

terrad status

if [ $resetKey -eq 1 ]; then
    mnemonic="notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius"
    printf "${mnemonic}\n" | terrad keys add test1 --recover
    echo
    if ![ $? -eq 0 ]; then
        printf "y\n${mnemonic}\n" | terrad keys add test1 --recover
    fi
fi
echo

# Upload smart contract code to blockchain
# Extract code id from output
OUTPUT=$(printf "y\n" | terrad tx wasm store artifacts/riskless.wasm --from test1 --chain-id=localterra --gas=auto --fees=100000uluna --broadcast-mode=block 2>&1)
pat='\"key"\:\"code_id\",\"value\":\"[0-9]+\"'
[[ $OUTPUT =~ $pat ]] # $pat must be unquoted
OUTPUT="${BASH_REMATCH}"
pat='[0-9]+'
[[ $OUTPUT =~ $pat ]] # $pat must be unquoted
codeId="${BASH_REMATCH}"
echo "Code ID: $codeId"
terrad query wasm code $codeId

# Instantiate smart contract and extract
# address from output
echo
OUTPUT=$(printf "y\n" | terrad tx wasm instantiate $codeId '{"admin":"terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v","anchor_money_market_address":"","a_ust_address":""}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block 2>&1)
echo $BASH_REMATCH
contractAddress="$(cut -d':' -f2 <<<${BASH_REMATCH[0]})"
echo "Contract Address: $contractAddress"

terrad tx wasm execute $contractAddress '{"CreateProject":{"name":"PoullTheMan","target_principal_amount":1000000000, "target_yield_amount":500000000, "project_deadline": 1661817600}}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block

terrad tx wasm instantiate 2 '{"admin":"terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v","anchor_money_market_address":"","a_ust_address":""}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block 2>&1)
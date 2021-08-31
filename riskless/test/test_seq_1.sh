#!/usr/bin

## Terra CLI Commands
mnemonic="notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius"
echo -e mnemonic | terracli keys add test1 --recover

# exec terracli keys add test1 --recover
# notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius
# terrad tx wasm store artifacts/riskless.wasm --from test1 --chain-id=localterra --gas=auto --fees=100000uluna --broadcast-mode=block
# terrad query wasm code 1
# terrad tx wasm instantiate 1 '{"admin":"terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v","anchor_money_market_address":"","a_ust_address":""}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block
# terrad tx wasm execute 1 '{"CreateProject":{"name":"PoullTheMan","target_principal_amount":1000000000, "target_yield_amount":500000000, "project_deadline": 1661817600}}' --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block
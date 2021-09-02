## Spacecamp Phase 1 Demo

Factory Instantiation Message

{
    "project_code_id":1,
    "anchor_money_market_address":"terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp",
    "a_ust_address": "terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp"
} 

Factory Contract Instantiation Command

terrad tx wasm instantiate 2 "{\"project_code_id\":1,\"anchor_money_market_address\":\"terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp\",\"a_ust_address\":\"terra17lmam6zguazs5q5u6z5mmx76uj63gldnse2pdp\"}" --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block

Create Project Execution Message

{
    "create_project":{
        "name":"BestFilmEver",
        "target_principal_amount":"10000000000",
        "target_yield_amount":"2000000000",
        "project_deadline":"1652172458"
    }
}

Execute Project Creation

terrad tx wasm execute terra1qxxlalvsdjd07p07y3rc5fu6ll8k4tme7cye8y "{\"create_project\":{\"name\":\"BestFilmEver\",\"target_principal_amount\":\"10000000000\",\"target_yield_amount\":\"2000000000\",\"project_deadline\":\"1652172458\"}}" --from test1 --chain-id=localterra --fees=10000uluna --gas=auto --broadcast-mode=block

Query Project Address

terrad query wasm contract-store terra1qxxlalvsdjd07p07y3rc5fu6ll8k4tme7cye8y "{\"get_project_contract_address\":{\"name\":\"BestFilmEver\"}}"

Query Project Status

terrad query wasm contract-store terra1hqrdl6wstt8qzshwc6mrumpjk9338k0l93hqyd "{\"get_project_status\":{}}"

Fund Project

terrad tx wasm execute terra1hqrdl6wstt8qzshwc6mrumpjk9338k0l93hqyd "{\"fund_project\":{}}" 100000000uusd --from test1 --chain-id=localterra --gas-prices=0.015uluna --gas=auto --gas-adjustment=1.4 --broadcast-mode=block

Query User Balance For Particular Project

terrad query wasm contract-store terra1hqrdl6wstt8qzshwc6mrumpjk9338k0l93hqyd "{\"get_user_balance\":{\"user\":\"terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v\"}}"

Withdraw Yield


## Helpful Resources

[CosmWasm Videos + Workshops](https://docs.cosmwasm.com/tutorials/videos-workshops)




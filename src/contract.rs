#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsgResponse, SubMsgResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, PalomaMsg, QueryMsg, SendTx};
use crate::state::{State, CHAIN_SETTINGS, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:palomadex-trader-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WITHDRAW_LIQUIDITY_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owners: msg
            .owners
            .iter()
            .map(|x| deps.api.addr_validate(x).unwrap())
            .collect(),
        retry_delay: msg.retry_delay,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::Exchange {
            dex_router,
            operations,
            minimum_receive,
            to,
            max_spread,
            funds,
        } => execute::exchange(
            deps,
            info,
            dex_router,
            operations,
            minimum_receive,
            to,
            max_spread,
            funds,
        ),
        ExecuteMsg::SetChainSetting {
            chain_id,
            compass_job_id,
            main_job_id,
        } => execute::set_chain_setting(deps, info, chain_id, compass_job_id, main_job_id),
        ExecuteMsg::SetPaloma { chain_id } => execute::set_paloma(deps, info, chain_id),
        ExecuteMsg::UpdateRefundWallet {
            chain_id,
            new_refund_wallet,
        } => execute::update_refund_wallet(deps, info, chain_id, new_refund_wallet),
        ExecuteMsg::UpdateGasFee {
            chain_id,
            new_gas_fee,
        } => execute::update_gas_fee(deps, info, chain_id, new_gas_fee),
        ExecuteMsg::UpdateServiceFeeCollector {
            chain_id,
            new_service_fee_collector,
        } => execute::update_service_fee_collector(deps, info, chain_id, new_service_fee_collector),
        ExecuteMsg::UpdateServiceFee {
            chain_id,
            new_service_fee,
        } => execute::update_service_fee(deps, info, chain_id, new_service_fee),
        ExecuteMsg::UpdateConfig { retry_delay } => execute::update_config(deps, info, retry_delay),
        ExecuteMsg::AddOwner { owners } => execute::add_owner(deps, info, owners),
        ExecuteMsg::RemoveOwner { owner } => execute::remove_owner(deps, info, owner),
        ExecuteMsg::SendToken {
            chain_id,
            token,
            to,
            amount,
            nonce,
        } => execute::send_token(deps, env, info, chain_id, token, to, amount, nonce),
        ExecuteMsg::AddLiquidity {
            pair,
            coins,
            slippage_tolerance,
        } => execute::add_liquidity(deps, info, pair, coins, slippage_tolerance),
        ExecuteMsg::RemoveLiquidity {
            chain_id,
            pair,
            amount,
            receiver,
        } => execute::remove_liquidity(deps, env, info, chain_id, pair, amount, receiver),
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{Addr, Decimal, ReplyOn, SubMsg, Uint128, Uint256, WasmMsg};
    use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};

    use super::*;
    use crate::{
        msg::{
            Asset, AssetInfo, ExecuteJob, ExternalExecuteMsg, ExternalQueryMsg, PairInfo,
            SwapOperation,
        },
        state::{ChainSetting, CHAIN_SETTINGS, MESSAGE_TIMESTAMP},
    };
    use std::str::FromStr;

    #[allow(clippy::too_many_arguments)]
    pub fn exchange(
        deps: DepsMut,
        info: MessageInfo,
        dex_router: Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<String>,
        max_spread: Option<Decimal>,
        funds: Vec<Coin>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: dex_router.to_string(),
                msg: to_json_binary(&ExternalExecuteMsg::ExecuteSwapOperations {
                    operations,
                    minimum_receive,
                    to,
                    max_spread,
                })?,
                funds,
            }))
            .add_attribute("action", "exchange"))
    }

    pub fn add_liquidity(
        deps: DepsMut,
        info: MessageInfo,
        pair: Addr,
        coins: Vec<Coin>,
        slippage_tolerance: Option<Decimal>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pair.to_string(),
                msg: to_json_binary(&ExternalExecuteMsg::ProvideLiquidity {
                    assets: coins
                        .iter()
                        .map(|coin| Asset {
                            info: AssetInfo::NativeToken {
                                denom: coin.denom.clone(),
                            },
                            amount: coin.amount,
                        })
                        .collect(),
                    slippage_tolerance,
                    receiver: None,
                })?,
                funds: coins,
            }))
            .add_attribute("action", "add_liquidity"))
    }

    pub fn remove_liquidity(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        chain_id: String,
        pair: Addr,
        amount: Uint128,
        receiver: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );

        let pair_info: PairInfo = deps
            .querier
            .query_wasm_smart(pair.clone(), &ExternalQueryMsg::Pair {})?;
        let liquidity_token = pair_info.liquidity_token;

        let mut coins: Vec<Coin> = vec![];
        pair_info.asset_infos.iter().for_each(|asset| {
            if let AssetInfo::NativeToken { denom } = asset {
                coins.push(
                    deps.querier
                        .query_balance(env.contract.address.clone(), denom)
                        .unwrap(),
                );
            }
        });

        let payload = to_json_binary(&(coins, receiver, chain_id))?;

        Ok(Response::new()
            .add_submessage(SubMsg {
                id: WITHDRAW_LIQUIDITY_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: liquidity_token.to_string(),
                    msg: to_json_binary(&ExternalExecuteMsg::Send {
                        contract: pair.to_string(),
                        amount,
                        msg: to_json_binary(&ExternalExecuteMsg::WithdrawLiquidity {
                            assets: vec![],
                        })?,
                    })?,
                    funds: vec![],
                }),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "remove_liquidity"))
    }

    pub fn set_chain_setting(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        compass_job_id: String,
        main_job_id: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        CHAIN_SETTINGS.save(
            deps.storage,
            chain_id.clone(),
            &ChainSetting {
                compass_job_id: compass_job_id.clone(),
                main_job_id: main_job_id.clone(),
            },
        )?;

        Ok(Response::new().add_attribute("action", "set_chain_setting"))
    }

    pub fn set_paloma(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement SetPaloma
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );

        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_paloma".to_string(),
                vec![Function {
                    name: "set_paloma".to_string(),
                    inputs: vec![],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("set_paloma")
                            .unwrap()
                            .encode_input(&[])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "set_paloma"))
    }
    pub fn update_refund_wallet(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_refund_wallet: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let update_refund_wallet_address: Address =
            Address::from_str(new_refund_wallet.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_refund_wallet".to_string(),
                vec![Function {
                    name: "update_refund_wallet".to_string(),
                    inputs: vec![Param {
                        name: "new_refund_wallet".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("update_refund_wallet")
                            .unwrap()
                            .encode_input(&[Token::Address(update_refund_wallet_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_refund_wallet"))
    }

    pub fn update_gas_fee(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_gas_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_gas_fee".to_string(),
                vec![Function {
                    name: "update_gas_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_gas_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("update_gas_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(Uint::from_big_endian(
                                &new_gas_fee.to_be_bytes(),
                            ))])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_gas_fee"))
    }

    pub fn update_service_fee_collector(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_service_fee_collector: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let update_service_fee_collector_address: Address =
            Address::from_str(new_service_fee_collector.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee_collector".to_string(),
                vec![Function {
                    name: "update_service_fee_collector".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee_collector".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee_collector")
                            .unwrap()
                            .encode_input(&[Token::Address(update_service_fee_collector_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee_collector"))
    }

    pub fn update_service_fee(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_service_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee".to_string(),
                vec![Function {
                    name: "update_service_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(Uint::from_big_endian(
                                &new_service_fee.to_be_bytes(),
                            ))])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee"))
    }

    pub fn update_config(
        deps: DepsMut,
        info: MessageInfo,
        retry_delay: Option<u64>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        if let Some(retry_delay) = retry_delay {
            state.retry_delay = retry_delay;
        }
        STATE.save(deps.storage, &state)?;
        Ok(Response::new().add_attribute("action", "update_config"))
    }

    pub fn add_owner(
        deps: DepsMut,
        info: MessageInfo,
        owners: Vec<String>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        for owner in owners.iter() {
            let owner = deps.api.addr_validate(owner)?;
            if !state.owners.iter().any(|x| x == owner) {
                state.owners.push(owner);
            }
        }
        STATE.save(deps.storage, &state)?;
        Ok(Response::new().add_attribute("action", "update_config"))
    }

    pub fn remove_owner(
        deps: DepsMut,
        info: MessageInfo,
        owner: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let owner = deps.api.addr_validate(&owner)?;
        assert!(
            state.owners.iter().any(|x| x == owner),
            "Owner does not exist"
        );
        state.owners.retain(|x| x != owner);
        STATE.save(deps.storage, &state)?;
        Ok(Response::new().add_attribute("action", "update_config"))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send_token(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        chain_id: String,
        token: String,
        to: String,
        amount: Uint128,
        nonce: Uint128,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "send_token".to_string(),
                vec![Function {
                    name: "send_token".to_string(),
                    inputs: vec![
                        Param {
                            name: "token".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "to".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "amount".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                        Param {
                            name: "nonce".to_string(),
                            kind: ParamType::Uint(256),
                            internal_type: None,
                        },
                    ],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        let tokens = &[
            Token::Address(Address::from_str(token.as_str()).unwrap()),
            Token::Address(Address::from_str(to.as_str()).unwrap()),
            Token::Uint(Uint::from_big_endian(&amount.to_be_bytes())),
            Token::Uint(Uint::from_big_endian(&nonce.to_be_bytes())),
        ];

        let retry_delay = state.retry_delay;
        if let Some(timestamp) =
            MESSAGE_TIMESTAMP.may_load(deps.storage, (chain_id.clone(), nonce.to_string()))?
        {
            if timestamp.plus_seconds(retry_delay).lt(&env.block.time) {
                MESSAGE_TIMESTAMP.save(
                    deps.storage,
                    (chain_id.clone(), nonce.to_string()),
                    &env.block.time,
                )?;
            } else {
                return Err(ContractError::Pending {});
            }
        } else {
            MESSAGE_TIMESTAMP.save(
                deps.storage,
                (chain_id.clone(), nonce.to_string()),
                &env.block.time,
            )?;
        }

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS
                        .load(deps.storage, chain_id.clone())?
                        .main_job_id,
                    payload: Binary::new(
                        contract
                            .function("send_token")
                            .unwrap()
                            .encode_input(tokens.as_slice())
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "send_token"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::ChainSetting { chain_id } => {
            to_json_binary(&CHAIN_SETTINGS.load(deps.storage, chain_id)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        #[allow(deprecated)]
        Reply {
            id: WITHDRAW_LIQUIDITY_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => {
            let (coins, receiver, chain_id): (Vec<Coin>, String, String) = from_json(payload)?;

            let amount0 = deps
                .querier
                .query_balance(env.contract.address.clone(), coins[0].clone().denom)?
                .amount
                - coins[0].amount;
            let amount1 = deps
                .querier
                .query_balance(env.contract.address.clone(), coins[1].clone().denom)?
                .amount
                - coins[1].amount;
            let amount0 = amount0.to_string() + coins[0].denom.as_str();
            let amount1 = amount1.to_string() + coins[1].denom.as_str();
            Ok(Response::new()
                .add_messages(vec![
                    CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                        send_tx: SendTx {
                            remote_chain_destination_address: receiver.clone(),
                            amount: amount0,
                            chain_reference_id: chain_id.clone(),
                        },
                    }),
                    CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                        send_tx: SendTx {
                            remote_chain_destination_address: receiver,
                            amount: amount1,
                            chain_reference_id: chain_id,
                        },
                    }),
                ])
                .add_attribute("action", "withdraw_liquidity"))
        }
        _ => Err(ContractError::UnknownReply {}),
    }
}

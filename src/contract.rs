#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdResult, SubMsgResponse, SubMsgResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, PalomaMsg, QueryMsg, SendTx};
use crate::state::{
    IncentivesSetting, State, CHAIN_SETTINGS, INCENTIVES_SETTING, LP_BALANCES, STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:palomadex-trader-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REMOVE_LIQUIDITY_REPLY_ID: u64 = 1;
const EXECUTE_REPLY_ID: u64 = 2;
const ADD_LIQUIDITY_REPLY_ID: u64 = 3;
const EXECUTE_FOR_SINGLE_LIQUIDITY_REPLY_ID: u64 = 4;
const CHECK_PADEX_DIFF_REPLY_ID: u64 = 5;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let incentives_setting = IncentivesSetting {
        incentivizer: deps.api.addr_validate(msg.incentivizer.as_str())?,
        padex: msg.padex,
        vepades: msg.vepades,
    };
    INCENTIVES_SETTING.save(deps.storage, &incentives_setting)?;
    Ok(Response::new().add_attribute("action", "migrate"))
}

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
    let incentives_setting = IncentivesSetting {
        incentivizer: deps.api.addr_validate(msg.incentivizer.as_str())?,
        padex: msg.padex,
        vepades: msg.vepades,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    INCENTIVES_SETTING.save(deps.storage, &incentives_setting)?;
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
            max_spread,
            funds,
            chain_id,
            recipient,
        } => execute::exchange(
            deps,
            env,
            info,
            dex_router,
            operations,
            minimum_receive,
            max_spread,
            funds,
            chain_id,
            recipient,
        ),
        ExecuteMsg::SendToEVM {
            chain_id,
            amounts,
            recipient,
        } => execute::send_to_evm(deps, env, info, chain_id, amounts, recipient),
        ExecuteMsg::Deposit {
            incentivizer,
            token,
            amount,
            depositor,
        } => execute::deposit(deps, env, info, incentivizer, token, amount, depositor),
        ExecuteMsg::Withdraw {
            incentivizer,
            token,
            amount,
            recipient,
        } => execute::withdraw(deps, env, info, incentivizer, token, amount, recipient),
        ExecuteMsg::ClaimRewards {
            incentivizer,
            tokens,
            recipient,
        } => execute::claim_rewards(deps, env, info, incentivizer, tokens, recipient),
        ExecuteMsg::CreateLock {
            vepadex,
            coin,
            end_lock_time,
            user,
        } => execute::create_lock(deps, env, info, vepadex, coin, end_lock_time, user),
        ExecuteMsg::IncreaseLockAmount {
            vepadex,
            user,
            coin,
        } => execute::increase_lock_amount(deps, env, info, vepadex, user, coin),
        ExecuteMsg::Unlock { vepadex, user } => execute::unlock(deps, env, info, vepadex, user),
        ExecuteMsg::IncreaseEndLockTime {
            vepadex,
            end_lock_time,
            user,
        } => execute::increase_end_lock_time(deps, info, vepadex, end_lock_time, user),
        ExecuteMsg::AddLpToken {
            lp_token,
            user,
            amount,
        } => execute::add_lp_token(deps, info, lp_token, user, amount),
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
            tokens,
            to,
            amounts,
            nonce,
        } => execute::send_token(deps, env, info, chain_id, tokens, to, amounts, nonce),
        ExecuteMsg::AddLiquidity {
            pair,
            coins,
            slippage_tolerance,
            depositor,
        } => execute::add_liquidity(deps, env, info, pair, coins, slippage_tolerance, depositor),
        ExecuteMsg::RemoveLiquidity {
            chain_id,
            pair,
            amount,
            receiver,
        } => execute::remove_liquidity(deps, env, info, chain_id, pair, amount, receiver),
        ExecuteMsg::CancelTx { transaction_id } => {
            execute::cancel_tx(deps, env, info, transaction_id)
        }
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{Addr, Decimal, Decimal256, ReplyOn, SubMsg, Uint128, Uint256, WasmMsg};
    use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
    use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};

    use super::*;
    use crate::{
        msg::{
            Asset, AssetInfo, CancelTx, ConfigResponse, Cw20Msg, ExecuteJob, ExternalExecuteMsg,
            ExternalQueryMsg, FeeInfoResponse, IncentivizerExecuteMsg, PairInfo, PairType,
            PoolResponse, SwapOperation, VePadexExecuteMsg,
        },
        state::{ChainSetting, CHAIN_SETTINGS, LP_BALANCES, MESSAGE_TIMESTAMP},
    };
    use std::str::FromStr;

    #[allow(clippy::too_many_arguments)]
    pub fn exchange(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        dex_router: Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        max_spread: Option<Decimal>,
        funds: Vec<Coin>,
        chain_id: String,
        recipient: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );

        let coin: Coin;

        let SwapOperation::AstroSwap { ask_asset_info, .. } = operations.last().unwrap();

        if let AssetInfo::NativeToken { denom } = ask_asset_info {
            coin = deps
                .querier
                .query_balance(env.contract.address.clone(), denom.clone())?;
        } else {
            return Err(ContractError::UnsupportedCw20 {});
        }

        let payload = to_json_binary(&(recipient, chain_id, coin))?;
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: EXECUTE_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: dex_router.to_string(),
                    msg: to_json_binary(&ExternalExecuteMsg::ExecuteSwapOperations {
                        operations,
                        minimum_receive,
                        to: None,
                        max_spread,
                    })?,
                    funds,
                }),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "exchange"))
    }

    pub fn add_liquidity(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        pair: Addr,
        coins: Vec<Coin>,
        slippage_tolerance: Option<Decimal>,
        depositor: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let pair_info: PairInfo = deps
            .querier
            .query_wasm_smart(pair.clone(), &ExternalQueryMsg::Pair {})?;
        let init_lp_balance: BalanceResponse = deps.querier.query_wasm_smart(
            pair_info.liquidity_token.clone(),
            &Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )?;

        if coins.len() == 2 || pair_info.pair_type != (PairType::Xyk {}) {
            let payload = to_json_binary(&(
                depositor,
                pair_info.liquidity_token.to_string(),
                init_lp_balance.balance,
            ))?;
            Ok(Response::new()
                .add_submessage(SubMsg {
                    id: ADD_LIQUIDITY_REPLY_ID,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
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
                    }),
                    payload,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                })
                .add_attribute("action", "add_liquidity"))
        } else {
            assert!(coins.len() == 1, "Only 1 or 2 coins are supported");
            let pool_response: PoolResponse = deps
                .querier
                .query_wasm_smart(pair.to_string(), &ExternalQueryMsg::Pool {})?;
            let input_coin = coins[0].clone();
            let (reserve_in, reserve_out) = if pool_response.assets[0].info
                == (AssetInfo::NativeToken {
                    denom: input_coin.denom.clone(),
                }) {
                (
                    pool_response.assets[0].clone(),
                    pool_response.assets[1].clone(),
                )
            } else {
                (
                    pool_response.assets[1].clone(),
                    pool_response.assets[0].clone(),
                )
            };
            let config_response: ConfigResponse = deps
                .querier
                .query_wasm_smart(pair.to_string(), &ExternalQueryMsg::Config {})?;
            let fee_info_response: FeeInfoResponse = deps.querier.query_wasm_smart(
                config_response.factory_addr.to_string(),
                &ExternalQueryMsg::FeeInfo {
                    pair_type: PairType::Xyk {},
                },
            )?;
            let fee_bps = fee_info_response.total_fee_bps;

            let swap_amount: Uint128 =
                calculate_swap_amount(input_coin.amount, reserve_in.amount, fee_bps);

            let coins = vec![
                Coin {
                    denom: input_coin.denom.clone(),
                    amount: input_coin.amount - swap_amount,
                },
                deps.querier.query_balance(env.contract.address.clone(), {
                    if let AssetInfo::NativeToken { denom } = reserve_out.info.clone() {
                        denom
                    } else {
                        return Err(ContractError::UnsupportedCw20 {});
                    }
                })?,
            ];

            let payload = to_json_binary(&(
                pair.clone(),
                depositor,
                pair_info.liquidity_token.to_string(),
                init_lp_balance.balance,
                coins,
            ))?;
            Ok(Response::new()
                .add_submessage(SubMsg {
                    id: EXECUTE_FOR_SINGLE_LIQUIDITY_REPLY_ID,
                    msg: CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: pair.to_string(),
                        msg: to_json_binary(&ExternalExecuteMsg::ExecuteSwapOperations {
                            operations: vec![SwapOperation::AstroSwap {
                                offer_asset_info: reserve_in.info,
                                ask_asset_info: reserve_out.info,
                            }],
                            minimum_receive: None,
                            to: None,
                            max_spread: None,
                        })?,
                        funds: vec![Coin {
                            denom: input_coin.denom,
                            amount: swap_amount,
                        }],
                    }),
                    payload,
                    gas_limit: None,
                    reply_on: ReplyOn::Success,
                })
                .add_attribute("action", "add_liquidity"))
        }
    }

    fn calculate_swap_amount(input_amount: Uint128, reserve_in: Uint128, fee_bps: u16) -> Uint128 {
        let receive_rate = Decimal256::from_ratio(10000 - fee_bps, 10000u16);
        (((Decimal256::one() + receive_rate).pow(2)
            + Decimal256::raw(4)
                * receive_rate
                * Decimal256::new(input_amount.into())
                * Decimal256::new(reserve_in.into()))
        .sqrt()
            - Decimal256::one()
            - receive_rate)
            .checked_div(Decimal256::raw(2))
            .unwrap()
            .checked_mul(Decimal256::new(reserve_in.into()))
            .unwrap()
            .to_uint_floor()
            .try_into()
            .unwrap()
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
        let lp_token = pair_info.liquidity_token;

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
        let lp_balance =
            LP_BALANCES.load(deps.storage, (receiver.clone(), lp_token.to_string()))?;
        if lp_balance < amount {
            return Err(ContractError::InsufficientLiquidity {});
        }
        LP_BALANCES.update(
            deps.storage,
            (receiver.clone(), lp_token.to_string()),
            |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() - amount) },
        )?;
        let payload = to_json_binary(&(coins, receiver, chain_id, lp_token.to_string()))?;

        Ok(Response::new()
            .add_submessage(SubMsg {
                id: REMOVE_LIQUIDITY_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: lp_token.to_string(),
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

    pub fn send_to_evm(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        chain_id: String,
        amounts: Vec<String>,
        recipient: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let messages = amounts
            .iter()
            .map(|amount| {
                let coin = Coin::from_str(amount).unwrap();
                if coin.denom == incentives_setting.padex {
                    LP_BALANCES
                        .update(
                            deps.storage,
                            (recipient.clone(), incentives_setting.padex.clone()),
                            |lp_balance: Option<Uint128>| -> StdResult<_> {
                                let mut balance = lp_balance.unwrap_or_default();
                                if balance > coin.amount {
                                    balance -= coin.amount;
                                }
                                Ok(balance)
                            },
                        )
                        .unwrap();
                }
                CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                    send_tx: Some(SendTx {
                        remote_chain_destination_address: recipient.clone(),
                        amount: amount.clone(),
                        chain_reference_id: chain_id.clone(),
                    }),
                    cancel_tx: None,
                })
            })
            .collect::<Vec<_>>();
        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "send_to_evm"))
    }

    pub fn deposit(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        incentivizer: Addr,
        token: String,
        amount: Uint128,
        depositor: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        LP_BALANCES.update(
            deps.storage,
            (depositor.clone(), token.clone()),
            |lp_balance: Option<Uint128>| -> StdResult<_> {
                let balance = lp_balance.unwrap_or_default();
                assert!(balance >= amount, "Insufficient balance");
                Ok(balance - amount)
            },
        )?;
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(depositor.clone(), padex_balance))?;
        let msg = WasmMsg::Execute {
            contract_addr: token.clone(),
            msg: to_json_binary(&Cw20ExecuteMsg::Send {
                contract: deps
                    .api
                    .addr_validate(incentivizer.as_str())
                    .unwrap()
                    .to_string(),
                amount,
                msg: to_json_binary(&Cw20Msg::Deposit {
                    recipient: Some(depositor),
                })?,
            })?,
            funds: vec![],
        };
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "deposit")
            .add_attribute("token", token)
            .add_attribute("amount", amount.to_string()))
    }

    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        incentivizer: Addr,
        token: String,
        amount: Uint128,
        recipient: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        LP_BALANCES.update(
            deps.storage,
            (recipient.clone(), token.clone()),
            |lp_balance: Option<Uint128>| -> StdResult<_> {
                let balance = lp_balance.unwrap_or_default();
                Ok(balance + amount)
            },
        )?;
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(recipient.clone(), padex_balance))?;
        let msg = WasmMsg::Execute {
            contract_addr: deps.api.addr_validate(incentivizer.as_str())?.to_string(),
            msg: to_json_binary(&IncentivizerExecuteMsg::Withdraw {
                lp_token: token.clone(),
                amount,
                user: Some(recipient),
            })?,
            funds: vec![],
        };
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "withdraw")
            .add_attribute("token", token)
            .add_attribute("amount", amount.to_string()))
    }

    pub fn claim_rewards(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        incentivizer: Addr,
        lp_tokens: Vec<String>,
        user: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let msg = WasmMsg::Execute {
            contract_addr: deps.api.addr_validate(incentivizer.as_str())?.to_string(),
            msg: to_json_binary(&IncentivizerExecuteMsg::ClaimRewards {
                lp_tokens,
                user: Some(user.clone()),
            })?,
            funds: vec![],
        };
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(user, padex_balance))?;
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "claim_rewards"))
    }

    pub fn create_lock(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        vepadex: Addr,
        coin: Coin,
        end_lock_time: u64,
        user: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(user.clone(), padex_balance))?;
        let msg = WasmMsg::Execute {
            contract_addr: vepadex.to_string(),
            msg: to_json_binary(&VePadexExecuteMsg::CreateLock {
                end_lock_time,
                user: Some(user),
            })?,
            funds: vec![coin],
        };
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "create_lock"))
    }

    pub fn increase_lock_amount(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        vepadex: Addr,
        user: String,
        coin: Coin,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(user.clone(), padex_balance))?;
        let msg = WasmMsg::Execute {
            contract_addr: vepadex.to_string(),
            msg: to_json_binary(&VePadexExecuteMsg::IncreaseLockAmount { user: Some(user) })?,
            funds: vec![coin],
        };
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "increase_lock_amount"))
    }

    pub fn unlock(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        vepadex: Addr,
        user: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let incentives_setting = INCENTIVES_SETTING.load(deps.storage)?;
        let padex_balance = deps
            .querier
            .query_balance(env.contract.address.clone(), incentives_setting.padex)
            .unwrap();
        let payload = to_json_binary(&(user.clone(), padex_balance))?;
        let msg = WasmMsg::Execute {
            contract_addr: vepadex.to_string(),
            msg: to_json_binary(&VePadexExecuteMsg::Withdraw { user: Some(user) })?,
            funds: vec![],
        };
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: CHECK_PADEX_DIFF_REPLY_ID,
                msg: CosmosMsg::Wasm(msg),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("action", "unlock"))
    }

    pub fn increase_end_lock_time(
        deps: DepsMut,
        info: MessageInfo,
        vepadex: Addr,
        end_lock_time: u64,
        user: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        let msg = WasmMsg::Execute {
            contract_addr: vepadex.to_string(),
            msg: to_json_binary(&VePadexExecuteMsg::IncreaseEndLockTime {
                end_lock_time,
                user: Some(user),
            })?,
            funds: vec![],
        };
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("action", "increase_end_lock_time"))
    }

    pub fn add_lp_token(
        deps: DepsMut,
        info: MessageInfo,
        lp_token: String,
        user: String,
        amount: Uint128,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        LP_BALANCES.update(
            deps.storage,
            (user.clone(), lp_token.clone()),
            |lp_balance: Option<Uint128>| -> StdResult<_> {
                Ok(lp_balance.unwrap_or_default() + amount)
            },
        )?;
        Ok(Response::new()
            .add_attribute("action", "add_lp_token")
            .add_attribute("lp_token", lp_token)
            .add_attribute("user", user)
            .add_attribute("amount", amount.to_string()))
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
        tokens: Vec<String>,
        to: String,
        amounts: Vec<Uint128>,
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
                            name: "tokens".to_string(),
                            kind: ParamType::Array(Box::new(ParamType::Address)),
                            internal_type: None,
                        },
                        Param {
                            name: "to".to_string(),
                            kind: ParamType::Address,
                            internal_type: None,
                        },
                        Param {
                            name: "amounts".to_string(),
                            kind: ParamType::Array(Box::new(ParamType::Uint(256))),
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

        let tokens = tokens
            .iter()
            .map(|token| Token::Address(Address::from_str(token.as_str()).unwrap()))
            .collect::<Vec<_>>();
        let amounts = amounts
            .iter()
            .map(|amount| Token::Uint(Uint::from_big_endian(&amount.to_be_bytes())))
            .collect::<Vec<_>>();

        let tokens = &[
            Token::Array(tokens),
            Token::Address(Address::from_str(to.as_str()).unwrap()),
            Token::Array(amounts),
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

    pub fn cancel_tx(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        transaction_id: u64,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(
            state.owners.iter().any(|x| x == info.sender),
            "Unauthorized"
        );
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                send_tx: None,
                cancel_tx: Some(CancelTx { transaction_id }),
            }))
            .add_attribute("action", "cancel_tx")
            .add_attribute("transaction_id", transaction_id.to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_json_binary(&STATE.load(deps.storage)?),
        QueryMsg::ChainSetting { chain_id } => {
            to_json_binary(&CHAIN_SETTINGS.load(deps.storage, chain_id)?)
        }
        QueryMsg::LpQuery { user, lp_token } => {
            let lp_balance = LP_BALANCES
                .may_load(deps.storage, (user, lp_token))?
                .unwrap_or_default();
            to_json_binary(&lp_balance)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        #[allow(deprecated)]
        Reply {
            id: REMOVE_LIQUIDITY_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => reply::remove_liquidity(deps, env, payload),
        #[allow(deprecated)]
        Reply {
            id: EXECUTE_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => reply::execute_reply(deps, env, payload),
        #[allow(deprecated)]
        Reply {
            id: ADD_LIQUIDITY_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => reply::add_liquidity(deps, env, payload),
        #[allow(deprecated)]
        Reply {
            id: EXECUTE_FOR_SINGLE_LIQUIDITY_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => reply::exchange_for_single_liqudity(deps, env, payload),
        #[allow(deprecated)]
        Reply {
            id: CHECK_PADEX_DIFF_REPLY_ID,
            payload,
            gas_used: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    events: _,
                    data: _,
                    msg_responses: _,
                }),
        } => reply::check_padex_diff(deps, payload),
        _ => Err(ContractError::UnknownReply {}),
    }
}

pub mod reply {
    use cosmwasm_std::{ReplyOn, SubMsg, Uint128, WasmMsg};
    use cw20::{BalanceResponse, Cw20QueryMsg};

    use crate::{
        msg::{Asset, AssetInfo, ExternalExecuteMsg},
        state::LP_BALANCES,
    };

    use super::*;

    pub fn remove_liquidity(
        deps: DepsMut,
        env: Env,
        payload: Binary,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let (mut coins, receiver, chain_id, lp_token): (Vec<Coin>, String, String, String) =
            from_json(payload)?;
        coins[0].amount = deps
            .querier
            .query_balance(env.contract.address.clone(), coins[0].clone().denom)?
            .amount
            - coins[0].amount;
        coins[1].amount = deps
            .querier
            .query_balance(env.contract.address.clone(), coins[1].clone().denom)?
            .amount
            - coins[1].amount;
        Ok(Response::new()
            .add_messages(vec![
                CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                    send_tx: Some(SendTx {
                        remote_chain_destination_address: receiver.clone(),
                        amount: coins[0].to_string(),
                        chain_reference_id: chain_id.clone(),
                    }),
                    cancel_tx: None,
                }),
                CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                    send_tx: Some(SendTx {
                        remote_chain_destination_address: receiver,
                        amount: coins[1].to_string(),
                        chain_reference_id: chain_id,
                    }),
                    cancel_tx: None,
                }),
            ])
            .add_attribute("lp_token", lp_token)
            .add_attribute("coin0", coins[0].to_string())
            .add_attribute("coin1", coins[1].to_string())
            .add_attribute("action", "remove_liquidity"))
    }

    pub fn execute_reply(
        deps: DepsMut,
        env: Env,
        payload: Binary,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let (recipient, chain_id, coin): (String, String, Coin) = from_json(payload)?;
        let mut increased_coin = deps
            .querier
            .query_balance(env.contract.address.clone(), coin.denom.clone())?;
        increased_coin.amount -= coin.amount;
        assert!(!increased_coin.amount.is_zero(), "Not enough output coin");
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                send_tx: Some(SendTx {
                    remote_chain_destination_address: recipient,
                    amount: increased_coin.to_string(),
                    chain_reference_id: chain_id,
                }),
                cancel_tx: None,
            }))
            .add_attribute("coin_out", increased_coin.to_string())
            .add_attribute("action", "execute_reply"))
    }
    pub fn add_liquidity(
        deps: DepsMut,
        env: Env,
        payload: Binary,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let (depositor, lp_token, init_lp_balance): (String, String, Uint128) = from_json(payload)?;
        let result_lp_balance: BalanceResponse = deps.querier.query_wasm_smart(
            lp_token.clone(),
            &Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            },
        )?;
        let lp_amount = result_lp_balance.balance - init_lp_balance;
        LP_BALANCES.update(
            deps.storage,
            (depositor.clone(), lp_token.clone()),
            |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() + lp_amount) },
        )?;
        Ok(Response::new()
            .add_attribute("lp_token", lp_token)
            .add_attribute("lp_amount", lp_amount)
            .add_attribute("action", "add_liquidity"))
    }

    pub fn exchange_for_single_liqudity(
        deps: DepsMut,
        env: Env,
        payload: Binary,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let (pair, depositor, lp_token, init_lp_balance, coins): (
            String,
            String,
            String,
            Uint128,
            Vec<Coin>,
        ) = from_json(payload)?;
        let output_coin = Coin {
            denom: coins[1].denom.clone(),
            amount: deps
                .querier
                .query_balance(&env.contract.address, coins[1].denom.clone())?
                .amount
                - coins[1].amount,
        };
        assert!(!output_coin.amount.is_zero(), "Not enough output coin");
        let payload = to_json_binary(&(depositor, lp_token, init_lp_balance))?;
        let coins = vec![
            Coin {
                denom: coins[0].denom.clone(),
                amount: coins[0].amount,
            },
            Coin {
                denom: output_coin.denom.clone(),
                amount: output_coin.amount,
            },
        ];
        Ok(Response::new()
            .add_submessage(SubMsg {
                id: ADD_LIQUIDITY_REPLY_ID,
                msg: CosmosMsg::Wasm(WasmMsg::Execute {
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
                        slippage_tolerance: None,
                        receiver: None,
                    })?,
                    funds: coins.clone(),
                }),
                payload,
                gas_limit: None,
                reply_on: ReplyOn::Success,
            })
            .add_attribute("coin0", coins[0].to_string())
            .add_attribute("coin1", coins[1].to_string())
            .add_attribute("action", "exchange_for_single_liqudity"))
    }
    pub fn check_padex_diff(
        deps: DepsMut,
        payload: Binary,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let (user, padex_balance): (String, Coin) = from_json(payload)?;
        let new_balance = deps.querier.query_balance(
            deps.api.addr_validate(user.as_str())?,
            padex_balance.denom.clone(),
        )?;
        if new_balance.amount > padex_balance.amount {
            let diff_balance = new_balance.amount - padex_balance.amount;
            if !diff_balance.is_zero() {
                LP_BALANCES.update(
                    deps.storage,
                    (user.clone(), padex_balance.denom.clone()),
                    |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() + diff_balance) },
                )?;
            }
        } else {
            let diff_balance = padex_balance.amount - new_balance.amount;
            if !diff_balance.is_zero() {
                LP_BALANCES.update(
                    deps.storage,
                    (user.clone(), padex_balance.denom.clone()),
                    |balance| -> StdResult<_> { Ok(balance.unwrap_or_default() - diff_balance) },
                )?;
            }
        }
        Ok(Response::new()
            .add_attribute("action", "claim_rewards")
            .add_attribute("user", user)
            .add_attribute("padex_balance", padex_balance.to_string()))
    }
}

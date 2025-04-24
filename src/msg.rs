#[allow(unused_imports)]
use crate::state::{ChainSetting, State};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin, CustomMsg, Decimal, Uint128, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub retry_delay: u64,
    pub owners: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Exchange {
        dex_router: Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        max_spread: Option<Decimal>,
        funds: Vec<Coin>,
        recipient: String,
    },
    AddLiquidity {
        pair: Addr,
        coins: Vec<Coin>,
        slippage_tolerance: Option<Decimal>,
    },
    RemoveLiquidity {
        chain_id: String,
        pair: Addr,
        amount: Uint128,
        receiver: String,
    },
    SetChainSetting {
        chain_id: String,
        compass_job_id: String,
        main_job_id: String,
    },
    SetPaloma {
        chain_id: String,
    },
    UpdateRefundWallet {
        chain_id: String,
        new_refund_wallet: String,
    },
    UpdateGasFee {
        chain_id: String,
        new_gas_fee: Uint256,
    },
    UpdateServiceFeeCollector {
        chain_id: String,
        new_service_fee_collector: String,
    },
    UpdateServiceFee {
        chain_id: String,
        new_service_fee: Uint256,
    },
    UpdateConfig {
        retry_delay: Option<u64>,
    },
    AddOwner {
        owners: Vec<String>,
    },
    RemoveOwner {
        owner: String,
    },
}

#[cw_serde]
pub enum SwapOperation {
    AstroSwap {
        /// Information about the asset being swapped
        offer_asset_info: AssetInfo,
        /// Information about the asset we swap to
        ask_asset_info: AssetInfo,
    },
}

#[cw_serde]
#[derive(Hash, Eq)]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },
    /// Native token
    NativeToken { denom: String },
}

#[cw_serde]
pub enum ExternalExecuteMsg {
    ExecuteSwapOperations {
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        to: Option<String>,
        max_spread: Option<Decimal>,
    },
    ProvideLiquidity {
        assets: Vec<Asset>,
        slippage_tolerance: Option<Decimal>,
        receiver: Option<String>,
    },
    WithdrawLiquidity {
        #[serde(default)]
        assets: Vec<Asset>,
    },
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
}

#[cw_serde]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}
#[cw_serde]
pub enum PalomaMsg {
    /// Message struct for cross-chain calls.
    SchedulerMsg {
        execute_job: ExecuteJob,
    },
    SkywayMsg {
        send_tx: SendTx,
    },
}

#[cw_serde]
pub struct ExecuteJob {
    pub job_id: String,
    pub payload: Binary,
}

#[cw_serde]
pub struct SendTx {
    pub remote_chain_destination_address: String,
    pub amount: String,
    pub chain_reference_id: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Query the current state of the contract
    #[returns(State)]
    State {},
    /// Query the current chain settings
    #[returns(ChainSetting)]
    ChainSetting { chain_id: String },
}

#[cw_serde]
pub enum ExternalQueryMsg {
    Pair {},
}

/// This structure stores the main parameters for an palomadex pair
#[cw_serde]
pub struct PairInfo {
    /// Asset information for the assets in the pool
    pub asset_infos: Vec<AssetInfo>,
    /// Pair contract address
    pub contract_addr: Addr,
    /// Pair LP token address
    pub liquidity_token: Addr,
    /// The pool type (xyk, stableswap etc) available in [`PairType`]
    pub pair_type: PairType,
}

#[derive(Eq)]
#[cw_serde]
pub enum PairType {
    /// XYK pair type
    Xyk {},
    /// Stable pair type
    Stable {},
    /// Custom pair type
    Custom(String),
}

impl CustomMsg for PalomaMsg {}

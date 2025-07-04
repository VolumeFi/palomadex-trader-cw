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
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Exchange {
        dex_router: Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        max_spread: Option<Decimal>,
        funds: Vec<Coin>,
        chain_id: String,
        recipient: String,
    },
    SendToken {
        chain_id: String,
        tokens: Vec<String>,
        to: String,
        amounts: Vec<Uint128>,
        nonce: Uint128,
    },
    AddLiquidity {
        pair: Addr,
        coins: Vec<Coin>,
        slippage_tolerance: Option<Decimal>,
        depositor: String,
    },
    RemoveLiquidity {
        chain_id: String,
        pair: Addr,
        amount: Uint128,
        receiver: String,
    },
    SendToEVM {
        chain_id: String,
        amounts: Vec<String>,
        recipient: String,
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
    CancelTx {
        transaction_id: u64,
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
    SchedulerMsg { execute_job: ExecuteJob },
    SkywayMsg {
        send_tx: Option<SendTx>,
        cancel_tx: Option<CancelTx>,
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
pub struct CancelTx {
    pub transaction_id: u64,
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

    #[returns(Uint128)]
    LpQuery { user: String, lp_token: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ExternalQueryMsg {
    #[returns(PairInfo)]
    Pair {},
    #[returns(PoolResponse)]
    Pool {},
    #[returns(ConfigResponse)]
    Config {},
    #[returns(FeeInfoResponse)]
    FeeInfo {
        /// The pair type for which we return fee information. Pair type is a [`PairType`] struct
        pair_type: PairType,
    },
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

#[cw_serde]
pub struct PoolResponse {
    /// The assets in the pool together with asset amounts
    pub assets: Vec<Asset>,
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
}

#[cw_serde]
pub struct ConfigResponse {
    /// Last timestamp when the cumulative prices in the pool were updated
    pub block_time_last: u64,
    /// The pool's parameters
    pub params: Option<Binary>,
    /// The contract owner
    pub owner: Addr,
    /// The factory contract address
    pub factory_addr: Addr,
}

#[cw_serde]
pub struct FeeInfoResponse {
    /// Contract address to send governance fees to
    pub fee_address: Option<Addr>,
    /// Total amount of fees (in bps) charged on a swap
    pub total_fee_bps: u16,
    /// Amount of fees (in bps) sent to the Maker contract
    pub maker_fee_bps: u16,
}

impl CustomMsg for PalomaMsg {}

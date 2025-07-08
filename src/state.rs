use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owners: Vec<Addr>,
    pub retry_delay: u64,
}

#[cw_serde]
pub struct ChainSetting {
    pub compass_job_id: String,
    pub main_job_id: String,
}

#[cw_serde]
pub struct IncentivesSetting {
    pub incentivizer: Addr,
    pub padex: String,
    pub vepades: String,
}

pub const CHAIN_SETTINGS: Map<String, ChainSetting> = Map::new("chain_settings");
pub const STATE: Item<State> = Item::new("state");
pub const LP_BALANCES: Map<(String, String), Uint128> = Map::new("lp_balances");
pub const MESSAGE_TIMESTAMP: Map<(String, String), Timestamp> = Map::new("message_timestamp");
pub const INCENTIVES_SETTING: Item<IncentivesSetting> = Item::new("incentives_setting");

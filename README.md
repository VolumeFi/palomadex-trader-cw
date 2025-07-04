# Palomadex Trader Contract - Security Audit Documentation

## Overview

The Palomadex Trader Contract is a CosmWasm smart contract that facilitates cross-chain trading and liquidity operations on the Paloma network. This contract acts as a bridge between different blockchain networks, enabling token swaps, liquidity provision, and cross-chain transfers.

**Contract Name:** `crates.io:palomadex-trader-cw`  
**Version:** 0.1.0  
**Author:** wc117 <williamchang89117@gmail.com>

## Architecture

The contract is structured with the following key components:

- **Entry Points:** `instantiate`, `execute`, `query`, `reply`, `migrate`
- **Execute Functions:** Core trading and administrative functions
- **Reply Handlers:** Process asynchronous operation results
- **State Management:** Persistent storage for contract state and settings

## Security Model

### Access Control
- **Owner-based authorization:** All administrative functions require owner privileges
- **Multi-owner support:** Contract can have multiple owners for enhanced security
- **Owner validation:** All owner addresses are validated before storage

### Critical Security Considerations
- **Cross-chain operations:** Involves external chain interactions with potential for failures
- **Liquidity management:** Direct manipulation of LP token balances
- **Fee handling:** Service and gas fee configurations affect economic security
- **Nonce management:** Prevents replay attacks on cross-chain operations

## Function Documentation

### Entry Point Functions

#### `instantiate`
**Purpose:** Initializes the contract with initial configuration  
**Access:** Public (contract deployment)  
**Security Level:** Critical

**Parameters:**
- `retry_delay: u64` - Delay between retry attempts for failed operations
- `owners: Vec<String>` - List of initial contract owners

**Security Considerations:**
- Owner addresses are validated before storage
- Sets contract version for migration tracking
- Initial state is immutable after deployment

**Example Usage:**
```json
{
  "retry_delay": 3600,
  "owners": ["paloma1abc...", "paloma1def..."]
}
```

#### `migrate`
**Purpose:** Handles contract upgrades and migrations  
**Access:** Public (contract admin)  
**Security Level:** Critical

**Parameters:**
- `deps: DepsMut` - Contract dependencies
- `_env: Env` - Contract environment
- `_msg: MigrateMsg` - Migration message (empty)

**Security Considerations:**
- Updates contract version
- Preserves existing state during migration
- No state modifications allowed

#### `execute`
**Purpose:** Main entry point for all contract operations  
**Access:** Public (authenticated users)  
**Security Level:** High

**Parameters:**
- `deps: DepsMut` - Contract dependencies
- `env: Env` - Contract environment
- `info: MessageInfo` - Transaction information
- `msg: ExecuteMsg` - Operation to execute

**Security Considerations:**
- Routes to specific execute functions based on message type
- Each function has its own authorization checks
- Returns `Response<PalomaMsg>` for cross-chain operations

### Trading Functions

#### `exchange`
**Purpose:** Executes token swaps through DEX routers  
**Access:** Owners only  
**Security Level:** High

**Parameters:**
- `dex_router: Addr` - DEX router contract address
- `operations: Vec<SwapOperation>` - Swap operations to execute
- `minimum_receive: Option<Uint128>` - Minimum tokens to receive
- `max_spread: Option<Decimal>` - Maximum allowed spread
- `funds: Vec<Coin>` - Funds to send with transaction
- `chain_id: String` - Target chain identifier
- `recipient: String` - Recipient address on target chain

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Input validation:** Validates DEX router and operations
- **Slippage protection:** Enforces minimum receive amounts
- **Cross-chain risk:** Operations may fail on target chain
- **Fund safety:** Uses submessage with reply for atomicity

**Example Usage:**
```json
{
  "exchange": {
    "dex_router": "paloma1router...",
    "operations": [
      {
        "astro_swap": {
          "offer_asset_info": {"native_token": {"denom": "uluna"}},
          "ask_asset_info": {"native_token": {"denom": "uusdc"}}
        }
      }
    ],
    "minimum_receive": "1000000",
    "max_spread": "0.01",
    "funds": [{"denom": "uluna", "amount": "1000000"}],
    "chain_id": "ethereum",
    "recipient": "0x1234..."
  }
}
```

#### `add_liquidity`
**Purpose:** Adds liquidity to DEX pairs  
**Access:** Owners only  
**Security Level:** High

**Parameters:**
- `pair: Addr` - Pair contract address
- `coins: Vec<Coin>` - Coins to add as liquidity
- `slippage_tolerance: Option<Decimal>` - Maximum slippage allowed
- `depositor: String` - Address to credit LP tokens to

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Pair validation:** Queries pair information before execution
- **LP tracking:** Tracks LP token balances per depositor
- **Single coin handling:** Supports single coin liquidity with swap
- **Slippage protection:** Enforces maximum slippage tolerance

**Complex Logic:**
- For single coin liquidity, calculates optimal swap amount using `calculate_swap_amount`
- Uses mathematical formula to determine swap ratio for balanced liquidity
- Handles both XYK and stable pairs differently

**Example Usage:**
```json
{
  "add_liquidity": {
    "pair": "paloma1pair...",
    "coins": [
      {"denom": "uluna", "amount": "1000000"},
      {"denom": "uusdc", "amount": "1000000"}
    ],
    "slippage_tolerance": "0.01",
    "depositor": "paloma1user..."
  }
}
```

#### `remove_liquidity`
**Purpose:** Removes liquidity from DEX pairs  
**Access:** Owners only  
**Security Level:** High

**Parameters:**
- `chain_id: String` - Target chain for receiving tokens
- `pair: Addr` - Pair contract address
- `amount: Uint128` - LP tokens to burn
- `receiver: String` - Address to receive underlying tokens

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Balance validation:** Checks LP token balance before removal
- **Cross-chain transfer:** Sends tokens to target chain
- **Atomic operation:** Uses submessage with reply for consistency

**Example Usage:**
```json
{
  "remove_liquidity": {
    "chain_id": "ethereum",
    "pair": "paloma1pair...",
    "amount": "1000000",
    "receiver": "0x1234..."
  }
}
```

### Cross-Chain Functions

#### `send_to_evm`
**Purpose:** Sends tokens to EVM-compatible chains  
**Access:** Owners only  
**Security Level:** High

**Parameters:**
- `chain_id: String` - Target EVM chain identifier
- `amounts: Vec<String>` - Token amounts to send
- `recipient: String` - Recipient address on target chain

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Multiple transfers:** Supports sending multiple amounts in single transaction
- **Cross-chain risk:** Relies on Paloma bridge infrastructure
- **No validation:** Amounts are passed as strings without validation

**Example Usage:**
```json
{
  "send_to_evm": {
    "chain_id": "ethereum",
    "amounts": ["1000000uluna", "1000000uusdc"],
    "recipient": "0x1234..."
  }
}
```

#### `send_token`
**Purpose:** Sends tokens to external chains with nonce protection  
**Access:** Owners only  
**Security Level:** High

**Parameters:**
- `chain_id: String` - Target chain identifier
- `tokens: Vec<String>` - Token contract addresses
- `to: String` - Recipient address
- `amounts: Vec<Uint128>` - Token amounts
- `nonce: Uint128` - Unique transaction identifier

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Nonce protection:** Prevents replay attacks using timestamp tracking
- **Retry delay:** Enforces minimum time between retries
- **Input encoding:** Encodes parameters for EVM contract calls
- **Job scheduling:** Uses Paloma scheduler for execution

**Example Usage:**
```json
{
  "send_token": {
    "chain_id": "ethereum",
    "tokens": ["0x1234...", "0x5678..."],
    "to": "0xabcd...",
    "amounts": ["1000000", "2000000"],
    "nonce": "12345"
  }
}
```

### Administrative Functions

#### `set_chain_setting`
**Purpose:** Configures chain-specific settings  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier
- `compass_job_id: String` - Compass job identifier
- `main_job_id: String` - Main job identifier

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Storage update:** Modifies persistent chain settings
- **No validation:** Job IDs are not validated

**Example Usage:**
```json
{
  "set_chain_setting": {
    "chain_id": "ethereum",
    "compass_job_id": "compass_eth_001",
    "main_job_id": "main_eth_001"
  }
}
```

#### `set_paloma`
**Purpose:** Sets Paloma configuration for a chain  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Contract encoding:** Creates EVM contract interface
- **Job execution:** Schedules job on Paloma network
- **No parameters:** Function takes no parameters

**Example Usage:**
```json
{
  "set_paloma": {
    "chain_id": "ethereum"
  }
}
```

#### `update_refund_wallet`
**Purpose:** Updates refund wallet address for a chain  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier
- `new_refund_wallet: String` - New refund wallet address

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Address validation:** Validates EVM address format
- **Contract encoding:** Encodes function call for EVM contract
- **Job scheduling:** Uses Paloma scheduler for execution

**Example Usage:**
```json
{
  "update_refund_wallet": {
    "chain_id": "ethereum",
    "new_refund_wallet": "0x1234..."
  }
}
```

#### `update_gas_fee`
**Purpose:** Updates gas fee configuration for a chain  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier
- `new_gas_fee: Uint256` - New gas fee amount

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Fee validation:** No validation of fee amount
- **Contract encoding:** Encodes uint256 parameter for EVM
- **Job scheduling:** Uses Paloma scheduler for execution

**Example Usage:**
```json
{
  "update_gas_fee": {
    "chain_id": "ethereum",
    "new_gas_fee": "20000000000000000"
  }
}
```

#### `update_service_fee_collector`
**Purpose:** Updates service fee collector address  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier
- `new_service_fee_collector: String` - New fee collector address

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Address validation:** Validates EVM address format
- **Economic impact:** Affects fee collection mechanism

**Example Usage:**
```json
{
  "update_service_fee_collector": {
    "chain_id": "ethereum",
    "new_service_fee_collector": "0x1234..."
  }
}
```

#### `update_service_fee`
**Purpose:** Updates service fee amount  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `chain_id: String` - Chain identifier
- `new_service_fee: Uint256` - New service fee amount

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Fee validation:** No validation of fee amount
- **Economic impact:** Directly affects user costs

**Example Usage:**
```json
{
  "update_service_fee": {
    "chain_id": "ethereum",
    "new_service_fee": "1000000000000000"
  }
}
```

#### `update_config`
**Purpose:** Updates contract configuration  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `retry_delay: Option<u64>` - New retry delay (optional)

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **State modification:** Updates persistent contract state
- **Timing impact:** Affects retry behavior for failed operations

**Example Usage:**
```json
{
  "update_config": {
    "retry_delay": 7200
  }
}
```

#### `add_owner`
**Purpose:** Adds new contract owners  
**Access:** Existing owners only  
**Security Level:** Critical

**Parameters:**
- `owners: Vec<String>` - New owner addresses to add

**Security Considerations:**
- **Authorization:** Requires existing owner privileges
- **Address validation:** Validates all new owner addresses
- **Duplicate prevention:** Checks for existing owners before adding
- **Privilege escalation:** Grants full administrative access

**Example Usage:**
```json
{
  "add_owner": {
    "owners": ["paloma1newowner..."]
  }
}
```

#### `remove_owner`
**Purpose:** Removes contract owners  
**Access:** Existing owners only  
**Security Level:** Critical

**Parameters:**
- `owner: String` - Owner address to remove

**Security Considerations:**
- **Authorization:** Requires existing owner privileges
- **Address validation:** Validates owner address
- **Existence check:** Ensures owner exists before removal
- **Privilege reduction:** Removes administrative access

**Example Usage:**
```json
{
  "remove_owner": {
    "owner": "paloma1oldowner..."
  }
}
```

#### `cancel_tx`
**Purpose:** Cancels pending cross-chain transactions  
**Access:** Owners only  
**Security Level:** Medium

**Parameters:**
- `transaction_id: u64` - Transaction ID to cancel

**Security Considerations:**
- **Authorization:** Requires owner privileges
- **Transaction tracking:** Relies on external transaction tracking
- **No validation:** Does not verify transaction existence

**Example Usage:**
```json
{
  "cancel_tx": {
    "transaction_id": 12345
  }
}
```

### Query Functions

#### `query`
**Purpose:** Handles all contract queries  
**Access:** Public  
**Security Level:** Low

**Parameters:**
- `deps: Deps` - Contract dependencies
- `_env: Env` - Contract environment
- `msg: QueryMsg` - Query message

**Available Queries:**
- `State` - Returns contract state
- `ChainSetting` - Returns chain-specific settings
- `LpQuery` - Returns LP token balance for user

**Example Usage:**
```json
{
  "state": {}
}
```

### Reply Handler Functions

#### `reply`
**Purpose:** Handles asynchronous operation results  
**Access:** Internal (contract only)  
**Security Level:** High

**Parameters:**
- `deps: DepsMut` - Contract dependencies
- `env: Env` - Contract environment
- `msg: Reply` - Reply message with operation result

**Reply Types:**
- `REMOVE_LIQUIDITY_REPLY_ID` - Handles liquidity removal results
- `EXECUTE_REPLY_ID` - Handles swap execution results
- `ADD_LIQUIDITY_REPLY_ID` - Handles liquidity addition results
- `EXECUTE_FOR_SINGLE_LIQUIDITY_REPLY_ID` - Handles single coin liquidity results

#### `remove_liquidity` (Reply Handler)
**Purpose:** Processes liquidity removal results  
**Access:** Internal  
**Security Level:** High

**Security Considerations:**
- **Balance calculation:** Calculates actual received amounts
- **Cross-chain transfer:** Sends tokens to target chain
- **Atomic operation:** Ensures consistency of state changes

#### `execute_reply` (Reply Handler)
**Purpose:** Processes swap execution results  
**Access:** Internal  
**Security Level:** High

**Security Considerations:**
- **Balance verification:** Ensures sufficient output tokens
- **Cross-chain transfer:** Sends swapped tokens to recipient
- **Amount calculation:** Calculates actual received amount

#### `add_liquidity` (Reply Handler)
**Purpose:** Processes liquidity addition results  
**Access:** Internal  
**Security Level:** High

**Security Considerations:**
- **LP tracking:** Updates LP token balances for depositor
- **Balance calculation:** Calculates actual LP tokens received
- **State consistency:** Ensures accurate balance tracking

#### `exchange_for_single_liqudity` (Reply Handler)
**Purpose:** Processes single coin liquidity with swap results  
**Access:** Internal  
**Security Level:** High

**Security Considerations:**
- **Swap verification:** Ensures swap produced sufficient output
- **Liquidity provision:** Continues with liquidity addition
- **Complex flow:** Handles multi-step operation

### Utility Functions

#### `calculate_swap_amount`
**Purpose:** Calculates optimal swap amount for single coin liquidity  
**Access:** Internal  
**Security Level:** Medium

**Parameters:**
- `input_amount: Uint128` - Input token amount
- `reserve_in: Uint128` - Reserve of input token
- `fee_bps: u16` - Fee in basis points

**Security Considerations:**
- **Mathematical precision:** Uses Decimal256 for calculations
- **Fee consideration:** Accounts for trading fees
- **No overflow protection:** Relies on Uint128 arithmetic

## State Management

### Storage Structures

#### `State`
```rust
pub struct State {
    pub owners: Vec<Addr>,      // Contract owners
    pub retry_delay: u64,       // Retry delay for failed operations
}
```

#### `ChainSetting`
```rust
pub struct ChainSetting {
    pub compass_job_id: String, // Compass job identifier
    pub main_job_id: String,    // Main job identifier
}
```

### Storage Maps

- `STATE: Item<State>` - Contract state
- `CHAIN_SETTINGS: Map<String, ChainSetting>` - Chain-specific settings
- `LP_BALANCES: Map<(String, String), Uint128>` - LP token balances per user
- `MESSAGE_TIMESTAMP: Map<(String, String), Timestamp>` - Message timestamps for nonce protection

## Error Handling

### Custom Errors

- `Unauthorized` - Access denied
- `Pending` - Operation is pending (nonce protection)
- `UnknownReply` - Invalid reply received
- `UnsupportedCw20` - CW20 tokens not supported
- `InsufficientLiquidity` - Insufficient LP tokens

## Security Recommendations

### Critical Issues
1. **Owner Privileges:** All administrative functions require owner access - ensure secure key management
2. **Cross-chain Risk:** Operations may fail on target chains - implement proper error handling
3. **Liquidity Management:** Direct LP token manipulation - verify balance calculations
4. **Fee Configuration:** Service and gas fees affect economics - validate fee amounts

### Medium Issues
1. **Input Validation:** Some functions lack input validation - add parameter checks
2. **Error Handling:** Limited error handling for external calls - improve error recovery
3. **State Consistency:** Complex state updates - ensure atomicity

### Low Issues
1. **Documentation:** Some functions lack detailed documentation - improve code comments
2. **Testing:** Complex logic requires comprehensive testing - add unit and integration tests

## Testing Recommendations

1. **Unit Tests:** Test each function with various input parameters
2. **Integration Tests:** Test cross-chain operations end-to-end
3. **Security Tests:** Test access control and authorization
4. **Edge Cases:** Test boundary conditions and error scenarios
5. **Gas Optimization:** Test gas usage for all operations

## Deployment Considerations

1. **Initial Configuration:** Set appropriate retry delays and owner addresses
2. **Chain Settings:** Configure job IDs for all supported chains
3. **Fee Configuration:** Set reasonable service and gas fees
4. **Monitoring:** Implement monitoring for cross-chain operations
5. **Upgrade Path:** Plan for contract upgrades and migrations

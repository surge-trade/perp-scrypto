# Surge

Surge is a perpetual futures decentralized exchange. It uses an oracle price feed in combination with a USD liquidity pool to execute trades. Trades use a two-step process to avoid front running. First, the trader submits a request to the exchange, then a keeper executes the trade. All trades are settled in USD, which is labeled as the base token. Surge aims to provide highly efficient liquidity and good execution for the trader while ensuring a stable yield for liquidity providers. Since the pool is the counter-party for all trades, it is important to keep open interest between longs and shorts as balanced as possible. The difference between long and short open interest is referred to as the skew. In the ideal state where the combined open interest is high but the skew is zero, the pool is delta neutral and is assured to make a profit. The protocol seeks to maintain this state by funding and fee incentives.

## Architecture

### Oracle

Surge uses a pull-based oracle model. When a keeper processes a trade, the latest price is fetched from the oracle and submitted to the exchange. Prices are signed by the oracle and verified on-chain. If the age of the price is too old, the exchange will not accept it. Surge allows for multiple oracles to be used, but all oracles must agree on the price to avoid arbitrage. Oracles have an N of N trust model.

### Keeper

Surge uses a keeper to process requests such as trades and removing collateral, as well as auto-deleveraging, liquidations, and updating funding rates. The keeper is expected to save all requests and execute them when possible. There may be significant time between the request submission and the keeper executing the trade, such as in the case of a limit order with its execution price not yet met. Keepers have a 1 of N trust model.

### On-Ledger Components

#### Exchange

Surge uses a modular architecture where the `exchange` is stateless and contains the core logic while peripheral components are responsible for holding state and tokens. The `exchange` controls these peripheral components using the `authority` token. This allows for upgradeability of the core logic by simply deploying a new version of the `exchange` and passing the `authority` token to the new component.

#### Margin Pool

The `margin pool` component is responsible for being the central counterparty for all positions and debt within the system. It also acts as a bank for all base tokens in the system.

#### Margin Account

Individual `margin accounts` components act as trading accounts for users. Each account holds the user's collateral, positions, unsettled debt, and requests. Accounts have three auth levels to make them more convenient to use without compromising security.

- Level 1: Can change the account's auth settings.
- Level 2: Can remove collateral from the account.
- Level 3: Can submit trades and cancel requests.

#### Config

The `config` component is responsible for holding the exchange's settings. This includes things such as tradable pairs, valid collaterals, fees, and funding rates.

#### Referral Generator

The `referral generator` component is responsible for tracking and verifying referral codes. Users can be given a referral NFT which then allows them to create referral codes with claimable tokens. The referred user can then claim by using the referral code when creating their account. This gives the referred user a fee rebate on their trades and the owner of the referral NFT a share of the trading fees.

#### Fee Distributor

The `fee distributor` component acts as an account for protocol and treasury fees.

#### Permission Registry

The `permission registry` component is responsible for mapping auth to usable `margin accounts`. This allows the user to seamlessly log in on a new device.

#### Fee Delegator

The `fee delegator` component is responsible for paying network fees for users. In exchange, the user has debt assigned to their `margin account`. This allows users to not have to own the network token in order to use Surge.

#### Token Wrapper

The `token wrapper` component is responsible for wrapping and unwrapping the child tokens into the base token. Initially, this will be the xUSDC. This allows for the possibility of changing to a different stablecoin in the future if desired. Note: only one child token should be allowed at a time to avoid arbitrage. The `token wrapper` also enables flash loans.

#### Env Registry

The `env registry` component acts as an on-ledger store for variables used by the front end.

### Resources

- `base token`: Wrapped USDC.
- `LP token`: LP token for providing liquidity to the pool.
- `referral NFT`: NFT that allows for the creation of referral codes.
- `protocol token`: Utility token for the protocol.
- `keeper reward token`: Reward token for the keeper to incentivize submitting transactions.
- `authority token`: Controls the peripheral components as well as the `LP token` and `referral NFT`.
- `base authority token`: Controls the `base token`.

## Actions

### Owner Actions

- `deposit_authority`: Deposit authority tokens into the exchange component (restricted to owner).
- `withdraw_authority`: Withdraw authority tokens from the exchange component (restricted to owner).
- `signal_upgrade`: Signal an upgrade to a new exchange component (restricted to owner).
- `update_exchange_config`: Update the exchange configuration (restricted to owner).
- `update_pair_configs`: Update the configuration for trading pairs (restricted to owner).
- `update_collateral_configs`: Update the configuration for collateral assets (restricted to owner).
- `remove_collateral_config`: Remove a collateral asset configuration (restricted to owner).

### Admin Actions

- `collect_treasury`: Collect funds from the treasury (restricted to treasury admin).
- `collect_fee_delegator`: Collect fees from the fee delegator (restricted to fee delegator admin).
- `mint_referral`: Mint a new referral NFT (restricted to referral admin).
- `mint_referral_with_allocation`: Mint a new referral NFT with allocation (restricted to referral admin).
- `update_referral`: Update an existing referral (restricted to referral admin).
- `add_referral_allocation`: Add allocation to an existing referral (restricted to referral admin).

### User Actions

- `swap_protocol_fee`: Swap protocol token for protocol fees.
- `add_liquidity`: Add liquidity to the pool.
- `remove_liquidity`: Remove liquidity from the pool.
- `create_referral_codes`: Create referral codes.
- `create_referral_codes_from_allocation`: Create referral codes from referral allocation.
- `collect_referral_rewards`: Collect referral rewards.
- `create_account`: Create a new margin account.
- `set_level_1_auth`, `set_level_2_auth`, `set_level_3_auth`: Set different levels of authentication for an account.
- `add_collateral`: Add collateral to an account.
- `remove_collateral_request`: Request to remove collateral.
- `margin_order_request`: Request to trade.
- `margin_order_tp_sl_request`: Combination of requests to trade with take profit and stop loss attached.
- `cancel_requests`: Cancel requests.

### Keeper Actions

- `process_request`: Process a pending request.
- `swap_debt`: Swap debt to pay off an account debt in return for collateral.
- `liquidate`: Liquidate an account.
- `auto_deleverage`: Automatically deleverage positions.
- `update_pairs`: Update trading pair information.

### Public Methods

- `get_pairs`: Get information about trading pairs.
- `get_permissions`: Get permission information.
- `get_account_details`: Get details of a margin account.
- `get_pool_details`: Get details of the liquidity pool.
- `get_pair_details`: Get details of a specific trading pair.
- `get_referral_details`: Get details of a referral.
- `get_exchange_config`: Get the current exchange configuration.
- `get_pair_configs`: Get configurations of all trading pairs.
- `get_pair_configs_len`: Get the number of trading pair configurations.
- `get_collateral_configs`: Get configurations of all collateral assets.
- `get_collaterals`: Get a list of all collateral assets.
- `get_protocol_balance`: Get the balance of the protocol.
- `get_treasury_balance`: Get the balance of the treasury.

### Adding and Removing Liquidity

Users can provide liquidity to the pool in the form of `base tokens`. Liquidity providers take the opposite position of every trade and collect fees and a share of funding. In the ideal state where the combined open interest is high but the skew is zero, the pool is delta neutral and liquidity providers are assured to make a profit. Adding and removing liquidity is atomic but has a small fee to prevent arbitrage.

### Adding and Removing Collateral

Before a user can trade, they must first add collateral to their account. Adding collateral is an atomic action, but removing collateral requires submitting a request that is then executed by a keeper. This is to ensure the user meets all margin requirements.

### Margin Orders

In order to trade, a user submits a margin order request. The request is then executed by a keeper when possible, creating a position within the user's margin account.

Key Features:

- Cross-margin leverage
- Directional price limits
- Reduce-only option
- Slippage protection
- Delayed execution
- Order expiration
- Finite State Machine (FSM) and order chaining

These features enable traders to create sophisticated order strategies. The `margin_order_request` method offers maximum flexibility, while `margin_order_tp_sl_request` simplifies the creation of take-profit and stop-loss orders.

### Auto-Deleveraging

Auto-deleveraging is a mechanism that automatically closes positions to rebalance the pool when market skew becomes excessive. This feature helps maintain system stability during significant price movements or market imbalances. Key aspects of the ADL process include:

1. Activation: ADL is triggered when the skew ratio exceeds a predefined cap.
2. Target selection: Positions are evaluated based on their profit-and-loss (PNL) percentage.
3. Dynamic threshold: The PNL threshold for ADL eligibility is calculated using a cubic function that considers the current skew ratio. This ensures that as skew increases, more positions become eligible for ADL. See: <https://www.desmos.com/calculator/oa47dko39m>
4. Full closure: When a position qualifies for ADL, it is closed entirely.
5. Effectiveness check: The system verifies that the ADL action actually reduces the overall skew.
6. Fee settlement: Normal trading fees are applied to ADL actions, maintaining fairness for all participants.

This approach ensures that the most profitable positions are deleveraged first, with the threshold dynamically adjusting based on market conditions. The ADL mechanism helps prevent extreme imbalances and maintains the overall health of the trading system.

### Liquidations

A liquidation occurs when an account's total value falls below the required maintenance margin. The process involves:

1. Closing all open positions and calculating the resulting profit or loss (PnL).
2. Valuing all collateral assets at a discounted rate.
3. Comparing the account's total value (PnL + discounted collateral + virtual balance) against the required margin.
4. If the account value is below the required margin, the liquidation proceeds:
    - All collateral is withdrawn from the account.
    - A liquidator provides payment in the base currency to cover the discounted collateral value.
    - The account is settled using the PnL and the payment received for collateral.
5. If there's still a negative balance after settlement, this becomes a loss for the liquidity pool:
    - The remaining debt is forgiven (zeroed out for the account).
    - The pool absorbs this loss, socializing it among liquidity providers.

This process ensures under-collateralized positions are closed promptly, minimizing risk to the system. The liquidator takes on market risk in exchange for potentially acquiring discounted assets, while the pool acts as the final backstop for any unrecoverable losses.

### Pair Updates

Regular pair updates are crucial for maintaining accurate exchange state, including the pool's total PnL and skew, as well as for updating funding rates. Updates are triggered by:

- A trade being executed.
- An auto-deleverage being executed.
- A liquidation being executed.
- The last update being too old.
- The price moving past the accepted change threshold.

These updates ensure the system remains current and responsive to market conditions.

### Fees

Trade fees are applied whenever a position is opened or closed either by an order, auto-deleveraging, or liquidation. The fee algorithm calculates a dynamic trading fee based on several factors.

#### Fee Calculation

The fee is calculated based on two components:

a. fee_0: A base fee proportional to the absolute value of the trade.
b. fee_1: A dynamic fee that scales with market skew and trade size.

The algorithm works as follows:

1. fee_0 is calculated as a flat percentage (pair_config.fee_0) of the absolute trade value.
2. fee_1 is more complex:
    - It scales with the current market skew (skew), which is the difference between long and short positions multiplied by the current price.
    - It also scales quadratically with the trade value itself (value).
    - This component aims to charge higher fees for trades that increase market imbalance.
3. The base fee (fee_0) and dynamic fee (fee_1) are added together.
4. The sum is then multiplied by a rebate factor (fee_rebate), which can potentially reduce the fee for certain users (e.g., high-volume traders or users with referral bonuses).
5. Finally, the resulting fee is clamped between 0 and fee_max, ensuring it never exceeds a certain percentage of the trade value.

This algorithm allows for flexible fee structures that can adapt to market conditions, discourage market imbalances, and provide incentives for desired user behaviors, all while maintaining a predictable maximum fee.

### Funding

Funding is accumulated on each open position.

#### Funding Rate Calculation

The funding rate is calculated based on two components:

a. funding_1_rate: Proportional to the current market skew (imbalance between long and short positions).
b. funding_2_rate: Based on an accumulated rate that changes over time, influenced by market skew.

#### Funding Distribution

- If the funding rate is positive, long positions pay short positions.
- If the funding rate is negative, short positions pay long positions.
- A portion of the funding (funding_share) is kept by the pool.
- An additional funding component (funding_pool) is charged as a flat rate to both long and short positions.

#### Funding Indices

The algorithm updates funding_long_index and funding_short_index, which track the cumulative funding for long and short positions respectively.
These indices are used to calculate the funding payments for individual positions based on when they were opened.

#### Time-based Calculation

All funding calculations are proportional to the time period (period) since the last update.

#### Zero Open Interest Handling

If either long or short open interest is zero, no funding is calculated to avoid division by zero.
This algorithm aims to incentivize market balance by making it more expensive to hold positions on the overweight side of the market, while also generating fees for the protocol and liquidity providers. The use of cumulative indices allows for efficient calculation of funding payments for individual positions, regardless of when they were opened.

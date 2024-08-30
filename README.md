# Surge

Surge is perpetual futures decentralized exchange. It uses a oracle price feed in combination with a USD liquidity pool to execute trades. Trades use a two step process to avoid front running. First the trader submits a request to the exchange, then a keeper then executes the trade. All trades are settled in USD which is labeled as the base token. Surge aims to provide highly efficient liquidity and good execution for the trader, while ensure a stable yield for liquidity providers. Since the pool is the counter-party for all trades it is important to keep open interest between longs and shorts as balanced as possible. The difference between long and short open interest is referred to as the skew. In the ideal state where the combined open interest is high but the skew is zero, the pool is delta neutral and is assured to make a profit. The protocol seeks to maintain this state by funding and fee incentives.

## Architecture

### Oracle

Surge uses a pull based oracle model. When a keeper processes a trade the latest price is fetched from the oracle and submitted to the exchange. Prices are signed by the oracle and verified on chain. If the age of the price is too old the exchange will not accept it. Surge allows for multiple oracles to be used but all oracles must agree on the price to avoid arbitrage. Oracles have a N of N trust model..

### Keeper

Surge uses a keeper to process requests such as trades and removing collateral, as well as auto-deleveraging, liquidations, and updating funding rates. The keeper is expected to save all requests and execute them possible. There maybe significant time between the request submission and the keeper executing the trade such as in the case of a limit order with is execution price not yet met. Keepers have a 1 of N trust model.

### On Ledger Components

#### Exchange

Surge uses a modular architecture where the `exchange` is stateless and contains the core logic while peripheral components are responsible for holding state and tokens. The `exchange` controls these peripheral components using the `authority` token. This allows for upgradeability of the core logic but simple deploying a new version of the `exchange` and passing the `authority` token to the new component.

#### Margin Pool

The `margin pool` component is responsible for being the central counter party for all positions and debt within the system. It also acts as a bank for all base tokens in the system.

#### Margin Account

Individual `margin accounts` components act as trading accounts for users. Each account hold the users collateral, positions, unsettled debt, and requests. Accounts have three auth levels to make them more convenient to use without compromising security. 

- Level 1: Can change the accounts auth settings.
- Level 2: Can remove collateral from the account.
- Level 3: Can submit trades and cancel requests.

#### Config

The `config` component is responsible for holding the exchange's settings. This includes things such as tradable pairs, valid collaterals, fees and funding rates, etc.

#### Referral Generator

The `referral generator` component is responsible tracking verifying referral codes. Users can be given a referral nft which then allows them to create referral codes with claimable tokens. The referred user can then claim by using the referral code when creating their account. The gives the referred user a fee rebate on their trades and owner of the referral nft a share of the trading fees.

#### Fee Distributor

The `fee distributor` component acts as an account for protocol and treasury fees.

#### Permission Registry

The `permission registry` component is responsible mapping auth to usable `margin accounts`. This allows for the user to seamlessly login on a new device.

#### Fee Delegator

The `fee delegator` component is responsible for paying network fees for users. In exchange the user has debt assigned to their `margin account`. This allows for users to not have to own the network token in order to use surge.

#### Token Wrapper

The `token wrapper` component is responsible for wrapping and unwrapping the child tokens into the base token. Initially this will be the xUSDC. This allows for the possibility of changing to a different stablecoin in the future if desired. Note: only one child token should be allowed at a time to avoid arbitrage. The `token wrapper` also enables flash loans.

#### Env Registry

The `env registry` component is acts an on ledger store for variables used by the front end.

### Resources

- `base token`: Wrapped USDC.
- `LP token`: LP token for the providing liquidity to the pool.
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

Users can provide liquidity to the pool in the form of `base tokens`. Liquidity providers take the opposite position of every trade and collect fees and a share of funding. In the ideal state where the combined open interest is high but the skew is zero, the pool is delta neutral and liquidity providers are assured to make a profit. Add and remove liquidity is atomic but has a small fee to prevent arbitrage.

### Adding and Removing Collateral

Before a user can trade they must first add collateral to their account. Adding collateral is an atomic action but removing collateral requires submitting a request that is then executed by a keeper. This is to insure the user meets all margin requirements.

### Margin Orders

In order to trade a user submits a margin order request. The request is then executed by a keeper when possible, creating a position within the users margin account.

Key features of margin orders:

- Leverage: Trade using cross margin collateral.
- Execution Price: Directional price limit for the order.
- Reduce Only: Option to ensure an order only reduces an existing position.
- Slippage Protection: Set maximum allowed slippage to protect against unexpected fees.
- Delayed Execution: Option to delay order execution for a specified time.
- Expiry: Set an expiration time for orders to be automatically cancelled if not filled.
- FSM and Chaining: Orders can be submitted as either dormant or active. The execution of one order can then activate or cancel other orders.

Using these features traders can easily create complex orders while specifying the precise conditions for order execution. The `margin_order_request` method allows for maximum flexibility while the `margin_order_tp_sl_request` method allows for the easy creation of take profit and stop loss orders.

### Auto-Deleveraging

Auto-deleveraging is a feature that allows for the automatic closing of positions in the case the pool is out of balance. This feature helps to ensure the system remains in a healthy state in the case of large price movements. Positions with the highest ROI are closed first. The ROI threshold required for auto-deleveraging deceases as the skew increases.

### Liquidations

Liquidations occur when an account's margin falls below the required maintenance margin. All positions are closed and all collateral in the account is sold at a discounted rate to pay off the debt. If the value of the collateral is not enough to pay off the debt the remaining debt is forgiven and realized as a loss by the pool.

### Fees

Trade fees are applied whenever a position is opened or closed either by a order, auto deleveraging, or liquidation. Fees calculations include:

1. Flat percentage fee
2. Price impact fee

The price impact fee determined from the skew increase squared, but maybe negative in the case the trade reduces the skew. The total fee is clamped between 0% and maximum fee to prevent excessive fees. The fee is split between the pool, protocol, treasury, and referral. Referrals also offer a rebate that is reduces the total fee paid.

### Funding

Funding is accumulated on each open position. Funding calculations include:

1. Skew based funding
2. Integral of skew based funding
3. Flat borrowing rate paid to the pool
4. Share of funding paid to the pool

In the case where the funding rate is positive, longs will pay shorts. In the case where the funding rate is negative, shorts will pay longs.

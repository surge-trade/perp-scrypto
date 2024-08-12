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

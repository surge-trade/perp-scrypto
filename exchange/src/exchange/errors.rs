pub const ERROR_COLLATERAL_INVALID: &str = "Invalid collateral";
pub const ERROR_SKEW_TOO_HIGH: &str = "Skew exceeds maximum";
pub const ERROR_INSUFFICIENT_MARGIN: &str = "Insufficient margin";


pub const ERROR_LEVERAGE_TOO_LOW: &str = "Leverage exceeds minimum";
pub const ERROR_LEVERAGE_TOO_HIGH: &str = "Leverage exceeds maximum";

pub const ERROR_MARGIN_ORDER_PRICE_LIMIT: &str = "Price limit not met";

pub const ERROR_INVALID_PAYMENT: &str = "Invalid payment resource";
pub const ERROR_INVALID_LP_TOKEN: &str = "Invalid LP token";

pub const ERROR_REMOVE_COLLATERAL_NEGATIVE_BALANCE: &str = "Cannot remove collateral with debt";
pub const ERROR_REMOVE_COLLATERAL_INSUFFICIENT_POOL_TOKENS: &str = "Insufficient pool balance for withdrawal";

pub const ERROR_LIQUIDATION_SUFFICIENT_MARGIN: &str = "Sufficient margin for liquidation";

pub const ERROR_ADL_SKEW_TOO_LOW: &str = "Skew ratio is too low for ADL";
pub const ERROR_ADL_PNL_BELOW_THRESHOLD: &str = "PnL not positive";
pub const ERROR_ADL_SKEW_NOT_REDUCED: &str = "Skew ratio not reduced";
pub const ERROR_ADL_NO_POSITION: &str = "No position to close";

pub const ERROR_SWAP_NOT_ENOUGH_DEBT: &str = "Not enough debt to swap";

pub const ERROR_POSITIONS_TOO_MANY: &str = "Too many positions";

pub const ERROR_INVALID_ACCOUNT: &str = "Invalid account";
pub const ERROR_INVALID_MARGIN_ACCOUNT: &str = "Invalid margin account";
pub const ERROR_INVALID_POOL: &str = "Invalid pool";
pub const ERROR_INVALID_ORACLE: &str = "Invalid oracle";
pub const ERROR_INVALID_REQUEST_STATUS: &str = "Invalid request status";

pub const ERROR_MISSING_ACCOUNT_POSITION: &str = "Account position not found";
pub const ERROR_MISSING_POOL_POSITION: &str = "Position not found";
pub const ERROR_MISSING_PAIR_CONFIG: &str = "Pair config not found";
pub const ERROR_MISSING_PRICE: &str = "Price not found";
pub const ERROR_MISSING_RESOURCE_FEED: &str = "Resource feed not found";
pub const ERROR_MISSING_REQUEST: &str = "Request not found";
pub const ERROR_MISSING_AUTH: &str = "Authorization role not found";

pub const ERROR_ARITHMETIC: &str = "Arithmetic error";

pub const ERROR_REQUEST_ENCODING: &str = "Request encoding error";
pub const ERROR_REQUEST_DECODING: &str = "Request decoding error";

pub const ERROR_REQUEST_NOT_ACTIVE: &str = "Request not active";
pub const ERROR_REQUEST_NOT_DORMANT: &str = "Request not dormant";
pub const ERROR_CANNOT_MAKE_DORMANT: &str = "Cannot make request dormant";

pub const ERROR_REQUEST_EXPIRED: &str = "Request expired";
pub const ERROR_REQUEST_BEFORE_LIQUIDATION: &str = "Request before liquidation";

pub const PANIC_NEGATIVE_COLLATERAL: &str = "Negative collateral";

pub const ERROR_INVALID_AUTHORITY: &str = "Invalid authority";
pub const ERROR_INVALID_FEE_OATH: &str = "Invalid fee oath";

pub const ERROR_SKEW_TOO_HIGH: &str = "Skew exceeds maximum";
pub const ERROR_PAIR_OI_TOO_HIGH: &str = "Pair OI exceeds maximum";
pub const ERROR_INSUFFICIENT_MARGIN: &str = "Insufficient margin";

pub const ERROR_PAIR_DISABLED: &str = "Pair disabled";

pub const ERROR_MARGIN_ORDER_PRICE_LIMIT: &str = "Price limit not met";

pub const ERROR_INVALID_PAYMENT: &str = "Invalid payment resource";
pub const ERROR_INVALID_LP_TOKEN: &str = "Invalid LP token";

pub const ERROR_WITHDRAWAL_INSUFFICIENT_BALANCE: &str = "Insufficient balance for withdrawal";
pub const ERROR_WITHDRAWAL_INSUFFICIENT_POOL_TOKENS: &str = "Insufficient pool balance for withdrawal";

pub const ERROR_LIQUIDATION_SUFFICIENT_MARGIN: &str = "Sufficient margin for liquidation";
pub const ERROR_LIQUIDATION_INSUFFICIENT_PAYMENT: &str = "Insufficient payment for liquidation";

pub const ERROR_ADL_SKEW_TOO_LOW: &str = "Skew ratio is too low for ADL";
pub const ERROR_ADL_PNL_BELOW_THRESHOLD: &str = "PnL not positive";
pub const ERROR_ADL_SKEW_NOT_REDUCED: &str = "Skew ratio not reduced";
pub const ERROR_ADL_NO_POSITION: &str = "No position to close";

pub const ERROR_SWAP_NO_DEBT: &str = "No debt to swap";

pub const ERROR_CLAIMS_TOO_MANY: &str = "Claims list too big";
pub const ERROR_ACTIVATE_REQUESTS_TOO_MANY: &str = "Activate requests list too big";
pub const ERROR_CANCEL_REQUESTS_TOO_MANY: &str = "Cancel requests list too big";

pub const ERROR_INVALID_COLLATERAL: &str = "Invalid collateral";
pub const ERROR_POSITIONS_TOO_MANY: &str = "Too many positions";
pub const ERROR_COLLATERALS_TOO_MANY: &str = "Too many collaterals";
pub const ERROR_ACTIVE_REQUESTS_TOO_MANY: &str = "Too many active requests";

pub const ERROR_INVALID_ACCOUNT: &str = "Invalid account";
pub const ERROR_INVALID_MARGIN_ACCOUNT: &str = "Invalid margin account";
pub const ERROR_INVALID_REQUEST_STATUS: &str = "Invalid request status";

pub const ERROR_MISSING_POOL_POSITION: &str = "Pool position not found";
pub const ERROR_MISSING_PAIR_CONFIG: &str = "Pair config not found";
pub const ERROR_MISSING_PRICE: &str = "Price not found";
pub const ERROR_MISSING_RESOURCE_FEED: &str = "Resource feed not found";
pub const ERROR_MISSING_REQUEST: &str = "Request not found";
pub const ERROR_MISSING_AUTH: &str = "Authorization role not found";

pub const ERROR_ARITHMETIC: &str = "Arithmetic error";

pub const ERROR_REQUEST_ENCODING: &str = "Request encoding error";
pub const ERROR_REQUEST_DECODING: &str = "Request decoding error";

pub const ERROR_CANCEL_REQUEST_NOT_ACTIVE_OR_DORMANT: &str = "Request not active or dormant";

pub const ERROR_PROCESS_REQUEST_NOT_ACTIVE: &str = "Request not active";
pub const ERROR_PROCESS_REQUEST_BEFORE_VALID_START: &str = "Request before valid start";
pub const ERROR_PROCESS_REQUEST_BEFORE_SUBMISSION: &str = "Request before submission";

pub const ERROR_INVALID_REFERRAL_DATA: &str = "Invalid referral data";
pub const ERROR_INVALID_REFERRAL: &str = "Invalid referral";
pub const ERROR_REFERRAL_LIMIT_REACHED: &str = "Referral limit reached";

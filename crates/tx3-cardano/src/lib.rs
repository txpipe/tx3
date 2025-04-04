mod compile;
mod resolve;

pub mod ledgers;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error coercing {0} into {1}")]
    CoerceError(String, String),

    #[error("invalid asset expression '{0}'")]
    InvalidAssetExpression(String),

    #[error("missing address")]
    MissingAddress,

    #[error("invalid address expression '{0}'")]
    InvalidAddressExpression(String),

    #[error("ledger internal error: {0}")]
    LedgerInternalError(String),

    #[error("tx '{0}' not found")]
    TxNotFound(String),

    #[error("definition '{0}' not found")]
    DefinitionNotFound(String),

    #[error("party '{0}' not assigned")]
    PartyNotAssigned(String),

    #[error("arg '{0}' not assigned")]
    ArgNotAssigned(String),

    #[error("invalid address")]
    InvalidAddress(#[from] pallas::ledger::addresses::Error),

    #[error("mapping error {0}")]
    MappingError(String),

    #[error("missing amount")]
    MissingAmount,

    #[error("missing asset name")]
    MissingAssetName,

    #[error("asset value too high")]
    AssetValueTooHigh,

    #[error("outputs too high")]
    OutputsTooHigh,

    #[error("error applying tx values")]
    ApplyError(#[from] tx3_lang::applying::Error),

    #[error("no AST analysis performed")]
    NoAstAnalysis,

    #[error("inputs not resolved")]
    InputsNotResolved,

    #[error("can't resolve symbol '{0:?}'")]
    CantResolveSymbol(tx3_lang::ast::Symbol),

    #[error("max optimize rounds reached")]
    MaxOptimizeRoundsReached,
}

pub type Network = pallas::ledger::primitives::NetworkId;

pub struct PParams {
    pub network: Network,
    pub min_fee_coefficient: u64,
    pub min_fee_constant: u64,
    pub coins_per_utxo_byte: u64,
    // TODO: cost models, execution prices
}

pub use compile::compile_tx;
pub use resolve::resolve_tx;
pub use resolve::Ledger;

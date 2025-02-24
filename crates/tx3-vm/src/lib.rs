#[cfg(feature = "cardano")]
pub mod cardano;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    #[error("missing address")]
    MissingAddress,

    #[error("asset value too high")]
    AssetValueTooHigh,

    #[error("outputs too high")]
    OutputsTooHigh,

    #[error("error coercing {0} into {1}")]
    CoerceError(String, String),

    #[error("error applying tx values")]
    ApplyError(#[from] tx3_lang::applying::Error),

    #[error("no AST analysis performed")]
    NoAstAnalysis,

    #[error("inputs not resolved")]
    InputsNotResolved,

    #[error("can't resolve symbol '{0:?}'")]
    CantResolveSymbol(tx3_lang::ast::Symbol),

    #[error("invalid asset expression '{0}'")]
    InvalidAssetExpression(String),

    #[error("invalid address expression '{0}'")]
    InvalidAddressExpression(String),

    #[error("ledger internal error: {0}")]
    LedgerInternalError(String),

    #[error("max optimize rounds reached")]
    MaxOptimizeRoundsReached,
}

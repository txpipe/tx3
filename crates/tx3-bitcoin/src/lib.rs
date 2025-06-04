mod compile;
mod resolve;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error coercing {0} into {1}")]
    CoerceError(String, String),

    #[error("missing amount")]
    MissingAmount,
}

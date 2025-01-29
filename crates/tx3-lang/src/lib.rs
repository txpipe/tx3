use pest_derive::Parser;

pub mod ast;
pub mod parser;

#[derive(Parser)]
#[grammar = "tx3.pest"]
pub struct Tx3Parser;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Invalid type: {0}")]
    InvalidType(String),
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    #[error("Invalid binary operator: {0}")]
    InvalidBinaryOperator(String),
    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(Rule),
}

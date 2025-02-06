use pest::Parser;
use pest_derive::Parser;

use crate::ast::{AstNode as _, Program};

#[derive(Parser)]
#[grammar = "tx3.pest"]
pub(crate) struct Tx3Grammar;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

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

pub fn parse_file(path: &str) -> Result<Program, Error> {
    let input = std::fs::read_to_string(path)?;
    let pairs = Tx3Grammar::parse(Rule::program, &input)?;
    Ok(Program::parse(pairs.into_iter().next().unwrap())?)
}

pub fn parse_string(input: &str) -> Result<Program, Error> {
    let pairs = Tx3Grammar::parse(Rule::program, input)?;
    Ok(Program::parse(pairs.into_iter().next().unwrap())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_file() {
        let _ = parse_file("tests/swap.tx3").unwrap();
    }

    #[test]
    fn smoke_test_parse_string() {
        let _ = parse_string("tx swap() {}").unwrap();
    }
}

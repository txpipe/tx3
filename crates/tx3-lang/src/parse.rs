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
    use assert_json_diff::assert_json_eq;
    use paste::paste;

    #[test]
    fn smoke_test_parse_file() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let _ = parse_file(&format!("{}/../..//examples/transfer.tx3", manifest_dir)).unwrap();
    }

    #[test]
    fn smoke_test_parse_string() {
        let _ = parse_string("tx swap() {}").unwrap();
    }

    fn make_snapshot_if_missing(example: &str, program: &Program) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = format!("{}/../../examples/{}.ast", manifest_dir, example);

        if !std::fs::exists(&path).unwrap() {
            let ast = serde_json::to_string_pretty(program).unwrap();
            std::fs::write(&path, ast).unwrap();
        }
    }

    fn test_parsing_example(example: &str) {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file = format!("{}/../../examples/{}.tx3", manifest_dir, example);
        let program = parse_file(&test_file).unwrap();

        make_snapshot_if_missing(example, &program);

        let ast_file = format!("{}/../../examples/{}.ast", manifest_dir, example);
        let ast = std::fs::read_to_string(ast_file).unwrap();

        let expected: Program = serde_json::from_str(&ast).unwrap();

        assert_json_eq!(program, expected);
    }

    #[macro_export]
    macro_rules! test_parsing {
        ($name:ident) => {
            paste! {
                #[test]
                fn [<test_example_ $name>]() {
                    test_parsing_example(stringify!($name));
                }
            }
        };
    }

    test_parsing!(lang_tour);

    test_parsing!(transfer);

    test_parsing!(swap);

    test_parsing!(asteria);

    // test_parsing!(vesting);

    // TODO
    // test_parsing!(faucet);
}

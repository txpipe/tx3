//! Parses the Tx3 language.
//!
//! This module takes a string and parses it into Tx3 AST.

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::ast::*;

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

pub trait AstNode: Sized {
    const RULE: Rule;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error>;
}

impl AstNode for Program {
    const RULE: Rule = Rule::program;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let inner = pair.into_inner();

        let mut program = Self {
            txs: Vec::new(),
            assets: Vec::new(),
            types: Vec::new(),
            parties: Vec::new(),
            policies: Vec::new(),
            scope: None,
        };

        for pair in inner {
            match pair.as_rule() {
                Rule::tx_def => program.txs.push(TxDef::parse(pair)?),
                Rule::asset_def => program.assets.push(AssetDef::parse(pair)?),
                Rule::record_def => program.types.push(TypeDef::parse(pair)?),
                Rule::variant_def => program.types.push(TypeDef::parse(pair)?),
                Rule::party_def => program.parties.push(PartyDef::parse(pair)?),
                Rule::policy_def => program.policies.push(PolicyDef::parse(pair)?),
                Rule::EOI => break,
                x => unreachable!("Unexpected rule in program: {:?}", x),
            }
        }

        Ok(program)
    }
}

impl AstNode for ParameterList {
    const RULE: Rule = Rule::parameter_list;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let inner = pair.into_inner();

        let mut parameters = Vec::new();

        for param in inner {
            let mut inner = param.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let r#type = Type::parse(inner.next().unwrap())?;

            parameters.push(ParamDef { name, r#type });
        }

        Ok(ParameterList { parameters })
    }
}

impl AstNode for TxDef {
    const RULE: Rule = Rule::tx_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let name = inner.next().unwrap().as_str().to_string();
        let parameters = ParameterList::parse(inner.next().unwrap())?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut burn = None;
        let mut mint = None;
        let mut adhoc = Vec::new();

        for item in inner {
            match item.as_rule() {
                Rule::input_block => inputs.push(InputBlock::parse(item)?),
                Rule::output_block => outputs.push(OutputBlock::parse(item)?),
                Rule::burn_block => burn = Some(BurnBlock::parse(item)?),
                Rule::mint_block => mint = Some(MintBlock::parse(item)?),
                Rule::chain_specific_block => adhoc.push(ChainSpecificBlock::parse(item)?),
                x => unreachable!("Unexpected rule in tx_def: {:?}", x),
            }
        }

        Ok(TxDef {
            name,
            parameters,
            inputs,
            outputs,
            burn,
            mint,
            adhoc,
            scope: None,
        })
    }
}

impl AstNode for Identifier {
    const RULE: Rule = Rule::identifier;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(Identifier {
            value: pair.as_str().to_string(),
            symbol: None,
        })
    }
}

impl AstNode for StringLiteral {
    const RULE: Rule = Rule::string;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(StringLiteral {
            value: pair.as_str()[1..pair.as_str().len() - 1].to_string(),
        })
    }
}

impl AstNode for HexStringLiteral {
    const RULE: Rule = Rule::hex_string;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(HexStringLiteral {
            value: pair.as_str()[2..].to_string(),
        })
    }
}

impl AstNode for PartyDef {
    const RULE: Rule = Rule::party_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(PartyDef { name: identifier })
    }
}

impl AstNode for InputBlockField {
    const RULE: Rule = Rule::input_block_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::input_block_from => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::From(DataExpr::parse(pair)?);
                Ok(x)
            }
            Rule::input_block_datum_is => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::DatumIs(Type::parse(pair)?);
                Ok(x)
            }
            Rule::input_block_min_amount => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::MinAmount(AssetExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::input_block_redeemer => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::Redeemer(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::input_block_ref => {
                let pair = pair.into_inner().next().unwrap();
                let x = InputBlockField::Ref(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            x => unreachable!("Unexpected rule in input_block: {:?}", x),
        }
    }
}

impl AstNode for InputBlock {
    const RULE: Rule = Rule::input_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let name = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(|x| InputBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(InputBlock {
            name,
            is_many: false,
            fields,
        })
    }
}

impl AstNode for OutputBlockField {
    const RULE: Rule = Rule::output_block_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::output_block_to => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::To(Box::new(DataExpr::parse(pair)?));
                Ok(x)
            }
            Rule::output_block_amount => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::Amount(AssetExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::output_block_datum => {
                let pair = pair.into_inner().next().unwrap();
                let x = OutputBlockField::Datum(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            x => unreachable!("Unexpected rule in output_block_field: {:?}", x),
        }
    }
}

impl AstNode for OutputBlock {
    const RULE: Rule = Rule::output_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let has_name = inner
            .peek()
            .map(|x| x.as_rule() == Rule::identifier)
            .unwrap_or_default();

        let name = has_name.then(|| inner.next().unwrap().as_str().to_string());

        let fields = inner
            .map(|x| OutputBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OutputBlock { name, fields })
    }
}

impl AstNode for MintBlockField {
    const RULE: Rule = Rule::mint_block_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::mint_block_amount => {
                let pair = pair.into_inner().next().unwrap();
                let x = MintBlockField::Amount(AssetExpr::parse(pair)?.into());
                Ok(x)
            }
            Rule::mint_block_redeemer => {
                let pair = pair.into_inner().next().unwrap();
                let x = MintBlockField::Redeemer(DataExpr::parse(pair)?.into());
                Ok(x)
            }
            x => unreachable!("Unexpected rule in output_block_field: {:?}", x),
        }
    }
}

impl AstNode for MintBlock {
    const RULE: Rule = Rule::mint_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let inner = pair.into_inner();

        let fields = inner
            .map(|x| MintBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MintBlock { fields })
    }
}

impl AstNode for BurnBlock {
    const RULE: Rule = Rule::burn_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let inner = pair.into_inner();

        let fields = inner
            .map(|x| MintBlockField::parse(x))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BurnBlock { fields })
    }
}

impl AstNode for RecordField {
    const RULE: Rule = Rule::record_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        let r#type = Type::parse(inner.next().unwrap())?;

        Ok(RecordField {
            name: identifier,
            r#type,
        })
    }
}

impl AstNode for PolicyDef {
    const RULE: Rule = Rule::policy_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let value = PolicyValue::parse(inner.next().unwrap())?;

        Ok(PolicyDef { name, value })
    }
}

impl AstNode for PolicyValue {
    const RULE: Rule = Rule::policy_value;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::policy_import => Ok(PolicyValue::Import(StringLiteral::parse(
                pair.into_inner().next().unwrap(),
            )?)),
            Rule::hex_string => Ok(PolicyValue::HexString(HexStringLiteral::parse(pair)?)),
            x => unreachable!("Unexpected rule in policy_value: {:?}", x),
        }
    }
}

impl AstNode for AssetConstructor {
    const RULE: Rule = Rule::asset_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let amount = DataExpr::parse(inner.next().unwrap())?;

        Ok(AssetConstructor {
            r#type,
            amount: Box::new(amount),
        })
    }
}

impl AssetExpr {
    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(AssetExpr::Identifier(Identifier::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(AssetExpr::Constructor(AssetConstructor::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(AssetExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::asset_constructor => AssetExpr::constructor_parse(pair),
            Rule::property_access => AssetExpr::property_access_parse(pair),
            Rule::identifier => AssetExpr::identifier_parse(pair),
            x => Err(Error::UnexpectedRule(x)),
        }
    }
}

impl AstNode for AssetExpr {
    const RULE: Rule = Rule::asset_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::term_parse(inner.next().unwrap())?;

        while let Some(term) = inner.next() {
            let operator = BinaryOperator::parse(term)?;
            let next_expr = Self::term_parse(inner.next().unwrap())?;

            final_expr = AssetExpr::BinaryOp(AssetBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }
}

impl AstNode for PropertyAccess {
    const RULE: Rule = Rule::property_access;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let object = Identifier::parse(inner.next().unwrap())?;

        let mut identifiers = Vec::new();
        identifiers.push(Identifier::parse(inner.next().unwrap())?);

        for identifier in inner {
            identifiers.push(Identifier::parse(identifier)?);
        }

        Ok(PropertyAccess {
            object,
            path: identifiers,
            scope: None,
        })
    }
}

impl AstNode for RecordConstructorField {
    const RULE: Rule = Rule::record_constructor_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;
        let value = DataExpr::parse(inner.next().unwrap())?;

        Ok(RecordConstructorField {
            name,
            value: Box::new(value),
        })
    }
}

impl AstNode for DatumConstructor {
    const RULE: Rule = Rule::datum_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let case = VariantCaseConstructor::parse(inner.next().unwrap())?;

        Ok(DatumConstructor {
            r#type,
            case,
            scope: None,
        })
    }
}

impl VariantCaseConstructor {
    fn implicit_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let inner = pair.into_inner();

        let mut fields = Vec::new();
        let mut spread = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::record_constructor_field => {
                    fields.push(RecordConstructorField::parse(pair)?);
                }
                Rule::spread_expression => {
                    spread = Some(DataExpr::parse(pair.into_inner().next().unwrap())?);
                }
                x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
            }
        }

        Ok(VariantCaseConstructor {
            name: Identifier::new("Default"),
            fields,
            spread: spread.map(|x| Box::new(x)),
            scope: None,
        })
    }

    fn explicit_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;

        let mut fields = Vec::new();
        let mut spread = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::record_constructor_field => {
                    fields.push(RecordConstructorField::parse(pair)?);
                }
                Rule::spread_expression => {
                    spread = Some(DataExpr::parse(pair.into_inner().next().unwrap())?);
                }
                x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
            }
        }

        Ok(VariantCaseConstructor {
            name,
            fields,
            spread: spread.map(|x| Box::new(x)),
            scope: None,
        })
    }
}

impl AstNode for VariantCaseConstructor {
    const RULE: Rule = Rule::variant_case_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::implicit_variant_case_constructor => Self::implicit_parse(pair),
            Rule::explicit_variant_case_constructor => Self::explicit_parse(pair),
            x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
        }
    }
}

impl DataExpr {
    fn number_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(DataExpr::Number(pair.as_str().parse().unwrap()))
    }

    fn bool_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(DataExpr::Bool(pair.as_str().parse().unwrap()))
    }

    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(DataExpr::Identifier(Identifier::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(DataExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        Ok(DataExpr::Constructor(DatumConstructor::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::number => DataExpr::number_parse(pair),
            Rule::string => Ok(DataExpr::String(StringLiteral::parse(pair)?)),
            Rule::bool => DataExpr::bool_parse(pair),
            Rule::hex_string => Ok(DataExpr::HexString(HexStringLiteral::parse(pair)?)),
            Rule::datum_constructor => DataExpr::constructor_parse(pair),
            Rule::unit => Ok(DataExpr::Unit),
            Rule::identifier => DataExpr::identifier_parse(pair),
            Rule::property_access => DataExpr::property_access_parse(pair),
            x => Err(Error::UnexpectedRule(x)),
        }
    }
}

impl AstNode for DataExpr {
    const RULE: Rule = Rule::data_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::term_parse(inner.next().unwrap())?;

        while let Some(term) = inner.next() {
            let operator = BinaryOperator::parse(term)?;
            let next_expr = Self::term_parse(inner.next().unwrap())?;

            final_expr = DataExpr::BinaryOp(DataBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }
}

impl AstNode for BinaryOperator {
    const RULE: Rule = Rule::binary_operator;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_str() {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            x => Err(Error::InvalidBinaryOperator(x.to_string())),
        }
    }
}

impl AstNode for Type {
    const RULE: Rule = Rule::r#type;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "Bytes" => Ok(Type::Bytes),
            t => Ok(Type::Custom(Identifier::new(t.to_string()))),
        }
    }
}

impl TypeDef {
    fn parse_variant_format(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let cases = inner
            .map(VariantCase::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDef {
            name: identifier,
            cases,
        })
    }

    fn parse_record_format(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(RecordField::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDef {
            name: identifier.clone(),
            cases: vec![VariantCase {
                name: "Default".to_string(),
                fields,
            }],
        })
    }
}

impl AstNode for TypeDef {
    const RULE: Rule = Rule::type_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::variant_def => Ok(Self::parse_variant_format(pair)?),
            Rule::record_def => Ok(Self::parse_record_format(pair)?),
            x => unreachable!("Unexpected rule in type_def: {:?}", x),
        }
    }
}

impl VariantCase {
    fn struct_case_parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let fields = inner
            .map(RecordField::parse)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: identifier,
            fields,
        })
    }

    fn unit_case_parse(pair: pest::iterators::Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(Self {
            name: identifier,
            fields: vec![],
        })
    }
}

impl AstNode for VariantCase {
    const RULE: Rule = Rule::variant_case;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let case = match pair.as_rule() {
            Rule::variant_case_struct => Self::struct_case_parse(pair),
            Rule::variant_case_tuple => todo!("parse variant case tuple"),
            Rule::variant_case_unit => Self::unit_case_parse(pair),
            x => unreachable!("Unexpected rule in datum_variant: {:?}", x),
        }?;

        Ok(case)
    }
}

impl AstNode for AssetDef {
    const RULE: Rule = Rule::asset_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();
        let policy = HexStringLiteral::parse(inner.next().unwrap())?;
        let asset_name = inner.next().unwrap().as_str().to_string();

        Ok(AssetDef {
            name: identifier,
            policy,
            asset_name,
        })
    }
}

impl AstNode for ChainSpecificBlock {
    const RULE: Rule = Rule::chain_specific_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, Error> {
        let mut inner = pair.into_inner();

        let block = inner.next().unwrap();

        match block.as_rule() {
            Rule::cardano_block => {
                let block = crate::cardano::CardanoBlock::parse(block)?;
                Ok(ChainSpecificBlock::Cardano(block))
            }
            x => unreachable!("Unexpected rule in chain_specific_block: {:?}", x),
        }
    }
}

/// Parses a Tx3 source file into a Program AST.
///
/// # Arguments
///
/// * `path` - Path to the Tx3 source file to parse
///
/// # Returns
///
/// * `Result<Program, Error>` - The parsed Program AST or an error
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file contents are not valid Tx3 syntax
/// - The AST construction fails
///
/// # Example
///
/// ```no_run
/// use tx3_lang::parsing::parse_file;
/// let program = parse_file("path/to/program.tx3").unwrap();
/// ```
pub fn parse_file(path: &str) -> Result<Program, Error> {
    let input = std::fs::read_to_string(path)?;
    let pairs = Tx3Grammar::parse(Rule::program, &input)?;
    Ok(Program::parse(pairs.into_iter().next().unwrap())?)
}

/// Parses a Tx3 source string into a Program AST.
///
/// # Arguments
///
/// * `input` - String containing Tx3 source code
///
/// # Returns
///
/// * `Result<Program, Error>` - The parsed Program AST or an error
///
/// # Errors
///
/// Returns an error if:
/// - The input string is not valid Tx3 syntax
/// - The AST construction fails
///
/// # Example
///
/// ```
/// use tx3_lang::parsing::parse_string;
/// let program = parse_string("tx swap() {}").unwrap();
/// ```
pub fn parse_string(input: &str) -> Result<Program, Error> {
    let pairs = Tx3Grammar::parse(Rule::program, input)?;
    Ok(Program::parse(pairs.into_iter().next().unwrap())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use paste::paste;
    use pest::Parser;

    #[test]
    fn smoke_test_parse_file() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let _ = parse_file(&format!("{}/../..//examples/transfer.tx3", manifest_dir)).unwrap();
    }

    #[test]
    fn smoke_test_parse_string() {
        let _ = parse_string("tx swap() {}").unwrap();
    }

    macro_rules! input_to_ast_check {
        ($ast:ty, $name:expr, $input:expr, $expected:expr) => {
            paste::paste! {
                #[test]
                fn [<test_parse_ $ast:snake _ $name>]() {
                    let pairs = super::Tx3Grammar::parse(<$ast>::RULE, $input).unwrap();
                    let single_match = pairs.into_iter().next().unwrap();
                    let result = <$ast>::parse(single_match).unwrap();

                    assert_eq!(result, $expected);
                }
            }
        };
    }

    input_to_ast_check!(BinaryOperator, "plus", "+", BinaryOperator::Add);

    input_to_ast_check!(BinaryOperator, "minus", "-", BinaryOperator::Subtract);

    input_to_ast_check!(
        TypeDef,
        "type_def_record",
        "type MyRecord {
            field1: Int,
            field2: Bytes,
        }",
        TypeDef {
            name: "MyRecord".to_string(),
            cases: vec![VariantCase {
                name: "Default".to_string(),
                fields: vec![
                    RecordField::new("field1", Type::Int),
                    RecordField::new("field2", Type::Bytes)
                ],
            }],
        }
    );

    input_to_ast_check!(
        TypeDef,
        "type_def_variant",
        "type MyVariant {
            Case1 {
                field1: Int,
                field2: Bytes,
            },
            Case2,
        }",
        TypeDef {
            name: "MyVariant".to_string(),
            cases: vec![
                VariantCase {
                    name: "Case1".to_string(),
                    fields: vec![
                        RecordField::new("field1", Type::Int),
                        RecordField::new("field2", Type::Bytes)
                    ],
                },
                VariantCase {
                    name: "Case2".to_string(),
                    fields: vec![],
                },
            ],
        }
    );

    input_to_ast_check!(
        StringLiteral,
        "literal_string",
        "\"Hello, world!\"",
        StringLiteral::new("Hello, world!".to_string())
    );

    input_to_ast_check!(
        HexStringLiteral,
        "hex_string",
        "0xAFAFAF",
        HexStringLiteral::new("AFAFAF".to_string())
    );

    input_to_ast_check!(
        StringLiteral,
        "literal_string_address",
        "\"addr1qx234567890abcdefghijklmnopqrstuvwxyz\"",
        StringLiteral::new("addr1qx234567890abcdefghijklmnopqrstuvwxyz".to_string())
    );

    input_to_ast_check!(
        DataExpr,
        "identifier",
        "my_party",
        DataExpr::Identifier(Identifier::new("my_party".to_string()))
    );

    input_to_ast_check!(DataExpr, "literal_bool_true", "true", DataExpr::Bool(true));

    input_to_ast_check!(
        DataExpr,
        "literal_bool_false",
        "false",
        DataExpr::Bool(false)
    );

    input_to_ast_check!(DataExpr, "unit_value", "())", DataExpr::Unit);

    input_to_ast_check!(
        PropertyAccess,
        "single_property",
        "subject.property",
        PropertyAccess::new("subject", &["property"])
    );

    input_to_ast_check!(
        PropertyAccess,
        "multiple_properties",
        "subject.property.subproperty",
        PropertyAccess::new("subject", &["property", "subproperty"])
    );

    input_to_ast_check!(
        PolicyDef,
        "policy_def_hex",
        "policy MyPolicy = 0xAFAFAF;",
        PolicyDef {
            name: "MyPolicy".to_string(),
            value: PolicyValue::HexString(HexStringLiteral::new("AFAFAF".to_string())),
        }
    );

    input_to_ast_check!(
        PolicyDef,
        "policy_def_import",
        "policy MyPolicy = import(\"aiken_code\");",
        PolicyDef {
            name: "MyPolicy".to_string(),
            value: PolicyValue::Import(StringLiteral::new("aiken_code".to_string())),
        }
    );

    input_to_ast_check!(
        AssetDef,
        "hex_hex",
        "asset MyToken = 0xef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe.0xef7a1ceb;",
        AssetDef {
            name: "MyToken".to_string(),
            policy: HexStringLiteral::new(
                "ef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe".to_string()
            ),
            asset_name: "0xef7a1ceb".to_string(),
        }
    );

    input_to_ast_check!(
        AssetDef,
        "hex_ascii",
        "asset MyToken = 0xef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe.MYTOKEN;",
        AssetDef {
            name: "MyToken".to_string(),
            policy: HexStringLiteral::new(
                "ef7a1cebb2dc7de884ddf82f8fcbc91fe9750dcd8c12ec7643a99bbe".to_string()
            ),
            asset_name: "MYTOKEN".to_string(),
        }
    );

    input_to_ast_check!(
        AssetConstructor,
        "type_and_literal",
        "MyToken(15)",
        AssetConstructor {
            r#type: Identifier::new("MyToken"),
            amount: Box::new(DataExpr::Number(15)),
        }
    );

    input_to_ast_check!(
        DataExpr,
        "addition",
        "5 + var1",
        DataExpr::BinaryOp(DataBinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(DataExpr::Number(5)),
            right: Box::new(DataExpr::Identifier(Identifier::new("var1"))),
        })
    );

    input_to_ast_check!(
        DatumConstructor,
        "datum_constructor_record",
        "MyRecord {
            field1: 10,
            field2: abc,
        }",
        DatumConstructor {
            r#type: Identifier::new("MyRecord"),
            case: VariantCaseConstructor {
                name: Identifier::new("Default"),
                fields: vec![
                    RecordConstructorField {
                        name: Identifier::new("field1"),
                        value: Box::new(DataExpr::Number(10)),
                    },
                    RecordConstructorField {
                        name: Identifier::new("field2"),
                        value: Box::new(DataExpr::Identifier(Identifier::new("abc"))),
                    },
                ],
                spread: None,
                scope: None,
            },
            scope: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "datum_constructor_variant",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
        }",
        DatumConstructor {
            r#type: Identifier::new("ShipCommand"),
            case: VariantCaseConstructor {
                name: Identifier::new("MoveShip"),
                fields: vec![
                    RecordConstructorField {
                        name: Identifier::new("delta_x"),
                        value: Box::new(DataExpr::Identifier(Identifier::new("delta_x"))),
                    },
                    RecordConstructorField {
                        name: Identifier::new("delta_y"),
                        value: Box::new(DataExpr::Identifier(Identifier::new("delta_y"))),
                    },
                ],
                spread: None,
                scope: None,
            },
            scope: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "datum_constructor_variant_with_spread",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
            ...abc
        }",
        DatumConstructor {
            r#type: Identifier::new("ShipCommand"),
            case: VariantCaseConstructor {
                name: Identifier::new("MoveShip"),
                fields: vec![
                    RecordConstructorField {
                        name: Identifier::new("delta_x"),
                        value: Box::new(DataExpr::Identifier(Identifier::new("delta_x"))),
                    },
                    RecordConstructorField {
                        name: Identifier::new("delta_y"),
                        value: Box::new(DataExpr::Identifier(Identifier::new("delta_y"))),
                    },
                ],
                spread: Some(Box::new(DataExpr::Identifier(Identifier::new(
                    "abc".to_string()
                )))),
                scope: None,
            },
            scope: None,
        }
    );

    input_to_ast_check!(
        OutputBlock,
        "output_block_anonymous",
        r#"output {
            to: my_party,
            amount: Ada(100),
        }"#,
        OutputBlock {
            name: None,
            fields: vec![
                OutputBlockField::To(Box::new(DataExpr::Identifier(Identifier::new(
                    "my_party".to_string(),
                )))),
                OutputBlockField::Amount(Box::new(AssetExpr::Constructor(AssetConstructor {
                    r#type: Identifier::new("Ada"),
                    amount: Box::new(DataExpr::Number(100)),
                }))),
            ],
        }
    );

    input_to_ast_check!(
        ChainSpecificBlock,
        "chain_specific_block_cardano",
        "cardano::vote_delegation_certificate {
            drep: 0x1234567890,
            stake: 0x1234567890,
        }",
        ChainSpecificBlock::Cardano(crate::cardano::CardanoBlock::VoteDelegationCertificate(
            crate::cardano::VoteDelegationCertificate {
                drep: DataExpr::HexString(HexStringLiteral::new("1234567890".to_string())),
                stake: DataExpr::HexString(HexStringLiteral::new("1234567890".to_string())),
            },
        ))
    );

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

    test_parsing!(vesting);

    test_parsing!(faucet);
}

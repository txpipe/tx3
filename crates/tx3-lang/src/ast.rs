use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::parse::{Error as ParseError, Rule};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identifier(pub String);

impl AstNode for Identifier {
    const RULE: Rule = Rule::identifier;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(Identifier(pair.as_str().to_string()))
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub trait AstNode: Sized {
    const RULE: Rule;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub txs: Vec<TxDef>,
    pub datums: Vec<DatumDef>,
    pub assets: Vec<AssetDef>,
    pub parties: Vec<PartyDef>,
}

impl AstNode for Program {
    const RULE: Rule = Rule::program;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut program = Self {
            txs: Vec::new(),
            assets: Vec::new(),
            datums: Vec::new(),
            parties: Vec::new(),
        };

        for pair in inner {
            match pair.as_rule() {
                Rule::tx_def => program.txs.push(TxDef::parse(pair)?),
                Rule::asset_def => program.assets.push(AssetDef::parse(pair)?),
                Rule::datum_def => program.datums.push(DatumDef::parse(pair)?),
                Rule::party_def => program.parties.push(PartyDef::parse(pair)?),
                Rule::EOI => break,
                x => unreachable!("Unexpected rule in program: {:?}", x),
            }
        }

        Ok(program)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterList {
    pub parameters: Vec<Parameter>,
}

impl AstNode for ParameterList {
    const RULE: Rule = Rule::parameter_list;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut parameters = Vec::new();

        for param in inner {
            let mut inner = param.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let typ = Type::parse(inner.next().unwrap())?;

            parameters.push(Parameter { name, typ });
        }

        Ok(ParameterList { parameters })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxDef {
    pub name: Identifier,
    pub parameters: ParameterList,
    pub inputs: Vec<InputBlock>,
    pub outputs: Vec<OutputBlock>,
    pub burns: Option<BurnBlock>,
    pub mints: Option<MintBlock>,
}

impl AstNode for TxDef {
    const RULE: Rule = Rule::tx_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;
        let parameters = ParameterList::parse(inner.next().unwrap())?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut burns = None;
        let mut mints = None;

        for item in inner {
            match item.as_rule() {
                Rule::input_block => inputs.push(InputBlock::parse(item)?),
                Rule::output_block => outputs.push(OutputBlock::parse(item)?),
                Rule::burn_block => burns = Some(BurnBlock::parse(item)?),
                Rule::mint_block => mints = Some(MintBlock::parse(item)?),
                x => unreachable!("Unexpected rule in tx_def: {:?}", x),
            }
        }

        Ok(TxDef {
            name,
            parameters,
            inputs,
            outputs,
            burns,
            mints,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputBlock {
    pub name: Identifier,
    pub is_many: bool,
    pub from: Option<Identifier>,
    pub datum_is: Option<Type>,
    pub min_amount: Option<Box<AssetExpr>>,
    pub redeemer: Option<Box<DataExpr>>,
}

impl AstNode for InputBlock {
    const RULE: Rule = Rule::input_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;

        let mut input_block = InputBlock {
            name,
            is_many: false,
            from: None,
            datum_is: None,
            min_amount: None,
            redeemer: None,
        };

        for field_option in inner {
            let field = field_option.into_inner().next().unwrap();

            match field.as_rule() {
                Rule::input_block_from => {
                    input_block.from = Some(Identifier::parse(field.into_inner().next().unwrap())?);
                }
                Rule::input_block_datum_is => {
                    input_block.datum_is = Some(Type::parse(field.into_inner().next().unwrap())?);
                }
                Rule::input_block_min_amount => {
                    input_block.min_amount =
                        Some(AssetExpr::parse(field.into_inner().next().unwrap())?.into());
                }
                Rule::input_block_redeemer => {
                    input_block.redeemer =
                        Some(DataExpr::parse(field.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in input_block: {:?}", x),
            }
        }

        Ok(input_block)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputBlock {
    pub to: Identifier,
    pub amount: Option<Box<AssetExpr>>,
    pub datum: Option<Box<DataExpr>>,
}

impl AstNode for OutputBlock {
    const RULE: Rule = Rule::output_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut output_block = OutputBlock {
            to: Identifier(String::new()),
            amount: None,
            datum: None,
        };

        for field_option in inner {
            let field = field_option.into_inner().next().unwrap();

            match field.as_rule() {
                Rule::output_block_to => {
                    output_block.to = Identifier::parse(field.into_inner().next().unwrap())?
                }
                Rule::output_block_amount => {
                    output_block.amount =
                        Some(AssetExpr::parse(field.into_inner().next().unwrap())?.into());
                }
                Rule::output_block_datum => {
                    output_block.datum =
                        Some(DataExpr::parse(field.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in output_block: {:?}", x),
            }
        }

        Ok(output_block)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MintBlock {
    pub amount: Box<AssetExpr>,
}

impl AstNode for MintBlock {
    const RULE: Rule = Rule::mint_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut amount = None;

        for item in inner {
            match item.as_rule() {
                Rule::mint_block_amount => {
                    amount = Some(AssetExpr::parse(item.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in mint_block: {:?}", x),
            }
        }

        Ok(MintBlock {
            amount: amount.unwrap(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BurnBlock {
    pub amount: Box<AssetExpr>,
}

impl AstNode for BurnBlock {
    const RULE: Rule = Rule::burn_block;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let inner = pair.into_inner();

        let mut amount = None;

        for item in inner {
            match item.as_rule() {
                Rule::burn_block_amount => {
                    amount = Some(AssetExpr::parse(item.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in burn_block: {:?}", x),
            }
        }

        Ok(BurnBlock {
            amount: amount.unwrap(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumField {
    pub name: String,
    pub typ: Type,
}

impl AstNode for DatumField {
    const RULE: Rule = Rule::datum_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        let typ = Type::parse(inner.next().unwrap())?;

        Ok(DatumField {
            name: identifier,
            typ,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyDef {
    pub name: String,
}

impl AstNode for PartyDef {
    const RULE: Rule = Rule::party_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(PartyDef { name: identifier })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyField {
    pub name: String,
    pub party_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetConstructor {
    pub r#type: Identifier,
    pub amount: Box<DataExpr>,
    pub name: Option<Box<DataExpr>>,
}

impl AstNode for AssetConstructor {
    const RULE: Rule = Rule::asset_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let amount = DataExpr::parse(inner.next().unwrap())?;
        let name = inner.next().map(|x| DataExpr::parse(x)).transpose()?;

        Ok(AssetConstructor {
            r#type,
            amount: Box::new(amount),
            name: name.map(|x| Box::new(x)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetBinaryOp {
    pub left: Box<AssetExpr>,
    pub operator: BinaryOperator,
    pub right: Box<AssetExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetExpr {
    Constructor(AssetConstructor),
    BinaryOp(AssetBinaryOp),
    PropertyAccess(PropertyAccess),
    Identifier(Identifier),
}

impl AssetExpr {
    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::Identifier(Identifier::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::Constructor(AssetConstructor::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(AssetExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::asset_constructor => AssetExpr::constructor_parse(pair),
            Rule::property_access => AssetExpr::property_access_parse(pair),
            Rule::identifier => AssetExpr::identifier_parse(pair),
            x => Err(ParseError::UnexpectedRule(x)),
        }
    }
}

impl AstNode for AssetExpr {
    const RULE: Rule = Rule::asset_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyAccess {
    pub object: Identifier,
    pub path: Vec<Identifier>,
}

impl AstNode for PropertyAccess {
    const RULE: Rule = Rule::property_access;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
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
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumConstructorField {
    pub name: Identifier,
    pub value: Box<DataExpr>,
}

impl AstNode for DatumConstructorField {
    const RULE: Rule = Rule::datum_constructor_field;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let name = Identifier::parse(inner.next().unwrap())?;
        let value = DataExpr::parse(inner.next().unwrap())?;

        Ok(DatumConstructorField {
            name,
            value: Box::new(value),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumConstructor {
    pub r#type: Identifier,
    pub variant: Identifier,
    pub fields: Vec<DatumConstructorField>,
    pub spread: Option<Box<DataExpr>>,
}

impl AstNode for DatumConstructor {
    const RULE: Rule = Rule::datum_constructor;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Identifier::parse(inner.next().unwrap())?;
        let variant = Identifier::parse(inner.next().unwrap())?;

        let mut fields = Vec::new();
        let mut spread = None;

        for pair in inner {
            match pair.as_rule() {
                Rule::datum_constructor_field => {
                    fields.push(DatumConstructorField::parse(pair)?);
                }
                Rule::spread_expression => {
                    spread = Some(DataExpr::parse(pair.into_inner().next().unwrap())?);
                }
                x => unreachable!("Unexpected rule in datum_constructor: {:?}", x),
            }
        }

        Ok(DatumConstructor {
            r#type,
            variant,
            fields,
            spread: spread.map(|x| Box::new(x)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataBinaryOp {
    pub left: Box<DataExpr>,
    pub operator: BinaryOperator,
    pub right: Box<DataExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataExpr {
    None,
    Number(i64),
    Bool(bool),
    String(String),
    HexString(String),
    Constructor(DatumConstructor),
    Identifier(Identifier),
    PropertyAccess(PropertyAccess),
    BinaryOp(DataBinaryOp),
}

impl DataExpr {
    fn string_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::String(
            pair.as_str()[1..pair.as_str().len() - 1].to_string(),
        ))
    }

    fn number_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Number(pair.as_str().parse().unwrap()))
    }

    fn bool_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Bool(pair.as_str().parse().unwrap()))
    }

    fn hex_string_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::HexString(pair.as_str().to_string()))
    }

    fn identifier_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Identifier(Identifier::parse(pair)?))
    }

    fn property_access_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::PropertyAccess(PropertyAccess::parse(pair)?))
    }

    fn constructor_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        Ok(DataExpr::Constructor(DatumConstructor::parse(pair)?))
    }

    fn term_parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_rule() {
            Rule::number => DataExpr::number_parse(pair),
            Rule::string => DataExpr::string_parse(pair),
            Rule::bool => DataExpr::bool_parse(pair),
            Rule::hex_string => DataExpr::hex_string_parse(pair),
            Rule::datum_constructor => DataExpr::constructor_parse(pair),
            Rule::identifier => DataExpr::identifier_parse(pair),
            Rule::property_access => DataExpr::property_access_parse(pair),
            x => Err(ParseError::UnexpectedRule(x)),
        }
    }
}

impl AstNode for DataExpr {
    const RULE: Rule = Rule::data_expr;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

impl AstNode for BinaryOperator {
    const RULE: Rule = Rule::binary_operator;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_str() {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            x => Err(ParseError::InvalidBinaryOperator(x.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Bytes,
    Custom(String),
}

impl AstNode for Type {
    const RULE: Rule = Rule::r#type;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        match pair.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "Bytes" => Ok(Type::Bytes),
            t => Ok(Type::Custom(t.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumDef {
    pub name: String,
    pub variants: Vec<DatumVariant>,
}

impl DatumDef {
    fn struct_variant_parse(pair: pest::iterators::Pair<Rule>) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let mut fields = Vec::new();

        for field in inner {
            fields.push(DatumField::parse(field)?);
        }

        Ok(DatumVariant {
            name: identifier,
            fields,
        })
    }

    fn unit_variant_parse(pair: pest::iterators::Pair<Rule>) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(DatumVariant {
            name: identifier,
            fields: vec![],
        })
    }

    fn datum_variant_parse(pair: pest::iterators::Pair<Rule>) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let flavor = inner.next().unwrap();

        match flavor.as_rule() {
            Rule::datum_variant_struct => Self::struct_variant_parse(flavor),
            Rule::datum_variant_tuple => todo!("parse datum variant tuple"),
            Rule::datum_variant_unit => Self::unit_variant_parse(flavor),
            x => unreachable!("Unexpected rule in datum_variant: {:?}", x),
        }
    }
}

impl AstNode for DatumDef {
    const RULE: Rule = Rule::datum_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let mut variants = Vec::new();

        for variant in inner {
            variants.push(Self::datum_variant_parse(variant)?);
        }

        Ok(DatumDef {
            name: identifier,
            variants,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumVariant {
    pub name: String,
    pub fields: Vec<DatumField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetDef {
    pub name: String,
    pub policy: String,
    pub asset_name: Option<String>,
}

impl AstNode for AssetDef {
    const RULE: Rule = Rule::asset_def;

    fn parse(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();
        let policy = inner.next().unwrap().as_str().to_string();
        let asset_name = inner.next().map(|x| x.as_str().to_string());

        Ok(AssetDef {
            name: identifier,
            policy,
            asset_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pest::Parser;

    macro_rules! input_to_ast_check {
        ($ast:ty, $name:expr, $input:expr, $expected:expr) => {
            paste::paste! {
                #[test]
                fn [<test_parse_ $ast:snake _ $name>]() {
                    let pairs = crate::parse::Tx3Grammar::parse(<$ast>::RULE, $input).unwrap();
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
        DataExpr,
        "literal_string",
        "\"Hello, world!\"",
        DataExpr::String("Hello, world!".to_string())
    );

    input_to_ast_check!(DataExpr, "literal_bool_true", "true", DataExpr::Bool(true));

    input_to_ast_check!(
        DataExpr,
        "literal_bool_false",
        "false",
        DataExpr::Bool(false)
    );

    input_to_ast_check!(
        PropertyAccess,
        "single_property",
        "subject.property",
        PropertyAccess {
            object: Identifier("subject".to_string()),
            path: vec![Identifier("property".to_string())],
        }
    );

    input_to_ast_check!(
        PropertyAccess,
        "multiple_properties",
        "subject.property.subproperty",
        PropertyAccess {
            object: Identifier("subject".to_string()),
            path: vec![
                Identifier("property".to_string()),
                Identifier("subproperty".to_string())
            ],
        }
    );

    input_to_ast_check!(
        AssetConstructor,
        "type_and_literal",
        "MyToken(15)",
        AssetConstructor {
            r#type: Identifier("MyToken".to_string()),
            amount: Box::new(DataExpr::Number(15)),
            name: None,
        }
    );

    input_to_ast_check!(
        AssetConstructor,
        "type_and_literal_with_name",
        "MyClass(15, \"TokenName\")",
        AssetConstructor {
            r#type: Identifier("MyClass".to_string()),
            amount: Box::new(DataExpr::Number(15)),
            name: Some(Box::new(DataExpr::String("TokenName".to_string()))),
        }
    );

    input_to_ast_check!(
        DataExpr,
        "addition",
        "5 + var1",
        DataExpr::BinaryOp(DataBinaryOp {
            operator: BinaryOperator::Add,
            left: Box::new(DataExpr::Number(5)),
            right: Box::new(DataExpr::Identifier(Identifier("var1".to_string()))),
        })
    );

    input_to_ast_check!(
        DatumConstructor,
        "struct_variant",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
        }",
        DatumConstructor {
            r#type: Identifier("ShipCommand".to_string()),
            variant: Identifier("MoveShip".to_string()),
            fields: vec![
                DatumConstructorField {
                    name: Identifier("delta_x".to_string()),
                    value: Box::new(DataExpr::Identifier(Identifier("delta_x".to_string()))),
                },
                DatumConstructorField {
                    name: Identifier("delta_y".to_string()),
                    value: Box::new(DataExpr::Identifier(Identifier("delta_y".to_string()))),
                },
            ],
            spread: None,
        }
    );

    input_to_ast_check!(
        DatumConstructor,
        "struct_variant_with_spread",
        "ShipCommand::MoveShip {
            delta_x: delta_x,
            delta_y: delta_y,
            ...abc
        }",
        DatumConstructor {
            r#type: Identifier("ShipCommand".to_string()),
            variant: Identifier("MoveShip".to_string()),
            fields: vec![
                DatumConstructorField {
                    name: Identifier("delta_x".to_string()),
                    value: Box::new(DataExpr::Identifier(Identifier("delta_x".to_string()))),
                },
                DatumConstructorField {
                    name: Identifier("delta_y".to_string()),
                    value: Box::new(DataExpr::Identifier(Identifier("delta_y".to_string()))),
                },
            ],
            spread: Some(Box::new(DataExpr::Identifier(Identifier(
                "abc".to_string()
            )))),
        }
    );
}

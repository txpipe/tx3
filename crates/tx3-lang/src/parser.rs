use crate::ast::{
    AssetBinaryOp, AssetConstructor, AssetDef, AssetExpr, BinaryOperator, BurnBlock, DataBinaryOp,
    DataExpr, DatumConstructor, DatumDef, DatumField, DatumVariant, Identifier, InputBlock,
    MintBlock, OutputBlock, Parameter, PartyDef, Program, PropertyAccess, TxDef, Type,
};
use pest::Parser;
use pest_derive::Parser;

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
}

impl Tx3Parser {
    pub fn parse_program(input: &str) -> Result<Program, ParseError> {
        let pairs = Self::parse(Rule::program, input)?;

        let mut program = Program {
            txs: Vec::new(),
            assets: Vec::new(),
            datums: Vec::new(),
            parties: Vec::new(),
        };

        for pair in pairs.into_iter() {
            match pair.as_rule() {
                Rule::program => {
                    for item in pair.into_inner() {
                        match item.as_rule() {
                            Rule::tx_def => program.txs.push(Self::parse_tx_def(item)?),
                            Rule::asset_def => program.assets.push(Self::parse_asset_def(item)?),
                            Rule::datum_def => program.datums.push(Self::parse_datum_def(item)?),
                            Rule::party_def => program.parties.push(Self::parse_party_def(item)?),
                            Rule::EOI => break,
                            x => unreachable!("Unexpected rule in program: {:?}", x),
                        }
                    }
                }
                Rule::EOI => break,
                x => unreachable!("Unexpected rule: {:?}", x),
            }
        }

        Ok(program)
    }

    fn parse_asset_def(pair: pest::iterators::Pair<Rule>) -> Result<AssetDef, ParseError> {
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

    fn parse_datum_def(pair: pest::iterators::Pair<Rule>) -> Result<DatumDef, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let mut variants = Vec::new();

        for variant in inner {
            variants.push(Self::parse_datum_variant(variant)?);
        }

        Ok(DatumDef {
            name: identifier,
            variants,
        })
    }

    fn parse_datum_variant(pair: pest::iterators::Pair<Rule>) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let flavor = inner.next().unwrap();

        match flavor.as_rule() {
            Rule::datum_variant_struct => Self::parse_datum_variant_struct(flavor),
            Rule::datum_variant_tuple => todo!("parse datum variant tuple"),
            Rule::datum_variant_unit => Self::parse_datum_variant_unit(flavor),
            x => unreachable!("Unexpected rule in datum_variant: {:?}", x),
        }
    }

    fn parse_datum_variant_struct(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let mut fields = Vec::new();

        for field in inner {
            fields.push(Self::parse_datum_field(field)?);
        }

        Ok(DatumVariant {
            name: identifier,
            fields,
        })
    }

    fn parse_datum_variant_unit(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<DatumVariant, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(DatumVariant {
            name: identifier,
            fields: vec![],
        })
    }

    fn parse_datum_field(pair: pest::iterators::Pair<Rule>) -> Result<DatumField, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        let typ = Self::parse_type(inner.next().unwrap())?;

        Ok(DatumField {
            name: identifier,
            typ,
        })
    }

    fn parse_party_def(pair: pest::iterators::Pair<Rule>) -> Result<PartyDef, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();

        Ok(PartyDef { name: identifier })
    }

    fn parse_type(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        match pair.as_str() {
            "Int" => Ok(Type::Int),
            "Bytes" => Ok(Type::Bytes),
            t => Ok(Type::Custom(t.to_string())),
        }
    }

    fn parse_parameters(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Parameter>, ParseError> {
        let mut parameters = Vec::new();

        for param in pair.into_inner() {
            let mut inner = param.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let typ = Self::parse_type(inner.next().unwrap())?;

            parameters.push(Parameter { name, typ });
        }

        Ok(parameters)
    }

    fn parse_binary_operator(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<BinaryOperator, ParseError> {
        match pair.as_str() {
            "+" => Ok(BinaryOperator::Add),
            "-" => Ok(BinaryOperator::Subtract),
            x => Err(ParseError::InvalidBinaryOperator(x.to_string())),
        }
    }

    fn parse_identifier(pair: pest::iterators::Pair<Rule>) -> Result<Identifier, ParseError> {
        Ok(pair.as_str().to_string())
    }

    fn parse_property_access(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<PropertyAccess, ParseError> {
        let mut inner = pair.into_inner();

        let object = Self::parse_identifier(inner.next().unwrap())?;

        let mut identifiers = Vec::new();
        identifiers.push(Self::parse_identifier(inner.next().unwrap())?);

        for identifier in inner {
            identifiers.push(Self::parse_identifier(identifier)?);
        }

        Ok(PropertyAccess {
            object,
            path: identifiers,
        })
    }

    fn parse_datum_constructor(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<DatumConstructor, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Self::parse_identifier(inner.next().unwrap())?;
        let variant = Self::parse_identifier(inner.next().unwrap())?;

        let mut fields = Vec::new();

        for field in inner.next().unwrap().into_inner() {
            let mut inner = field.into_inner();

            let name = Self::parse_identifier(inner.next().unwrap())?;
            let value = Self::parse_data_expr(inner.next().unwrap())?;

            fields.push((name, Box::new(value)));
        }

        let spread = inner.next().map(|x| Self::parse_data_expr(x)).transpose()?;

        Ok(DatumConstructor {
            r#type,
            variant,
            fields,
            spread: spread.map(|x| Box::new(x)),
        })
    }

    fn parse_data_term(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr, ParseError> {
        match pair.as_rule() {
            Rule::number => Ok(DataExpr::Number(pair.as_str().parse().unwrap())),
            Rule::string => Ok(DataExpr::String(pair.as_str().to_string())),
            Rule::hex_string => Ok(DataExpr::HexString(pair.as_str().to_string())),
            Rule::datum_constructor => {
                Ok(DataExpr::Constructor(Self::parse_datum_constructor(pair)?))
            }
            Rule::identifier => Ok(DataExpr::Identifier(Self::parse_identifier(pair)?)),
            Rule::property_access => {
                Ok(DataExpr::PropertyAccess(Self::parse_property_access(pair)?))
            }
            x => unreachable!("Unexpected rule in data_term: {:?}", x),
        }
    }

    fn parse_data_expr(pair: pest::iterators::Pair<Rule>) -> Result<DataExpr, ParseError> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::parse_data_term(inner.next().unwrap())?;

        for term in inner {
            let mut inner = term.into_inner();

            let operator = Self::parse_binary_operator(inner.next().unwrap())?;
            let next_expr = Self::parse_data_term(inner.next().unwrap())?;

            final_expr = DataExpr::BinaryOp(DataBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }

    fn parse_asset_constructor(
        pair: pest::iterators::Pair<Rule>,
    ) -> Result<AssetConstructor, ParseError> {
        let mut inner = pair.into_inner();

        let r#type = Self::parse_identifier(inner.next().unwrap())?;

        inner.next().unwrap();
        let amount = DataExpr::Number(42); // Self::parse_data_expr(inner.next().unwrap())?;
        let name = inner.next().map(|x| Self::parse_data_expr(x)).transpose()?;

        Ok(AssetConstructor {
            r#type,
            amount: Box::new(amount),
            name: name.map(|x| Box::new(x)),
        })
    }

    fn parse_asset_term(pair: pest::iterators::Pair<Rule>) -> Result<AssetExpr, ParseError> {
        match pair.as_rule() {
            Rule::asset_constructor => {
                Ok(AssetExpr::Constructor(Self::parse_asset_constructor(pair)?))
            }
            Rule::identifier => Ok(AssetExpr::Identifier(Self::parse_identifier(pair)?)),
            Rule::property_access => Ok(AssetExpr::PropertyAccess(Self::parse_property_access(
                pair,
            )?)),
            x => unreachable!("Unexpected rule in asset_term: {:?}", x),
        }
    }

    fn parse_asset_expr(pair: pest::iterators::Pair<Rule>) -> Result<AssetExpr, ParseError> {
        let mut inner = pair.into_inner();

        let mut final_expr = Self::parse_asset_term(inner.next().unwrap())?;

        for term in inner {
            let operator = Self::parse_binary_operator(inner.next().unwrap())?;
            let next_expr = Self::parse_asset_term(inner.next().unwrap())?;

            final_expr = AssetExpr::BinaryOp(AssetBinaryOp {
                operator,
                left: Box::new(final_expr),
                right: Box::new(next_expr),
            });
        }

        Ok(final_expr)
    }

    fn parse_input_block(pair: pest::iterators::Pair<Rule>) -> Result<InputBlock, ParseError> {
        let mut inner = pair.into_inner();

        let identifier = inner.next().unwrap().as_str().to_string();

        let mut input_block = InputBlock {
            name: identifier,
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
                    input_block.from =
                        Some(field.into_inner().next().unwrap().as_str().to_string());
                }
                Rule::input_block_datum_is => {
                    input_block.datum_is =
                        Some(Self::parse_type(field.into_inner().next().unwrap())?);
                }
                Rule::input_block_min_amount => {
                    input_block.min_amount =
                        Some(Self::parse_asset_expr(field.into_inner().next().unwrap())?.into());
                }
                Rule::input_block_redeemer => {
                    input_block.redeemer =
                        Some(Self::parse_data_expr(field.into_inner().next().unwrap())?.into());
                }
                x => unreachable!("Unexpected rule in input_block: {:?}", x),
            }
        }

        Ok(input_block)
    }

    fn parse_output_block(pair: pest::iterators::Pair<Rule>) -> Result<OutputBlock, ParseError> {
        let inner = pair.into_inner();

        let mut output_block = OutputBlock {
            to: String::new(),
            amount: None,
            datum: None,
        };

        for field_option in inner {
            let field = field_option.into_inner().next().unwrap();

            match field.as_rule() {
                Rule::output_block_to => output_block.to = field.as_str().to_string(),
                Rule::output_block_amount => {
                    output_block.amount = Some(Self::parse_asset_expr(field)?.into());
                }
                Rule::output_block_datum => {
                    output_block.datum = Some(Self::parse_data_expr(field)?.into());
                }
                x => unreachable!("Unexpected rule in output_block: {:?}", x),
            }
        }

        Ok(output_block)
    }

    fn parse_burn_block(pair: pest::iterators::Pair<Rule>) -> Result<BurnBlock, ParseError> {
        let inner = pair.into_inner();

        let mut amount = None;

        for item in inner {
            match item.as_rule() {
                Rule::burn_block_amount => {
                    amount = Some(Self::parse_asset_expr(item)?.into());
                }
                x => unreachable!("Unexpected rule in burn_block: {:?}", x),
            }
        }

        Ok(BurnBlock {
            amount: amount.unwrap(),
        })
    }

    fn parse_mint_block(pair: pest::iterators::Pair<Rule>) -> Result<MintBlock, ParseError> {
        todo!()
    }

    fn parse_tx_def(pair: pest::iterators::Pair<Rule>) -> Result<TxDef, ParseError> {
        let mut inner = pair.into_inner();
        let identifier = inner.next().unwrap().as_str().to_string();
        let parameters = Self::parse_parameters(inner.next().unwrap())?;

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut burns = None;
        let mut mints = None;

        for item in inner {
            match item.as_rule() {
                Rule::input_block => inputs.push(Self::parse_input_block(item)?),
                Rule::output_block => outputs.push(Self::parse_output_block(item)?),
                Rule::burn_block => burns = Some(Self::parse_burn_block(item)?),
                Rule::mint_block => mints = Some(Self::parse_mint_block(item)?),
                x => unreachable!("Unexpected rule in tx_def: {:?}", x),
            }
        }

        Ok(TxDef {
            name: identifier,
            parameters,
            inputs,
            outputs,
            burns,
            mints,
        })
    }
}

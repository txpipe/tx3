use crate::ast::{
    BinaryOperator, DatumDef, DatumField, Expr, InputData, OutputData, Parameter, PartyDef,
    PartyField, TemplateData, TxComponent, Type,
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
    // Add more error variants as needed
}

impl Tx3Parser {
    pub fn parse_program(input: &str) -> Result<Vec<Expr>, ParseError> {
        let pairs = Self::parse(Rule::program, input)?;
        let mut definitions = Vec::new();

        for pair in pairs.into_iter() {
            match pair.as_rule() {
                Rule::program => {
                    for item in pair.into_inner() {
                        match item.as_rule() {
                            Rule::datum_def => definitions.push(Self::parse_datum_def(item)?),
                            Rule::party_def => definitions.push(Self::parse_party_def(item)?),
                            Rule::tx_template => definitions.push(Self::parse_template(item)?),
                            Rule::EOI => break,
                            x => unreachable!("Unexpected rule in program: {:?}", x),
                        }
                    }
                }
                Rule::EOI => break,
                x => unreachable!("Unexpected rule: {:?}", x),
            }
        }

        Ok(definitions)
    }

    fn parse_datum_def(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();

        let mut fields = Vec::new();

        for field in inner {
            let mut field_inner = field.into_inner();
            let field_name = field_inner.next().unwrap().as_str().to_string();
            let field_type = Self::parse_type(field_inner.next().unwrap())?;
            fields.push(DatumField {
                name: field_name,
                typ: field_type,
            });
        }

        Ok(Expr::Datum(DatumDef { name, fields }))
    }

    fn parse_party_def(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let mut fields = Vec::new();

        if let Some(field_list) = inner.next() {
            for field in field_list.into_inner() {
                let mut field_inner = field.into_inner();
                let field_name = field_inner.next().unwrap().as_str().to_string();
                let party_type = field_inner.next().unwrap().as_str().to_string();
                fields.push(PartyField {
                    name: field_name,
                    party_type,
                });
            }
        }

        Ok(Expr::Party(PartyDef { name, fields }))
    }

    fn parse_template(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();

        let parameters = if let Some(param_list) = inner.next() {
            if param_list.as_rule() == Rule::parameter_list {
                Self::parse_parameters(param_list)?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let mut body = Vec::new();
        for component in inner {
            match component.as_rule() {
                Rule::input => body.push(TxComponent::Input(Self::parse_input(component)?)),
                Rule::output => body.push(TxComponent::Output(Self::parse_output(component)?)),
                x => unreachable!("Unexpected rule in template body: {:?}", x),
            }
        }

        Ok(Expr::Template(TemplateData {
            name,
            parameters,
            body,
        }))
    }

    fn parse_input(pair: pest::iterators::Pair<Rule>) -> Result<InputData, ParseError> {
        let mut inner = pair.into_inner();
        let is_many = inner.next().unwrap().as_str().contains("*");
        let name = inner.next().unwrap().as_str().to_string();

        let mut input = InputData {
            name,
            is_many,
            from: None,
            datum_is: None,
            min_amount: None,
            redeemer: None,
        };

        while let Some(field) = inner.next() {
            match field.as_rule() {
                Rule::from_field => {
                    input.from = Some(field.into_inner().next().unwrap().as_str().to_string());
                }
                Rule::datum_is_field => {
                    input.datum_is = Some(field.into_inner().next().unwrap().as_str().to_string());
                }
                Rule::min_amount_field => {
                    let expr = field.into_inner().next().unwrap();
                    input.min_amount = Some(Box::new(Self::parse_expr(expr)?));
                }
                Rule::redeemer_field => {
                    let expr = field.into_inner().next().unwrap();
                    input.redeemer = Some(Box::new(Self::parse_expr(expr)?));
                }
                x => unreachable!("Unexpected rule in input: {:?}", x),
            }
        }

        Ok(input)
    }

    fn parse_output(pair: pest::iterators::Pair<Rule>) -> Result<OutputData, ParseError> {
        let mut inner = pair.into_inner();

        let mut output = OutputData {
            to: String::new(),
            amount: None,
            datum: None,
        };

        while let Some(field) = inner.next() {
            match field.as_rule() {
                Rule::to_field => {
                    output.to = field.into_inner().next().unwrap().as_str().to_string();
                }
                Rule::amount_field => {
                    let expr = field.into_inner().next().unwrap();
                    output.amount = Some(Box::new(Self::parse_expr(expr)?));
                }
                Rule::datum_field => {
                    let expr = field.into_inner().next().unwrap();
                    output.datum = Some(Box::new(Self::parse_expr(expr)?));
                }
                x => unreachable!("Unexpected rule in output: {:?}", x),
            }
        }

        if output.to.is_empty() {
            return Err(ParseError::MissingRequiredField("to".to_string()));
        }

        Ok(output)
    }

    fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
        match pair.as_rule() {
            Rule::number => Ok(Expr::Number(pair.as_str().parse().unwrap())),
            Rule::identifier => Ok(Expr::Identifier(pair.as_str().to_string())),
            Rule::sub_field => {
                let mut parts = pair.into_inner();
                let base = parts.next().unwrap();
                let mut fields = Vec::new();

                for field in parts {
                    fields.push(field.as_str().to_string());
                }

                Ok(Expr::SubField {
                    base: Box::new(Self::parse_expr(base)?),
                    fields,
                })
            }
            Rule::datum_constructor => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let mut fields = Vec::new();
                let mut spread = None;

                while let Some(field) = inner.next() {
                    if field.as_rule() == Rule::value_deconstructor {
                        let expr = field.into_inner().next().unwrap();
                        spread = Some(Box::new(Self::parse_expr(expr)?));
                        break;
                    }

                    let name = field.as_str().to_string();
                    let expr = inner.next().unwrap();
                    fields.push((name, Box::new(Self::parse_expr(expr)?)));
                }

                Ok(Expr::DatumConstructor {
                    name,
                    fields,
                    spread,
                })
            }
            Rule::expr => {
                let mut inner = pair.into_inner();
                let first = inner.next().unwrap();

                let mut result = Self::parse_expr(first)?;
                dbg!(&result);
                // Process any infix operators
                while let Some(op) = inner.next() {
                    dbg!(&op);
                    let right = inner.next().unwrap();
                    let operator = match op.as_str() {
                        "+" => BinaryOperator::Add,
                        "-" => BinaryOperator::Subtract,
                        _ => unreachable!(),
                    };
                    result = Expr::BinaryOp {
                        left: Box::new(result),
                        operator,
                        right: Box::new(Self::parse_expr(right)?),
                    };
                }

                Ok(result)
            }
            x => unreachable!("Unexpected rule in expression: {:?}", x),
        }
    }

    fn parse_type(pair: pest::iterators::Pair<Rule>) -> Result<Type, ParseError> {
        match pair.as_str() {
            "Int" => Ok(Type::Int),
            "Token" => Ok(Type::Token),
            "Datum" => Ok(Type::Datum),
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
}

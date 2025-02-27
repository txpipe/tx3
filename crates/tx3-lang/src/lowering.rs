//! Lowers the Tx3 language to the intermediate representation.
//!
//! This module takes an AST and performs lowering on it. It converts the AST
//! into the intermediate representation (IR) of the Tx3 language.

use crate::ast;
use crate::ir;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing analyze phase")]
    MissingAnalyzePhase,

    #[error("symbol '{0}' expected to be '{1}'")]
    InvalidSymbol(String, &'static str),

    #[error("invalid ast: {0}")]
    InvalidAst(String),

    #[error("failed to decode hex string: {0}")]
    DecodeHexError(#[from] hex::FromHexError),
}

fn expect_type_def(ident: &ast::Identifier) -> Result<&ast::TypeDef, Error> {
    let symbol = ident.symbol.as_ref().ok_or(Error::MissingAnalyzePhase)?;

    symbol
        .as_type_def()
        .ok_or(Error::InvalidSymbol(ident.value.clone(), "TypeDef"))
}

fn expect_case_def(ident: &ast::Identifier) -> Result<&ast::VariantCase, Error> {
    let symbol = ident.symbol.as_ref().ok_or(Error::MissingAnalyzePhase)?;

    symbol
        .as_variant_case()
        .ok_or(Error::InvalidSymbol(ident.value.clone(), "VariantCase"))
}

fn expect_field_def(ident: &ast::Identifier) -> Result<&ast::RecordField, Error> {
    let symbol = ident.symbol.as_ref().ok_or(Error::MissingAnalyzePhase)?;

    symbol
        .as_field_def()
        .ok_or(Error::InvalidSymbol(ident.value.clone(), "FieldDef"))
}

fn coerce_identifier_into_asset_def(identifier: &ast::Identifier) -> Result<ast::AssetDef, Error> {
    if let Some(ast::Symbol::AssetDef(x)) = &identifier.symbol {
        Ok(x.as_ref().clone())
    } else {
        Err(Error::InvalidSymbol(identifier.value.clone(), "AssetDef"))
    }
}

fn coerce_identifier_into_asset_expr(
    identifier: &ast::Identifier,
) -> Result<ir::Expression, Error> {
    match &identifier.symbol {
        Some(ast::Symbol::Input(x)) => Ok(ir::Expression::EvalInputAssets(x.clone())),
        Some(ast::Symbol::Fees) => Ok(ir::Expression::FeeQuery),
        _ => Err(Error::InvalidSymbol(identifier.value.clone(), "AssetExpr")),
    }
}

pub(crate) trait IntoLower {
    type Output;

    fn into_lower(&self) -> Result<Self::Output, Error>;
}

impl<T> IntoLower for Option<T>
where
    T: IntoLower,
{
    type Output = Option<T::Output>;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        self.as_ref().map(|x| x.into_lower()).transpose()
    }
}

impl<T> IntoLower for Box<T>
where
    T: IntoLower,
{
    type Output = T::Output;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        self.as_ref().into_lower()
    }
}

impl IntoLower for ast::DatumConstructor {
    type Output = ir::StructExpr;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let type_def = expect_type_def(&self.r#type)?;

        let constructor = type_def
            .find_case_index(&self.case.name.value)
            .ok_or(Error::InvalidAst("case not found".to_string()))?;

        let case_def = expect_case_def(&self.case.name)?;

        let mut fields = vec![];

        for field_def in case_def.fields.iter() {
            let value = self.case.find_field_value(&field_def.name);

            if let Some(value) = value {
                fields.push(value.into_lower()?);
            } else {
                todo!();
            }
        }

        Ok(ir::StructExpr {
            constructor,
            fields,
        })
    }
}

impl IntoLower for ast::PolicyDef {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match &self.value {
            ast::PolicyValue::HexString(x) => Ok(ir::Expression::Policy(hex::decode(&x.value)?)),
            ast::PolicyValue::Import(_) => todo!(),
        }
    }
}

impl IntoLower for ast::DataExpr {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let out = match self {
            ast::DataExpr::None => ir::Expression::None,
            ast::DataExpr::Number(x) => Self::Output::Number(*x as i128),
            ast::DataExpr::Bool(x) => ir::Expression::Bool(*x),
            ast::DataExpr::String(x) => ir::Expression::Bytes(x.value.as_bytes().to_vec()),
            ast::DataExpr::HexString(x) => ir::Expression::Bytes(hex::decode(&x.value)?),
            ast::DataExpr::Constructor(x) => ir::Expression::Struct(x.into_lower()?),
            ast::DataExpr::Unit => ir::Expression::Struct(ir::StructExpr::unit()),
            ast::DataExpr::Identifier(x) => match &x.symbol {
                Some(ast::Symbol::ParamVar(n, ty)) => {
                    ir::Expression::EvalParameter(n.to_lowercase().clone(), ty.as_ref().clone())
                }
                Some(ast::Symbol::PartyDef(x)) => {
                    ir::Expression::EvalParameter(x.name.to_lowercase().clone(), ast::Type::Address)
                }
                Some(ast::Symbol::PolicyDef(x)) => x.into_lower()?,
                _ => {
                    dbg!(&x);
                    todo!();
                }
            },
            ast::DataExpr::PropertyAccess(x) => todo!(),
            ast::DataExpr::BinaryOp(x) => todo!(),
        };

        Ok(out)
    }
}

impl IntoLower for ast::AssetConstructor {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let asset_def = coerce_identifier_into_asset_def(&self.r#type)?;

        let asset_name = ir::Expression::Bytes(asset_def.asset_name.as_bytes().to_vec());

        let amount = self.amount.into_lower()?;

        Ok(ir::Expression::Assets(vec![ir::AssetExpr {
            policy: hex::decode(&asset_def.policy.value)?,
            asset_name,
            amount,
        }]))
    }
}

impl IntoLower for ast::AssetBinaryOp {
    type Output = ir::BinaryOp;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let left = self.left.into_lower()?;
        let right = self.right.into_lower()?;

        Ok(ir::BinaryOp {
            left,
            right,
            op: match self.operator {
                ast::BinaryOperator::Add => ir::BinaryOpKind::Add,
                ast::BinaryOperator::Subtract => ir::BinaryOpKind::Sub,
            },
        })
    }
}

impl IntoLower for ast::AssetExpr {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match self {
            ast::AssetExpr::Constructor(x) => x.into_lower(),
            ast::AssetExpr::BinaryOp(x) => {
                Ok(ir::Expression::EvalCustom(Box::new(x.into_lower()?)))
            }
            ast::AssetExpr::Identifier(x) => coerce_identifier_into_asset_expr(x),
            ast::AssetExpr::PropertyAccess(x) => todo!(),
        }
    }
}

impl IntoLower for ast::InputBlockField {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match self {
            ast::InputBlockField::From(x) => x.into_lower(),
            ast::InputBlockField::DatumIs(_) => todo!(),
            ast::InputBlockField::MinAmount(x) => x.into_lower(),
            ast::InputBlockField::Redeemer(x) => x.into_lower(),
            ast::InputBlockField::Ref(x) => x.into_lower(),
        }
    }
}

impl IntoLower for ast::InputBlock {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let ir = ir::InputQuery {
            name: self.name.to_lowercase().clone(),
            address: self.find("from").map(|x| x.into_lower()).transpose()?,
            min_amount: self
                .find("min_amount")
                .map(|x| x.into_lower())
                .transpose()?,
        };

        Ok(ir::Expression::InputQuery(Box::new(ir)))
    }
}

impl IntoLower for ast::OutputBlockField {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match self {
            ast::OutputBlockField::To(x) => x.into_lower(),
            ast::OutputBlockField::Amount(x) => x.into_lower(),
            ast::OutputBlockField::Datum(x) => x.into_lower(),
        }
    }
}

impl IntoLower for ast::OutputBlock {
    type Output = ir::Output;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        Ok(ir::Output {
            address: self.find("to").map(|x| x.into_lower()).transpose()?,
            datum: self.find("datum").map(|x| x.into_lower()).transpose()?,
            amount: self.find("amount").map(|x| x.into_lower()).transpose()?,
        })
    }
}

impl IntoLower for ast::MintBlockField {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match self {
            ast::MintBlockField::Amount(x) => x.into_lower(),
            ast::MintBlockField::Redeemer(x) => x.into_lower(),
        }
    }
}

impl IntoLower for ast::MintBlock {
    type Output = ir::Mint;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        Ok(ir::Mint {
            amount: self.find("amount").map(|x| x.into_lower()).transpose()?,
            redeemer: self.find("redeemer").map(|x| x.into_lower()).transpose()?,
        })
    }
}

impl IntoLower for ast::ChainSpecificBlock {
    type Output = ir::AdHocDirective;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        match self {
            ast::ChainSpecificBlock::Cardano(x) => x.into_lower(),
        }
    }
}

pub fn lower_tx(ast: &ast::TxDef) -> Result<ir::Tx, Error> {
    let ir = ir::Tx {
        inputs: ast
            .inputs
            .iter()
            .map(|x| x.into_lower())
            .collect::<Result<Vec<_>, _>>()?,
        outputs: ast
            .outputs
            .iter()
            .map(|x| x.into_lower())
            .collect::<Result<Vec<_>, _>>()?,
        mint: ast.mint.as_ref().map(|x| x.into_lower()).transpose()?,
        adhoc: ast
            .adhoc
            .iter()
            .map(|x| x.into_lower())
            .collect::<Result<Vec<_>, _>>()?,
        fees: ir::Expression::FeeQuery,
    };

    Ok(ir)
}

/// Lowers the Tx3 language to the intermediate representation.
///
/// This function takes an AST and converts it into the intermediate
/// representation (IR) of the Tx3 language.
///
/// # Arguments
///
/// * `ast` - The AST to lower
///
/// # Returns
///
/// * `Result<ir::Program, Error>` - The lowered intermediate representation
pub fn lower(ast: &ast::Program, template: &str) -> Result<ir::Tx, Error> {
    let tx = ast
        .txs
        .iter()
        .find(|x| x.name == template)
        .ok_or(Error::InvalidAst("tx not found".to_string()))?;

    lower_tx(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file = format!("{}/../../examples/transfer.tx3", manifest_dir);
        let mut ast = crate::parsing::parse_file(&test_file).unwrap();
        crate::analyzing::analyze(&mut ast).unwrap();
        let ir = lower(&ast, "transfer").unwrap();

        dbg!(ir);
    }
}

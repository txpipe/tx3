use crate::analyze::Symbol;
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
    if let Some(Symbol::AssetDef(x)) = &identifier.symbol {
        Ok(x.as_ref().clone())
    } else {
        Err(Error::InvalidSymbol(identifier.value.clone(), "AssetDef"))
    }
}

fn coerce_identifier_into_asset_expr(
    identifier: &ast::Identifier,
) -> Result<ir::Expression, Error> {
    match &identifier.symbol {
        Some(Symbol::Input(x)) => Ok(ir::Expression::EvalInputAssets(x.clone())),
        Some(Symbol::Fees) => Ok(ir::Expression::EvalFees),
        _ => Err(Error::InvalidSymbol(identifier.value.clone(), "AssetExpr")),
    }
}

trait IntoLower {
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
            ast::PolicyValue::HexString(x) => Ok(ir::Expression::Policy(x.value.clone())),
            ast::PolicyValue::Import(_) => todo!(),
        }
    }
}

impl IntoLower for ast::DataExpr {
    type Output = ir::Expression;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let out = match self {
            ast::DataExpr::None => todo!(),
            ast::DataExpr::Number(x) => Self::Output::Number(*x as i128),
            ast::DataExpr::Bool(_) => todo!(),
            ast::DataExpr::String(_) => todo!(),
            ast::DataExpr::HexString(_) => todo!(),
            ast::DataExpr::Constructor(x) => ir::Expression::Struct(x.into_lower()?),
            ast::DataExpr::Unit => ir::Expression::Struct(ir::StructExpr::unit()),
            ast::DataExpr::Identifier(x) => match &x.symbol {
                Some(Symbol::ParamVar(x)) => ir::Expression::EvalParameter(x.clone()),
                Some(Symbol::PartyDef(x)) => ir::Expression::EvalParty(x.name.clone()),
                Some(Symbol::PolicyDef(x)) => x.into_lower()?,
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

        let asset_name = match (self.asset_name.as_ref(), asset_def.asset_name) {
            (Some(x), _) => x.into_lower()?,
            (_, Some(x)) => ir::Expression::Bytes(x.as_bytes().to_vec()),
            _ => return Err(Error::InvalidAst("no asset name".to_string())),
        };

        let amount = self.amount.into_lower()?;

        Ok(ir::Expression::BuildAsset(ir::AssetConstructor {
            policy: asset_def.policy,
            asset_name: Some(Box::new(asset_name)),
            amount: Some(Box::new(amount)),
        }))
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
    type Output = ir::InputQuery;

    fn into_lower(&self) -> Result<Self::Output, Error> {
        let ir = ir::InputQuery {
            name: self.name.clone(),
            address: self.find("from").map(|x| x.into_lower()).transpose()?,
            min_amount: self
                .find("min_amount")
                .map(|x| x.into_lower())
                .transpose()?,
        };

        Ok(ir)
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

pub fn lower_tx(ast: &ast::TxDef) -> Result<ir::Tx, Error> {
    let ir = ir::Tx {
        name: ast.name.clone(),
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
        mints: vec![],
    };

    Ok(ir)
}

pub fn lower(ast: &ast::Program) -> Result<ir::Program, Error> {
    let ir = ir::Program {
        txs: ast
            .txs
            .iter()
            .map(lower_tx)
            .collect::<Result<Vec<_>, _>>()?,
    };

    Ok(ir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file = format!("{}/../../examples/transfer.tx3", manifest_dir);
        let mut ast = crate::parse::parse_file(&test_file).unwrap();
        crate::analyze::analyze(&mut ast).unwrap();
        let ir = lower(&ast).unwrap();

        dbg!(ir);
    }
}

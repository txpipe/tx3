//! Semantic analysis of the Tx3 language.
//!
//! This module takes an AST and performs semantic analysis on it. It checks for
//! duplicate definitions, unknown symbols, and other semantic errors.

use std::{collections::HashMap, rc::Rc};

use thiserror::Error;

use crate::ast::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("duplicate definition: {0}")]
    DuplicateDefinition(String),

    #[error("not in scope: {0}")]
    NotInScope(String),

    #[error("needs parent scope")]
    NeedsParentScope,

    #[error("invalid symbol, expected {0}, got {1}")]
    InvalidSymbol(&'static str, String),
}

trait Analyzable {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error>;
}

impl<T: Analyzable> Analyzable for Option<T> {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        if let Some(item) = self {
            item.analyze(parent)?;
        }

        Ok(())
    }
}

impl<T: Analyzable> Analyzable for Box<T> {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.as_mut().analyze(parent)?;

        Ok(())
    }
}

impl Analyzable for DataBinaryOp {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.left.analyze(parent.clone())?;
        self.right.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for RecordConstructorField {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.name.analyze(parent.clone())?;
        self.value.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for VariantCaseConstructor {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.name.analyze(parent.clone())?;

        let mut scope = Scope::new(parent);

        let case = match &self.name.symbol {
            Some(Symbol::VariantCase(x)) => x,
            Some(x) => return Err(Error::InvalidSymbol("VariantCase", self.name.value.clone())),
            _ => unreachable!(),
        };

        for field in case.fields.iter() {
            scope.track_record_field(field);
        }

        self.scope = Some(Rc::new(scope));

        for field in self.fields.iter_mut() {
            field.analyze(self.scope.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for DatumConstructor {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.r#type.analyze(parent.clone())?;

        let mut scope = Scope::new(parent);

        let type_def = match &self.r#type.symbol {
            Some(Symbol::TypeDef(x)) => x,
            Some(x) => return Err(Error::InvalidSymbol("TypeDef", self.r#type.value.clone())),
            _ => unreachable!(),
        };

        for case in type_def.cases.iter() {
            scope.track_variant_case(case);
        }

        self.scope = Some(Rc::new(scope));

        self.case.analyze(self.scope.clone())?;

        Ok(())
    }
}

impl Analyzable for DataExpr {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            DataExpr::Constructor(x) => x.analyze(parent),
            DataExpr::Identifier(x) => x.analyze(parent),
            DataExpr::PropertyAccess(x) => x.analyze(parent),
            DataExpr::BinaryOp(x) => x.analyze(parent),
            _ => Ok(()),
        }
    }
}

impl Analyzable for AssetBinaryOp {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.left.analyze(parent.clone())?;
        self.right.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for AssetConstructor {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.amount.analyze(parent.clone())?;
        self.r#type.analyze(parent.clone())?;
        self.asset_name.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for PropertyAccess {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.object.analyze(parent.clone())?;

        self.scope = Some(Rc::new(Scope::new(parent)));

        for path in self.path.iter_mut() {
            path.analyze(self.scope.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for AssetExpr {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            AssetExpr::Identifier(x) => x.analyze(parent),
            AssetExpr::Constructor(x) => x.analyze(parent),
            AssetExpr::BinaryOp(x) => x.analyze(parent),
            AssetExpr::PropertyAccess(x) => x.analyze(parent),
        }
    }
}

impl Analyzable for Identifier {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        let symbol = parent.map(|p| p.resolve(&self.value)).transpose()?;

        self.symbol = symbol;

        Ok(())
    }
}

impl Analyzable for Type {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            Type::Custom(x) => x.analyze(parent),
            _ => Ok(()),
        }
    }
}

impl Analyzable for InputBlockField {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            InputBlockField::From(x) => x.analyze(parent.clone())?,
            InputBlockField::DatumIs(x) => x.analyze(parent.clone())?,
            InputBlockField::MinAmount(x) => x.analyze(parent.clone())?,
            InputBlockField::Redeemer(x) => x.analyze(parent.clone())?,
            InputBlockField::Ref(x) => x.analyze(parent.clone())?,
        }

        Ok(())
    }
}

impl Analyzable for InputBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        for field in self.fields.iter_mut() {
            field.analyze(parent.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for OutputBlockField {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            OutputBlockField::To(x) => x.analyze(parent.clone())?,
            OutputBlockField::Amount(x) => x.analyze(parent.clone())?,
            OutputBlockField::Datum(x) => x.analyze(parent.clone())?,
        }

        Ok(())
    }
}

impl Analyzable for OutputBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        for field in self.fields.iter_mut() {
            field.analyze(parent.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for RecordField {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.r#type.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for VariantCase {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        for field in self.fields.iter_mut() {
            field.analyze(parent.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for TypeDef {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        for case in self.cases.iter_mut() {
            case.analyze(parent.clone())?;
        }

        Ok(())
    }
}

impl Analyzable for TxDef {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        let mut scope = Scope::new(parent);

        scope.symbols.insert("fees".to_string(), Symbol::Fees);

        for param in self.parameters.parameters.iter() {
            scope.track_param_var(&param.name);
        }

        for input in self.inputs.iter() {
            scope.track_input(input);
        }

        self.scope = Some(Rc::new(scope));

        for input in self.inputs.iter_mut() {
            input.analyze(self.scope.clone())?;
        }

        for output in self.outputs.iter_mut() {
            output.analyze(self.scope.clone())?;
        }

        Ok(())
    }
}

static ADA: std::sync::LazyLock<AssetDef> = std::sync::LazyLock::new(|| AssetDef {
    name: "Ada".to_string(),
    policy: "Ada".to_string(),
    asset_name: Some("ada".to_string()),
});

impl Analyzable for Program {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        let mut scope = Scope::new(parent);

        for party in self.parties.iter() {
            scope.track_party_def(party);
        }

        for policy in self.policies.iter() {
            scope.track_policy_def(policy);
        }

        scope.track_asset_def(&ADA);

        for asset in self.assets.iter() {
            scope.track_asset_def(asset);
        }

        for type_def in self.types.iter() {
            scope.track_type_def(type_def);
        }

        self.scope = Some(Rc::new(scope));

        for type_def in self.types.iter_mut() {
            type_def.analyze(self.scope.clone())?;
        }

        for tx in self.txs.iter_mut() {
            tx.analyze(self.scope.clone())?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn new(parent: Option<Rc<Scope>>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }

    pub fn track_type_def(&mut self, type_: &TypeDef) {
        self.symbols
            .insert(type_.name.clone(), Symbol::TypeDef(Box::new(type_.clone())));
    }

    pub fn track_variant_case(&mut self, case: &VariantCase) {
        self.symbols.insert(
            case.name.clone(),
            Symbol::VariantCase(Box::new(case.clone())),
        );
    }

    pub fn track_record_field(&mut self, field: &RecordField) {
        self.symbols.insert(
            field.name.clone(),
            Symbol::RecordField(Box::new(field.clone())),
        );
    }

    pub fn track_party_def(&mut self, party: &PartyDef) {
        self.symbols.insert(
            party.name.clone(),
            Symbol::PartyDef(Box::new(party.clone())),
        );
    }

    pub fn track_policy_def(&mut self, policy: &PolicyDef) {
        self.symbols.insert(
            policy.name.clone(),
            Symbol::PolicyDef(Box::new(policy.clone())),
        );
    }

    pub fn track_asset_def(&mut self, asset: &AssetDef) {
        self.symbols.insert(
            asset.name.clone(),
            Symbol::AssetDef(Box::new(asset.clone())),
        );
    }

    pub fn track_param_var(&mut self, param: &str) {
        self.symbols
            .insert(param.to_string(), Symbol::ParamVar(param.to_string()));
    }

    pub fn track_input(&mut self, input: &InputBlock) {
        self.symbols
            .insert(input.name.clone(), Symbol::Input(input.name.clone()));
    }

    pub fn resolve(&self, name: &str) -> Result<Symbol, Error> {
        if let Some(symbol) = self.symbols.get(name) {
            Ok(symbol.clone())
        } else if let Some(parent) = &self.parent {
            parent.resolve(name)
        } else {
            Err(Error::NotInScope(name.to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    ParamVar(String),
    Input(String),
    PartyDef(Box<PartyDef>),
    PolicyDef(Box<PolicyDef>),
    AssetDef(Box<AssetDef>),
    TypeDef(Box<TypeDef>),
    RecordField(Box<RecordField>),
    VariantCase(Box<VariantCase>),
    Fees,
}

impl Symbol {
    pub fn as_type_def(&self) -> Option<&TypeDef> {
        match self {
            Symbol::TypeDef(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn as_variant_case(&self) -> Option<&VariantCase> {
        match self {
            Symbol::VariantCase(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn as_field_def(&self) -> Option<&RecordField> {
        match self {
            Symbol::RecordField(x) => Some(x.as_ref()),
            _ => None,
        }
    }
}

pub fn analyze(ast: &mut Program) -> Result<(), Error> {
    ast.analyze(None)?;

    Ok(())
}

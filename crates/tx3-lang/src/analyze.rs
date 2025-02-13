//! Semantic analysis of the Tx3 language.
//!
//! This module takes an AST and performs semantic analysis on it. It checks for
//! duplicate definitions, unknown symbols, and other semantic errors.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    DuplicateDefinition(String),
    NotInScope(String),
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

impl Analyzable for DatumConstructor {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.r#type.analyze(parent)?;

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

impl Analyzable for AddressExpr {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            AddressExpr::Identifier(x) => x.analyze(parent),
            _ => Ok(()),
        }
    }
}

impl Analyzable for Type {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        match self {
            Type::Int => todo!(),
            Type::Bool => todo!(),
            Type::Bytes => todo!(),
            Type::Custom(x) => x.analyze(parent),
        }
    }
}

impl Analyzable for InputBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.min_amount.analyze(parent.clone())?;
        self.datum_is.analyze(parent.clone())?;
        self.redeemer.analyze(parent.clone())?;
        self.from.analyze(parent.clone())?;

        Ok(())
    }
}

impl Analyzable for OutputBlock {
    fn analyze(&mut self, parent: Option<Rc<Scope>>) -> Result<(), Error> {
        self.to.analyze(parent.clone())?;
        self.amount.analyze(parent.clone())?;

        Ok(())
    }
}

static FEES: std::sync::LazyLock<ParamDef> = std::sync::LazyLock::new(|| ParamDef {
    name: "fees".to_string(),
    r#type: Type::Int,
});

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

        scope.track_asset_def(&ADA);

        for asset in self.assets.iter() {
            scope.track_asset_def(asset);
        }

        for datum in self.datums.iter() {
            scope.track_datum_def(datum);
        }

        self.scope = Some(Rc::new(scope));

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

    pub fn track_datum_def(&mut self, datum: &DatumDef) {
        self.symbols.insert(
            datum.name.clone(),
            Symbol::DatumDef(Box::new(datum.clone())),
        );
    }

    pub fn track_party_def(&mut self, party: &PartyDef) {
        self.symbols.insert(
            party.name.clone(),
            Symbol::PartyDef(Box::new(party.clone())),
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
    AssetDef(Box<AssetDef>),
    DatumDef(Box<DatumDef>),
    Fees,
}

pub fn analyze(ast: &mut Program) -> Result<(), Error> {
    ast.analyze(None)?;

    Ok(())
}

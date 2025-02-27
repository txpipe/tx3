//! The Tx3 language abstract syntax tree (AST).
//!
//! This module defines the abstract syntax tree (AST) for the Tx3 language.
//! It provides the structure for representing Tx3 programs, including
//! transactions, types, assets, and other constructs.
//!
//! This module is not intended to be used directly by end-users. See
//! [`parse_file`](crate::parse_file) and [`parse_string`](crate::parse_string)
//! for parsing Tx3 source code into an AST.

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, PartialEq, Eq)]
pub struct Scope {
    pub(crate) symbols: HashMap<String, Symbol>,
    pub(crate) parent: Option<Rc<Scope>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    ParamVar(String, Box<Type>),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub value: String,

    // analysis
    #[serde(skip)]
    pub(crate) symbol: Option<Symbol>,
}

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            symbol: None,
        }
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub txs: Vec<TxDef>,
    pub types: Vec<TypeDef>,
    pub assets: Vec<AssetDef>,
    pub parties: Vec<PartyDef>,
    pub policies: Vec<PolicyDef>,

    // analysis
    #[serde(skip)]
    pub(crate) scope: Option<Rc<Scope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterList {
    pub parameters: Vec<ParamDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxDef {
    pub name: String,
    pub parameters: ParameterList,
    pub inputs: Vec<InputBlock>,
    pub outputs: Vec<OutputBlock>,
    pub burn: Option<BurnBlock>,
    pub mint: Option<MintBlock>,
    pub adhoc: Vec<ChainSpecificBlock>,

    // analysis
    #[serde(skip)]
    pub(crate) scope: Option<Rc<Scope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StringLiteral {
    pub value: String,
}

impl StringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HexStringLiteral {
    pub value: String,
}

impl HexStringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputBlockField {
    From(DataExpr),
    DatumIs(Type),
    MinAmount(AssetExpr),
    Redeemer(DataExpr),
    Ref(DataExpr),
}

impl InputBlockField {
    fn key(&self) -> &str {
        match self {
            InputBlockField::From(_) => "from",
            InputBlockField::DatumIs(_) => "datum_is",
            InputBlockField::MinAmount(_) => "min_amount",
            InputBlockField::Redeemer(_) => "redeemer",
            InputBlockField::Ref(_) => "ref",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputBlock {
    pub name: String,
    pub is_many: bool,
    pub fields: Vec<InputBlockField>,
}

impl InputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&InputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputBlockField {
    To(Box<DataExpr>),
    Amount(Box<AssetExpr>),
    Datum(Box<DataExpr>),
}

impl OutputBlockField {
    fn key(&self) -> &str {
        match self {
            OutputBlockField::To(_) => "to",
            OutputBlockField::Amount(_) => "amount",
            OutputBlockField::Datum(_) => "datum",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputBlock {
    pub name: Option<String>,
    pub fields: Vec<OutputBlockField>,
}

impl OutputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&OutputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MintBlockField {
    Amount(Box<AssetExpr>),
    Redeemer(Box<DataExpr>),
}

impl MintBlockField {
    fn key(&self) -> &str {
        match self {
            MintBlockField::Amount(_) => "amount",
            MintBlockField::Redeemer(_) => "redeemer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MintBlock {
    pub fields: Vec<MintBlockField>,
}

impl MintBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&MintBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BurnBlock {
    pub fields: Vec<MintBlockField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RecordField {
    pub name: String,
    pub r#type: Type,
}

impl RecordField {
    pub fn new(name: &str, r#type: Type) -> Self {
        Self {
            name: name.to_string(),
            r#type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PartyDef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyField {
    pub name: String,
    pub party_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PolicyDef {
    pub name: String,
    pub value: PolicyValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PolicyValue {
    Import(StringLiteral),
    HexString(HexStringLiteral),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetConstructor {
    pub r#type: Identifier,
    pub amount: Box<DataExpr>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyAccess {
    pub object: Identifier,
    pub path: Vec<Identifier>,

    // analysis
    #[serde(skip)]
    pub(crate) scope: Option<Rc<Scope>>,
}

impl PropertyAccess {
    pub fn new(object: &str, path: &[&str]) -> Self {
        Self {
            object: Identifier::new(object),
            path: path.iter().map(|x| Identifier::new(*x)).collect(),
            scope: None,
        }
    }
}

impl PropertyAccess {
    /// Shift the property access to the next property in the path.
    pub fn shift(mut self) -> Option<Self> {
        if self.path.is_empty() {
            return None;
        }

        let new_object = self.path.remove(0);
        self.object = new_object;

        Some(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordConstructorField {
    pub name: Identifier,
    pub value: Box<DataExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumConstructor {
    pub r#type: Identifier,
    pub case: VariantCaseConstructor,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariantCaseConstructor {
    pub name: Identifier,
    pub fields: Vec<RecordConstructorField>,
    pub spread: Option<Box<DataExpr>>,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

impl VariantCaseConstructor {
    pub fn find_field_value(&self, field: &str) -> Option<&DataExpr> {
        self.fields
            .iter()
            .find(|x| x.name.value == field)
            .map(|x| x.value.as_ref())
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
    Unit,
    Number(i64),
    Bool(bool),
    String(StringLiteral),
    HexString(HexStringLiteral),
    Constructor(DatumConstructor),
    Identifier(Identifier),
    PropertyAccess(PropertyAccess),
    BinaryOp(DataBinaryOp),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Bytes,
    Address,
    Custom(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParamDef {
    pub name: String,
    pub r#type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub name: String,
    pub cases: Vec<VariantCase>,
}

impl TypeDef {
    pub(crate) fn find_case_index(&self, case: &str) -> Option<usize> {
        self.cases.iter().position(|x| x.name == case)
    }

    pub(crate) fn find_case(&self, case: &str) -> Option<&VariantCase> {
        self.cases.iter().find(|x| x.name == case)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantCase {
    pub name: String,
    pub fields: Vec<RecordField>,
}

impl VariantCase {
    pub(crate) fn find_field_index(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|x| x.name == field)
    }

    pub(crate) fn find_field(&self, field: &str) -> Option<&RecordField> {
        self.fields.iter().find(|x| x.name == field)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetDef {
    pub name: String,
    pub policy: HexStringLiteral,
    pub asset_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainSpecificBlock {
    Cardano(crate::cardano::CardanoBlock),
}

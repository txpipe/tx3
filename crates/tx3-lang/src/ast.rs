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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    ParamVar(String, Box<Type>),
    Input(String, Box<Type>),
    PartyDef(Box<PartyDef>),
    PolicyDef(Box<PolicyDef>),
    AssetDef(Box<AssetDef>),
    TypeDef(Box<TypeDef>),
    RecordField(Box<RecordField>),
    VariantCase(Box<VariantCase>),
    Fees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    dummy: bool,
    pub start: usize,
    pub end: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self::DUMMY
    }
}

impl Eq for Span {}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        if self.dummy || other.dummy {
            return true;
        }

        self.start == other.start && self.end == other.end
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl Span {
    pub const DUMMY: Self = Self {
        dummy: true,
        start: 0,
        end: 0,
    };

    pub fn new(start: usize, end: usize) -> Self {
        Self {
            dummy: false,
            start,
            end,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.dummy || (self.start == 0 && self.end == 0)
    }
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

    pub fn as_policy_def(&self) -> Option<&PolicyDef> {
        match self {
            Symbol::PolicyDef(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    pub fn target_type(&self) -> Option<Type> {
        match self {
            Symbol::ParamVar(_, ty) => Some(ty.as_ref().clone()),
            Symbol::RecordField(x) => Some(x.r#type.clone()),
            Symbol::Input(_, ty) => Some(ty.as_ref().clone()),
            x => {
                dbg!(x);
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identifier {
    pub value: String,
    pub span: Span,

    // analysis
    #[serde(skip)]
    pub(crate) symbol: Option<Symbol>,
}

impl Identifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            symbol: None,
            span: Span::DUMMY,
        }
    }

    pub fn try_symbol(&self) -> Result<&Symbol, crate::lowering::Error> {
        match &self.symbol {
            Some(symbol) => Ok(symbol),
            None => Err(crate::lowering::Error::MissingAnalyzePhase),
        }
    }

    pub fn target_type(&self) -> Option<Type> {
        self.symbol.as_ref().and_then(|x| x.target_type())
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Program {
    pub txs: Vec<TxDef>,
    pub types: Vec<TypeDef>,
    pub assets: Vec<AssetDef>,
    pub parties: Vec<PartyDef>,
    pub policies: Vec<PolicyDef>,
    pub span: Span,

    // analysis
    #[serde(skip)]
    pub(crate) scope: Option<Rc<Scope>>,

    #[serde(skip)]
    pub source_code: Option<ropey::Rope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterList {
    pub parameters: Vec<ParamDef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxDef {
    pub name: String,
    pub parameters: ParameterList,
    pub references: Vec<ReferenceBlock>,
    pub inputs: Vec<InputBlock>,
    pub outputs: Vec<OutputBlock>,
    pub validity: Option<ValidityBlock>,
    pub burn: Option<BurnBlock>,
    pub mints: Vec<MintBlock>,
    pub signers: Option<SignersBlock>,
    pub adhoc: Vec<ChainSpecificBlock>,
    pub span: Span,
    pub collateral: Vec<CollateralBlock>,
    pub metadata: Option<MetadataBlock>,

    // analysis
    #[serde(skip)]
    pub(crate) scope: Option<Rc<Scope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StringLiteral {
    pub value: String,
    pub span: Span,
}

impl StringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            span: Span::DUMMY,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HexStringLiteral {
    pub value: String,
    pub span: Span,
}

impl HexStringLiteral {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            span: Span::DUMMY,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollateralBlockField {
    From(AddressExpr),
    MinAmount(AssetExpr),
    Ref(DataExpr),
}

impl CollateralBlockField {
    fn key(&self) -> &str {
        match self {
            CollateralBlockField::From(_) => "from",
            CollateralBlockField::MinAmount(_) => "min_amount",
            CollateralBlockField::Ref(_) => "ref",
        }
    }

    pub fn as_address_expr(&self) -> Option<&AddressExpr> {
        match self {
            CollateralBlockField::From(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_asset_expr(&self) -> Option<&AssetExpr> {
        match self {
            CollateralBlockField::MinAmount(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_data_expr(&self) -> Option<&DataExpr> {
        match self {
            CollateralBlockField::Ref(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollateralBlock {
    pub fields: Vec<CollateralBlockField>,
    pub span: Span,
}

impl CollateralBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&CollateralBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputBlockField {
    From(AddressExpr),
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

    pub fn as_address_expr(&self) -> Option<&AddressExpr> {
        match self {
            InputBlockField::From(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_asset_expr(&self) -> Option<&AssetExpr> {
        match self {
            InputBlockField::MinAmount(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_data_expr(&self) -> Option<&DataExpr> {
        match self {
            InputBlockField::Redeemer(x) => Some(x),
            InputBlockField::Ref(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_datum_type(&self) -> Option<&Type> {
        match self {
            InputBlockField::DatumIs(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceBlock {
    pub name: String,
    pub r#ref: DataExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataBlockField {
    pub key: DataExpr,
    pub value: DataExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataBlock {
    pub fields: Vec<MetadataBlockField>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputBlock {
    pub name: String,
    pub is_many: bool,
    pub fields: Vec<InputBlockField>,
    pub span: Span,
}

impl InputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&InputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }

    pub(crate) fn datum_is(&self) -> Option<&Type> {
        self.find("datum_is").and_then(|x| x.as_datum_type())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputBlockField {
    To(Box<AddressExpr>),
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
    pub span: Span,
}

impl OutputBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&OutputBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidityBlockField {
    UntilSlot(Box<DataExpr>),
    SinceSlot(Box<DataExpr>),
}

impl ValidityBlockField {
    fn key(&self) -> &str {
        match self {
            ValidityBlockField::UntilSlot(_) => "until_slot",
            ValidityBlockField::SinceSlot(_) => "since_slot",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidityBlock {
    pub fields: Vec<ValidityBlockField>,
    pub span: Span,
}

impl ValidityBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&ValidityBlockField> {
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
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignersBlock {
    pub signers: Vec<DataExpr>,
    pub span: Span,
}

impl MintBlock {
    pub(crate) fn find(&self, key: &str) -> Option<&MintBlockField> {
        self.fields.iter().find(|x| x.key() == key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BurnBlock {
    pub fields: Vec<MintBlockField>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordField {
    pub name: String,
    pub r#type: Type,
    pub span: Span,
}

impl RecordField {
    pub fn new(name: &str, r#type: Type) -> Self {
        Self {
            name: name.to_string(),
            r#type,
            span: Span::DUMMY,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PartyDef {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyField {
    pub name: String,
    pub party_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDef {
    pub name: String,
    pub value: PolicyValue,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyField {
    Hash(DataExpr),
    Script(DataExpr),
    Ref(DataExpr),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyConstructor {
    pub fields: Vec<PolicyField>,
    pub span: Span,
}

impl PolicyConstructor {
    pub(crate) fn find_field(&self, field: &str) -> Option<&PolicyField> {
        self.fields.iter().find(|x| match x {
            PolicyField::Hash(_) => field == "hash",
            PolicyField::Script(_) => field == "script",
            PolicyField::Ref(_) => field == "ref",
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyValue {
    Constructor(PolicyConstructor),
    Assign(HexStringLiteral),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticAssetConstructor {
    pub r#type: Identifier,
    pub amount: Box<DataExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnyAssetConstructor {
    pub policy: Box<DataExpr>,
    pub asset_name: Box<DataExpr>,
    pub amount: Box<DataExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetBinaryOp {
    pub left: Box<AssetExpr>,
    pub operator: BinaryOperator,
    pub right: Box<AssetExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetExpr {
    StaticConstructor(StaticAssetConstructor),
    AnyConstructor(AnyAssetConstructor),
    BinaryOp(AssetBinaryOp),
    PropertyAccess(PropertyAccess),
    Identifier(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyAccess {
    pub object: Identifier,
    pub path: Vec<Identifier>,
    pub span: Span,

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
            span: Span::DUMMY,
        }
    }

    pub fn target_type(&self) -> Option<Type> {
        self.path.last().and_then(|x| x.target_type())
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
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructConstructor {
    pub r#type: Identifier,
    pub case: VariantCaseConstructor,
    pub span: Span,

    // analysis
    #[serde(skip)]
    pub scope: Option<Rc<Scope>>,
}

impl StructConstructor {
    pub fn target_type(&self) -> Option<Type> {
        self.r#type.symbol.as_ref().and_then(|x| x.target_type())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariantCaseConstructor {
    pub name: Identifier,
    pub fields: Vec<RecordConstructorField>,
    pub spread: Option<Box<DataExpr>>,
    pub span: Span,

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
pub struct ListConstructor {
    pub elements: Vec<DataExpr>,
    pub span: Span,
}

impl ListConstructor {
    pub fn target_type(&self) -> Option<Type> {
        self.elements.first().and_then(|x| x.target_type())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoRef {
    pub txid: Vec<u8>,
    pub index: u64,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataBinaryOp {
    pub left: Box<DataExpr>,
    pub operator: BinaryOperator,
    pub right: Box<DataExpr>,
    pub span: Span,
}

impl DataBinaryOp {
    pub fn target_type(&self) -> Option<Type> {
        self.left.target_type()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataExpr {
    None,
    Unit,
    Number(i64),
    Bool(bool),
    String(StringLiteral),
    HexString(HexStringLiteral),
    StructConstructor(StructConstructor),
    ListConstructor(ListConstructor),
    Identifier(Identifier),
    PropertyAccess(PropertyAccess),
    BinaryOp(DataBinaryOp),
    UtxoRef(UtxoRef),
}

impl DataExpr {
    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            DataExpr::Identifier(x) => Some(x),
            _ => None,
        }
    }

    pub fn target_type(&self) -> Option<Type> {
        match self {
            DataExpr::Identifier(x) => x.target_type(),
            DataExpr::None => None,
            DataExpr::Unit => Some(Type::Unit),
            DataExpr::Number(_) => Some(Type::Int),
            DataExpr::Bool(_) => Some(Type::Bool),
            DataExpr::String(_) => Some(Type::Bytes),
            DataExpr::HexString(_) => Some(Type::Bytes),
            DataExpr::StructConstructor(x) => x.target_type(),
            DataExpr::ListConstructor(x) => x.target_type(),
            DataExpr::PropertyAccess(x) => x.target_type(),
            DataExpr::BinaryOp(x) => x.target_type(),
            DataExpr::UtxoRef(_) => Some(Type::UtxoRef),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AddressExpr {
    String(StringLiteral),
    HexString(HexStringLiteral),
    Identifier(Identifier),
}

impl AddressExpr {
    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            AddressExpr::Identifier(x) => Some(x),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    Undefined,
    Unit,
    Int,
    Bool,
    Bytes,
    Address,
    UtxoRef,
    AnyAsset,
    List(Box<Type>),
    Custom(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParamDef {
    pub name: String,
    pub r#type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeDef {
    pub name: String,
    pub cases: Vec<VariantCase>,
    pub span: Span,
}

impl TypeDef {
    pub(crate) fn find_case_index(&self, case: &str) -> Option<usize> {
        self.cases.iter().position(|x| x.name == case)
    }

    #[allow(dead_code)]
    pub(crate) fn find_case(&self, case: &str) -> Option<&VariantCase> {
        self.cases.iter().find(|x| x.name == case)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VariantCase {
    pub name: String,
    pub fields: Vec<RecordField>,
    pub span: Span,
}

impl VariantCase {
    #[allow(dead_code)]
    pub(crate) fn find_field_index(&self, field: &str) -> Option<usize> {
        self.fields.iter().position(|x| x.name == field)
    }

    #[allow(dead_code)]
    pub(crate) fn find_field(&self, field: &str) -> Option<&RecordField> {
        self.fields.iter().find(|x| x.name == field)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetDef {
    pub name: String,
    pub policy: DataExpr,
    pub asset_name: DataExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainSpecificBlock {
    Cardano(crate::cardano::CardanoBlock),
}

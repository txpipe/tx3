use serde::{Deserialize, Serialize};

pub type Identifier = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub txs: Vec<TxDef>,
    pub datums: Vec<DatumDef>,
    pub assets: Vec<AssetDef>,
    pub parties: Vec<PartyDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxDef {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub inputs: Vec<InputBlock>,
    pub outputs: Vec<OutputBlock>,
    pub burns: Option<BurnBlock>,
    pub mints: Option<MintBlock>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputBlock {
    pub to: Identifier,
    pub amount: Option<Box<AssetExpr>>,
    pub datum: Option<Box<DataExpr>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BurnBlock {
    pub amount: Box<AssetExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MintBlock {
    pub amount: Box<AssetExpr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumField {
    pub name: String,
    pub typ: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyDef {
    pub name: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatumConstructor {
    pub r#type: String,
    pub variant: String,
    pub fields: Vec<(String, Box<DataExpr>)>,
    pub spread: Option<Box<DataExpr>>,
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
    String(String),
    HexString(String),
    Constructor(DatumConstructor),
    Identifier(String),
    PropertyAccess(PropertyAccess),
    BinaryOp(DataBinaryOp),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    Int,
    Bytes,
    Custom(String),
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

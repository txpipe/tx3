//! The Tx3 language intermediate representation (IR).
//!
//! This module defines the intermediate representation (IR) for the Tx3
//! language. It provides the structure for representing Tx3 programs in a more
//! abstract form, suitable for further processing or execution.
//!
//! This module is not intended to be used directly by end-users. See
//! [`lower`](crate::lower) for lowering an AST to the intermediate
//! representation.

#[derive(Debug, Clone)]
pub struct StructExpr {
    pub constructor: usize,
    pub fields: Vec<Expression>,
}

impl StructExpr {
    pub fn unit() -> Self {
        Self {
            constructor: 0,
            fields: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOpKind {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub op: BinaryOpKind,
}

#[derive(Debug, Clone)]
pub struct AssetConstructor {
    pub policy: String,
    pub asset_name: Option<Box<Expression>>,
    pub amount: Option<Box<Expression>>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Struct(StructExpr),
    Bytes(Vec<u8>),
    Number(i128),
    Address(String),
    Policy(String),
    BuildAsset(AssetConstructor),
    EvalParty(String),
    EvalParameter(String),
    EvalInputDatum(String),
    EvalInputAssets(String),
    EvalCustom(Box<BinaryOp>),
    EvalFees,
}

#[derive(Debug, Clone)]
pub struct InputQuery {
    pub name: String,
    pub address: Option<Expression>,
    pub min_amount: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub address: Option<Expression>,
    pub datum: Option<Expression>,
    pub amount: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Mint {
    pub amount: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct Tx {
    pub name: String,
    pub inputs: Vec<InputQuery>,
    pub outputs: Vec<Output>,
    pub mints: Vec<Mint>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub txs: Vec<Tx>,
}

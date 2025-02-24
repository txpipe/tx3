//! The Tx3 language intermediate representation (IR).
//!
//! This module defines the intermediate representation (IR) for the Tx3
//! language. It provides the structure for representing Tx3 programs in a more
//! abstract form, suitable for further processing or execution.
//!
//! This module is not intended to be used directly by end-users. See
//! [`lower`](crate::lower) for lowering an AST to the intermediate
//! representation.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{ast, Utxo, UtxoRef};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BinaryOpKind {
    Add,
    Sub,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub left: Expression,
    pub right: Expression,
    pub op: BinaryOpKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AssetExpr {
    pub policy: Vec<u8>,
    pub asset_name: Expression,
    pub amount: Expression,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    None,
    Struct(StructExpr),
    Bytes(Vec<u8>),
    Number(i128),
    Bool(bool),
    String(String),
    Address(Vec<u8>),
    Policy(Vec<u8>),
    UtxoRefs(Vec<UtxoRef>),
    UtxoSet(HashSet<Utxo>),
    Assets(Vec<AssetExpr>),
    EvalParty(String),
    EvalParameter(String, ast::Type),
    EvalInputDatum(String),
    EvalInputAssets(String),
    EvalCustom(Box<BinaryOp>),
    InputQuery(Box<InputQuery>),
    FeeQuery,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InputQuery {
    pub name: String,
    pub address: Option<Expression>,
    pub min_amount: Option<Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    pub address: Option<Expression>,
    pub datum: Option<Expression>,
    pub amount: Option<Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mint {
    pub amount: Option<Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tx {
    pub fees: Expression,
    pub inputs: Vec<Expression>,
    pub outputs: Vec<Output>,
    pub mints: Vec<Mint>,
}

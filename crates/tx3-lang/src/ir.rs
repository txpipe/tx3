//! The Tx3 language intermediate representation (IR).
//!
//! This module defines the intermediate representation (IR) for the Tx3
//! language. It provides the structure for representing Tx3 programs in a more
//! abstract form, suitable for further processing or execution.
//!
//! This module is not intended to be used directly by end-users. See
//! [`lower`](crate::lower) for lowering an AST to the intermediate
//! representation.

use std::collections::{HashMap, HashSet};

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

/// An ad-hoc compile directive.
///
/// It's a generic, pass-through structure that the final chain-specific
/// compiler can use to compile custom structures. Tx3 won't attempt to process
/// this IR structure for anything other than trying to apply / reduce its
/// expressions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AdHocDirective {
    pub name: String,
    pub data: HashMap<String, Expression>,
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

    EvalParameter(String, ast::Type),
    EvalInputDatum(String),
    EvalInputAssets(String),
    EvalCustom(Box<BinaryOp>),

    // queries
    InputQuery(Box<InputQuery>),
    FeeQuery,

    // pass-through
    AdHocDirective(Box<AdHocDirective>),
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
    pub redeemer: Option<Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tx {
    pub fees: Expression,
    pub inputs: Vec<Expression>,
    pub outputs: Vec<Output>,
    pub mint: Option<Mint>,
    pub adhoc: Vec<AdHocDirective>,
}

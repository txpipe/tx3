//! The Tx3 language
//!
//! This crate provides the parser, analyzer and lowering logic for the Tx3
//! language.
//!
//! # Parsing
//!
//! ```
//! let program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! ```
//!
//! # Analyzing
//!
//! ```
//! let mut program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! tx3_lang::analyze(&mut program).unwrap();
//! ```
//!
//! # Lowering
//!
//! ```
//! let mut program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! tx3_lang::analyze(&mut program).unwrap();
//! let ir = tx3_lang::lower(&program).unwrap();
//! ```

pub mod analyzing;
pub mod applying;
pub mod ast;
pub mod ir;
pub mod lowering;
pub mod parsing;

#[macro_export]
macro_rules! include_tx3_build {
    ($package: tt) => {
        include!(concat!(env!("OUT_DIR"), concat!("/", $package, ".rs")));
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct UtxoRef {
    pub txid: Vec<u8>,
    pub index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Utxo {
    pub r#ref: UtxoRef,
    pub address: Vec<u8>,
    pub datum: Option<ir::Expression>,
    pub assets: Vec<ir::AssetExpr>,
}

impl std::hash::Hash for Utxo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#ref.hash(state);
    }
}

impl PartialEq for Utxo {
    fn eq(&self, other: &Self) -> bool {
        self.r#ref == other.r#ref
    }
}

impl Eq for Utxo {}

pub type UtxoSet = HashSet<Utxo>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ArgValue {
    Int(i128),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    Address(Vec<u8>),
    UtxoSet(UtxoSet),
}

pub struct Protocol {
    ast: ast::Program,
    global_args: std::collections::HashMap<String, ArgValue>,
}

impl Protocol {
    pub fn load_file(path: &str) -> Result<Self, parsing::Error> {
        let ast = parsing::parse_file(path)?;

        Ok(Self {
            ast,
            global_args: std::collections::HashMap::new(),
        })
    }

    pub fn load_string(code: &str) -> Result<Self, parsing::Error> {
        let ast = parsing::parse_string(code)?;

        Ok(Self {
            ast,
            global_args: std::collections::HashMap::new(),
        })
    }

    pub fn analyze(&mut self) -> Result<(), analyzing::Error> {
        analyzing::analyze(&mut self.ast)
    }

    pub fn set_global_arg(&mut self, name: &str, value: ArgValue) {
        self.global_args.insert(name.to_string(), value);
    }

    pub fn new_tx(&self, template: &str) -> Result<ProtoTx, lowering::Error> {
        let ir = lowering::lower(&self.ast, template)?;
        let mut tx = ProtoTx::from(ir);

        if !self.global_args.is_empty() {
            for (k, v) in &self.global_args {
                tx.set_arg(k, v.clone());
            }
        }

        // TODO: merge lower and apply errors?
        let tx = tx.apply().unwrap();

        Ok(tx.into())
    }
}

use std::collections::HashSet;

pub use applying::{apply_args, apply_fees, apply_inputs, find_params, find_queries, reduce};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtoTx {
    ir: ir::Tx,
    args: std::collections::HashMap<String, ArgValue>,
    inputs: std::collections::HashMap<String, UtxoSet>,
    fees: Option<u64>,

    #[serde(skip)]
    params: std::sync::OnceLock<std::collections::HashMap<String, ast::Type>>,

    #[serde(skip)]
    queries: std::sync::OnceLock<std::collections::HashMap<String, ir::InputQuery>>,
}

impl From<ir::Tx> for ProtoTx {
    fn from(ir: ir::Tx) -> Self {
        Self {
            ir,
            args: std::collections::HashMap::new(),
            inputs: std::collections::HashMap::new(),
            fees: None,
            params: std::sync::OnceLock::new(),
            queries: std::sync::OnceLock::new(),
        }
    }
}

impl ProtoTx {
    pub fn params(&self) -> &std::collections::HashMap<String, ast::Type> {
        self.params.get_or_init(|| find_params(&self.ir))
    }

    pub fn queries(&self) -> &std::collections::HashMap<String, ir::InputQuery> {
        self.queries.get_or_init(|| find_queries(&self.ir))
    }

    pub fn set_arg(&mut self, name: &str, value: ArgValue) {
        self.args.insert(name.to_string(), value);
    }

    pub fn set_input(&mut self, name: &str, value: UtxoSet) {
        self.inputs.insert(name.to_string(), value);
    }

    pub fn set_fees(&mut self, value: u64) {
        self.fees = Some(value);
    }

    pub fn missing_args(&self) -> impl Iterator<Item = (&str, &ast::Type)> {
        self.params()
            .iter()
            .filter(|(k, _)| !self.args.contains_key(k.as_str()))
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn missing_queries(&self) -> impl Iterator<Item = (&str, &ir::InputQuery)> {
        self.queries()
            .iter()
            .filter(|(k, _)| !self.inputs.contains_key(k.as_str()))
            .map(|(k, v)| (k.as_str(), v))
    }

    pub fn apply(self) -> Result<Self, applying::Error> {
        let tx = apply_args(self.ir, &self.args)?;

        let tx = if let Some(fees) = self.fees {
            apply_fees(tx, fees)?
        } else {
            tx
        };

        let tx = apply_inputs(tx, &self.inputs)?;

        let tx = reduce(tx)?;

        Ok(tx.into())
    }
}

impl AsRef<ir::Tx> for ProtoTx {
    fn as_ref(&self) -> &ir::Tx {
        &self.ir
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn happy_path() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let code = format!("{manifest_dir}/../../examples/transfer.tx3");
        let mut protocol = Protocol::load_file(&code).unwrap();
        protocol.analyze().unwrap();

        protocol.set_global_arg("Sender", ArgValue::Address(b"sender".to_vec()));

        let mut tx = protocol.new_tx("transfer").unwrap();

        dbg!(&tx.params());
        dbg!(&tx.queries());

        tx.set_arg("quantity", ArgValue::Int(100_000_000));

        let mut tx = tx.apply().unwrap();

        dbg!(&tx.params());
        dbg!(&tx.queries());

        tx.set_input(
            "source",
            HashSet::from([Utxo {
                r#ref: UtxoRef {
                    txid: b"fafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafa"
                        .to_vec(),
                    index: 0,
                },
                address: b"abababa".to_vec(),
                datum: None,
                assets: vec![ir::AssetExpr {
                    policy: b"abababa".to_vec(),
                    asset_name: ir::Expression::String("asset".to_string()),
                    amount: ir::Expression::Number(100),
                }],
            }]),
        );

        let tx = tx.apply().unwrap();

        dbg!(&tx.params());
        dbg!(&tx.queries());
    }
}

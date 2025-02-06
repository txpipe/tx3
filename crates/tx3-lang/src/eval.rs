use std::collections::HashMap;

use crate::ast::{self, Program, TxDef};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tx '{0}' not found")]
    TxNotFound(String),
}

pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Debug)]
pub struct TxEval {
    pub payload: Vec<u8>,
    pub fee: u64,
    pub ex_units: u64,
}

pub struct Utxo;

pub type Address = String;

pub trait Context {
    fn resolve_input(&self, input: &ast::InputBlock) -> Result<Vec<Utxo>, Error>;
}

pub struct Vm<C: Context> {
    pub program: Program,
    pub parties: HashMap<String, Address>,
    pub context: C,
}

impl<C: Context> Vm<C> {
    pub fn new(program: Program, parties: HashMap<String, Address>, context: C) -> Self {
        Self {
            program,
            parties,
            context,
        }
    }

    pub fn eval_tx(&mut self, tx_id: &str, args: &HashMap<String, Value>) -> Result<TxEval, Error> {
        let tx = self
            .program
            .txs
            .iter()
            .find(|tx| tx.name.as_ref() == tx_id)
            .ok_or(Error::TxNotFound(tx_id.to_string()))?;

        self.validate_parameters(tx, args)?;

        let inputs = self.resolve_inputs(tx)?;

        let outputs = self.construct_outputs(tx)?;

        Ok(TxEval {
            payload: vec![],
            fee: 0,
            ex_units: 0,
        })
    }

    fn validate_parameters(&self, tx: &TxDef, args: &HashMap<String, Value>) -> Result<(), Error> {
        // TODO: validate parameters
        Ok(())
    }

    fn resolve_inputs(&self, tx: &TxDef) -> Result<Vec<Vec<Utxo>>, Error> {
        let mut resolved = Vec::new();

        for block in &tx.inputs {
            let value = self.context.resolve_input(block)?;
            resolved.push(value);
        }

        Ok(resolved)
    }

    fn construct_outputs(&self, tx: &TxDef) -> Result<Vec<Utxo>, Error> {
        Ok(vec![Utxo, Utxo])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext;

    impl Context for TestContext {
        fn resolve_input(&self, input: &ast::InputBlock) -> Result<Vec<Utxo>, Error> {
            Ok(vec![Utxo, Utxo])
        }
    }

    #[test]
    fn smoke_test_swap() {
        let program = crate::parse::parse_file("tests/swap.tx3").unwrap();

        let parties = HashMap::from([
            ("dex".to_string(), "addr1xxx".to_string()),
            ("buyer".to_string(), "addr2xxx".to_string()),
        ]);

        let context = TestContext;

        let mut vm = Vm::new(program, parties, context);

        let params = HashMap::from([
            ("ask".to_string(), Value::Int(100)),
            ("bid".to_string(), Value::Int(100)),
        ]);

        let eval = vm.eval_tx("swap", &params).unwrap();

        println!("{:?}", eval);
    }
}

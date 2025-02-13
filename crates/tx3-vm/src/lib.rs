use std::collections::HashMap;

use pallas::ledger::primitives::conway;

use tx3_lang::{analyze::Symbol, ast, ir};

#[cfg(feature = "cardano")]
mod cardano;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tx '{0}' not found")]
    TxNotFound(String),

    #[error("definition '{0}' not found")]
    DefinitionNotFound(String),

    #[error("party '{0}' not assigned")]
    PartyNotAssigned(String),

    #[error("arg '{0}' not assigned")]
    ArgNotAssigned(String),

    #[error("invalid address")]
    InvalidAddress(#[from] pallas::ledger::addresses::Error),

    #[error("mapping error {0}")]
    MappingError(String),

    #[error("missing amount")]
    MissingAmount,

    #[error("missing asset name")]
    MissingAssetName,

    #[error("missing address")]
    MissingAddress,

    #[error("asset value too high")]
    AssetValueTooHigh,

    #[error("outputs too high")]
    OutputsTooHigh,

    #[error("error coercing {0} into {1}")]
    CoerceError(String, String),

    #[error("no AST analysis performed")]
    NoAstAnalysis,

    #[error("inputs not resolved")]
    InputsNotResolved,

    #[error("can't resolve symbol '{0:?}'")]
    CantResolveSymbol(Symbol),

    #[error("invalid asset expression '{0}'")]
    InvalidAssetExpression(String),

    #[error("invalid address expression '{0}'")]
    InvalidAddressExpression(String),
}

#[derive(Debug, Clone)]
pub enum ArgValue {
    Int(i128),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub policy: String,
    pub name: String,
    pub quantity: u64,
}

#[derive(Debug, Default)]
pub struct TxEval {
    pub payload: Vec<u8>,
    pub fee: u64,
    pub ex_units: u64,
}

// #[derive(Debug, Clone)]
// pub struct Utxo {
//     pub tx_id: Vec<u8>,
//     pub index: u64,
//     pub datum: Option<Value>,
//     pub assets: Vec<Asset>,
// }

pub type Utxo = (
    conway::TransactionInput,
    conway::PostAlonzoTransactionOutput,
);

pub type Address = String;

struct PParams {
    a: u64,
    b: u64,
    // TODO: cost models, execution prices
}

pub trait Ledger {
    fn get_pparams(&self) -> Result<PParams, Error>;
    fn resolve_input(&self, input: &ir::InputQuery) -> Result<Vec<Utxo>, Error>;
}

pub struct Vm<L: Ledger> {
    ledger: L,
    entrypoint: ir::Tx,
    parties: HashMap<String, Address>,
    inputs: HashMap<String, Vec<Utxo>>,
    args: HashMap<String, ArgValue>,
    pparams: Option<PParams>,
    eval: TxEval,
}

impl<L: Ledger> Vm<L> {
    pub fn new(
        entrypoint: ir::Tx,
        parties: HashMap<String, Address>,
        args: HashMap<String, ArgValue>,
        ledger: L,
    ) -> Result<Self, Error> {
        Ok(Self {
            entrypoint,
            parties,
            args,
            ledger,
            eval: Default::default(),
            pparams: Default::default(),
            inputs: Default::default(),
        })
    }

    fn validate_parameters(&self) -> Result<(), Error> {
        // TODO: validate parameters
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext;

    impl Ledger for TestContext {
        fn get_pparams(&self) -> Result<PParams, Error> {
            Ok(PParams { a: 1, b: 2 })
        }

        fn resolve_input(&self, input: &ir::InputQuery) -> Result<Vec<Utxo>, Error> {
            Ok(vec![
                (
                    conway::TransactionInput {
                        transaction_id: "267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c".parse().unwrap(),
                        index: 0,
                    },
                    conway::PostAlonzoTransactionOutput {
                        address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec().into(),
                        value: conway::Value::Coin(500_000_000),
                        datum_option: None,
                        script_ref: None,
                    },
                ),
                (
                    conway::TransactionInput {
                        transaction_id: "267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c".parse().unwrap(),
                        index: 1,
                    },
                    conway::PostAlonzoTransactionOutput {
                        address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec().into(),
                        value: conway::Value::Coin(301_000_000),
                        datum_option: None,
                        script_ref: None,
                    }
                ),
            ])
        }
    }

    #[test]
    fn smoke_test_transfer() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let test_file = format!("{}/../../examples/transfer.tx3", manifest_dir);
        let mut program = tx3_lang::parse::parse_file(&test_file).unwrap();
        tx3_lang::analyze::analyze(&mut program).unwrap();
        let ir = tx3_lang::lowering::lower(&mut program).unwrap();

        let parties = HashMap::from([
            ("Sender".to_string(), "addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2".to_string()),
            ("Receiver".to_string(), "addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2".to_string()),
        ]);

        let args = HashMap::from([("quantity".to_string(), ArgValue::Int(100_000_000))]);

        let context = TestContext;

        let entrypoint = ir.txs.iter().find(|tx| tx.name == "transfer").unwrap();

        let vm = Vm::new(entrypoint.clone(), parties, args, context).unwrap();
        let tx = vm.execute().unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }
}

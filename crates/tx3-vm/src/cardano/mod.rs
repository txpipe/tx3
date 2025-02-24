use std::collections::HashSet;

use eval::{compile_tx, eval_pass};
use tx3_lang::{
    ir::{AssetExpr, InputQuery},
    Utxo, UtxoRef, UtxoSet,
};

use crate::Error;

mod eval;

#[derive(Debug, Default)]
pub struct TxEval {
    pub payload: Vec<u8>,
    pub fee: u64,
    pub ex_units: u64,
}

pub type Network = pallas::ledger::addresses::Network;

pub struct PParams {
    pub network: pallas::ledger::addresses::Network,
    pub min_fee_coefficient: u64,
    pub min_fee_constant: u64,
    pub coins_per_utxo_byte: u64,
    // TODO: cost models, execution prices
}

pub trait Ledger {
    fn get_pparams(&self) -> Result<PParams, Error>;
    fn resolve_input(&self, query: &InputQuery) -> Result<UtxoSet, Error>;
}

fn optimize_tx<L: Ledger>(
    tx: &tx3_lang::ProtoTx,
    pparams: &PParams,
    ledger: &L,
    best_fees: u64,
) -> Result<Option<TxEval>, Error> {
    let mut attempt = tx.clone();

    for (name, query) in tx.missing_queries() {
        let utxos = ledger.resolve_input(query)?;
        attempt.set_input(name, utxos);
    }

    attempt.set_fees(best_fees);

    let attempt = attempt.apply()?;

    let attempt = compile_tx(attempt.as_ref(), pparams)?;
    let eval = eval_pass(&attempt, pparams)?;

    if eval.fee != best_fees {
        return Ok(Some(eval));
    }

    Ok(None)
}

pub fn build_tx<T: Ledger>(
    tx: tx3_lang::ProtoTx,
    ledger: T,
    max_optimize_rounds: usize,
) -> Result<TxEval, Error> {
    let pparams = ledger.get_pparams()?;
    let mut last_eval = TxEval::default();
    let mut rounds = 0;

    while let Some(better) = optimize_tx(&tx, &pparams, &ledger, last_eval.fee)? {
        last_eval = better;

        if rounds > max_optimize_rounds {
            return Err(Error::MaxOptimizeRoundsReached);
        }

        rounds += 1;
    }

    Ok(last_eval)
}

pub struct MockLedger;

impl Ledger for MockLedger {
    fn get_pparams(&self) -> Result<PParams, Error> {
        Ok(PParams {
            network: pallas::ledger::addresses::Network::Testnet,
            min_fee_coefficient: 1,
            min_fee_constant: 2,
            coins_per_utxo_byte: 1,
        })
    }

    fn resolve_input(&self, input: &InputQuery) -> Result<UtxoSet, Error> {
        let utxos = vec![
            Utxo {
                r#ref: UtxoRef {
                    txid: hex::decode("267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c").unwrap(),
                    index: 0,
                },
                address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec().into(),
                assets: vec![AssetExpr {
                    policy: vec![],
                    asset_name: tx3_lang::ir::Expression::Bytes(b"".to_vec()),
                    amount: tx3_lang::ir::Expression::Number(500_000_000)
                }],
                datum: None,
            },
            Utxo {
                r#ref: UtxoRef {
                    txid: hex::decode("267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c").unwrap(),
                    index: 1,
                },
                address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec().into(),
                assets: vec![AssetExpr {
                    policy: vec![],
                    asset_name: tx3_lang::ir::Expression::Bytes(b"".to_vec()),
                    amount: tx3_lang::ir::Expression::Number(301_000_000)
                }],
                datum: None,
            },
        ];

        Ok(HashSet::from_iter(utxos))
    }
}

#[cfg(test)]
mod tests {
    use tx3_lang::{ArgValue, Protocol};

    use super::*;

    fn load_protocol(example_name: &str) -> Result<Protocol, Error> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let code = format!("{manifest_dir}/../../examples/{example_name}.tx3");
        let mut protocol = Protocol::load_file(&code).unwrap();
        protocol.analyze().unwrap();

        Ok(protocol)
    }

    fn address_to_bytes(address: &str) -> ArgValue {
        ArgValue::Address(
            pallas::ledger::addresses::Address::from_bech32(address)
                .unwrap()
                .to_vec(),
        )
    }

    #[test]
    fn smoke_test_transfer() {
        let protocol = load_protocol("transfer").unwrap();

        let mut tx = protocol.new_tx("transfer").unwrap();

        tx.set_arg("Sender", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"));
        tx.set_arg("Receiver", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"));
        tx.set_arg("quantity", ArgValue::Int(100_000_000));

        let tx = tx.apply().unwrap();

        let tx = build_tx(tx, MockLedger, 3).unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }

    #[test]
    fn smoke_test_vesting() {
        let protocol = load_protocol("vesting").unwrap();

        let mut tx = protocol.new_tx("lock").unwrap();

        tx.set_arg("Owner", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"));
        tx.set_arg("Beneficiary", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"));
        tx.set_arg("quantity", ArgValue::Int(100_000_000));
        tx.set_arg("until", ArgValue::Int(1713288000));

        let tx = tx.apply().unwrap();

        let tx = build_tx(tx, MockLedger, 3).unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }
}

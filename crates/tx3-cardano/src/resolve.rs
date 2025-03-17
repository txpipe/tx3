use pallas::ledger::primitives::conway as primitives;
use tx3_lang::ir::InputQuery;

use crate::{compile::compile_tx, Error, PParams};

#[derive(Debug, Default)]
pub struct TxEval {
    pub payload: Vec<u8>,
    pub fee: u64,
    pub ex_units: u64,
}

#[trait_variant::make(Send)]
pub trait Ledger {
    async fn get_pparams(&self) -> Result<PParams, Error>;

    async fn resolve_input(
        &self,
        pattern: utxorpc::spec::cardano::TxOutputPattern,
    ) -> Result<utxorpc::UtxoPage<utxorpc::Cardano>, Error>;
}

fn expr_to_address_pattern(
    value: &tx3_lang::ir::Expression,
) -> utxorpc::spec::cardano::AddressPattern {
    let address = match value {
        tx3_lang::ir::Expression::Address(address) => address.clone(),
        tx3_lang::ir::Expression::String(address) => {
            pallas::ledger::addresses::Address::from_bech32(address)
                .unwrap()
                .to_vec()
        }
        _ => return Default::default(),
    };

    utxorpc::spec::cardano::AddressPattern {
        exact_address: address.into(),
        ..Default::default()
    }
}

fn input_query_to_pattern(query: &InputQuery) -> utxorpc::spec::cardano::TxOutputPattern {
    let address = query.address.as_ref().map(expr_to_address_pattern);

    utxorpc::spec::cardano::TxOutputPattern {
        address,
        ..Default::default()
    }
}

fn utxo_from_u5c_to_tx3(u: utxorpc::ChainUtxo<utxorpc::spec::cardano::TxOutput>) -> tx3_lang::Utxo {
    tx3_lang::Utxo {
        r#ref: tx3_lang::UtxoRef {
            txid: u.txo_ref.as_ref().unwrap().hash.clone().into(),
            index: u.txo_ref.as_ref().unwrap().index,
        },
        address: u.parsed.as_ref().unwrap().address.clone().into(),
        datum: None, //u.parsed.unwrap().datum.into(),
        assets: vec![tx3_lang::ir::AssetExpr {
            policy: vec![],
            asset_name: tx3_lang::ir::Expression::Bytes(vec![]),
            amount: tx3_lang::ir::Expression::Number(u.parsed.as_ref().unwrap().coin as i128),
        }], //u.parsed.unwrap().assets.into(),
        script: u
            .parsed
            .as_ref()
            .and_then(|x| x.script.as_ref())
            .and_then(|x| x.script.as_ref())
            .map(|x| {
                let mut buf = vec![];
                x.encode(&mut buf);
                buf
            })
            .map(tx3_lang::ir::Expression::Bytes),
    }
}

fn eval_size_fees(tx: &[u8], pparams: &PParams) -> Result<u64, Error> {
    Ok(tx.len() as u64 * pparams.min_fee_coefficient + pparams.min_fee_constant + 200_000)
}

#[allow(dead_code)]
fn eval_redeemer_fees(_tx: &primitives::Tx, _pparams: &PParams) -> Result<u64, Error> {
    // pallas::ledger::validate::phase_two::evaluate_tx(tx.into(), pparams, utxos,
    // slot_config);
    todo!()
}

async fn eval_pass<L: Ledger>(
    tx: &tx3_lang::ProtoTx,
    pparams: &PParams,
    ledger: &L,
    best_fees: u64,
) -> Result<Option<TxEval>, Error> {
    let mut attempt = tx.clone();
    attempt.set_fees(best_fees);

    attempt = attempt.apply()?;

    for (name, query) in tx.find_queries() {
        let utxos = ledger
            .resolve_input(input_query_to_pattern(&query))
            .await?
            .items
            .into_iter()
            .map(utxo_from_u5c_to_tx3)
            .collect();

        // TODO: actually filter utxos

        attempt.set_input(&name, utxos);
    }

    let attempt = attempt.apply()?;

    let tx = compile_tx(attempt.as_ref(), pparams)?;

    let payload = pallas::codec::minicbor::to_vec(&tx).unwrap();

    let size_fees = eval_size_fees(&payload, pparams)?;

    //let redeemer_fees = eval_redeemer_fees(tx, pparams)?;

    let eval = TxEval {
        payload,
        fee: size_fees, // TODO: add redeemer fees
        ex_units: 0,
    };

    if eval.fee != best_fees {
        return Ok(Some(eval));
    }

    Ok(None)
}

pub async fn resolve_tx<T: Ledger>(
    tx: tx3_lang::ProtoTx,
    ledger: T,
    max_optimize_rounds: usize,
) -> Result<TxEval, Error> {
    let pparams = ledger.get_pparams().await?;
    let mut last_eval = TxEval::default();
    let mut rounds = 0;

    // one initial pass to reduce any available params;
    let tx = tx.apply()?;

    while let Some(better) = eval_pass(&tx, &pparams, &ledger, last_eval.fee).await? {
        last_eval = better;

        if rounds > max_optimize_rounds {
            return Err(Error::MaxOptimizeRoundsReached);
        }

        rounds += 1;
    }

    Ok(last_eval)
}

#[cfg(test)]
mod tests {
    use tx3_lang::{ArgValue, Protocol};

    use super::*;
    use crate::ledgers::mock::MockLedger;

    fn load_protocol(example_name: &str) -> Protocol {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let code = format!("{manifest_dir}/../../examples/{example_name}.tx3");
        Protocol::from_file(&code).load().unwrap()
    }

    fn address_to_bytes(address: &str) -> ArgValue {
        ArgValue::Address(
            pallas::ledger::addresses::Address::from_bech32(address)
                .unwrap()
                .to_vec(),
        )
    }

    #[tokio::test]
    async fn smoke_test_transfer() {
        let protocol = load_protocol("transfer");

        let tx = protocol.new_tx("transfer")
            .unwrap()
            .with_arg("Sender", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"))
            .with_arg("Receiver", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"))
            .with_arg("quantity", ArgValue::Int(100_000_000))
            .apply()
            .unwrap();

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }

    #[tokio::test]
    async fn smoke_test_vesting() {
        let protocol = load_protocol("vesting");

        let tx = protocol.new_tx("lock")
            .unwrap()
            .with_arg("Owner", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"))
            .with_arg("Beneficiary", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"))
            .with_arg("quantity", ArgValue::Int(100_000_000))
            .with_arg("until", ArgValue::Int(1713288000))
            .apply()
            .unwrap();

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }

    #[tokio::test]
    async fn smoke_test_vesting_unlock() {
        let protocol = load_protocol("vesting");

        let tx = protocol.new_tx("unlock")
            .unwrap()
            .with_arg("beneficiary", address_to_bytes("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2"))
            .with_arg("locked_utxo", ArgValue::UtxoRef(tx3_lang::UtxoRef {
                txid: hex::decode("682d6d95495403b491737b95dae5c1f060498d9efc91a592962134f880398be2").unwrap(),
                index: 1,
            }))
            .with_arg("timelock_script", ArgValue::UtxoRef(tx3_lang::UtxoRef {
                txid: hex::decode("682d6d95495403b491737b95dae5c1f060498d9efc91a592962134f880398be2").unwrap(),
                index: 0,
            }))
            .apply()
            .unwrap();

        dbg!(&tx.find_params());
        dbg!(&tx.find_queries());

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }

    #[tokio::test]
    async fn faucet_test() {
        let protocol = load_protocol("faucet");

        let mut tx = protocol
            .new_tx("claim_with_password")
            .unwrap()
            .apply()
            .unwrap();

        tx.set_arg("quantity", 1.into());
        tx.set_arg("password", hex::decode("abc1").unwrap().into());
        tx.set_arg("requester", "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x".into());

        dbg!(&tx.find_params());

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(&tx.payload));
        println!("{}", tx.fee);
    }
}

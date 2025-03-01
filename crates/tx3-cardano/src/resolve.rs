use tx3_lang::{
    ir::{self, InputQuery},
    UtxoSet,
};

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
    async fn resolve_input(&self, query: &InputQuery) -> Result<UtxoSet, Error>;
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

    for (name, query) in tx.queries() {
        let utxos = ledger.resolve_input(query).await?;
        attempt.set_input(name, utxos);
    }

    let attempt = attempt.apply()?;

    let tx = compile_tx(attempt.as_ref(), pparams)?;

    let payload = pallas::codec::minicbor::to_vec(&tx).unwrap();

    let fee =
        (payload.len() as u64 * pparams.min_fee_coefficient) + pparams.min_fee_constant + 200_000;

    let eval = TxEval {
        payload,
        fee,
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

    #[tokio::test]
    async fn smoke_test_transfer() {
        let protocol = load_protocol("transfer").unwrap();

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
        let protocol = load_protocol("vesting").unwrap();

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
    async fn faucet_test() {
        let protocol = load_protocol("faucet").unwrap();

        let mut tx = protocol
            .new_tx("claim_with_password")
            .unwrap()
            .apply()
            .unwrap();

        tx.set_arg("quantity", 1.into());
        tx.set_arg("password", hex::decode("abc1").unwrap().into());
        tx.set_arg("requester", "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x".into());

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(&tx.payload));
        println!("{}", tx.fee);
    }
}

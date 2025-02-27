use eval::{compile_tx, eval_pass};
use tx3_lang::{ir::InputQuery, UtxoRef, UtxoSet};

use crate::Error;

mod eval;
pub mod ledgers;

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
    async fn get_pparams(&self) -> Result<PParams, Error>;
    async fn resolve_input(&self, query: &InputQuery) -> Result<UtxoSet, Error>;
}

async fn optimize_tx<L: Ledger>(
    tx: &tx3_lang::ProtoTx,
    pparams: &PParams,
    ledger: &L,
    best_fees: u64,
) -> Result<Option<TxEval>, Error> {
    let mut attempt = tx.clone();

    for (name, query) in tx.queries() {
        let utxos = ledger.resolve_input(query).await?;
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

pub async fn build_tx<T: Ledger>(
    tx: tx3_lang::ProtoTx,
    ledger: T,
    max_optimize_rounds: usize,
) -> Result<TxEval, Error> {
    let pparams = ledger.get_pparams().await?;
    let mut last_eval = TxEval::default();
    let mut rounds = 0;

    while let Some(better) = optimize_tx(&tx, &pparams, &ledger, last_eval.fee).await? {
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
    use crate::cardano::ledgers::mock::MockLedger;

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

        let tx = build_tx(tx, MockLedger, 3).await.unwrap();

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

        let tx = build_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }

    #[tokio::test]
    async fn buidlr_fest() {
        let protocol = load_protocol("buidlr_fest").unwrap();

        let tx = protocol.new_tx("purchase_ticket")
            .unwrap()
            .with_arg("Participant", address_to_bytes("addr1q9mjjtht4tqffckvmnacjv0hw9xvh7ts25hfvejcj75dza3zm8mrlcdv4atxwl3fpl6y8fwe9d2zjcsja9a353dsntdqlu3vxk"))
            .with_arg("EventOrganizer", address_to_bytes("addr1zyzpenlg0vywj7zdh9dzdeggaer94zvckfncv9c3886c36yafhxhu32dys6pvn6wlw8dav6cmp4pmtv7cc3yel9uu0nqhcjd29"))
            .with_arg("drep", ArgValue::Bytes(hex::decode("55215f98aa7d9e289d215a55f62d258c6c3a71ab847b76de9ddbe661").unwrap().to_vec()))
            .with_arg("ticket_price", ArgValue::Int(150_000_000))
            .apply()
            .unwrap();

        let ledger = ledgers::u5c::Ledger::new(ledgers::u5c::Config {
            endpoint_url: "https://mainnet.utxorpc-v0.demeter.run".to_string(),
            api_key: "dmtr_utxorpc1wgnnj0qcfj32zxsz2uc8d4g7uclm2s2w".to_string(),
            network_id: 1,
        })
        .await
        .unwrap();

        let tx = build_tx(tx, ledger, 5).await.unwrap();

        println!("{}", hex::encode(&tx.payload));
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

        let tx = build_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(&tx.payload));
        println!("{}", tx.fee);
    }
}

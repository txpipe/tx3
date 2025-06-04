#[derive(Debug, Default)]
pub struct TxEval {
    pub payload: Vec<u8>,
    pub fee: u64,
    pub ex_units: u64,
}

#[trait_variant::make(Send)]
pub trait Ledger {
    async fn get_pparams(&self) -> Result<PParams, Error>;
    async fn resolve_input(&self, query: &InputQuery) -> Result<tx3_lang::UtxoSet, Error>;
}

pub fn resolve_tx(
    tx: tx3_lang::ProtoTx,
    ledger: impl Ledger,
    max_optimize_rounds: usize,
) -> Result<TxEval, Error> {
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::Address;
    use tx3_lang::{ArgValue, Protocol};

    fn load_protocol(example_name: &str) -> Protocol {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let code = format!("{manifest_dir}/../../examples/{example_name}.tx3");
        Protocol::from_file(&code).load().unwrap()
    }

    fn address_to_bytes(address: &str) -> ArgValue {
        let address = Address::from_str(address)
            .unwrap()
            .require_network(bitcoin::Network::Testnet)
            .unwrap();

        ArgValue::Bytes(address.script_pubkey().as_bytes().to_vec())
    }

    #[tokio::test]
    async fn smoke_test_minscript() {
        let protocol = load_protocol("bitcoin_miniscript");

        let tx = protocol
            .new_tx("transfer")
            .unwrap()
            .with_arg(
                "Alice",
                address_to_bytes("tb1pl0e4ywt8u483dg400scfpg9alh2v6xvju0zqw07reg6z00y9jnkq3cjanz"),
            )
            .with_arg(
                "Bob",
                address_to_bytes("tb1pthj7rvazre5k8sgvs0vxujmd5jewv5numuv05dfnu8sjt9w33peqv6dzz7"),
            )
            .with_arg("quantity", ArgValue::Int(100_000_000))
            .apply()
            .unwrap();

        let tx = resolve_tx(tx, MockLedger, 3).await.unwrap();

        println!("{}", hex::encode(tx.payload));
        println!("{}", tx.fee);
    }
}

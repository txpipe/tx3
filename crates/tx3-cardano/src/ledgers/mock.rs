use crate::{resolve::Ledger, Error, PParams};

pub struct MockLedger;

impl Ledger for MockLedger {
    async fn get_pparams(&self) -> Result<PParams, Error> {
        Ok(PParams {
            network: pallas::ledger::addresses::Network::Testnet,
            min_fee_coefficient: 1,
            min_fee_constant: 2,
            coins_per_utxo_byte: 1,
        })
    }

    async fn resolve_input(
        &self,
        _input: utxorpc::spec::cardano::TxOutputPattern,
    ) -> Result<utxorpc::UtxoPage<utxorpc::Cardano>, Error> {
        let utxo = utxorpc::ChainUtxo::<utxorpc::spec::cardano::TxOutput> {
            parsed: utxorpc::spec::cardano::TxOutput {
                address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec().into(),
                coin: 500_000_000,
                assets: vec![],
                datum: None,
                script: None,
            }.into(),
            txo_ref: utxorpc::spec::query::TxoRef {
                hash: hex::decode(
                    "267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c",
                )
                .unwrap()
                .into(),
                index: 0,
            }
            .into(),
            native: vec![].into(),
        };

        Ok(utxorpc::UtxoPage {
            items: vec![utxo.clone(), utxo.clone()],
            next: None,
        })
    }
}

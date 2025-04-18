use tx3_lang::{ir::InputQuery, UtxoSet};

use crate::{resolve::Ledger, Error, PParams};

pub struct MockLedger;

impl Ledger for MockLedger {
    async fn get_pparams(&self) -> Result<PParams, Error> {
        Ok(PParams {
            network: crate::Network::Testnet,
            min_fee_coefficient: 1,
            min_fee_constant: 2,
            coins_per_utxo_byte: 1,
        })
    }

    async fn resolve_input(&self, _input: &InputQuery) -> Result<UtxoSet, Error> {
        let utxo = tx3_lang::Utxo {
        r#ref: tx3_lang::UtxoRef {
            txid: hex::decode(
                    "267aae354f0d14d82877fa5720f7ddc9b0e3eea3cd2a0757af77db4d975ba81c",
                )
                .unwrap().clone(),
            index: 0,
        },
        address: pallas::ledger::addresses::Address::from_bech32("addr1qx0rs5qrvx9qkndwu0w88t0xghgy3f53ha76kpx8uf496m9rn2ursdm3r0fgf5pmm4lpufshl8lquk5yykg4pd00hp6quf2hh2").unwrap().to_vec(),
        datum: None,
        assets: vec![tx3_lang::ir::AssetExpr {
            policy: tx3_lang::ir::Expression::None,
            asset_name: tx3_lang::ir::Expression::None,
            amount: tx3_lang::ir::Expression::Number(500_000_000_i128),
        }],
        script: None
    };

        Ok(UtxoSet::from([utxo]))
    }
}

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tokio::sync::Mutex;
use utxorpc::CardanoQueryClient;

use crate::PParams;

impl From<utxorpc::Error> for crate::Error {
    fn from(error: utxorpc::Error) -> Self {
        crate::Error::LedgerInternalError(error.to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub endpoint_url: String,
    pub api_key: String,
    pub network_id: u8,
}

#[derive(Clone)]
pub struct Ledger {
    queries: Arc<Mutex<utxorpc::CardanoQueryClient>>,
    network_id: u8,
}

impl Ledger {
    pub async fn new(config: Config) -> Result<Self, crate::Error> {
        let queries = utxorpc::ClientBuilder::new()
            .uri(&config.endpoint_url)?
            .metadata("dmtr-api-key", config.api_key)?
            .build::<CardanoQueryClient>()
            .await;

        Ok(Self {
            queries: Arc::new(Mutex::new(queries)),
            network_id: config.network_id,
        })
    }

    // pub async fn read_utxos(
    //     &mut self,
    //     refs: Vec<wit::TxoRef>,
    // ) -> Result<Vec<wit::Utxo>, wit::LedgerError> {
    //     let refs = refs.into_iter().map(|r| r.into()).collect();
    //     let utxos = self.queries.read_utxos(refs).await?;
    //     Ok(utxos.into_iter().map(|u| u.into()).collect())
    // }
}

impl crate::resolve::Ledger for Ledger {
    async fn get_pparams(&self) -> Result<PParams, crate::Error> {
        let req = utxorpc::spec::query::ReadParamsRequest::default();

        let res = self
            .queries
            .lock()
            .await
            .read_params(req)
            .await
            .map_err(|err| crate::Error::LedgerInternalError(format!("{:?}", err)))?;

        let params = res.into_inner().values.and_then(|v| v.params).ok_or(
            crate::Error::LedgerInternalError("unexpected response from read_params".to_string()),
        )?;

        match params {
            utxorpc::spec::query::any_chain_params::Params::Cardano(params) => Ok(PParams {
                network: pallas::ledger::addresses::Network::from(self.network_id),
                min_fee_coefficient: params.min_fee_coefficient,
                min_fee_constant: params.min_fee_constant,
                coins_per_utxo_byte: params.coins_per_utxo_byte,
            }),
        }
    }

    async fn resolve_input(
        &self,
        pattern: utxorpc::spec::cardano::TxOutputPattern,
    ) -> Result<utxorpc::UtxoPage<utxorpc::Cardano>, crate::Error> {
        let utxos = self
            .queries
            .lock()
            .await
            .match_utxos(pattern, None, 1)
            .await?;

        Ok(utxos)
    }
}

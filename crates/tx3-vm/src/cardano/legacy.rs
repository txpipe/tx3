use pallas_primitives::conway;
use pallas_traverse::MultiEraOutput;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::{HashMap, HashSet};

use super::*;

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ReferenceScript {
    pub ref_txo: conway::TransactionInput,
    pub hash: Hash<28>,
    #[serde_as(as = "DisplayFromStr")]
    pub address: Address,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
pub struct TxoRef {
    pub hash: Hash<32>,
    pub index: u64,
}

impl std::str::FromStr for TxoRef {
    type Err = BuildError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hash, index) = s.split_once("#").ok_or(BuildError::MalformedTxoRef)?;
        let hash = Hash::from_str(hash).map_err(|_| BuildError::MalformedTxoRef)?;
        let index = index.parse().map_err(|_| BuildError::MalformedTxoRef)?;
        Ok(TxoRef::new(hash, index))
    }
}

pub trait ValueExpr: 'static + Send + Sync {
    fn eval(&self, ctx: &BuildContext) -> Result<conway::Value, BuildError>;

    fn eval_as_mint(&self, ctx: &BuildContext) -> Result<conway::Mint, BuildError> {
        let value = self.eval(ctx)?;

        match value {
            conway::Value::Multiasset(_, assets) => asset_math::multiasset_coin_to_mint(assets),
            conway::Value::Coin(_) => Err(BuildError::Conflicting),
        }
    }

    fn eval_as_burn(&self, ctx: &BuildContext) -> Result<conway::Mint, BuildError> {
        let value = self.eval(ctx)?;

        match value {
            conway::Value::Multiasset(_, assets) => asset_math::multiasset_coin_to_burn(assets),
            conway::Value::Coin(_) => Err(BuildError::Conflicting),
        }
    }
}

pub trait MintExpr: 'static + Send + Sync {
    fn eval(&self, ctx: &BuildContext) -> Result<Option<conway::Mint>, BuildError>;
    fn eval_redeemer(&self, ctx: &BuildContext) -> Result<Option<conway::Redeemer>, BuildError>;
}

#[derive(Default)]
pub struct MintBuilder {
    pub assets: Vec<Box<dyn ValueExpr>>,
    pub burn: Vec<Box<dyn ValueExpr>>,
    pub redeemer: Option<Box<dyn PlutusDataExpr>>,
}

impl MintBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_asset(mut self, asset: impl ValueExpr) -> Self {
        self.assets.push(Box::new(asset));
        self
    }

    pub fn with_burn(mut self, burn: impl ValueExpr) -> Self {
        self.burn.push(Box::new(burn));
        self
    }

    pub fn using_redeemer(mut self, redeemer: impl PlutusDataExpr) -> Self {
        self.redeemer = Some(Box::new(redeemer));
        self
    }
}

impl MintExpr for MintBuilder {
    fn eval(&self, ctx: &BuildContext) -> Result<Option<primitives::Mint>, BuildError> {
        let out = HashMap::new();

        let out = self.assets.iter().try_fold(out, |mut acc, v| {
            let v = v.eval_as_mint(ctx)?;
            asset_math::fold_multiassets(&mut acc, v);
            Result::<_, BuildError>::Ok(acc)
        })?;

        let out = self.burn.iter().try_fold(out, |mut acc, v| {
            let v = v.eval_as_burn(ctx)?;
            asset_math::fold_multiassets(&mut acc, v);
            Result::<_, BuildError>::Ok(acc)
        })?;

        let mint: Vec<_> = out
            .into_iter()
            .filter_map(|(policy, assets)| {
                let assets = assets.into_iter().collect();
                Some((policy, NonEmptyKeyValuePairs::from_vec(assets)?))
            })
            .collect();

        Ok(NonEmptyKeyValuePairs::from_vec(mint))
    }

    fn eval_redeemer(&self, ctx: &BuildContext) -> Result<Option<conway::Redeemer>, BuildError> {
        let Some(mint) = self.eval(ctx)? else {
            return Ok(None);
        };

        if mint.is_empty() {
            return Err(BuildError::Incomplete);
        }

        if mint.len() > 1 {
            return Err(BuildError::Conflicting);
        }

        let (policy, _) = mint.iter().next().unwrap();

        let data = self
            .redeemer
            .as_ref()
            .ok_or(BuildError::Incomplete)?
            .eval(ctx)?;

        let out = conway::Redeemer {
            tag: conway::RedeemerTag::Mint,
            index: ctx.mint_redeemer_index(*policy)?,
            ex_units: ctx.eval_ex_units(*policy, &data),
            data,
        };

        Ok(Some(out))
    }
}

pub trait ScriptExpr: 'static + Send + Sync {
    fn eval(&self, ctx: &BuildContext) -> Result<conway::ScriptRef, BuildError>;
}

impl ScriptExpr for conway::ScriptRef {
    fn eval(&self, _ctx: &BuildContext) -> Result<conway::ScriptRef, BuildError> {
        Ok(self.clone())
    }
}

impl ScriptExpr for conway::PlutusScript<3> {
    fn eval(&self, _ctx: &BuildContext) -> Result<conway::ScriptRef, BuildError> {
        Ok(conway::ScriptRef::PlutusV3Script(self.clone()))
    }
}

impl TxBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_reference_input(mut self, input: impl InputExpr) -> Self {
        self.reference_inputs.push(Box::new(input));
        self
    }

    pub fn with_input(mut self, input: impl InputExpr) -> Self {
        self.inputs.push(Box::new(input));
        self
    }

    pub fn with_output(mut self, output: impl OutputExpr) -> Self {
        self.outputs.push(Box::new(output));
        self
    }

    pub fn with_mint(mut self, mint: impl MintExpr) -> Self {
        self.mint.push(Box::new(mint));
        self
    }

    pub fn with_fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }
}

impl TxExpr for TxBuilder {
    fn eval_body(&mut self, ctx: &BuildContext) -> Result<conway::TransactionBody, BuildError> {
        let out = conway::TransactionBody {
            inputs: self
                .inputs
                .iter()
                .map(|i| i.eval(ctx))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .into(),
            outputs: self
                .outputs
                .iter_mut()
                .map(|o| o.eval(ctx))
                .collect::<Result<Vec<_>, _>>()?,
            fee: ctx.estimated_fee,
            ttl: None,
            validity_interval_start: None,
            certificates: None,
            withdrawals: None,
            auxiliary_data_hash: None,
            mint: {
                let mints = self
                    .mint
                    .iter()
                    .map(|m| m.eval(ctx))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter_map(|m| m);

                asset_math::aggregate_assets(mints)
            },
            script_data_hash: None,
            collateral: None,
            required_signers: None,
            network_id: None,
            collateral_return: None,
            total_collateral: None,
            reference_inputs: {
                let refs: Vec<_> = self
                    .reference_inputs
                    .iter()
                    .map(|i| i.eval(ctx))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                NonEmptySet::from_vec(refs)
            },
            voting_procedures: None,
            proposal_procedures: None,
            treasury_value: None,
            donation: None,
        };

        Ok(out)
    }

    fn eval_witness_set(&mut self, ctx: &BuildContext) -> Result<conway::WitnessSet, BuildError> {
        let out = conway::WitnessSet {
            redeemer: {
                let redeemers: Vec<_> = self
                    .mint
                    .iter()
                    .map(|m| m.eval_redeemer(ctx))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .filter_map(|r| r)
                    .collect();

                if redeemers.is_empty() {
                    None
                } else {
                    Some(conway::Redeemers::List(conway::MaybeIndefArray::Def(
                        redeemers,
                    )))
                }
            },
            vkeywitness: None,
            native_script: None,
            bootstrap_witness: None,
            plutus_v1_script: None,
            plutus_data: None,
            plutus_v2_script: None,
            plutus_v3_script: None,
        };

        Ok(out)
    }
}

impl BuildContext {
    pub fn mint_redeemer_index(&self, policy: primitives::ScriptHash) -> Result<u32, BuildError> {
        if let Some(tx_body) = &self.tx_body {
            let mut out: Vec<_> = tx_body
                .mint
                .iter()
                .flat_map(|x| x.iter())
                .map(|(p, _)| *p)
                .collect();

            out.sort();
            out.dedup();

            if let Some(index) = out.iter().position(|p| *p == policy) {
                return Ok(index as u32);
            }
        }

        Err(BuildError::RedeemerTargetMissing)
    }

    pub fn eval_ex_units(
        &self,
        _script: primitives::ScriptHash,
        _data: &primitives::PlutusData,
    ) -> primitives::ExUnits {
        // TODO
        primitives::ExUnits { mem: 8, steps: 8 }
    }
}

use std::str::FromStr;

use pallas::{
    codec::utils::{Bytes, CborWrap},
    ledger::{
        addresses::Address as PallasAddress,
        primitives::{conway, Hash, NonEmptyKeyValuePairs, PlutusData, PositiveCoin},
    },
};
use plutus_data::IntoData as _;
use tx3_lang::ir;

use super::*;

mod asset_math;
mod plutus_data;

macro_rules! asset {
    ($policy:expr, $asset:expr, $amount:expr) => {
        ($policy, NonEmptyKeyValuePairs::Def(vec![($asset, $amount)]))
    };
}

macro_rules! value {
    ($coin:expr, $($asset:expr),*) => {
        {
            let assets = NonEmptyKeyValuePairs::from_vec(vec![$(($asset.0, $asset.1)),*]).unwrap();
            pallas::ledger::primitives::conway::Value::Multiasset($coin, assets)
        }
    };
    ($coin:expr) => {
        pallas::ledger::primitives::conway::Value::Coin($coin)
    };
}

fn coerce_data_into_bytes(data: &PlutusData) -> Result<Vec<u8>, Error> {
    match data {
        PlutusData::BoundedBytes(x) => Ok(x.clone().into()),
        _ => {
            return Err(Error::CoerceError(
                format!("{:?}", data),
                "Bytes".to_string(),
            ))
        }
    }
}

fn coerce_string_into_address(value: &str) -> Result<pallas::ledger::addresses::Address, Error> {
    pallas::ledger::addresses::Address::from_str(value)
        .map_err(|_| Error::CoerceError(value.to_string(), "Address".to_string()))
}

fn coerce_policy_into_address(
    hash: &str,
    network: pallas::ledger::addresses::Network,
) -> Result<pallas::ledger::addresses::Address, Error> {
    let policy = Hash::<28>::from_str(hash).unwrap();

    let address = pallas::ledger::addresses::ShelleyAddress::new(
        network,
        pallas::ledger::addresses::ShelleyPaymentPart::Script(policy),
        pallas::ledger::addresses::ShelleyDelegationPart::Null,
    );

    Ok(address.into())
}

fn coerce_arg_into_number(arg: &ArgValue) -> Result<i128, Error> {
    match arg {
        ArgValue::Int(x) => Ok(*x),
        _ => Err(Error::CoerceError(
            format!("{:?}", arg),
            "Number".to_string(),
        )),
    }
}

impl<L: Ledger> Vm<L> {
    fn resolve_party(&mut self, name: &str) -> Result<String, Error> {
        let party = self
            .parties
            .get(name)
            .ok_or(Error::DefinitionNotFound(name.to_string()))?
            .clone();

        Ok(party)
    }

    fn resolve_parameter(&mut self, name: &str) -> Result<ArgValue, Error> {
        let value = self
            .args
            .get(name)
            .ok_or(Error::DefinitionNotFound(name.to_string()))?
            .clone();

        Ok(value)
    }

    fn resolve_input_assets(&mut self, identifier: &str) -> Result<conway::Value, Error> {
        let utxos = self
            .inputs
            .get(identifier)
            .ok_or(Error::DefinitionNotFound(identifier.to_string()))?;

        let values = utxos.iter().map(|(_, utxo)| utxo.value.clone());
        let value = asset_math::aggregate_values(values);

        return Ok(value);
    }

    fn eval_data_expr(&mut self, ir: &ir::Expression) -> Result<PlutusData, Error> {
        match ir {
            ir::Expression::Struct(x) => {
                let fields = x
                    .fields
                    .iter()
                    .map(|f| self.eval_data_expr(f))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(plutus_data::constr(x.constructor as u64, fields))
            }
            ir::Expression::Bytes(_) => todo!(),
            ir::Expression::Number(_) => todo!(),
            ir::Expression::Address(_) => todo!(),
            ir::Expression::BuildAsset(_) => todo!(),
            ir::Expression::EvalParty(x) => {
                let party = self.resolve_party(x)?;
                let address = coerce_string_into_address(&party)?;
                Ok(address.to_vec().into_data())
            }
            ir::Expression::EvalParameter(x) => {
                let arg = self.resolve_parameter(x)?;

                match arg {
                    ArgValue::Int(x) => Ok(x.into_data()),
                    ArgValue::Bool(_) => todo!(),
                    ArgValue::String(_) => todo!(),
                }
            }
            ir::Expression::EvalInputDatum(_) => todo!(),
            ir::Expression::EvalCustom(_) => todo!(),
            ir::Expression::EvalFees => todo!(),
            _ => todo!(),
        }
    }

    fn eval_native_asset_constructor(
        &mut self,
        ctr: &ir::AssetConstructor,
    ) -> Result<conway::Value, Error> {
        dbg!(&ctr);
        let policy = Hash::<28>::from_str(ctr.policy.as_str()).unwrap();

        let asset_name = ctr
            .asset_name
            .as_ref()
            .map(|x| self.eval_into_bytes(x))
            .transpose()?
            .ok_or(Error::MissingAssetName)?;

        let amount = ctr
            .amount
            .as_ref()
            .map(|x| self.eval_into_number(x))
            .transpose()?
            .ok_or(Error::MissingAmount)?;

        let amount = PositiveCoin::try_from(amount as u64).unwrap();

        Ok(value!(0, asset!(policy, asset_name.clone(), amount)))
    }

    fn eval_ada_constructor(&mut self, ctr: &ir::AssetConstructor) -> Result<conway::Value, Error> {
        let amount = ctr
            .amount
            .as_ref()
            .map(|x| self.eval_into_number(x))
            .transpose()?
            .ok_or(Error::MissingAmount)?;

        Ok(value!(amount as u64))
    }

    fn eval_minutxo_constructor(
        &mut self,
        ctr: &ir::AssetConstructor,
    ) -> Result<conway::Value, Error> {
        // let ratio = self.ledger.get_pparams()?.coins_per_utxo_byte;
        // let output = self.eval.find_output(ctr.name.as_str())?;
        // let serialized = pallas_codec::minicbor::to_vec(output).unwrap();
        // let min_lovelace = (160u64 + serialized.len() as u64) * ratio;
        //
        // Ok(value!(min_lovelace as u64))

        todo!()
    }

    fn eval_asset_constructor(
        &mut self,
        ir: &ir::AssetConstructor,
    ) -> Result<conway::Value, Error> {
        if &ir.policy == "Ada" {
            self.eval_ada_constructor(ir)
        } else if ir.policy == "minutxo" {
            self.eval_minutxo_constructor(ir)
        } else {
            self.eval_native_asset_constructor(ir)
        }
    }

    fn eval_asset_binary_op(&mut self, ir: &ir::BinaryOp) -> Result<conway::Value, Error> {
        let left = self.eval_into_asset(&ir.left)?;
        let right = self.eval_into_asset(&ir.right)?;

        match ir.op {
            ir::BinaryOpKind::Add => asset_math::add_values(&left, &right),
            ir::BinaryOpKind::Sub => asset_math::subtract_value(&left, &right),
        }
    }

    fn eval_into_asset(&mut self, ir: &ir::Expression) -> Result<conway::Value, Error> {
        match ir {
            ir::Expression::BuildAsset(x) => self.eval_asset_constructor(x),
            ir::Expression::EvalInputAssets(x) => self.resolve_input_assets(x),
            ir::Expression::EvalCustom(x) => self.eval_asset_binary_op(x),
            ir::Expression::EvalFees => Ok(value!(self.eval.fee as u64)),
            _ => Err(Error::InvalidAssetExpression(format!("{:?}", ir))),
        }
    }

    fn eval_number_binary_op(&mut self, ir: &ir::BinaryOp) -> Result<i128, Error> {
        let left = self.eval_into_number(&ir.left)?;
        let right = self.eval_into_number(&ir.right)?;

        match ir.op {
            ir::BinaryOpKind::Add => Ok(left + right),
            ir::BinaryOpKind::Sub => Ok(left - right),
        }
    }

    fn eval_into_number(&mut self, ir: &ir::Expression) -> Result<i128, Error> {
        match ir {
            ir::Expression::Number(x) => Ok(*x),
            ir::Expression::EvalParameter(x) => {
                let raw = self.resolve_parameter(x)?;
                coerce_arg_into_number(&raw)
            }
            ir::Expression::EvalCustom(x) => self.eval_number_binary_op(x),
            ir::Expression::EvalFees => Ok(self.eval.fee as i128),
            _ => Err(Error::InvalidAssetExpression(format!("{:?}", ir))),
        }
    }

    fn eval_into_bytes(&mut self, ir: &ir::Expression) -> Result<Bytes, Error> {
        match ir {
            ir::Expression::Bytes(x) => Ok(Bytes::from(x.clone())),
            _ => Err(Error::InvalidAssetExpression(format!("{:?}", ir))),
        }
    }

    fn eval_into_address(&mut self, ir: &ir::Expression) -> Result<PallasAddress, Error> {
        match ir {
            ir::Expression::Address(x) => coerce_string_into_address(x),
            ir::Expression::EvalParty(x) => {
                let party = self.resolve_party(x)?;
                coerce_string_into_address(&party)
            }
            ir::Expression::Policy(x) => coerce_policy_into_address(x, self.ledger.get_network()),
            _ => Err(Error::InvalidAddressExpression(format!("{:?}", ir))),
        }
    }

    fn eval_output_block(&mut self, ir: &ir::Output) -> Result<conway::TransactionOutput, Error> {
        let address = ir
            .address
            .as_ref()
            .map(|x| self.eval_into_address(x))
            .transpose()?
            .ok_or(Error::MissingAddress)?;

        let value = ir
            .amount
            .as_ref()
            .map(|x| self.eval_into_asset(&x))
            .transpose()?
            .ok_or(Error::MissingAmount)?;

        let datum_option = ir
            .datum
            .as_ref()
            .map(|x| self.eval_data_expr(x))
            .transpose()?;

        let output = conway::TransactionOutput::PostAlonzo(conway::PostAlonzoTransactionOutput {
            address: address.to_vec().into(),
            value,
            datum_option: datum_option.map(|x| conway::DatumOption::Data(CborWrap(x))),
            script_ref: None, // TODO: add script ref
        });

        Ok(output)
    }

    fn eval_mint_block(&mut self) -> Result<Option<conway::Mint>, Error> {
        if self.entrypoint.mints.is_empty() {
            return Ok(None);
        }

        todo!()
    }

    fn build_inputs(&mut self) -> Result<Vec<conway::TransactionInput>, Error> {
        let inputs = self
            .inputs
            .values()
            .flat_map(|i| i.iter().cloned())
            .map(|(i, _)| i)
            .collect();

        Ok(inputs)
    }

    fn eval_outputs(&mut self) -> Result<Vec<conway::TransactionOutput>, Error> {
        let blocks = self.entrypoint.outputs.iter().cloned().collect::<Vec<_>>();

        let resolved = blocks
            .iter()
            .map(|o| self.eval_output_block(o))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(resolved)
    }

    fn eval_tx_body(&mut self) -> Result<conway::TransactionBody, Error> {
        let out = conway::TransactionBody {
            inputs: self.build_inputs()?.into(),
            outputs: self.eval_outputs()?.into(),
            fee: self.eval.fee,
            ttl: None,
            validity_interval_start: None,
            certificates: None,
            withdrawals: None,
            auxiliary_data_hash: None,
            mint: self.eval_mint_block()?,
            script_data_hash: None,
            collateral: None,
            required_signers: None,
            network_id: None,
            collateral_return: None,
            total_collateral: None,
            reference_inputs: None,
            // reference_inputs: {
            //     let refs: Vec<_> = self
            //         .reference_inputs
            //         .iter()
            //         .map(|i| i.eval(ctx))
            //         .collect::<Result<Vec<_>, _>>()?
            //         .into_iter()
            //         .flatten()
            //         .collect();

            //     NonEmptySet::from_vec(refs)
            // },
            voting_procedures: None,
            proposal_procedures: None,
            treasury_value: None,
            donation: None,
        };

        Ok(out)
    }

    fn eval_auxiliary_data(&mut self) -> Result<Option<conway::AuxiliaryData>, Error> {
        Ok(Some(conway::AuxiliaryData::PostAlonzo(
            pallas::ledger::primitives::alonzo::PostAlonzoAuxiliaryData {
                metadata: None,
                native_scripts: None,
                plutus_scripts: None,
            },
        )))
    }

    fn eval_mint_redeemers(&mut self) -> Result<Vec<conway::Redeemer>, Error> {
        if self.entrypoint.mints.is_empty() {
            return Ok(vec![]);
        }

        todo!()
    }

    fn eval_redeemers(&mut self) -> Result<Option<conway::Redeemers>, Error> {
        let mint_redeemers = self.eval_mint_redeemers()?;

        // TODO: chain other redeemers
        let redeemers = mint_redeemers;

        if redeemers.is_empty() {
            Ok(None)
        } else {
            Ok(Some(conway::Redeemers::List(conway::MaybeIndefArray::Def(
                redeemers,
            ))))
        }
    }

    fn eval_witness_set(&mut self) -> Result<conway::WitnessSet, Error> {
        let out = conway::WitnessSet {
            redeemer: self.eval_redeemers()?,
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

    fn resolve_inputs(&mut self) -> Result<(), Error> {
        for block in self.entrypoint.inputs.iter() {
            let inputs = self.ledger.resolve_input(block)?;
            self.inputs.insert(block.name.clone(), inputs.clone());
        }

        Ok(())
    }

    fn resolve_pparams(&mut self) -> Result<(), Error> {
        let pparams = self.ledger.get_pparams()?;
        self.pparams = Some(pparams);

        Ok(())
    }

    fn build_tx(&mut self) -> Result<conway::Tx, Error> {
        let transaction_body = self.eval_tx_body()?;
        let transaction_witness_set = self.eval_witness_set()?;
        let auxiliary_data = self.eval_auxiliary_data()?;

        let tx = conway::Tx {
            transaction_body,
            transaction_witness_set,
            auxiliary_data: auxiliary_data.into(),
            success: true,
        };

        Ok(tx)
    }

    fn eval_pass(&mut self) -> Result<(), Error> {
        let tx = self.build_tx()?;

        let pparams = self.pparams.as_ref().unwrap();
        let payload = pallas::codec::minicbor::to_vec(tx).unwrap();
        let fee = (payload.len() as u64 * pparams.a) + pparams.b;

        self.eval = TxEval {
            payload,
            fee,
            ex_units: 0,
        };

        Ok(())
    }

    pub fn execute(mut self) -> Result<TxEval, Error> {
        self.validate_parameters()?;
        self.resolve_inputs()?;
        self.resolve_pparams()?;

        for _ in 0..3 {
            self.eval_pass()?;
        }

        Ok(self.eval)
    }
}

use std::str::FromStr;

use pallas_primitives::{conway, Constr, Hash, NonEmptyKeyValuePairs, PlutusData, PositiveCoin};
use plutus_data::IntoData;

use super::*;
use crate::{analyze::Symbol, ast::*};

mod asset_math;
mod plutus_data;

fn coerce_data_into_positive_coin(data: &PlutusData) -> Result<PositiveCoin, Error> {
    match data {
        PlutusData::BigInt(x) => match x {
            conway::BigInt::Int(int) => {
                let value = i128::from(*int);
                let value = PositiveCoin::try_from(value as u64).map_err(|_| {
                    Error::CoerceError(value.to_string(), "PositiveCoin".to_string())
                })?;
                Ok(value)
            }
            _ => {
                return Err(Error::CoerceError(
                    format!("{:?}", data),
                    "PositiveCoin".to_string(),
                ))
            }
        },
        _ => {
            return Err(Error::CoerceError(
                format!("{:?}", data),
                "PositiveCoin".to_string(),
            ))
        }
    }
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

impl<C: Context> Vm<C> {
    fn eval_input_block(
        &mut self,
        ast: &InputBlock,
    ) -> Result<Vec<conway::TransactionInput>, Error> {
        let inputs = self.inputs.get(&ast.name).ok_or(Error::InputsNotResolved)?;

        Ok(inputs.iter().map(|(input, _)| input.clone()).collect())
    }

    // TODO: refactor list lookup into hashmap cache
    fn resolve_party(&mut self, name: &str) -> Result<PartyDef, Error> {
        let party = self
            .program
            .parties
            .iter()
            .find(|p| p.name == name)
            .ok_or(Error::DefinitionNotFound(name.to_string()))?
            .clone();

        Ok(party)
    }

    // TODO: refactor list lookup into hashmap cache
    fn resolve_asset_type(&mut self, ast: &Identifier) -> Result<AssetDef, Error> {
        let asset = self
            .program
            .assets
            .iter()
            .find(|a| a.name == ast.as_ref())
            .ok_or(Error::DefinitionNotFound(ast.as_ref().to_string()))?
            .clone();

        Ok(asset)
    }

    fn eval_asset_identifier(&mut self, identifier: &Identifier) -> Result<conway::Value, Error> {
        let symbol = identifier.symbol.as_ref().ok_or(Error::NoAstAnalysis)?;

        if let Symbol::ParamVar(x) = symbol {
            if x == "fees" {
                return Ok(conway::Value::Coin(555u64));
            }

            return Err(Error::CantResolveSymbol(symbol.clone()));
        }

        if let Symbol::Input(x) = symbol {
            let utxos = self
                .inputs
                .get(x)
                .ok_or(Error::CantResolveSymbol(symbol.clone()))?;

            let values = utxos.iter().map(|(_, utxo)| utxo.value.clone());
            let value = asset_math::aggregate_values(values);

            return Ok(value);
        }

        Err(Error::CantResolveSymbol(symbol.clone()))
    }

    fn eval_data_identifier(&mut self, ast: &Identifier) -> Result<PlutusData, Error> {
        let value = self
            .args
            .get(ast.as_ref())
            .ok_or(Error::ArgNotAssigned(ast.as_ref().to_string()))?
            .clone();

        match value {
            Value::Int(x) => Ok(x.into_data()),
            Value::Bool(x) => Ok(x.into_data()),
            _ => Err(Error::CoerceError(
                format!("{:?}", value),
                "Data".to_string(),
            )),
        }
    }

    fn eval_data_expr(&mut self, ast: &DataExpr) -> Result<PlutusData, Error> {
        match ast {
            DataExpr::None => Ok(().into_data()),
            DataExpr::Number(x) => Ok(x.into_data()),
            DataExpr::Bool(x) => Ok(x.into_data()),
            DataExpr::String(x) => todo!(),
            DataExpr::HexString(x) => todo!(),
            DataExpr::Constructor(x) => todo!(),
            DataExpr::Identifier(x) => self.eval_data_identifier(x),
            DataExpr::PropertyAccess(x) => todo!(),
            DataExpr::BinaryOp(x) => todo!(),
        }
    }

    fn eval_native_asset_constructor(
        &mut self,
        ast: &AssetConstructor,
    ) -> Result<conway::Value, Error> {
        let def = self.resolve_asset_type(&ast.r#type)?;

        let policy = Hash::try_from(hex::decode(def.policy.as_str()).unwrap().as_slice()).unwrap();

        let asset = match def.asset_name {
            Some(name) => name.as_bytes().to_vec(),
            None => match &ast.name {
                Some(name) => {
                    let value = self.eval_data_expr(name)?;
                    coerce_data_into_bytes(&value)?
                }
                None => return Err(Error::MissingAssetName),
            },
        };

        let quantity = self.eval_data_expr(&ast.amount)?;
        let quantity = coerce_data_into_positive_coin(&quantity)?;

        let asset = NonEmptyKeyValuePairs::from_vec(vec![(asset.into(), quantity)]).unwrap();
        let multiasset = conway::Multiasset::from_vec(vec![(policy, asset)]).unwrap();
        let value = conway::Value::Multiasset(0, multiasset);

        Ok(value)
    }

    fn eval_ada_constructor(&mut self, ast: &AssetConstructor) -> Result<conway::Value, Error> {
        let quantity = self.eval_data_expr(&ast.amount)?;

        let quantity = coerce_data_into_positive_coin(&quantity)?;

        let value = conway::Value::Coin(quantity.into());

        Ok(value)
    }

    fn eval_asset_constructor(&mut self, ast: &AssetConstructor) -> Result<conway::Value, Error> {
        if ast.r#type.as_ref() == "Ada" {
            self.eval_ada_constructor(ast)
        } else {
            self.eval_native_asset_constructor(ast)
        }
    }

    fn eval_asset_binary_op(&mut self, ast: &AssetBinaryOp) -> Result<conway::Value, Error> {
        let left = self.eval_asset_expr(&ast.left)?;
        let right = self.eval_asset_expr(&ast.right)?;

        match ast.operator {
            BinaryOperator::Add => asset_math::add_values(&left, &right),
            BinaryOperator::Subtract => asset_math::subtract_value(&left, &right),
        }
    }

    fn eval_asset_expr(&mut self, ast: &AssetExpr) -> Result<conway::Value, Error> {
        match ast {
            AssetExpr::Constructor(x) => self.eval_asset_constructor(x),
            AssetExpr::BinaryOp(x) => self.eval_asset_binary_op(x),
            AssetExpr::Identifier(x) => self.eval_asset_identifier(x),
            AssetExpr::PropertyAccess(_) => todo!(),
        }
    }

    fn eval_output_block(&mut self, ast: &OutputBlock) -> Result<conway::TransactionOutput, Error> {
        let party = self.resolve_party(ast.to.as_ref())?;

        let party = self
            .parties
            .get(party.name.as_str())
            .ok_or(Error::PartyNotAssigned(party.name.as_str().to_string()))?;

        let address = pallas_addresses::Address::from_str(party)?;

        let value = match &ast.amount {
            Some(amount) => self.eval_asset_expr(amount)?,
            None => return Err(Error::MissingAmount),
        };

        let output = conway::TransactionOutput::PostAlonzo(conway::PostAlonzoTransactionOutput {
            address: address.to_vec().into(),
            value,
            datum_option: None, // TODO: add datum
            script_ref: None,   // TODO: add script ref
        });

        Ok(output)
    }

    fn eval_mint_block(&mut self) -> Result<Option<conway::Mint>, Error> {
        if self.entrypoint.mint.is_none() {
            return Ok(None);
        }

        todo!()
    }

    fn eval_inputs(&mut self) -> Result<Vec<conway::TransactionInput>, Error> {
        let blocks = self.entrypoint.inputs.iter().cloned().collect::<Vec<_>>();

        let resolved = blocks
            .iter()
            .map(|i| self.eval_input_block(i))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(resolved)
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
            inputs: self.eval_inputs()?.into(),
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
            pallas_primitives::alonzo::PostAlonzoAuxiliaryData {
                metadata: None,
                native_scripts: None,
                plutus_scripts: None,
            },
        )))
    }

    fn eval_mint_redeemers(&mut self) -> Result<Vec<conway::Redeemer>, Error> {
        if self.entrypoint.mint.is_none() {
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

    pub fn resolve_inputs(&mut self) -> Result<(), Error> {
        for block in self.entrypoint.inputs.iter() {
            let inputs = self.context.resolve_input(block)?;
            self.inputs.insert(block.name.clone(), inputs.clone());
        }

        Ok(())
    }

    pub fn eval(&mut self) -> Result<TxEval, Error> {
        self.validate_parameters()?;

        self.resolve_inputs()?;

        let transaction_body = self.eval_tx_body()?;
        let transaction_witness_set = self.eval_witness_set()?;
        let auxiliary_data = self.eval_auxiliary_data()?;

        let tx = conway::Tx {
            transaction_body,
            transaction_witness_set,
            auxiliary_data: auxiliary_data.into(),
            success: true,
        };

        Ok(TxEval {
            payload: pallas_codec::minicbor::to_vec(tx).unwrap(),
            fee: 0,
            ex_units: 0,
        })
    }
}

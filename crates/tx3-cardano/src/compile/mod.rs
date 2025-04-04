use pallas::ledger::primitives::conway as primitives;
use std::str::FromStr;
use tx3_lang::ir;

use super::*;

pub(crate) mod asset_math;
pub(crate) mod plutus_data;

use plutus_data::{IntoData as _, TryIntoData as _};

macro_rules! asset {
    ($policy:expr, $asset:expr, $amount:expr) => {
        pallas::ledger::primitives::conway::NonEmptyKeyValuePairs::from_vec(vec![(
            $policy,
            pallas::ledger::primitives::conway::NonEmptyKeyValuePairs::from_vec(vec![(
                $asset, $amount,
            )])
            .unwrap(),
        )])
        .unwrap()
    };
}

macro_rules! value {
    ($coin:expr, $assets:expr) => {
        pallas::ledger::primitives::conway::Value::Multiasset($coin, $assets)
    };
    ($coin:expr) => {
        pallas::ledger::primitives::conway::Value::Coin($coin)
    };
}

fn coerce_string_into_address(value: &str) -> Result<pallas::ledger::addresses::Address, Error> {
    pallas::ledger::addresses::Address::from_str(value)
        .map_err(|_| Error::CoerceError(value.to_string(), "Address".to_string()))
}

fn coerce_bytes_into_address(value: &[u8]) -> Result<pallas::ledger::addresses::Address, Error> {
    pallas::ledger::addresses::Address::from_bytes(value)
        .map_err(|_| Error::CoerceError(hex::encode(value), "Address".to_string()))
}

fn coerce_policy_into_address(
    policy: &[u8],
    pparams: &PParams,
) -> Result<pallas::ledger::addresses::Address, Error> {
    let policy = primitives::Hash::from(policy);

    let network = match pparams.network {
        primitives::NetworkId::Testnet => pallas::ledger::addresses::Network::Testnet,
        primitives::NetworkId::Mainnet => pallas::ledger::addresses::Network::Mainnet,
    };

    let address = pallas::ledger::addresses::ShelleyAddress::new(
        network,
        pallas::ledger::addresses::ShelleyPaymentPart::Script(policy),
        pallas::ledger::addresses::ShelleyDelegationPart::Null,
    );

    Ok(address.into())
}

fn coerce_expr_into_number(expr: &ir::Expression) -> Result<i128, Error> {
    match expr {
        ir::Expression::Number(x) => Ok(*x),
        ir::Expression::Assets(x) if x.len() == 1 => coerce_expr_into_number(&x[0].amount),
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "Number".to_string(),
        )),
    }
}

fn coerce_expr_into_utxo_refs(expr: &ir::Expression) -> Result<Vec<tx3_lang::UtxoRef>, Error> {
    match expr {
        ir::Expression::UtxoRefs(x) => Ok(x.clone()),
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "UtxoRefs".to_string(),
        )),
    }
}

fn coerce_expr_into_assets(ir: &ir::Expression) -> Result<Vec<ir::AssetExpr>, Error> {
    match ir {
        ir::Expression::Assets(x) => Ok(x.clone()),
        _ => Err(Error::CoerceError(
            format!("{:?}", ir),
            "Assets".to_string(),
        )),
    }
}

fn coerce_address_into_stake_credential(
    address: &pallas::ledger::addresses::Address,
) -> Result<primitives::StakeCredential, Error> {
    match address {
        pallas::ledger::addresses::Address::Shelley(x) => match x.delegation() {
            pallas::ledger::addresses::ShelleyDelegationPart::Key(x) => {
                Ok(primitives::StakeCredential::AddrKeyhash(*x))
            }
            pallas::ledger::addresses::ShelleyDelegationPart::Script(x) => {
                Ok(primitives::StakeCredential::ScriptHash(*x))
            }
            _ => Err(Error::CoerceError(
                format!("{:?}", address),
                "StakeCredential".to_string(),
            )),
        },
        pallas::ledger::addresses::Address::Stake(x) => match x.payload() {
            pallas::ledger::addresses::StakePayload::Stake(x) => {
                Ok(primitives::StakeCredential::AddrKeyhash(*x))
            }
            pallas::ledger::addresses::StakePayload::Script(x) => {
                Ok(primitives::StakeCredential::ScriptHash(*x))
            }
        },
        _ => Err(Error::CoerceError(
            format!("{:?}", address),
            "StakeCredential".to_string(),
        )),
    }
}

fn coerce_expr_into_stake_credential(
    expr: &ir::Expression,
) -> Result<primitives::StakeCredential, Error> {
    match expr {
        ir::Expression::Address(x) => {
            let address = coerce_bytes_into_address(x)?;
            coerce_address_into_stake_credential(&address)
        }
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "StakeCredential".to_string(),
        )),
    }
}

fn coerce_expr_into_address(
    expr: &ir::Expression,
    pparams: &PParams,
) -> Result<pallas::ledger::addresses::Address, Error> {
    match expr {
        ir::Expression::Address(x) => coerce_bytes_into_address(x),
        ir::Expression::Hash(x) => coerce_policy_into_address(x, pparams),
        ir::Expression::Bytes(x) => coerce_bytes_into_address(x),
        ir::Expression::String(x) => coerce_string_into_address(x),
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "Address".to_string(),
        )),
    }
}

fn coerce_expr_into_bytes(ir: &ir::Expression) -> Result<primitives::Bytes, Error> {
    match ir {
        ir::Expression::Bytes(x) => Ok(primitives::Bytes::from(x.clone())),
        _ => Err(Error::CoerceError(format!("{:?}", ir), "Bytes".to_string())),
    }
}

fn coerce_expr_into_hash<const SIZE: usize>(
    ir: &ir::Expression,
) -> Result<primitives::Hash<SIZE>, Error> {
    match ir {
        ir::Expression::Bytes(x) => Ok(primitives::Hash::from(x.as_slice())),
        _ => Err(Error::CoerceError(format!("{:?}", ir), "Hash".to_string())),
    }
}

// fn extract_classes_from_multiasset(value: &conway::Value) -> Vec<AssetClass>
// {     let ma = match value {
//         conway::Value::Multiasset(_, ma) => ma.iter().cloned().collect(),
//         _ => vec![],
//     };

//     ma.into_iter()
//         .flat_map(|(policy, assets)| {
//             assets
//                 .iter()
//                 .map(|(name, _)| AssetClass {
//                     policy: policy.clone(),
//                     name: name.clone().into(),
//                 })
//                 .collect::<Vec<_>>()
//         })
//         .collect()
// }

fn compile_struct(ir: &ir::StructExpr) -> Result<primitives::PlutusData, Error> {
    let fields = ir
        .fields
        .iter()
        .map(compile_data_expr)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(plutus_data::constr(ir.constructor as u64, fields))
}

fn compile_data_expr(ir: &ir::Expression) -> Result<primitives::PlutusData, Error> {
    match ir {
        ir::Expression::None => Ok(().into_data()),
        ir::Expression::Bytes(x) => Ok(x.into_data()),
        ir::Expression::Number(x) => Ok(x.into_data()),
        ir::Expression::Bool(x) => Ok(x.into_data()),
        ir::Expression::String(x) => Ok(x.as_str().into_data()),
        ir::Expression::Struct(x) => compile_struct(x),
        ir::Expression::Address(x) => Ok(x.into_data()),
        _ => Err(Error::CoerceError(
            format!("{:?}", ir),
            "DataExpr".to_string(),
        )),
    }
}

fn compile_native_asset_for_output(
    ir: &ir::AssetExpr,
) -> Result<primitives::Multiasset<primitives::PositiveCoin>, Error> {
    let policy = primitives::Hash::from(ir.policy.as_slice());
    let asset_name = coerce_expr_into_bytes(&ir.asset_name)?;
    let amount = coerce_expr_into_number(&ir.amount)?;
    let amount = primitives::PositiveCoin::try_from(amount as u64).unwrap();

    let asset = asset!(policy, asset_name.clone(), amount);

    Ok(asset)
}

fn compile_native_asset_for_mint(
    ir: &ir::AssetExpr,
) -> Result<primitives::Multiasset<primitives::NonZeroInt>, Error> {
    let policy = primitives::Hash::from(ir.policy.as_slice());
    let asset_name = coerce_expr_into_bytes(&ir.asset_name)?;
    let amount = coerce_expr_into_number(&ir.amount)?;
    let amount = primitives::NonZeroInt::try_from(amount as i64).unwrap();

    let asset = asset!(policy, asset_name.clone(), amount);

    Ok(asset)
}

fn compile_ada_value(ir: &ir::AssetExpr) -> Result<primitives::Value, Error> {
    let amount = coerce_expr_into_number(&ir.amount)?;

    Ok(value!(amount as u64))
}

fn compile_value(ir: &ir::AssetExpr) -> Result<primitives::Value, Error> {
    if ir.policy.is_empty() {
        compile_ada_value(ir)
    } else {
        let asset = compile_native_asset_for_output(ir)?;
        Ok(value!(0, asset))
    }
}

// calculate min utxo lovelace according to spec
// https://cips.cardano.org/cip/CIP-55

/*
fn eval_minutxo_constructor(&self, ctr: &ir::AssetConstructor) -> Result<primitives::Value, Error> {
    let ratio = self.ledger.get_pparams()?.coins_per_utxo_byte;
    let output = self.eval.find_output(ctr.name.as_str())?;
    let serialized = pallas_codec::minicbor::to_vec(output).unwrap();
    let min_lovelace = (160u64 + serialized.len() as u64) * ratio;

    Ok(value!(min_lovelace as u64))

    todo!()
}
    */

fn compile_output_block(
    ir: &ir::Output,
    pparams: &PParams,
) -> Result<primitives::TransactionOutput, Error> {
    let address = ir
        .address
        .as_ref()
        .map(|x| coerce_expr_into_address(x, pparams))
        .transpose()?
        .ok_or(Error::MissingAddress)?;

    let asset_list = ir
        .amount
        .iter()
        .map(coerce_expr_into_assets)
        .collect::<Result<Vec<_>, _>>()?;

    let values = asset_list
        .iter()
        .flatten()
        .map(compile_value)
        .collect::<Result<Vec<_>, _>>()?;

    let value = asset_math::aggregate_values(values);

    let datum_option = ir.datum.as_ref().map(compile_data_expr).transpose()?;

    let output =
        primitives::TransactionOutput::PostAlonzo(primitives::PostAlonzoTransactionOutput {
            address: address.to_vec().into(),
            value,
            datum_option: datum_option
                .map(|x| primitives::DatumOption::Data(pallas::codec::utils::CborWrap(x))),
            script_ref: None, // TODO: add script ref
        });

    Ok(output)
}

fn compile_mint_block(tx: &ir::Tx) -> Result<Option<primitives::Mint>, Error> {
    let mint = if let Some(mint) = tx.mint.as_ref() {
        let assets = mint
            .amount
            .as_ref()
            .map(coerce_expr_into_assets)
            .transpose()?
            .iter()
            .flatten()
            .map(compile_native_asset_for_mint)
            .collect::<Result<Vec<_>, _>>()?;

        let value = asset_math::aggregate_assets(assets).unwrap();

        Some(value)
    } else {
        None
    };

    Ok(mint)
}

fn compile_inputs(tx: &ir::Tx) -> Result<Vec<primitives::TransactionInput>, Error> {
    let refs = tx
        .inputs
        .iter()
        .flat_map(|x| x.refs.iter())
        .map(|x| primitives::TransactionInput {
            transaction_id: x.txid.as_slice().into(),
            index: x.index as u64,
        })
        .collect();

    Ok(refs)
}

fn compile_outputs(
    tx: &ir::Tx,
    pparams: &PParams,
) -> Result<Vec<primitives::TransactionOutput>, Error> {
    let resolved = tx
        .outputs
        .iter()
        .map(|x| compile_output_block(x, pparams))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(resolved)
}

fn compile_vote_delegation_certificate(
    x: &ir::AdHocDirective,
) -> Result<primitives::Certificate, Error> {
    let stake = coerce_expr_into_stake_credential(&x.data["stake"])?;
    let drep = coerce_expr_into_bytes(&x.data["drep"])?;
    let drep = primitives::DRep::Key(drep.as_slice().into());

    Ok(primitives::Certificate::VoteDeleg(stake, drep))
}

fn compile_certs(tx: &ir::Tx) -> Result<Vec<primitives::Certificate>, Error> {
    tx.adhoc
        .iter()
        .filter_map(|x| match x.name.as_str() {
            "vote_delegation_certificate" => {
                let cert = compile_vote_delegation_certificate(x);
                Some(cert)
            }
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()
}

fn compile_reference_inputs(tx: &ir::Tx) -> Result<Vec<primitives::TransactionInput>, Error> {
    let refs = tx
        .inputs
        .iter()
        .filter_map(|x| x.policy.as_ref())
        .filter_map(|x| x.script.as_ref())
        .filter_map(|x| x.as_utxo_ref())
        .flat_map(coerce_expr_into_utxo_refs)
        .flatten()
        .map(|x| primitives::TransactionInput {
            transaction_id: x.txid.as_slice().into(),
            index: x.index as u64,
        })
        .collect();

    Ok(refs)
}

fn compile_tx_body(tx: &ir::Tx, pparams: &PParams) -> Result<primitives::TransactionBody, Error> {
    let out = primitives::TransactionBody {
        inputs: compile_inputs(tx)?.into(),
        outputs: compile_outputs(tx, pparams)?,
        fee: coerce_expr_into_number(&tx.fees)? as u64,
        certificates: primitives::NonEmptySet::from_vec(compile_certs(tx)?),
        mint: compile_mint_block(tx)?,
        reference_inputs: primitives::NonEmptySet::from_vec(compile_reference_inputs(tx)?),
        network_id: Some(pparams.network),
        ttl: None,
        validity_interval_start: None,
        withdrawals: None,
        auxiliary_data_hash: None,
        script_data_hash: None,
        collateral: None,
        required_signers: None,
        collateral_return: None,
        total_collateral: None,
        voting_procedures: None,
        proposal_procedures: None,
        treasury_value: None,
        donation: None,
    };

    Ok(out)
}

fn compile_auxiliary_data(_tx: &ir::Tx) -> Result<Option<primitives::AuxiliaryData>, Error> {
    // Ok(Some(primitives::AuxiliaryData::PostAlonzo(
    //     pallas::ledger::primitives::alonzo::PostAlonzoAuxiliaryData {
    //         metadata: None,
    //         native_scripts: None,
    //         plutus_scripts: None,
    //     },
    // )))

    Ok(None)
}

fn utxo_ref_matches(ref1: &tx3_lang::UtxoRef, ref2: &primitives::TransactionInput) -> bool {
    ref1.txid.eq(ref2.transaction_id.as_slice()) && ref1.index == ref2.index as u32
}

fn compile_single_spend_redeemer(
    input_id: &tx3_lang::UtxoRef,
    input_ir: &ir::Input,
    sorted_inputs: &[&primitives::TransactionInput],
) -> Result<primitives::Redeemer, Error> {
    let index = sorted_inputs
        .iter()
        .position(|x| utxo_ref_matches(input_id, x))
        .unwrap();

    let redeemer = primitives::Redeemer {
        tag: primitives::RedeemerTag::Spend,
        index: index as u32,
        ex_units: primitives::ExUnits { mem: 0, steps: 0 },
        data: input_ir.redeemer.try_into_data()?,
    };

    Ok(redeemer)
}

fn compile_spend_redeemers(
    tx: &ir::Tx,
    compiled_body: &primitives::TransactionBody,
) -> Result<Vec<primitives::Redeemer>, Error> {
    let mut compiled_inputs = compiled_body.inputs.iter().collect::<Vec<_>>();
    compiled_inputs.sort_by_key(|x| (x.transaction_id, x.index));

    tx.inputs
        .iter()
        .filter(|input| input.redeemer.is_some())
        .flat_map(|input| {
            input.refs.iter().map({
                let value = compiled_inputs.clone();
                move |ref_| compile_single_spend_redeemer(ref_, input, value.as_slice())
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn compile_mint_redeemers(_tx: &ir::Tx) -> Result<Vec<primitives::Redeemer>, Error> {
    // TODO

    // pub fn mint_redeemer_index(&self, policy: primitives::ScriptHash) ->
    // Result<u32, BuildError> {     if let Some(tx_body) = &self.tx_body {
    //         let mut out: Vec<_> = tx_body
    //             .mint
    //             .iter()
    //             .flat_map(|x| x.iter())
    //             .map(|(p, _)| *p)
    //             .collect();

    //         out.sort();
    //         out.dedup();

    //         if let Some(index) = out.iter().position(|p| *p == policy) {
    //             return Ok(index as u32);
    //         }
    //     }

    //     Err(BuildError::RedeemerTargetMissing)
    // }
    //
    // let out = primitives::Redeemer {
    //     tag: primitives::RedeemerTag::Mint,
    //     index: ctx.mint_redeemer_index(*policy)?,
    //     ex_units: ctx.eval_ex_units(*policy, &data),
    //     data,
    // };

    Ok(vec![])
}

fn compile_redeemers(
    tx: &ir::Tx,
    compiled_body: &primitives::TransactionBody,
) -> Result<Option<primitives::Redeemers>, Error> {
    let spend_redeemers = compile_spend_redeemers(tx, compiled_body)?;
    let mint_redeemers = compile_mint_redeemers(tx)?;

    // TODO: chain other redeemers
    let redeemers: Vec<_> = spend_redeemers.into_iter().chain(mint_redeemers).collect();

    if redeemers.is_empty() {
        Ok(None)
    } else {
        Ok(Some(primitives::Redeemers::List(
            primitives::MaybeIndefArray::Def(redeemers),
        )))
    }
}

fn compile_witness_set(
    tx: &ir::Tx,
    compiled_body: &primitives::TransactionBody,
) -> Result<primitives::WitnessSet, Error> {
    let out = primitives::WitnessSet {
        redeemer: compile_redeemers(tx, compiled_body)?,
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

pub fn compile_tx(tx: &ir::Tx, pparams: &PParams) -> Result<primitives::Tx, Error> {
    let transaction_body = compile_tx_body(tx, pparams)?;
    let transaction_witness_set = compile_witness_set(tx, &transaction_body)?;
    let auxiliary_data = compile_auxiliary_data(tx)?;

    let tx = primitives::Tx {
        transaction_body,
        transaction_witness_set,
        auxiliary_data: auxiliary_data.into(),
        success: true,
    };

    Ok(tx)
}

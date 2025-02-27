use std::str::FromStr;

use pallas::{
    codec::utils::{Bytes, CborWrap},
    ledger::{
        addresses::{Address as PallasAddress, ShelleyAddress},
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
        pallas::ledger::primitives::conway::NonEmptyKeyValuePairs::from_vec(vec![(
            $policy,
            NonEmptyKeyValuePairs::from_vec(vec![($asset, $amount)]).unwrap(),
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
    hash: &[u8],
    pparams: &PParams,
) -> Result<pallas::ledger::addresses::Address, Error> {
    let policy = hash.into();

    let address = pallas::ledger::addresses::ShelleyAddress::new(
        pparams.network,
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

fn coerce_expr_into_utxo_refs(expr: &ir::Expression) -> Result<Vec<UtxoRef>, Error> {
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
    address: &PallasAddress,
) -> Result<conway::StakeCredential, Error> {
    match address {
        PallasAddress::Shelley(x) => match x.delegation() {
            pallas::ledger::addresses::ShelleyDelegationPart::Key(x) => {
                Ok(conway::StakeCredential::AddrKeyhash(*x))
            }
            pallas::ledger::addresses::ShelleyDelegationPart::Script(x) => {
                Ok(conway::StakeCredential::ScriptHash(*x))
            }
            _ => Err(Error::CoerceError(
                format!("{:?}", address),
                "StakeCredential".to_string(),
            )),
        },
        PallasAddress::Stake(x) => match x.payload() {
            pallas::ledger::addresses::StakePayload::Stake(x) => {
                Ok(conway::StakeCredential::AddrKeyhash(*x))
            }
            pallas::ledger::addresses::StakePayload::Script(x) => {
                Ok(conway::StakeCredential::ScriptHash(*x))
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
) -> Result<conway::StakeCredential, Error> {
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
) -> Result<PallasAddress, Error> {
    match expr {
        ir::Expression::Address(x) => coerce_bytes_into_address(x),
        ir::Expression::Policy(x) => coerce_policy_into_address(x, pparams),
        ir::Expression::Bytes(x) => coerce_bytes_into_address(x),
        ir::Expression::String(x) => coerce_string_into_address(x),
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "Address".to_string(),
        )),
    }
}

fn coerce_expr_into_bytes(ir: &ir::Expression) -> Result<Bytes, Error> {
    match ir {
        ir::Expression::Bytes(x) => Ok(Bytes::from(x.clone())),
        _ => Err(Error::InvalidAssetExpression(format!("{:?}", ir))),
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

fn compile_struct(ir: &ir::StructExpr) -> Result<PlutusData, Error> {
    let fields = ir
        .fields
        .iter()
        .map(compile_data_expr)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(plutus_data::constr(ir.constructor as u64, fields))
}

fn compile_data_expr(ir: &ir::Expression) -> Result<PlutusData, Error> {
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
) -> Result<conway::Multiasset<conway::PositiveCoin>, Error> {
    let policy = Hash::from(ir.policy.as_slice());
    let asset_name = coerce_expr_into_bytes(&ir.asset_name)?;
    let amount = coerce_expr_into_number(&ir.amount)?;
    let amount = conway::PositiveCoin::try_from(amount as u64).unwrap();

    let asset = asset!(policy, asset_name.clone(), amount);

    Ok(asset)
}

fn compile_native_asset_for_mint(
    ir: &ir::AssetExpr,
) -> Result<conway::Multiasset<conway::NonZeroInt>, Error> {
    let policy = Hash::from(ir.policy.as_slice());
    let asset_name = coerce_expr_into_bytes(&ir.asset_name)?;
    let amount = coerce_expr_into_number(&ir.amount)?;
    let amount = conway::NonZeroInt::try_from(amount as i64).unwrap();

    let asset = asset!(policy, asset_name.clone(), amount);

    Ok(asset)
}

fn compile_ada_value(ir: &ir::AssetExpr) -> Result<conway::Value, Error> {
    let amount = coerce_expr_into_number(&ir.amount)?;

    Ok(value!(amount as u64))
}

fn compile_value(ir: &ir::AssetExpr) -> Result<conway::Value, Error> {
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
fn eval_minutxo_constructor(&self, ctr: &ir::AssetConstructor) -> Result<conway::Value, Error> {
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
) -> Result<conway::TransactionOutput, Error> {
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

    let datum_option = ir
        .datum
        .as_ref()
        .map(|x| compile_data_expr(x))
        .transpose()?;

    let output = conway::TransactionOutput::PostAlonzo(conway::PostAlonzoTransactionOutput {
        address: address.to_vec().into(),
        value,
        datum_option: datum_option.map(|x| conway::DatumOption::Data(CborWrap(x))),
        script_ref: None, // TODO: add script ref
    });

    Ok(output)
}

fn compile_mint_block(tx: &ir::Tx) -> Result<Option<conway::Mint>, Error> {
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

fn compile_inputs(tx: &ir::Tx) -> Result<Vec<conway::TransactionInput>, Error> {
    let refs = tx
        .inputs
        .iter()
        .map(coerce_expr_into_utxo_refs)
        .collect::<Result<Vec<_>, _>>()?;

    let inputs = refs
        .iter()
        .flatten()
        .map(|x| conway::TransactionInput {
            transaction_id: x.txid.as_slice().into(),
            index: x.index as u64,
        })
        .collect();

    Ok(inputs)
}

fn compile_outputs(
    tx: &ir::Tx,
    pparams: &PParams,
) -> Result<Vec<conway::TransactionOutput>, Error> {
    let blocks = tx.outputs.iter().cloned().collect::<Vec<_>>();

    let resolved = blocks
        .iter()
        .map(|x| compile_output_block(x, pparams))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(resolved)
}

fn compile_vote_delegation_certificate(
    x: &ir::AdHocDirective,
) -> Result<conway::Certificate, Error> {
    let stake = coerce_expr_into_stake_credential(&x.data["stake"])?;
    let drep = coerce_expr_into_bytes(&x.data["drep"])?;
    let drep = conway::DRep::Key(drep.as_slice().into());

    Ok(conway::Certificate::VoteDeleg(stake, drep))
}

fn compile_certs(tx: &ir::Tx) -> Result<Vec<conway::Certificate>, Error> {
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

fn compile_tx_body(tx: &ir::Tx, pparams: &PParams) -> Result<conway::TransactionBody, Error> {
    let out = conway::TransactionBody {
        inputs: compile_inputs(tx)?.into(),
        outputs: compile_outputs(tx, pparams)?.into(),
        fee: coerce_expr_into_number(&tx.fees)? as u64,
        ttl: None,
        validity_interval_start: None,
        certificates: conway::NonEmptySet::from_vec(compile_certs(tx)?),
        withdrawals: None,
        auxiliary_data_hash: None,
        mint: compile_mint_block(tx)?,
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

fn compile_auxiliary_data(tx: &ir::Tx) -> Result<Option<conway::AuxiliaryData>, Error> {
    // Ok(Some(conway::AuxiliaryData::PostAlonzo(
    //     pallas::ledger::primitives::alonzo::PostAlonzoAuxiliaryData {
    //         metadata: None,
    //         native_scripts: None,
    //         plutus_scripts: None,
    //     },
    // )))

    Ok(None)
}

fn compile_mint_redeemers(tx: &ir::Tx) -> Result<Vec<conway::Redeemer>, Error> {
    // TODO
    Ok(vec![])
}

fn compile_redeemers(tx: &ir::Tx) -> Result<Option<conway::Redeemers>, Error> {
    let mint_redeemers = compile_mint_redeemers(tx)?;

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

fn compile_witness_set(tx: &ir::Tx) -> Result<conway::WitnessSet, Error> {
    let out = conway::WitnessSet {
        redeemer: compile_redeemers(tx)?,
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

pub fn compile_tx(tx: &ir::Tx, pparams: &PParams) -> Result<conway::Tx, Error> {
    let transaction_body = compile_tx_body(tx, pparams)?;
    let transaction_witness_set = compile_witness_set(tx)?;
    let auxiliary_data = compile_auxiliary_data(tx)?;

    let tx = conway::Tx {
        transaction_body,
        transaction_witness_set,
        auxiliary_data: auxiliary_data.into(),
        success: true,
    };

    Ok(tx)
}

pub fn eval_pass(tx: &conway::Tx, pparams: &PParams) -> Result<TxEval, Error> {
    let payload = pallas::codec::minicbor::to_vec(tx).unwrap();
    let fee =
        (payload.len() as u64 * pparams.min_fee_coefficient) + pparams.min_fee_constant + 200_000;

    Ok(TxEval {
        payload,
        fee,
        ex_units: 0,
    })
}

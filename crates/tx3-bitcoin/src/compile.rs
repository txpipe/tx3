use bitcoin::{
    absolute::LockTime, hashes::Hash, transaction::Version, OutPoint, ScriptBuf, Sequence,
    Transaction, TxIn, TxOut,
};

use tx3_lang::ir;

use crate::Error;

pub fn expr_into_amount(expr: &ir::Expression) -> Result<bitcoin::Amount, Error> {
    match expr {
        ir::Expression::Number(x) => Ok(bitcoin::Amount::from_sat(*x as u64)),
        ir::Expression::Assets(x) if x.len() == 1 => expr_into_amount(&x[0].amount),
        _ => Err(Error::CoerceError(
            format!("{:?}", expr),
            "Number".to_string(),
        )),
    }
}

fn compile_single_output(output: &ir::Output) -> Result<TxOut, Error> {
    let value = output
        .amount
        .as_ref()
        .map_or(Err(Error::MissingAmount), expr_into_amount)?;

    let script_pubkey = output
        .address
        .as_ref()
        .map_or(Err(Error::MissingAddress), expr_into_script_pubkey)?;

    Ok(TxOut {
        value,
        script_pubkey,
    })
}

fn compile_outputs(tx: &ir::Tx) -> Result<Vec<TxOut>, Error> {
    tx.outputs.iter().map(compile_single_output).collect()
}

fn compile_inputs(tx: &ir::Tx) -> Result<Vec<TxIn>, Error> {
    tx.inputs
        .iter()
        .flat_map(|input| input.refs.iter())
        .map(|ref_| {
            let txid = Hash::hash(ref_.txid.as_slice());
            let vout = ref_.index as u32;

            TxIn {
                previous_output: OutPoint::new(txid, vout),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: Default::default(),
            }
        })
        .collect()
}

pub fn compile_tx(tx: &ir::Tx, pparams: &PParams) -> Result<Transaction, Error> {
    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: compile_inputs(tx)?,
        output: compile_outputs(tx)?,
    };

    Ok(tx)
}

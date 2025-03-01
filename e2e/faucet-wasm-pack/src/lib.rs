mod utils;

use wasm_bindgen::prelude::*;

tx3_lang::include_tx3_build!("faucet");

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub async fn greet() -> Vec<u8> {
    let params = ClaimWithPasswordParams {
        password: "password".as_bytes().to_vec(),
        quantity: 100,
        requester: tx3_lang::ArgValue::String("addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x".to_string()),
    };

    let tx = new_claim_with_password_tx(params).unwrap();

    let ledger = tx3_cardano::ledgers::mock::MockLedger;

    let eval = tx3_cardano::resolve_tx(tx, ledger, 3).await.unwrap();

    eval.payload
}

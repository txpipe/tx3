fn main() {
    let mut p = tx3_lang::Protocol::load_file("protocol.tx3").unwrap();
    p.analyze().unwrap();
    let ir = p.new_tx("claim_with_password").unwrap().ir_bytes();

    println!("{}", hex::encode(ir));
}

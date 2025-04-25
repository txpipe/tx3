const BASE_TEMPLATE: &str = include_str!("../templates/rust.hbs");

fn rust_type_for_field(ty: &tx3_lang::ir::Type) -> String {
    match ty {
        tx3_lang::ir::Type::Int => "i64".to_string(),
        tx3_lang::ir::Type::Bool => "bool".to_string(),
        tx3_lang::ir::Type::Bytes => "Vec<u8>".to_string(),
        tx3_lang::ir::Type::Unit => "()".to_string(),
        tx3_lang::ir::Type::Address => "tx3_lang::ArgValue".to_string(),
        tx3_lang::ir::Type::UtxoRef => "tx3_lang::ArgValue".to_string(),
        tx3_lang::ir::Type::Custom(name) => name.clone(),
        tx3_lang::ir::Type::AnyAsset => "tx3_lang::ArgValue".to_string(),
        tx3_lang::ir::Type::Undefined => unreachable!(),
    }
}

pub fn generate(job: &crate::Job) {
    crate::gen::execute(
        job,
        ("rust", BASE_TEMPLATE),
        Some(crate::gen::FormatOptions::new(
            None, Some(convert_case::Case::Snake), None
        )),
        |ty| rust_type_for_field(ty),
        "rs",
    );
}

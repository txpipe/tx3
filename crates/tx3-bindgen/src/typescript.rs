const BASE_TEMPLATE: &str = include_str!("../templates/typescript.hbs");

fn ts_type_for_field(ty: &tx3_lang::ir::Type) -> &'static str {
    match ty {
        tx3_lang::ir::Type::Int => "number",
        tx3_lang::ir::Type::Address => "string",
        tx3_lang::ir::Type::Bool => "boolean",
        tx3_lang::ir::Type::Bytes => "Uint8Array",
        tx3_lang::ir::Type::UtxoRef => "string",
        // Add other type mappings as needed
        _ => "unknown",
    }
}

pub fn generate(job: &crate::Job) {
    crate::gen::execute(
        job,
        ("typescript", BASE_TEMPLATE),
        None,
        |ty| ts_type_for_field(ty).to_string(),
        "ts"
    );
}

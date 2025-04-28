const BASE_TEMPLATE: &str = include_str!("../templates/golang.hbs");

fn go_type_for_field(ty: &tx3_lang::ir::Type) -> String {
    match ty {
        tx3_lang::ir::Type::Int => "int64".to_string(),
        tx3_lang::ir::Type::Bool => "bool".to_string(),
        tx3_lang::ir::Type::Bytes => "[]byte".to_string(),
        tx3_lang::ir::Type::Unit => "struct{}".to_string(),
        tx3_lang::ir::Type::Address => "string".to_string(),
        tx3_lang::ir::Type::UtxoRef => "string".to_string(),
        tx3_lang::ir::Type::Custom(name) => name.clone(),
        tx3_lang::ir::Type::AnyAsset => "string".to_string(),
        tx3_lang::ir::Type::Undefined => unreachable!(),
    }
}

pub fn generate(job: &crate::Job) {
    crate::gen::execute(
        job,
        ("golang", BASE_TEMPLATE),
        Some(crate::gen::FormatOptions::new(
            None, Some(convert_case::Case::Pascal), None, Some(convert_case::Case::Pascal)
        )),
        |ty| go_type_for_field(ty),
        "go",
    );
}
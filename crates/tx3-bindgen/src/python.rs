const BASE_TEMPLATE: &str = include_str!("../templates/python.hbs");

fn python_type_for_field(ty: &tx3_lang::ir::Type) -> String {
    match ty {
        tx3_lang::ir::Type::Int => "int".to_string(),
        tx3_lang::ir::Type::Bool => "bool".to_string(),
        tx3_lang::ir::Type::Bytes => "bytes".to_string(),
        tx3_lang::ir::Type::Unit => "None".to_string(),
        tx3_lang::ir::Type::Address => "str".to_string(),
        tx3_lang::ir::Type::UtxoRef => "str".to_string(),
        tx3_lang::ir::Type::Custom(name) => name.clone(),
        tx3_lang::ir::Type::AnyAsset => "str".to_string(),
        tx3_lang::ir::Type::Undefined => unreachable!(),
    }
}

pub fn generate(job: &crate::Job) {
    crate::gen::execute(
        job,
        ("python", BASE_TEMPLATE),
        Some(crate::gen::FormatOptions::new(
            None, Some(convert_case::Case::Snake), None, None
        )),
        |ty| python_type_for_field(ty),
        "py",
    );
}
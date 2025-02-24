use proc_macro2::Ident;
use quote::{format_ident, quote};

fn to_syn_type(ty: &tx3_lang::ast::Type) -> syn::Type {
    match ty {
        tx3_lang::ast::Type::Int => syn::parse_str("i64").unwrap(),
        tx3_lang::ast::Type::Bool => syn::parse_str("bool").unwrap(),
        tx3_lang::ast::Type::Bytes => syn::parse_str("Vec<u8>").unwrap(),
        tx3_lang::ast::Type::Custom(name) => syn::parse_str(&format!("{}", name.value)).unwrap(),
    }
}

/// Runs the build-time configuration for the dependent crate
pub fn compile_tx3(path: &str) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", path);

    let mut ast = tx3_lang::parse_file(path).unwrap();
    tx3_lang::analyze(&mut ast).unwrap();

    // Create output file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let filename = path.replace(".tx3", ".rs");
    let dest_path = std::path::Path::new(&out_dir).join(filename);
    let mut output = String::new();

    for tx in ast.txs.iter() {
        let ir = tx3_lang::lower(&ast, &tx.name).unwrap();
        let ir_bytes: Vec<u8> = bincode::serialize(&ir).unwrap();

        let ir_name = format_ident!("IR_{}", tx.name.to_uppercase());

        // Convert bytes to a byte string literal
        let ir_bytes_literal = syn::LitByteStr::new(&ir_bytes, proc_macro2::Span::call_site());

        let fn_name = format_ident!("execute_{}", tx.name.to_lowercase());

        let param_names: Vec<Ident> = tx
            .parameters
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();

        let param_types: Vec<syn::Type> = tx
            .parameters
            .parameters
            .iter()
            .map(|p| to_syn_type(&p.r#type))
            .collect();

        let tokens = quote! {
            pub const #ir_name: &[u8] = #ir_bytes_literal;

            pub fn #fn_name(#(#param_names: #param_types),*) -> Result<ProtoTx, VMError> {

            }
        };

        output.push_str(&tokens.to_string());
        output.push_str("\n\n");
    }

    std::fs::write(dest_path, output).unwrap();
}

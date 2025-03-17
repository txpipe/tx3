use convert_case::Casing;
use proc_macro2::Ident;
use quote::{format_ident, quote};

use crate::Job;

fn to_syn_type(ty: &tx3_lang::ir::Type) -> syn::Type {
    match ty {
        tx3_lang::ir::Type::Int => syn::parse_str("i64").unwrap(),
        tx3_lang::ir::Type::Bool => syn::parse_str("bool").unwrap(),
        tx3_lang::ir::Type::Bytes => syn::parse_str("Vec<u8>").unwrap(),
        tx3_lang::ir::Type::Address => syn::parse_str("tx3_lang::ArgValue").unwrap(),
        tx3_lang::ir::Type::UtxoRef => syn::parse_str("tx3_lang::ArgValue").unwrap(),
        tx3_lang::ir::Type::Custom(name) => syn::parse_str(name).unwrap(),
    }
}

pub fn generate(job: &Job) {
    let mut output = String::new();

    for tx_def in job.protocol.txs() {
        let proto_tx = job.protocol.new_tx(&tx_def.name).unwrap();
        let proto_bytes: Vec<u8> = proto_tx.ir_bytes();

        let bytes_name = format_ident!("PROTO_{}", tx_def.name.to_uppercase());

        let struct_name =
            format_ident!("{}Params", tx_def.name.to_case(convert_case::Case::Pascal));

        // Convert bytes to a byte string literal
        let proto_bytes_literal =
            syn::LitByteStr::new(&proto_bytes, proc_macro2::Span::call_site());

        let fn_name = format_ident!("new_{}_tx", tx_def.name.to_lowercase());

        let param_names: Vec<Ident> = proto_tx
            .find_params()
            .keys()
            .map(|name| format_ident!("{}", name.to_case(convert_case::Case::Snake)))
            .collect();

        let param_types: Vec<syn::Type> =
            proto_tx.find_params().values().map(to_syn_type).collect();

        let tokens = quote! {
            pub const #bytes_name: &[u8] = #proto_bytes_literal;

            pub struct #struct_name {
                #(#param_names: #param_types),*
            }

            pub fn #fn_name(params: #struct_name) -> Result<tx3_lang::ProtoTx, tx3_lang::applying::Error> {
                let mut proto_tx = tx3_lang::ProtoTx::from_ir_bytes(#bytes_name).unwrap();

                #(proto_tx.set_arg(stringify!(#param_names), params.#param_names.into());)*

                proto_tx.apply()
            }
        };

        output.push_str(&tokens.to_string());
        output.push_str("\n\n");
    }

    std::fs::write(job.dest_path.join(format!("{}.rs", job.name)), output).unwrap();
}

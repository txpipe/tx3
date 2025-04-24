use convert_case::{Case, Casing};
use handlebars::Handlebars;
use serde::{Serialize, Serializer};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

use super::Job;

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

#[derive(Serialize)]
struct TxParameter {
    name: String,
    type_name: String,
}

struct BytesHex(Vec<u8>);

impl Serialize for BytesHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

#[derive(Serialize)]
struct Transaction {
    name: String,
    pascal_name: String,
    function_name: String,
    constant_name: String,
    ir_bytes: BytesHex,
    parameters: Vec<TxParameter>,
}

pub fn generate(job: &Job) {
    let mut handlebars = Handlebars::new();
    
    // Register the main template for the TypeScript file
    handlebars.register_template_string("typescript", BASE_TEMPLATE)
        .expect("Failed to register typescript template");
    
    // Get each transaction from the API
    let transactions: Vec<Transaction> = job.protocol.txs()
        .map(|tx_def| {
            let tx_name = tx_def.name.as_str();
            let proto_tx = job.protocol.new_tx(&tx_def.name).unwrap();
            
            // Generate names in different formats
            let pascal_name = format!("{}Params", tx_name.to_case(Case::Pascal));
            let function_name = format!("{}Tx", tx_name.to_case(Case::Camel));
            let constant_name = format!("{}_IR", tx_name.to_case(Case::Constant));
            
            // Create the list of parameters
            let parameters = proto_tx.find_params()
                .iter()
                .map(|(key, type_)| {
                    TxParameter {
                        name: key.as_str().to_case(Case::Camel),
                        type_name: ts_type_for_field(type_).to_string(),
                    }
                })
                .collect();
            
            Transaction {
                name: tx_name.to_string(),
                pascal_name,
                function_name,
                constant_name,
                ir_bytes: BytesHex(proto_tx.ir_bytes()),
                parameters,
            }
        })
        .collect();
    
    // Convert headers and env_args for the template
    let headers: JsonValue = serde_json::to_value(
        job.trp_headers.iter()
            .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
            .collect::<HashMap<_, _>>()
    ).expect("Failed to convert headers to JSON");
    
    let env_args: JsonValue = serde_json::to_value(
        job.env_args.iter()
            .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
            .collect::<HashMap<_, _>>()
    ).expect("Failed to convert env_args to JSON");
    
    // Create the context for the template
    let data = json!({
        "trpEndpoint": job.trp_endpoint,
        "headers": headers,
        "envArgs": env_args,
        "transactions": transactions,
    });
    
    // Render the template with the context
    let output = handlebars.render("typescript", &data)
        .expect("Failed to render typescript template");
    
    // Check if destination directory exists, create it if it doesn't
    if !job.dest_path.exists() {
        std::fs::create_dir_all(&job.dest_path)
            .expect("Failed to create destination directory");
    }

    std::fs::write(job.dest_path.join(format!("{}.ts", job.name)), output)
        .expect("Failed to write TypeScript output");
}

use std::collections::HashMap;

use convert_case::{Case, Casing};
use serde::{Serialize, Serializer};
use handlebars::Handlebars;
use serde_json::{json, Value as JsonValue};

use super::Job;

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
    params_name: String,
    function_name: String,
    constant_name: String,
    ir_bytes: BytesHex,
    parameters: Vec<TxParameter>,
}

pub struct FormatOptions<'a> {
    param_case: Case<'a>,
    function_case: Case<'a>,
    constant_case: Case<'a>,
    parameters_case: Case<'a>,
}

impl<'a> Default for FormatOptions<'a> {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}

impl<'a> FormatOptions<'a> {
    pub fn new(
        param_case: Option<Case<'a>>,
        function_case: Option<Case<'a>>,
        constant_case: Option<Case<'a>>,
        parameters_case: Option<Case<'a>>,
    ) -> Self {
        Self {
            param_case: param_case.unwrap_or(Case::Pascal),
            function_case: function_case.unwrap_or(Case::Camel),
            constant_case: constant_case.unwrap_or(Case::Constant),
            parameters_case: parameters_case.unwrap_or(Case::Camel),
        }
    }
}

pub fn execute<'a>(
    job: &Job,
    template: (&str, &str),
    format_options: Option<FormatOptions<'a>>,
    get_type_for_field: fn(&tx3_lang::ir::Type) -> String,
    output_ext: &str,
) {
    let mut handlebars = Handlebars::new();

    handlebars.register_template_string(template.0, template.1)
        .expect(&format!("Failed to register {} template", template.0));

    let _format = format_options.unwrap_or_default();

    let transactions: Vec<Transaction> = job.protocol.txs()
        .map(|tx_def| {
            let tx_name = tx_def.name.as_str();
            let proto_tx = job.protocol.new_tx(&tx_def.name).unwrap();

            let params_name = format!("{}Params", tx_name).to_case(_format.param_case);
            let function_name = format!("{}Tx", tx_name).to_case(_format.function_case);
            let constant_name = format!("{}Ir", tx_name).to_case(_format.constant_case);

            let parameters = proto_tx.find_params()
                .iter()
                .map(|(key, type_)| {
                    TxParameter {
                        name: key.as_str().to_case(_format.parameters_case),
                        type_name: get_type_for_field(type_),
                    }
                })
                .collect();

            Transaction {
                name: tx_name.to_string(),
                params_name,
                function_name,
                constant_name,
                ir_bytes: BytesHex(proto_tx.ir_bytes()),
                parameters,
            }
        })
        .collect();

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

    let data = json!({
        "trpEndpoint": job.trp_endpoint,
        "headers": headers,
        "envArgs": env_args,
        "transactions": transactions,
    });

    let output = handlebars.render(template.0, &data)
        .expect(&format!("Failed to render {} template", template.0));

    if !job.dest_path.exists() {
        std::fs::create_dir_all(&job.dest_path)
            .expect("Failed to create destination directory");
    }

    std::fs::write(job.dest_path.join(format!("{}.{}", job.name, output_ext)), output)
        .expect(&format!("Failed to write {} output", template.0));
}
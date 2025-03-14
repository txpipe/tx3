use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use tx3_bindgen::{rust, typescript, Job};

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output directory for generated code
    #[arg(short, long)]
    output: String,

    /// Input tx3 files to process
    #[arg(short, long, required = true, num_args = 1..)]
    input: Vec<String>,

    /// Template to use for code generation (typescript, python, rust, balius)
    #[arg(short, long, value_parser = ["typescript", "python", "rust", "balius"])]
    template: String,

    /// TRP endpoint to use for code generation
    #[arg(short, long)]
    trp_endpoint: Option<String>,

    /// TRP headers to send to the TRP server
    #[arg(short = 'H', long, value_parser = parse_key_val::<String, String>)]
    trp_header: Vec<(String, String)>,

    /// Env args to use for tx resolution
    #[arg(short = 'E', long, value_parser = parse_key_val::<String, String>)]
    env_arg: Vec<(String, String)>,
}

fn main() {
    let cli = Cli::parse();

    for input_file in cli.input {
        let protocol = tx3_lang::Protocol::from_file(&input_file)
            .load()
            .unwrap_or_else(|e| {
                eprintln!("Failed to load protocol file {}: {}", input_file, e);
                std::process::exit(1);
            });

        let input_file = std::path::Path::new(&input_file);

        let job = Job {
            name: input_file
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            protocol,
            dest_path: cli.output.clone().into(),
            trp_endpoint: cli
                .trp_endpoint
                .clone()
                .unwrap_or("http://localhost:3000".to_string()),
            trp_headers: HashMap::from_iter(cli.trp_header.clone()),
            env_args: HashMap::from_iter(cli.env_arg.clone()),
        };

        match cli.template.as_str() {
            "typescript" => typescript::generate(&job),
            "rust" => rust::generate(&job),
            "python" => todo!(),
            "balius" => todo!(),
            _ => unreachable!(), // clap ensures we only get valid values
        }
    }
}

use std::path::Path;

use clap::Parser;
use tx3_bindgen::{rust, typescript};

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

        match cli.template.as_str() {
            "typescript" => typescript::generate(protocol, &Path::new(&cli.output)),
            "rust" => rust::generate(protocol, &Path::new(&cli.output)),
            "python" => todo!(),
            "balius" => todo!(),
            _ => unreachable!(), // clap ensures we only get valid values
        }
    }
}

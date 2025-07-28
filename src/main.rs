mod config;
mod generator;
mod parser;

use clap::Parser as ClapParser;
use config::Config;
use generator::ValidatorGenerator;
use glob::glob;
use parser::TypeScriptParser;
use std::path::{Path, PathBuf};

#[derive(ClapParser, Debug)]
#[command(name = "bagsakan")]
#[command(about = "Generate TypeScript validation functions from interfaces", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "bagsakan.toml")]
    config: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = Config::from_file(&args.config)?;
    println!("Using configuration:");
    println!("  Validator pattern: {}", config.validator_pattern);
    println!("  Source files: {}", config.source_files);
    println!("  Validator file: {}", config.validator_file);
    println!("  Use JS extensions: {}", config.use_js_extensions);

    let pattern_regex = config.get_pattern_regex();
    let mut parser = TypeScriptParser::new(&pattern_regex);

    println!("\nScanning TypeScript files...");
    let mut file_count = 0;

    for entry in glob(&config.source_files)? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    println!("  Parsing: {:?}", path);
                    parser.parse_file(&path)?;
                    file_count += 1;
                }
            }
            Err(e) => eprintln!("Error reading file: {:?}", e),
        }
    }

    println!("\nFound {} TypeScript files", file_count);
    println!("Found {} interfaces", parser.interfaces.len());
    println!(
        "Found {} validator function calls",
        parser.validator_functions.len()
    );

    for (name, interface) in &parser.interfaces {
        println!(
            "\nInterface {}: {} properties",
            name,
            interface.properties.len()
        );
        for prop in &interface.properties {
            println!(
                "  - {}: {} (optional: {})",
                prop.name, prop.type_annotation, prop.optional
            );
        }
    }

    if !parser.validator_functions.is_empty() {
        // Deduplicate validator function names for display
        let unique_validators: std::collections::HashSet<_> = parser
            .validator_functions
            .iter()
            .map(|vf| &vf.name)
            .collect();
        println!(
            "\nFound {} unique validator functions to generate",
            unique_validators.len()
        );

        let generator = ValidatorGenerator::new(parser.interfaces, config.use_js_extensions);
        let output =
            generator.generate_validators(&parser.validator_functions, &config.validator_file);

        let output_path = Path::new(&config.validator_file);
        generator.write_to_file(output_path, &output)?;

        println!(
            "\nGenerated validators written to: {}",
            config.validator_file
        );
    } else {
        println!(
            "\nNo validator function calls found matching pattern: {}",
            config.validator_pattern
        );
    }

    Ok(())
}

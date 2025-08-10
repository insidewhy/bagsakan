mod config;
mod generator;
mod parser;

use clap::{Parser as ClapParser, Subcommand};
use config::Config;
use generator::ValidatorGenerator;
use glob::glob;
use parser::TypeScriptParser;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(ClapParser, Debug)]
#[command(name = "bagsakan")]
#[command(about = "Generate TypeScript validation functions from interfaces", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "bagsakan.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a validator for a specific interface
    Add {
        /// Name of the interface to generate a validator for
        interface_name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    match args.command {
        Some(Commands::Add { interface_name }) => add_interface_validator(&config, &interface_name),
        None => scan_and_generate(&config),
    }
}

fn scan_and_generate(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using configuration:");
    println!("  Validator pattern: {}", config.validator_pattern);
    println!("  Source files: {}", config.source_files);
    println!("  Validator file: {}", config.validator_file);
    println!("  Use JS extensions: {}", config.use_js_extensions);
    println!(
        "  Follow external imports: {}",
        config.follow_external_imports
    );
    if !config.exclude_packages.is_empty() {
        println!("  Excluded packages: {:?}", config.exclude_packages);
    }
    if !config.conditions.is_empty() {
        println!("  Export conditions: {:?}", config.conditions);
    }

    let pattern_regex = config.get_pattern_regex();
    let mut parser = TypeScriptParser::new(
        &pattern_regex,
        config.follow_external_imports,
        config.exclude_packages.clone(),
        config.conditions.clone(),
    );

    println!("\nScanning TypeScript files...");
    let mut file_count = 0;

    // First, collect and mark all source files
    let source_paths: Vec<_> = glob(&config.source_files)?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.is_file())
        .collect();

    // Mark all source files
    for path in &source_paths {
        parser.mark_as_source_file(path);
    }

    // Now parse all source files
    for path in source_paths {
        println!("  Parsing: {:?}", path);
        parser.parse_file(&path)?;
        file_count += 1;
    }

    println!("\nFound {} TypeScript files", file_count);
    println!("Found {} interfaces", parser.interfaces.len());
    println!("Found {} enums", parser.enums.len());
    println!(
        "Found {} validator function calls",
        parser.validator_functions.len()
    );

    if !parser.validator_functions.is_empty() {
        // Get unique interface names that have validators requested
        let requested_interfaces: std::collections::HashSet<_> = parser
            .validator_functions
            .iter()
            .map(|vf| &vf.interface_name)
            .collect();

        // Check for missing interfaces
        let missing_interfaces: Vec<_> = requested_interfaces
            .iter()
            .filter(|name| !parser.interfaces.contains_key(name.as_str()))
            .collect();

        if !missing_interfaces.is_empty() {
            eprintln!("\nError: Cannot generate validators for missing interfaces:");
            for name in &missing_interfaces {
                eprintln!("  - {}", name);

                // Find where it was used
                for vf in &parser.validator_functions {
                    if vf.interface_name.as_str() == name.as_str() {
                        eprintln!("    Used in: {}", vf.name);
                    }
                }
            }
            eprintln!("\nHint: Make sure these interfaces are:");
            eprintln!("  1. Defined in your source files or imported packages");
            eprintln!("  2. Exported from their modules");
            eprintln!("  3. Not in excluded packages");
            eprintln!("\nRun with BAGSAKAN_DEBUG=1 for more details about import resolution.");

            std::process::exit(1);
        }

        println!(
            "\nGenerating {} validators for {} interfaces",
            parser.validator_functions.len(),
            requested_interfaces.len()
        );

        // Only show details for requested interfaces
        for interface_name in &requested_interfaces {
            if let Some(interface) = parser.interfaces.get(*interface_name) {
                println!(
                    "\n  {} ({} properties)",
                    interface_name,
                    interface.properties.len()
                );
            }
        }

        let generator =
            ValidatorGenerator::new(parser.interfaces, parser.enums, config.use_js_extensions);
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

fn add_interface_validator(
    config: &Config,
    interface_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Adding validator for interface: {}", interface_name);
    println!("Using configuration:");
    println!("  Validator file: {}", config.validator_file);
    println!("  Use JS extensions: {}", config.use_js_extensions);
    if config.follow_external_imports {
        println!("  Follow external imports: true");
    }

    // Create a parser to find the interface
    let pattern_regex = config.get_pattern_regex();
    let mut parser = TypeScriptParser::new(
        &pattern_regex,
        config.follow_external_imports,
        config.exclude_packages.clone(),
        config.conditions.clone(),
    );

    println!(
        "\nScanning TypeScript files for interface '{}'...",
        interface_name
    );

    // Scan all source files to find the interface
    let source_paths: Vec<_> = glob(&config.source_files)?
        .filter_map(|entry| entry.ok())
        .filter(|path| path.is_file())
        .collect();

    for path in &source_paths {
        parser.mark_as_source_file(path);
    }

    for path in source_paths {
        parser.parse_file(&path)?;
    }

    // Check if the interface was found
    if !parser.interfaces.contains_key(interface_name) {
        eprintln!("\nError: Interface '{}' not found.", interface_name);
        eprintln!("\nAvailable interfaces:");
        let mut interface_names: Vec<_> = parser.interfaces.keys().collect();
        interface_names.sort();
        for name in interface_names.iter().take(20) {
            eprintln!("  - {}", name);
        }
        if parser.interfaces.len() > 20 {
            eprintln!("  ... and {} more", parser.interfaces.len() - 20);
        }
        std::process::exit(1);
    }

    println!("\nFound interface '{}'", interface_name);

    // Generate the validator function name
    let validator_name = config.validator_pattern.replace("%(type)", interface_name);

    // Read existing validators file if it exists
    let output_path = Path::new(&config.validator_file);
    let existing_content = if output_path.exists() {
        fs::read_to_string(output_path)?
    } else {
        String::new()
    };

    // Check if validator already exists
    if existing_content.contains(&format!("function {}", validator_name)) {
        println!(
            "\nValidator '{}' already exists in {}",
            validator_name, config.validator_file
        );
        return Ok(());
    }

    // Parse existing validators to maintain them
    let mut existing_validators = Vec::new();
    if !existing_content.is_empty() {
        // Extract existing validator functions from the file
        for line in existing_content.lines() {
            if line.starts_with("export function validate") {
                if let Some(name_start) = line.find("validate") {
                    if let Some(paren_pos) = line[name_start..].find('(') {
                        let func_name = &line[name_start..name_start + paren_pos];
                        if let Some(interface_name_match) = func_name.strip_prefix("validate") {
                            existing_validators.push(parser::ValidatorFunction {
                                name: func_name.to_string(),
                                interface_name: interface_name_match.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Add the new validator
    let new_validator = parser::ValidatorFunction {
        name: validator_name.clone(),
        interface_name: interface_name.to_string(),
    };
    existing_validators.push(new_validator);

    // Sort validators alphabetically
    existing_validators.sort_by(|a, b| a.name.cmp(&b.name));

    // Generate the updated validators file
    let generator =
        ValidatorGenerator::new(parser.interfaces, parser.enums, config.use_js_extensions);
    let output = generator.generate_validators(&existing_validators, &config.validator_file);

    // Write the updated file
    generator.write_to_file(output_path, &output)?;

    println!(
        "\nAdded validator '{}' to {}",
        validator_name, config.validator_file
    );
    println!("Total validators in file: {}", existing_validators.len());

    Ok(())
}

mod cli;
mod verify;
mod writer;

use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands, TemplateArg};
use kaido_core::error;
use kaido_core::features;
use kaido_core::generator::ProjectGenerator;
use kaido_core::templates::GenerateOptions;
use verify::{AikenVerifier, AikidoVerifier};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            template,
            namespace,
            project_name,
            output,
            token_name,
            asset_name,
            time_lock,
            cancellable,
            partial_claim,
            features,
            datum,
            redeemer,
            purpose,
            sdk,
            skip_verify,
        } => {
            if let Err(e) = run_generate(
                template,
                &namespace,
                &project_name,
                output,
                token_name,
                asset_name,
                time_lock,
                cancellable,
                partial_claim,
                features,
                datum,
                redeemer,
                &purpose,
                sdk,
                skip_verify,
            ) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::List => {
            run_list();
        }
        Commands::Verify { path } => {
            if let Err(e) = run_verify(&path) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_generate(
    template_arg: TemplateArg,
    namespace: &str,
    project_name: &str,
    output: Option<String>,
    token_name: Option<String>,
    asset_name: Option<String>,
    time_lock: bool,
    cancellable: bool,
    partial_claim: bool,
    feature_strs: Vec<String>,
    datum: Option<String>,
    redeemer: Option<String>,
    purpose: &str,
    sdk: bool,
    skip_verify: bool,
) -> error::Result<()> {
    GenerateOptions::validate_namespace_and_project(namespace, project_name)
        .map_err(error::KaidoError::InvalidOption)?;

    let output_dir = PathBuf::from(output.unwrap_or_else(|| project_name.to_string()));

    println!("{} Generating Aiken project...", "Kaido".cyan().bold());

    let options = match template_arg {
        TemplateArg::Mint => {
            let tn = token_name.unwrap_or_else(|| project_name.to_string());
            let an = asset_name.unwrap_or_else(|| tn.to_uppercase().replace(' ', "_"));
            GenerateOptions::simple_mint(namespace, project_name, &tn, &an, time_lock)
        }
        TemplateArg::Vesting => {
            GenerateOptions::vesting(namespace, project_name, cancellable, partial_claim)
        }
        TemplateArg::Escrow => GenerateOptions::escrow(namespace, project_name),
        TemplateArg::Treasury => GenerateOptions::multisig_treasury(namespace, project_name),
        TemplateArg::Marketplace => GenerateOptions::nft_marketplace(namespace, project_name),
        TemplateArg::Staking => GenerateOptions::staking_pool(namespace, project_name),
        TemplateArg::Oracle => GenerateOptions::oracle_settlement(namespace, project_name),
        TemplateArg::Referral => GenerateOptions::referral_system(namespace, project_name),
        TemplateArg::Dex => GenerateOptions::dex_pool(namespace, project_name),
        TemplateArg::Lending => GenerateOptions::lending_pool(namespace, project_name),
        TemplateArg::Governance => GenerateOptions::dao_governance(namespace, project_name),
        TemplateArg::Streaming => GenerateOptions::streaming_payments(namespace, project_name),
        TemplateArg::Custom => {
            // Validate purpose
            if purpose != "spend" && purpose != "mint" {
                return Err(error::KaidoError::InvalidOption(format!(
                    "Invalid purpose '{}'. Must be 'spend' or 'mint'",
                    purpose
                )));
            }

            // Parse features
            let parsed_features = features::parse_features(&feature_strs)?;
            let resolved = features::compose::resolve_features(&parsed_features, purpose)?;
            let feature_names: Vec<String> =
                resolved.iter().map(|f| f.name().to_string()).collect();

            // Parse datum fields
            let datum_fields = if let Some(ref d) = datum {
                features::types::parse_datum_fields(d)?
            } else if purpose == "spend" {
                return Err(error::KaidoError::InvalidOption(
                    "--datum is required for spend-purpose custom validators".to_string(),
                ));
            } else {
                vec![]
            };

            // Parse redeemer actions
            let redeemer_actions = if let Some(ref r) = redeemer {
                features::types::parse_redeemer_actions(r)?
            } else {
                return Err(error::KaidoError::InvalidOption(
                    "--redeemer is required for custom validators".to_string(),
                ));
            };

            // Validate features against types
            features::types::validate_features_against_types(&resolved, &datum_fields, purpose)?;

            GenerateOptions::custom(
                namespace,
                project_name,
                purpose,
                datum_fields,
                redeemer_actions,
                feature_names,
            )
        }
    };

    let gen = ProjectGenerator::new()?;
    let result = gen.render(&options)?;

    println!(
        "  {} {}",
        "Template:".white().bold(),
        options.template.description()
    );
    println!("  {} {}", "Output:".white().bold(), output_dir.display());
    println!();

    // Write files to disk
    let paths = writer::write_project(&result, &output_dir)?;
    for path in &paths {
        println!("  {} {}", "+".green(), path.display());
    }

    // SDK generation
    if sdk {
        println!();
        println!("{} Generating TypeScript SDK...", "SDK".blue().bold());
        match gen.render_sdk(&options) {
            Ok(sdk_result) => {
                let sdk_paths = writer::write_project(&sdk_result, &output_dir)?;
                for path in &sdk_paths {
                    println!("  {} {}", "+".green(), path.display());
                }
            }
            Err(e) => {
                println!("  {} SDK generation failed: {}", "WARN".yellow().bold(), e);
            }
        }
    }

    println!();

    // Verification
    if !skip_verify {
        if !AikenVerifier::is_available() {
            return Err(error::KaidoError::InvalidOption(
                "aiken not found on PATH (required unless --skip-verify is set)".to_string(),
            ));
        }
        if !AikidoVerifier::is_available() {
            return Err(error::KaidoError::InvalidOption(
                "aikido not found on PATH (required unless --skip-verify is set)".to_string(),
            ));
        }

        println!("{} Running aiken build...", "Verify".yellow().bold());
        match AikenVerifier::build(&output_dir) {
            Ok(()) => {
                println!("  {} aiken build passed", "OK".green().bold());
            }
            Err(e) => {
                println!("  {} aiken build failed: {}", "FAIL".red().bold(), e);
                return Err(e);
            }
        }

        println!("{} Running aiken check...", "Verify".yellow().bold());
        match AikenVerifier::check(&output_dir) {
            Ok(()) => {
                println!(
                    "  {} aiken check passed (all tests green)",
                    "OK".green().bold()
                );
            }
            Err(e) => {
                println!("  {} aiken check failed: {}", "FAIL".red().bold(), e);
                return Err(e);
            }
        }

        println!("{} Running aikido scan...", "Audit".magenta().bold());
        match AikidoVerifier::scan(&output_dir) {
            Ok(result) => {
                if result.findings.is_empty() {
                    println!("  {} no findings", "OK".green().bold());
                } else {
                    println!(
                        "  {} {} finding(s) ({} high/critical)",
                        if result.high_or_critical > 0 {
                            "WARN".red().bold()
                        } else {
                            "INFO".yellow().bold()
                        },
                        result.findings.len(),
                        result.high_or_critical,
                    );
                    for f in &result.findings {
                        println!(
                            "    [{}] {}: {}",
                            f.severity.to_uppercase(),
                            f.detector,
                            f.message
                        );
                    }
                }
            }
            Err(e) => {
                println!("  {} aikido scan failed: {}", "FAIL".red().bold(), e);
                return Err(e);
            }
        }
    } else {
        println!(
            "  {} verification skipped (--skip-verify)",
            "SKIP".yellow().bold()
        );
    }

    println!();
    println!(
        "{} Project generated at {}",
        "Done!".green().bold(),
        output_dir.display()
    );
    println!();
    println!("  Next steps:");
    println!("    cd {}", output_dir.display());
    println!("    aiken check    # run tests");
    println!("    aiken build    # compile to plutus.json");

    Ok(())
}

fn run_list() {
    use kaido_core::templates::Template;

    println!("{} Available Templates", "Kaido".cyan().bold());
    println!();

    for template in Template::all() {
        println!(
            "  {} {}",
            template.slug().white().bold(),
            template.description()
        );
    }

    println!();
    println!("Usage: kaido generate --template <TEMPLATE> --namespace <NS> --project-name <NAME>");
}

fn run_verify(path: &str) -> error::Result<()> {
    let project_dir = PathBuf::from(path);

    if !project_dir.join("aiken.toml").exists() {
        return Err(error::KaidoError::InvalidOption(
            "No aiken.toml found — not an Aiken project".to_string(),
        ));
    }

    if !AikenVerifier::is_available() {
        return Err(error::KaidoError::InvalidOption(
            "aiken not found on PATH".to_string(),
        ));
    }
    if !AikidoVerifier::is_available() {
        return Err(error::KaidoError::InvalidOption(
            "aikido not found on PATH".to_string(),
        ));
    }

    println!("{} Verifying project at {}", "Kaido".cyan().bold(), path);

    println!("  Running aiken build...");
    AikenVerifier::build(&project_dir)?;
    println!("  {} aiken build", "OK".green().bold());

    println!("  Running aiken check...");
    AikenVerifier::check(&project_dir)?;
    println!("  {} aiken check", "OK".green().bold());

    if let Some(version) = AikenVerifier::version() {
        println!("  Compiler: {}", version);
    }

    // Aikido static analysis
    println!();
    println!("{} Running aikido scan...", "Audit".magenta().bold());
    match AikidoVerifier::scan(&project_dir) {
        Ok(result) => {
            if result.findings.is_empty() {
                println!("  {} no findings", "OK".green().bold());
            } else {
                println!(
                    "  {} {} finding(s) ({} high/critical)",
                    if result.high_or_critical > 0 {
                        "WARN".red().bold()
                    } else {
                        "INFO".yellow().bold()
                    },
                    result.findings.len(),
                    result.high_or_critical,
                );
                for f in &result.findings {
                    println!(
                        "    [{}] {}: {}",
                        f.severity.to_uppercase(),
                        f.detector,
                        f.message
                    );
                }
            }
        }
        Err(e) => {
            println!("  {} aikido scan failed: {}", "FAIL".red().bold(), e);
            return Err(e);
        }
    }

    if let Some(version) = AikidoVerifier::version() {
        println!("  Analyzer: {}", version);
    }

    println!();
    println!("{} All checks passed!", "Done!".green().bold());

    Ok(())
}

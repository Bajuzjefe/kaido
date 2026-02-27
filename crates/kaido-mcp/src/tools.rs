use std::fs;
use std::path::PathBuf;
use std::process::Command;

use kaido_core::features;
use kaido_core::generator::ProjectGenerator;
use kaido_core::templates::{GenerateOptions, Template};
use serde_json::Value;

/// MCP tool definitions for tools/list
pub fn tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "kaido_list_templates",
            "description": "List all available Aiken smart contract templates with descriptions",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        serde_json::json!({
            "name": "kaido_generate",
            "description": "Generate an Aiken smart contract project from a security-focused template. Supports 13 templates including mint, vesting, escrow, treasury, marketplace, staking, oracle, referral, dex, lending, governance, streaming, and custom composable validators.",
            "inputSchema": {
                "type": "object",
                "required": ["template", "namespace", "project_name"],
                "properties": {
                    "template": { "type": "string", "description": "Template: mint, vesting, escrow, treasury, marketplace, staking, oracle, referral, dex, lending, governance, streaming, custom" },
                    "namespace": { "type": "string", "description": "Project namespace (e.g., 'myorg')" },
                    "project_name": { "type": "string", "description": "Project name (e.g., 'my-token')" },
                    "output": { "type": "string", "description": "Output directory path" },
                    "token_name": { "type": "string", "description": "Token display name (mint template)" },
                    "asset_name": { "type": "string", "description": "On-chain asset name (mint template)" },
                    "time_lock": { "type": "boolean", "description": "Enable time-lock (mint template)" },
                    "cancellable": { "type": "boolean", "description": "Allow cancellation (vesting)" },
                    "partial_claim": { "type": "boolean", "description": "Allow partial claims (vesting)" },
                    "features": { "type": "string", "description": "Composable features CSV (custom template)" },
                    "datum": { "type": "string", "description": "Datum fields (custom, e.g., 'owner:ByteArray,amount:Int')" },
                    "redeemer": { "type": "string", "description": "Redeemer actions (custom, e.g., 'Claim,Cancel')" },
                    "purpose": { "type": "string", "description": "Validator purpose: 'spend' or 'mint' (custom)" },
                    "sdk": { "type": "boolean", "description": "Generate TypeScript SDK" },
                    "skip_verify": { "type": "boolean", "description": "Skip aiken/aikido verification" }
                }
            }
        }),
        serde_json::json!({
            "name": "kaido_verify",
            "description": "Verify an existing Aiken project compiles and passes tests. Runs aiken build, aiken check, and aikido static analysis.",
            "inputSchema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string", "description": "Path to the Aiken project directory" }
                }
            }
        }),
    ]
}

/// Dispatch a tool call by name
pub fn call_tool(name: &str, arguments: &Value) -> Result<String, String> {
    match name {
        "kaido_list_templates" => Ok(list_templates()),
        "kaido_generate" => Ok(generate(arguments)),
        "kaido_verify" => {
            let path = arguments
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            Ok(verify(path))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn list_templates() -> String {
    let templates: Vec<Value> = Template::all()
        .iter()
        .map(|t| {
            serde_json::json!({
                "slug": t.slug(),
                "description": t.description(),
            })
        })
        .collect();

    serde_json::to_string_pretty(&templates).unwrap_or_else(|_| "[]".to_string())
}

fn skip_verify_arg(args: &Value) -> bool {
    args.get("skip_verify")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn run_command(
    project_dir: &PathBuf,
    bin: &str,
    args: &[&str],
) -> Result<std::process::Output, String> {
    Command::new(bin)
        .args(args)
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to run '{} {}': {}", bin, args.join(" "), e))
}

fn parse_aikido_scan(
    success: bool,
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> Result<Value, String> {
    if stdout.trim().is_empty() {
        return Err(format!(
            "aikido returned empty output (exit {:?}). stderr:\n{}",
            code, stderr
        ));
    }

    let parsed: Value = serde_json::from_str(stdout).map_err(|e| {
        format!(
            "Failed to parse aikido JSON output: {}\nstdout:\n{}\nstderr:\n{}",
            e, stdout, stderr
        )
    })?;

    let findings_count = parsed
        .get("findings")
        .and_then(|v| v.as_array())
        .map(|v| v.len())
        .unwrap_or(0);
    if !success && findings_count == 0 {
        return Err(format!(
            "aikido exited non-zero ({:?}) without findings.\nstdout:\n{}\nstderr:\n{}",
            code, stdout, stderr
        ));
    }

    Ok(parsed)
}

fn run_verification(project_dir: &PathBuf) -> Result<Value, String> {
    let mut verification = serde_json::Map::new();

    let build = run_command(project_dir, "aiken", &["build"])?;
    if !build.status.success() {
        return Err(format!(
            "aiken build failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        ));
    }
    verification.insert("aiken_build".to_string(), serde_json::json!("passed"));

    let check = run_command(project_dir, "aiken", &["check"])?;
    if !check.status.success() {
        return Err(format!(
            "aiken check failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        ));
    }
    verification.insert("aiken_check".to_string(), serde_json::json!("passed"));

    let scan = run_command(
        project_dir,
        "aikido",
        &[".", "--format", "json", "--quiet", "--fail-on", "high"],
    )?;
    let scan_json = parse_aikido_scan(
        scan.status.success(),
        scan.status.code(),
        &String::from_utf8_lossy(&scan.stdout),
        &String::from_utf8_lossy(&scan.stderr),
    )?;
    verification.insert("aikido_scan".to_string(), scan_json);

    Ok(Value::Object(verification))
}

fn generate(args: &Value) -> String {
    let template = args.get("template").and_then(|v| v.as_str()).unwrap_or("");
    let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
    let project_name = args
        .get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let output = args.get("output").and_then(|v| v.as_str());
    let token_name = args.get("token_name").and_then(|v| v.as_str());
    let asset_name = args.get("asset_name").and_then(|v| v.as_str());
    let time_lock = args
        .get("time_lock")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let cancellable = args
        .get("cancellable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let partial_claim = args
        .get("partial_claim")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let features_str = args.get("features").and_then(|v| v.as_str());
    let datum = args.get("datum").and_then(|v| v.as_str());
    let redeemer = args.get("redeemer").and_then(|v| v.as_str());
    let purpose = args
        .get("purpose")
        .and_then(|v| v.as_str())
        .unwrap_or("spend");
    let sdk = args.get("sdk").and_then(|v| v.as_bool()).unwrap_or(false);
    let skip_verify = skip_verify_arg(args);

    let output_dir = PathBuf::from(output.unwrap_or(project_name));

    let options = match build_options(
        template,
        namespace,
        project_name,
        token_name,
        asset_name,
        time_lock,
        cancellable,
        partial_claim,
        features_str,
        datum,
        redeemer,
        purpose,
    ) {
        Ok(opts) => opts,
        Err(e) => return serde_json::json!({"error": e}).to_string(),
    };

    let gen = match ProjectGenerator::new() {
        Ok(g) => g,
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    let result = match gen.render(&options) {
        Ok(r) => r,
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    // Write files to disk
    let mut written_paths = Vec::new();
    for file in &result.files {
        let full_path = output_dir.join(&file.path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&full_path, &file.content) {
            return serde_json::json!({"error": format!("Write failed: {}", e)}).to_string();
        }
        written_paths.push(file.path.clone());
    }

    // SDK generation
    let mut sdk_paths = Vec::new();
    if sdk {
        if let Ok(sdk_result) = gen.render_sdk(&options) {
            for file in &sdk_result.files {
                let full_path = output_dir.join(&file.path);
                if let Some(parent) = full_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(&full_path, &file.content);
                sdk_paths.push(file.path.clone());
            }
        }
    }

    // Verification
    let verification = if skip_verify {
        serde_json::json!(null)
    } else {
        match run_verification(&output_dir) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::json!({
                    "error": format!("verification failed: {}", e),
                    "template": options.template.slug(),
                    "output_dir": output_dir.display().to_string(),
                    "files": written_paths,
                    "sdk_files": sdk_paths,
                })
                .to_string()
            }
        }
    };

    serde_json::json!({
        "template": options.template.slug(),
        "output_dir": output_dir.display().to_string(),
        "files": written_paths,
        "sdk_files": sdk_paths,
        "verification": verification,
    })
    .to_string()
}

fn verify(path: &str) -> String {
    let project_dir = PathBuf::from(path);

    if !project_dir.join("aiken.toml").exists() {
        return serde_json::json!({"error": "No aiken.toml found"}).to_string();
    }

    match run_verification(&project_dir) {
        Ok(v) => serde_json::json!({ "ok": true, "verification": v }).to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "error": format!("verification failed: {}", e)
        })
        .to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_options(
    template: &str,
    namespace: &str,
    project_name: &str,
    token_name: Option<&str>,
    asset_name: Option<&str>,
    time_lock: bool,
    cancellable: bool,
    partial_claim: bool,
    features_str: Option<&str>,
    datum: Option<&str>,
    redeemer: Option<&str>,
    purpose: &str,
) -> Result<GenerateOptions, String> {
    GenerateOptions::validate_namespace_and_project(namespace, project_name)?;

    match template {
        "mint" | "simple_mint" | "simple-mint" => {
            let tn = token_name.unwrap_or(project_name);
            let default_an = tn.to_uppercase().replace(' ', "_");
            let an = asset_name.unwrap_or(&default_an);
            Ok(GenerateOptions::simple_mint(
                namespace,
                project_name,
                tn,
                an,
                time_lock,
            ))
        }
        "vesting" => Ok(GenerateOptions::vesting(
            namespace,
            project_name,
            cancellable,
            partial_claim,
        )),
        "escrow" => Ok(GenerateOptions::escrow(namespace, project_name)),
        "treasury" | "multisig_treasury" => {
            Ok(GenerateOptions::multisig_treasury(namespace, project_name))
        }
        "marketplace" | "nft_marketplace" => {
            Ok(GenerateOptions::nft_marketplace(namespace, project_name))
        }
        "staking" | "staking_pool" => Ok(GenerateOptions::staking_pool(namespace, project_name)),
        "oracle" | "oracle_settlement" => {
            Ok(GenerateOptions::oracle_settlement(namespace, project_name))
        }
        "referral" | "referral_system" => {
            Ok(GenerateOptions::referral_system(namespace, project_name))
        }
        "dex" | "dex_pool" => Ok(GenerateOptions::dex_pool(namespace, project_name)),
        "lending" | "lending_pool" => Ok(GenerateOptions::lending_pool(namespace, project_name)),
        "governance" | "dao_governance" => {
            Ok(GenerateOptions::dao_governance(namespace, project_name))
        }
        "streaming" | "streaming_payments" => {
            Ok(GenerateOptions::streaming_payments(namespace, project_name))
        }
        "custom" => {
            if purpose != "spend" && purpose != "mint" {
                return Err(format!(
                    "Invalid purpose '{}'. Must be 'spend' or 'mint'",
                    purpose
                ));
            }

            let feature_strs: Vec<String> = features_str
                .unwrap_or("")
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            let parsed = features::parse_features(&feature_strs).map_err(|e| e.to_string())?;
            let resolved =
                features::compose::resolve_features(&parsed, purpose).map_err(|e| e.to_string())?;
            let names: Vec<String> = resolved.iter().map(|f| f.name().to_string()).collect();

            let datum_fields = if let Some(d) = datum {
                features::types::parse_datum_fields(d).map_err(|e| e.to_string())?
            } else if purpose == "spend" {
                return Err("datum is required for spend-purpose custom validators".to_string());
            } else {
                vec![]
            };

            let redeemer_actions = if let Some(r) = redeemer {
                features::types::parse_redeemer_actions(r).map_err(|e| e.to_string())?
            } else {
                return Err("redeemer is required for custom validators".to_string());
            };

            features::types::validate_features_against_types(&resolved, &datum_fields, purpose)
                .map_err(|e| e.to_string())?;

            Ok(GenerateOptions::custom(
                namespace,
                project_name,
                purpose,
                datum_fields,
                redeemer_actions,
                names,
            ))
        }
        _ => Err(format!(
            "Unknown template '{}'. Use kaido_list_templates to see available.",
            template
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_aikido_scan, skip_verify_arg};

    #[test]
    fn skip_verify_defaults_to_false() {
        let args = serde_json::json!({});
        assert!(!skip_verify_arg(&args));
    }

    #[test]
    fn skip_verify_reads_explicit_true() {
        let args = serde_json::json!({ "skip_verify": true });
        assert!(skip_verify_arg(&args));
    }

    #[test]
    fn parse_aikido_scan_accepts_non_zero_with_findings() {
        let stdout =
            r#"{"findings":[{"detector":"d","severity":"critical","description":"x"}],"total":1}"#;
        let parsed = parse_aikido_scan(false, Some(2), stdout, "").expect("must parse");
        assert_eq!(
            parsed["findings"].as_array().expect("findings array").len(),
            1
        );
    }

    #[test]
    fn parse_aikido_scan_rejects_empty_output() {
        let err = parse_aikido_scan(false, Some(1), "", "stderr").expect_err("must fail");
        assert!(err.contains("empty output"));
    }
}

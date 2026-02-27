use wasm_bindgen::prelude::*;

use crate::features::{self, Feature};
use crate::generator::ProjectGenerator;
use crate::templates::{GenerateOptions, Template};

/// List all available templates as JSON
#[wasm_bindgen]
pub fn list_templates() -> String {
    let templates: Vec<serde_json::Value> = Template::all()
        .iter()
        .map(|t| {
            serde_json::json!({
                "slug": t.slug(),
                "description": t.description(),
                "supports_sdk": t.supports_sdk(),
            })
        })
        .collect();

    serde_json::to_string(&templates).unwrap_or_else(|_| "[]".to_string())
}

/// Get detailed info for a specific template
#[wasm_bindgen]
pub fn get_template_info(slug: &str) -> String {
    let template = match slug.parse::<Template>() {
        Ok(t) => t,
        Err(_) => {
            return serde_json::json!({"error": format!("Unknown template: {}", slug)}).to_string()
        }
    };

    let options: Vec<&str> = match template {
        Template::SimpleMint => vec!["token_name", "asset_name", "time_lock"],
        Template::Vesting => vec!["cancellable", "partial_claim"],
        Template::Custom => vec!["purpose", "features", "datum", "redeemer"],
        _ => vec![],
    };

    serde_json::json!({
        "slug": template.slug(),
        "description": template.description(),
        "options": options,
        "supports_sdk": template.supports_sdk(),
    })
    .to_string()
}

/// List all composable features as JSON
#[wasm_bindgen]
pub fn list_features() -> String {
    let features: Vec<serde_json::Value> = Feature::all()
        .iter()
        .map(|f| {
            serde_json::json!({
                "name": f.name(),
                "description": f.description(),
                "purpose": f.purpose_restriction(),
                "depends_on": f.depends_on().iter().map(|d| d.name()).collect::<Vec<_>>(),
            })
        })
        .collect();

    serde_json::to_string(&features).unwrap_or_else(|_| "[]".to_string())
}

/// Generate an Aiken project from options JSON. Returns [{path, content}].
#[wasm_bindgen]
pub fn generate(options_json: &str) -> Result<String, String> {
    let args: serde_json::Value =
        serde_json::from_str(options_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let template = args.get("template").and_then(|v| v.as_str()).unwrap_or("");
    let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
    let project_name = args
        .get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let options = build_options(&args, template, namespace, project_name)?;

    let gen = ProjectGenerator::new().map_err(|e| e.to_string())?;
    let result = gen.render(&options).map_err(|e| e.to_string())?;

    let files: Vec<serde_json::Value> = result
        .files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path,
                "content": f.content,
            })
        })
        .collect();

    serde_json::to_string(&files).map_err(|e| e.to_string())
}

/// Generate TypeScript SDK files. Returns [{path, content}].
#[wasm_bindgen]
pub fn generate_sdk(options_json: &str) -> Result<String, String> {
    let args: serde_json::Value =
        serde_json::from_str(options_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let template = args.get("template").and_then(|v| v.as_str()).unwrap_or("");
    let namespace = args.get("namespace").and_then(|v| v.as_str()).unwrap_or("");
    let project_name = args
        .get("project_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let options = build_options(&args, template, namespace, project_name)?;

    let gen = ProjectGenerator::new().map_err(|e| e.to_string())?;
    let result = gen.render_sdk(&options).map_err(|e| e.to_string())?;

    let files: Vec<serde_json::Value> = result
        .files
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path,
                "content": f.content,
            })
        })
        .collect();

    serde_json::to_string(&files).map_err(|e| e.to_string())
}

/// Validate custom builder options (live validation for the wizard)
#[wasm_bindgen]
pub fn validate_custom(json: &str) -> String {
    let args: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({"valid": false, "errors": [e.to_string()]}).to_string()
        }
    };

    let purpose = args
        .get("purpose")
        .and_then(|v| v.as_str())
        .unwrap_or("spend");
    let features_str = args.get("features").and_then(|v| v.as_str()).unwrap_or("");
    let datum_str = args.get("datum").and_then(|v| v.as_str());
    let redeemer_str = args.get("redeemer").and_then(|v| v.as_str());

    let mut errors = Vec::new();

    // Validate purpose
    if purpose != "spend" && purpose != "mint" {
        errors.push(format!(
            "Invalid purpose '{}'. Must be 'spend' or 'mint'",
            purpose
        ));
    }

    // Parse and validate features
    let feature_strs: Vec<String> = features_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let resolved = match features::parse_features(&feature_strs) {
        Ok(parsed) => match features::compose::resolve_features(&parsed, purpose) {
            Ok(r) => Some(r),
            Err(e) => {
                errors.push(e.to_string());
                None
            }
        },
        Err(e) => {
            errors.push(e.to_string());
            None
        }
    };

    // Validate datum
    let datum_fields = if let Some(d) = datum_str {
        match features::types::parse_datum_fields(d) {
            Ok(f) => f,
            Err(e) => {
                errors.push(e.to_string());
                vec![]
            }
        }
    } else if purpose == "spend" {
        errors.push("Datum fields are required for spend validators".to_string());
        vec![]
    } else {
        vec![]
    };

    // Validate redeemer
    if let Some(r) = redeemer_str {
        if let Err(e) = features::types::parse_redeemer_actions(r) {
            errors.push(e.to_string());
        }
    } else {
        errors.push("At least one redeemer action is required".to_string());
    }

    // Cross-validate features against types
    if let Some(ref resolved) = resolved {
        if let Err(e) =
            features::types::validate_features_against_types(resolved, &datum_fields, purpose)
        {
            errors.push(e.to_string());
        }
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors,
    })
    .to_string()
}

fn build_options(
    args: &serde_json::Value,
    template: &str,
    namespace: &str,
    project_name: &str,
) -> Result<GenerateOptions, String> {
    match template {
        "mint" | "simple_mint" | "simple-mint" => {
            let token_name = args
                .get("token_name")
                .and_then(|v| v.as_str())
                .unwrap_or(project_name);
            let default_an = token_name.to_uppercase().replace(' ', "_");
            let asset_name = args
                .get("asset_name")
                .and_then(|v| v.as_str())
                .unwrap_or(&default_an);
            let time_lock = args
                .get("time_lock")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(GenerateOptions::simple_mint(
                namespace,
                project_name,
                token_name,
                asset_name,
                time_lock,
            ))
        }
        "vesting" => {
            let cancellable = args
                .get("cancellable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let partial_claim = args
                .get("partial_claim")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Ok(GenerateOptions::vesting(
                namespace,
                project_name,
                cancellable,
                partial_claim,
            ))
        }
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
            let purpose = args
                .get("purpose")
                .and_then(|v| v.as_str())
                .unwrap_or("spend");
            let features_str = args.get("features").and_then(|v| v.as_str()).unwrap_or("");
            let datum_str = args.get("datum").and_then(|v| v.as_str());
            let redeemer_str = args.get("redeemer").and_then(|v| v.as_str());

            let feature_strs: Vec<String> = features_str
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            let parsed = features::parse_features(&feature_strs).map_err(|e| e.to_string())?;
            let resolved =
                features::compose::resolve_features(&parsed, purpose).map_err(|e| e.to_string())?;
            let names: Vec<String> = resolved.iter().map(|f| f.name().to_string()).collect();

            let datum_fields = if let Some(d) = datum_str {
                features::types::parse_datum_fields(d).map_err(|e| e.to_string())?
            } else if purpose == "spend" {
                return Err("datum is required for spend-purpose custom validators".to_string());
            } else {
                vec![]
            };

            let redeemer_actions = if let Some(r) = redeemer_str {
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
        _ => Err(format!("Unknown template '{}'", template)),
    }
}

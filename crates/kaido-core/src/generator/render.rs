use std::collections::HashMap;
use tera::{Context, Tera};

use crate::error::{KaidoError, Result};
use crate::features;
use crate::features::compose;
use crate::templates::{GenerateOptions, Template};

/// A single generated file (path relative to project root + content)
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Path relative to the project root (e.g., "aiken.toml", "lib/myorg/my_token/types.ak")
    pub path: String,
    /// File content
    pub content: String,
}

/// Result of rendering templates — contains all files to write
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Generated files with relative paths and content
    pub files: Vec<GeneratedFile>,
    /// Template that was used
    pub template: Template,
}

/// Generates complete Aiken projects from templates (pure computation, no I/O)
pub struct ProjectGenerator {
    tera: Tera,
}

impl ProjectGenerator {
    /// Create a new generator with embedded templates
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register base templates
        tera.add_raw_template(
            "base/aiken.toml",
            include_str!("../../../../templates/base/aiken.toml.tera"),
        )?;
        tera.add_raw_template(
            "base/.aikido.toml",
            include_str!("../../../../templates/base/aikido.toml.tera"),
        )?;

        // Register simple_mint templates
        tera.add_raw_template(
            "simple_mint/types.ak",
            include_str!("../../../../templates/simple_mint/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "simple_mint/validator.ak",
            include_str!("../../../../templates/simple_mint/validator.ak.tera"),
        )?;

        // Register vesting templates
        tera.add_raw_template(
            "vesting/types.ak",
            include_str!("../../../../templates/vesting/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "vesting/validator.ak",
            include_str!("../../../../templates/vesting/validator.ak.tera"),
        )?;

        // Register escrow templates
        tera.add_raw_template(
            "escrow/types.ak",
            include_str!("../../../../templates/escrow/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "escrow/validator.ak",
            include_str!("../../../../templates/escrow/validator.ak.tera"),
        )?;

        // Register multisig_treasury templates
        tera.add_raw_template(
            "multisig_treasury/types.ak",
            include_str!("../../../../templates/multisig_treasury/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "multisig_treasury/validator.ak",
            include_str!("../../../../templates/multisig_treasury/validator.ak.tera"),
        )?;

        // Register oracle_settlement templates
        tera.add_raw_template(
            "oracle_settlement/types.ak",
            include_str!("../../../../templates/oracle_settlement/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "oracle_settlement/validator.ak",
            include_str!("../../../../templates/oracle_settlement/validator.ak.tera"),
        )?;

        // Register staking_pool templates
        tera.add_raw_template(
            "staking_pool/types.ak",
            include_str!("../../../../templates/staking_pool/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "staking_pool/validator.ak",
            include_str!("../../../../templates/staking_pool/validator.ak.tera"),
        )?;

        // Register nft_marketplace templates
        tera.add_raw_template(
            "nft_marketplace/types.ak",
            include_str!("../../../../templates/nft_marketplace/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "nft_marketplace/validator.ak",
            include_str!("../../../../templates/nft_marketplace/validator.ak.tera"),
        )?;

        // Register SDK base templates
        tera.add_raw_template(
            "sdk_base/package.json",
            include_str!("../../../../templates/sdk_base/package.json.tera"),
        )?;
        tera.add_raw_template(
            "sdk_base/tsconfig.json",
            include_str!("../../../../templates/sdk_base/tsconfig.json.tera"),
        )?;

        // SDK templates — simple_mint
        tera.add_raw_template("simple_mint/sdk/types.ts", include_str!("../../../../templates/simple_mint/sdk/types.ts.tera"))?;
        tera.add_raw_template("simple_mint/sdk/serialization.ts", include_str!("../../../../templates/simple_mint/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("simple_mint/sdk/client.ts", include_str!("../../../../templates/simple_mint/sdk/client.ts.tera"))?;
        tera.add_raw_template("simple_mint/sdk/index.ts", include_str!("../../../../templates/simple_mint/sdk/index.ts.tera"))?;

        // SDK templates — vesting
        tera.add_raw_template("vesting/sdk/types.ts", include_str!("../../../../templates/vesting/sdk/types.ts.tera"))?;
        tera.add_raw_template("vesting/sdk/serialization.ts", include_str!("../../../../templates/vesting/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("vesting/sdk/client.ts", include_str!("../../../../templates/vesting/sdk/client.ts.tera"))?;
        tera.add_raw_template("vesting/sdk/index.ts", include_str!("../../../../templates/vesting/sdk/index.ts.tera"))?;

        // SDK templates — escrow
        tera.add_raw_template("escrow/sdk/types.ts", include_str!("../../../../templates/escrow/sdk/types.ts.tera"))?;
        tera.add_raw_template("escrow/sdk/serialization.ts", include_str!("../../../../templates/escrow/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("escrow/sdk/client.ts", include_str!("../../../../templates/escrow/sdk/client.ts.tera"))?;
        tera.add_raw_template("escrow/sdk/index.ts", include_str!("../../../../templates/escrow/sdk/index.ts.tera"))?;

        // SDK templates — multisig_treasury
        tera.add_raw_template("multisig_treasury/sdk/types.ts", include_str!("../../../../templates/multisig_treasury/sdk/types.ts.tera"))?;
        tera.add_raw_template("multisig_treasury/sdk/serialization.ts", include_str!("../../../../templates/multisig_treasury/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("multisig_treasury/sdk/client.ts", include_str!("../../../../templates/multisig_treasury/sdk/client.ts.tera"))?;
        tera.add_raw_template("multisig_treasury/sdk/index.ts", include_str!("../../../../templates/multisig_treasury/sdk/index.ts.tera"))?;

        // SDK templates — nft_marketplace
        tera.add_raw_template("nft_marketplace/sdk/types.ts", include_str!("../../../../templates/nft_marketplace/sdk/types.ts.tera"))?;
        tera.add_raw_template("nft_marketplace/sdk/serialization.ts", include_str!("../../../../templates/nft_marketplace/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("nft_marketplace/sdk/client.ts", include_str!("../../../../templates/nft_marketplace/sdk/client.ts.tera"))?;
        tera.add_raw_template("nft_marketplace/sdk/index.ts", include_str!("../../../../templates/nft_marketplace/sdk/index.ts.tera"))?;

        // SDK templates — staking_pool
        tera.add_raw_template("staking_pool/sdk/types.ts", include_str!("../../../../templates/staking_pool/sdk/types.ts.tera"))?;
        tera.add_raw_template("staking_pool/sdk/serialization.ts", include_str!("../../../../templates/staking_pool/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("staking_pool/sdk/client.ts", include_str!("../../../../templates/staking_pool/sdk/client.ts.tera"))?;
        tera.add_raw_template("staking_pool/sdk/index.ts", include_str!("../../../../templates/staking_pool/sdk/index.ts.tera"))?;

        // SDK templates — oracle_settlement
        tera.add_raw_template("oracle_settlement/sdk/types.ts", include_str!("../../../../templates/oracle_settlement/sdk/types.ts.tera"))?;
        tera.add_raw_template("oracle_settlement/sdk/serialization.ts", include_str!("../../../../templates/oracle_settlement/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("oracle_settlement/sdk/client.ts", include_str!("../../../../templates/oracle_settlement/sdk/client.ts.tera"))?;
        tera.add_raw_template("oracle_settlement/sdk/index.ts", include_str!("../../../../templates/oracle_settlement/sdk/index.ts.tera"))?;

        // Register dex_pool templates
        tera.add_raw_template(
            "dex_pool/types.ak",
            include_str!("../../../../templates/dex_pool/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "dex_pool/validator.ak",
            include_str!("../../../../templates/dex_pool/validator.ak.tera"),
        )?;

        // Register lending_pool templates
        tera.add_raw_template(
            "lending_pool/types.ak",
            include_str!("../../../../templates/lending_pool/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "lending_pool/validator.ak",
            include_str!("../../../../templates/lending_pool/validator.ak.tera"),
        )?;

        // Register dao_governance templates
        tera.add_raw_template(
            "dao_governance/types.ak",
            include_str!("../../../../templates/dao_governance/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "dao_governance/validator.ak",
            include_str!("../../../../templates/dao_governance/validator.ak.tera"),
        )?;

        // Register streaming_payments templates
        tera.add_raw_template(
            "streaming_payments/types.ak",
            include_str!("../../../../templates/streaming_payments/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "streaming_payments/validator.ak",
            include_str!("../../../../templates/streaming_payments/validator.ak.tera"),
        )?;

        // Register referral_system templates
        tera.add_raw_template(
            "referral_system/types.ak",
            include_str!("../../../../templates/referral_system/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "referral_system/validation.ak",
            include_str!("../../../../templates/referral_system/validation.ak.tera"),
        )?;
        tera.add_raw_template(
            "referral_system/mint_validator.ak",
            include_str!("../../../../templates/referral_system/mint_validator.ak.tera"),
        )?;
        tera.add_raw_template(
            "referral_system/treasury_validator.ak",
            include_str!("../../../../templates/referral_system/treasury_validator.ak.tera"),
        )?;

        // SDK templates — referral_system
        tera.add_raw_template("referral_system/sdk/types.ts", include_str!("../../../../templates/referral_system/sdk/types.ts.tera"))?;
        tera.add_raw_template("referral_system/sdk/serialization.ts", include_str!("../../../../templates/referral_system/sdk/serialization.ts.tera"))?;
        tera.add_raw_template("referral_system/sdk/client.ts", include_str!("../../../../templates/referral_system/sdk/client.ts.tera"))?;
        tera.add_raw_template("referral_system/sdk/index.ts", include_str!("../../../../templates/referral_system/sdk/index.ts.tera"))?;

        // Register custom templates
        tera.add_raw_template(
            "custom/types.ak",
            include_str!("../../../../templates/custom/types.ak.tera"),
        )?;
        tera.add_raw_template(
            "custom/validator.ak",
            include_str!("../../../../templates/custom/validator.ak.tera"),
        )?;

        Ok(Self { tera })
    }

    /// Render a complete Aiken project (pure computation, returns file contents)
    pub fn render(&self, options: &GenerateOptions) -> Result<RenderResult> {
        let ctx = self.build_context(options);
        let mut files = Vec::new();

        // Lib directory prefix
        let lib_prefix = format!("lib/{}/{}", options.namespace, options.module_name);

        // Render aiken.toml
        let aiken_toml = self.tera.render("base/aiken.toml", &ctx)?;
        files.push(GeneratedFile {
            path: "aiken.toml".to_string(),
            content: aiken_toml,
        });
        let aikido_toml = self.tera.render("base/.aikido.toml", &ctx)?;
        files.push(GeneratedFile {
            path: ".aikido.toml".to_string(),
            content: aikido_toml,
        });

        // Custom templates handle their own rendering
        if options.template == Template::Custom {
            return self.render_custom(options, &lib_prefix, &files);
        }

        // Render template-specific files
        let slug = options.template.slug();

        // Types file
        let types_content = self.tera.render(&format!("{}/types.ak", slug), &ctx)?;
        files.push(GeneratedFile {
            path: format!("{}/types.ak", lib_prefix),
            content: types_content,
        });

        // Template-specific extra files
        if options.template == Template::ReferralSystem {
            // Validation helpers (lib file)
            let validation_content = self.tera.render(&format!("{}/validation.ak", slug), &ctx)?;
            files.push(GeneratedFile {
                path: format!("{}/validation.ak", lib_prefix),
                content: validation_content,
            });

            // Mint validator
            let mint_content = self.tera.render(&format!("{}/mint_validator.ak", slug), &ctx)?;
            files.push(GeneratedFile {
                path: format!("validators/{}_mint.ak", options.validator_name),
                content: mint_content,
            });

            // Treasury validator
            let treasury_content = self.tera.render(&format!("{}/treasury_validator.ak", slug), &ctx)?;
            files.push(GeneratedFile {
                path: format!("validators/{}_treasury.ak", options.validator_name),
                content: treasury_content,
            });
        } else {
            // Standard single-validator template
            let validator_content = self.tera.render(&format!("{}/validator.ak", slug), &ctx)?;
            files.push(GeneratedFile {
                path: format!("validators/{}.ak", options.validator_name),
                content: validator_content,
            });
        }

        Ok(RenderResult {
            files,
            template: options.template,
        })
    }

    /// Render TypeScript SDK files
    pub fn render_sdk(&self, options: &GenerateOptions) -> Result<RenderResult> {
        if !options.template.supports_sdk() {
            return Err(KaidoError::InvalidOption(format!(
                "TypeScript SDK is not available for '{}' template yet",
                options.template.slug()
            )));
        }

        let ctx = self.build_context(options);
        let slug = options.template.slug();
        let mut files = Vec::new();

        // Render shared base files
        let pkg = self.tera.render("sdk_base/package.json", &ctx)?;
        files.push(GeneratedFile {
            path: "sdk/package.json".to_string(),
            content: pkg,
        });

        let tsconfig = self.tera.render("sdk_base/tsconfig.json", &ctx)?;
        files.push(GeneratedFile {
            path: "sdk/tsconfig.json".to_string(),
            content: tsconfig,
        });

        // Render template-specific SDK files
        for file in &["types.ts", "serialization.ts", "client.ts", "index.ts"] {
            let template_name = format!("{}/sdk/{}", slug, file);
            let content = self.tera.render(&template_name, &ctx)?;
            files.push(GeneratedFile {
                path: format!("sdk/src/{}", file),
                content,
            });
        }

        Ok(RenderResult {
            files,
            template: options.template,
        })
    }

    /// Render a custom composable validator project
    fn render_custom(
        &self,
        options: &GenerateOptions,
        lib_prefix: &str,
        base_files: &[GeneratedFile],
    ) -> Result<RenderResult> {
        let mut files: Vec<GeneratedFile> = base_files.to_vec();

        // Parse features from stored names
        let feature_strs: Vec<String> = options.feature_names.clone();
        let parsed_features = features::parse_features(&feature_strs)?;
        let resolved = compose::resolve_features(&parsed_features, &options.purpose)?;

        // Compose features
        let composed = compose::compose(
            &resolved,
            &options.purpose,
            &options.datum_fields,
            &options.redeemer_actions,
            &options.validator_name,
        )?;

        // Build context
        let mut ctx = self.build_context(options);

        ctx.insert("purpose", &options.purpose);

        // Datum fields for template
        let datum_fields_ctx: Vec<HashMap<String, String>> = options.datum_fields.iter().map(|f| {
            let mut m = HashMap::new();
            m.insert("name".to_string(), f.name.clone());
            m.insert("aiken_type".to_string(), f.aiken_type.clone());
            m
        }).collect();
        ctx.insert("datum_fields", &datum_fields_ctx);

        // Redeemer actions for template
        let redeemer_actions_ctx: Vec<HashMap<String, serde_json::Value>> = options.redeemer_actions.iter().map(|a| {
            let mut m = HashMap::new();
            m.insert("name".to_string(), serde_json::Value::String(a.name.clone()));
            m.insert("has_fields".to_string(), serde_json::Value::Bool(!a.fields.is_empty()));
            let fields: Vec<serde_json::Value> = a.fields.iter().map(|(name, ty)| {
                let mut fm = serde_json::Map::new();
                fm.insert("name".to_string(), serde_json::Value::String(name.clone()));
                fm.insert("aiken_type".to_string(), serde_json::Value::String(ty.clone()));
                serde_json::Value::Object(fm)
            }).collect();
            m.insert("fields".to_string(), serde_json::Value::Array(fields));
            m
        }).collect();
        ctx.insert("redeemer_actions", &redeemer_actions_ctx);

        // Feature names for doc comment
        let feature_names = options.feature_names.join(", ");
        ctx.insert("feature_names", &feature_names);

        // Composed imports
        ctx.insert("composed_imports", &composed.imports.join("\n"));

        // Types import line
        let mut type_names = Vec::new();
        if options.purpose == "spend" {
            type_names.push("CustomDatum".to_string());
        }
        type_names.push("CustomRedeemer".to_string());
        for action in &options.redeemer_actions {
            type_names.push(action.name.clone());
        }
        let types_import = format!(
            "use {}/{}/types.{{{}}}",
            options.namespace,
            options.module_name,
            type_names.join(", ")
        );
        ctx.insert("types_import", &types_import);

        // Composed params
        let params_str: Vec<String> = composed.validator_params.iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect();
        ctx.insert("composed_params", &params_str.join(", "));

        // Composed preamble
        ctx.insert("composed_preamble", &composed.preamble);

        // Composed action checks
        ctx.insert("composed_action_checks", &composed.action_checks.join("\n"));

        // Test data
        ctx.insert("composed_test_helpers", &composed.test_helpers);
        ctx.insert("composed_test_cases", &composed.test_cases);

        // Render types file
        let types_content = self.tera.render("custom/types.ak", &ctx)?;
        files.push(GeneratedFile {
            path: format!("{}/types.ak", lib_prefix),
            content: types_content,
        });

        // Render validator file
        let validator_content = self.tera.render("custom/validator.ak", &ctx)?;
        files.push(GeneratedFile {
            path: format!("validators/{}.ak", options.validator_name),
            content: validator_content,
        });

        Ok(RenderResult {
            files,
            template: Template::Custom,
        })
    }

    fn build_context(&self, options: &GenerateOptions) -> Context {
        let mut ctx = Context::new();

        // Base fields
        ctx.insert("namespace", &options.namespace);
        ctx.insert("project_name", &options.project_name);
        ctx.insert("module_name", &options.module_name);
        ctx.insert("description", &options.description);
        ctx.insert("validator_name", &options.validator_name);

        // Simple Mint fields
        if let Some(ref name) = options.token_name {
            ctx.insert("token_name", name);
        }
        if let Some(ref name) = options.asset_name {
            ctx.insert("asset_name", name);
        }
        ctx.insert("time_lock", &options.time_lock);

        // Vesting fields
        ctx.insert("cancellable", &options.cancellable);
        ctx.insert("partial_claim", &options.partial_claim);

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_mint() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::simple_mint("myorg", "my-token", "MyToken", "MY_TOKEN", false);
        let result = gen.render(&opts).unwrap();

        assert!(result.files.iter().any(|f| f.path == "aiken.toml"));
        assert!(result.files.iter().any(|f| f.path == "lib/myorg/my_token/types.ak"));
        assert!(result.files.iter().any(|f| f.path.starts_with("validators/")));

        let toml = result.files.iter().find(|f| f.path == "aiken.toml").unwrap();
        assert!(toml.content.contains("myorg/my-token"));
        assert!(toml.content.contains("v1.1.21"));
        assert!(toml.content.contains("v3.0.0"));
    }

    #[test]
    fn test_render_simple_mint_with_timelock() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::simple_mint("myorg", "my-token", "MyToken", "MY_TOKEN", true);
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("lock_after"));
        assert!(validator.content.contains("is_entirely_before"));
        assert!(validator.content.contains("mint_after_lock_fails"));
    }

    #[test]
    fn test_render_vesting() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::vesting("myorg", "my-vesting", true, false);
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Claim"));
        assert!(validator.content.contains("Cancel"));
        assert!(validator.content.contains("owner_pkh"));
    }

    #[test]
    fn test_render_vesting_no_cancel() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::vesting("myorg", "my-vesting", false, false);
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Claim"));
        assert!(!validator.content.contains("Cancel"));
    }

    #[test]
    fn test_render_staking_pool() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::staking_pool("myorg", "my-staking");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Stake"));
        assert!(validator.content.contains("Unstake"));
        assert!(validator.content.contains("AddRewards"));
        assert!(validator.content.contains("admin_pkh"));
    }

    #[test]
    fn test_render_oracle_settlement() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::oracle_settlement("myorg", "my-settle");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Settle"));
        assert!(validator.content.contains("Reclaim"));
        assert!(validator.content.contains("oracle_pkh"));
        assert!(validator.content.contains("settlement_amount"));
    }

    #[test]
    fn test_render_escrow() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::escrow("myorg", "my-escrow");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Complete"));
        assert!(validator.content.contains("Reclaim"));
        assert!(validator.content.contains("Cancel"));
        assert!(validator.content.contains("seller_output"));
    }

    #[test]
    fn test_render_nft_marketplace() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::nft_marketplace("myorg", "my-market");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Buy"));
        assert!(validator.content.contains("Delist"));
        assert!(validator.content.contains("seller_pkh"));
        assert!(validator.content.contains("price_lovelace"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("ListingDatum"));
        assert!(types.content.contains("MarketplaceRedeemer"));
    }

    #[test]
    fn test_render_multisig_treasury() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::multisig_treasury("myorg", "my-treasury");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Deposit"));
        assert!(validator.content.contains("Withdraw"));
        assert!(validator.content.contains("sig_count"));
        assert!(validator.content.contains("2_000_000"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("TreasuryDatum"));
        assert!(types.content.contains("TreasuryRedeemer"));
    }

    #[test]
    fn test_render_referral_system() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::referral_system("myorg", "my-referral");
        let result = gen.render(&opts).unwrap();

        // Check all expected files
        assert!(result.files.iter().any(|f| f.path == "aiken.toml"));
        assert!(result.files.iter().any(|f| f.path.contains("types.ak")));
        assert!(result.files.iter().any(|f| f.path.contains("validation.ak")));
        assert!(result.files.iter().any(|f| f.path.contains("_mint.ak")));
        assert!(result.files.iter().any(|f| f.path.contains("_treasury.ak")));

        // Check types
        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("ConfigDatum"));
        assert!(types.content.contains("TreasuryDatum"));
        assert!(types.content.contains("MintRedeemer"));
        assert!(types.content.contains("MintReferralToken"));

        // Check validation
        let validation = result.files.iter().find(|f| f.path.contains("validation.ak")).unwrap();
        assert!(validation.content.contains("referral_token_name"));
        assert!(validation.content.contains("signed_by"));
        assert!(validation.content.contains("blake2b_256"));

        // Check mint validator
        let mint = result.files.iter().find(|f| f.path.contains("_mint.ak")).unwrap();
        assert!(mint.content.contains("MintProjectTokens"));
        assert!(mint.content.contains("MintReferralToken"));
        assert!(mint.content.contains("BurnToken"));
        assert!(mint.content.contains("UpdateConfig"));
        assert!(mint.content.contains("DestroyProject"));
        assert!(mint.content.contains("anti-sybil"));

        // Check treasury validator
        let treasury = result.files.iter().find(|f| f.path.contains("_treasury.ak")).unwrap();
        assert!(treasury.content.contains("Deposit"));
        assert!(treasury.content.contains("Withdraw"));
        assert!(treasury.content.contains("2_000_000"));
    }

    #[test]
    fn test_render_lib_path_structure() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::simple_mint("myorg", "my-token", "MyToken", "MY_TOKEN", false);
        let result = gen.render(&opts).unwrap();

        // CRITICAL: lib files must be at lib/{namespace}/{module_name}/types.ak
        // module_name is snake_case — Aiken fails silently on hyphens
        assert!(result.files.iter().any(|f| f.path == "lib/myorg/my_token/types.ak"));
    }

    // ---- Custom template tests ----

    #[test]
    fn test_render_custom_minimal_spend() {
        use crate::features::types::{DatumField, RedeemerAction};

        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::custom(
            "myorg",
            "my-custom",
            "spend",
            vec![
                DatumField { name: "admin".to_string(), aiken_type: "ByteArray".to_string() },
            ],
            vec![
                RedeemerAction { name: "Execute".to_string(), fields: vec![] },
            ],
            vec!["signature-auth".to_string()],
        );
        let result = gen.render(&opts).unwrap();

        assert!(result.files.iter().any(|f| f.path == "aiken.toml"));
        assert!(result.files.iter().any(|f| f.path.contains("types.ak")));
        assert!(result.files.iter().any(|f| f.path.starts_with("validators/")));

        // Check types
        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("CustomDatum"));
        assert!(types.content.contains("admin: ByteArray"));
        assert!(types.content.contains("CustomRedeemer"));
        assert!(types.content.contains("Execute"));

        // Check validator
        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("admin_pkh: ByteArray"));
        assert!(validator.content.contains("extra_signatories"));
        assert!(validator.content.contains("Execute"));
        assert!(validator.content.contains("signature-auth"));
    }

    #[test]
    fn test_render_custom_vesting_like() {
        use crate::features::types::{DatumField, RedeemerAction};

        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::custom(
            "myorg",
            "my-lock",
            "spend",
            vec![
                DatumField { name: "owner".to_string(), aiken_type: "ByteArray".to_string() },
                DatumField { name: "deadline".to_string(), aiken_type: "Int".to_string() },
            ],
            vec![
                RedeemerAction { name: "Claim".to_string(), fields: vec![] },
                RedeemerAction { name: "Cancel".to_string(), fields: vec![] },
            ],
            vec!["signature-auth".to_string(), "timelock".to_string()],
        );
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("admin_pkh"));
        assert!(validator.content.contains("is_entirely_after"));
        assert!(validator.content.contains("datum.deadline"));
        assert!(validator.content.contains("Claim"));
        assert!(validator.content.contains("Cancel"));
    }

    #[test]
    fn test_render_custom_treasury_like() {
        use crate::features::types::{DatumField, RedeemerAction};

        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::custom(
            "myorg",
            "my-vault",
            "spend",
            vec![
                DatumField { name: "admin".to_string(), aiken_type: "ByteArray".to_string() },
                DatumField { name: "balance".to_string(), aiken_type: "Int".to_string() },
            ],
            vec![
                RedeemerAction { name: "Deposit".to_string(), fields: vec![] },
                RedeemerAction { name: "Withdraw".to_string(), fields: vec![("amount".to_string(), "Int".to_string())] },
            ],
            vec!["signature-auth".to_string(), "datum-continuity".to_string(), "value-preservation".to_string(), "reference-safety".to_string()],
        );
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("own_input"));
        assert!(validator.content.contains("cont_output"));
        assert!(validator.content.contains("reference_script == None"));
        assert!(validator.content.contains("output_lovelace >= input_lovelace"));
        assert!(validator.content.contains("Deposit"));
        assert!(validator.content.contains("Withdraw"));

        // Check types has field with arguments
        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("Withdraw"));
        assert!(types.content.contains("amount: Int"));
    }

    #[test]
    fn test_render_custom_mint() {
        use crate::features::types::RedeemerAction;

        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::custom(
            "myorg",
            "my-token",
            "mint",
            vec![],
            vec![
                RedeemerAction { name: "Mint".to_string(), fields: vec![] },
                RedeemerAction { name: "Burn".to_string(), fields: vec![] },
            ],
            vec!["signature-auth".to_string(), "burn-verification".to_string()],
        );
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("mint(redeemer: CustomRedeemer, policy_id: PolicyId"));
        assert!(validator.content.contains("Mint"));
        assert!(validator.content.contains("Burn"));
        assert!(validator.content.contains("qty < 0"));

        // No CustomDatum in types for mint purpose
        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(!types.content.contains("CustomDatum"));
        assert!(types.content.contains("CustomRedeemer"));
    }

    #[test]
    fn test_render_dex_pool() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::dex_pool("myorg", "my-dex");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Swap"));
        assert!(validator.content.contains("AddLiquidity"));
        assert!(validator.content.contains("RemoveLiquidity"));
        assert!(validator.content.contains("UpdateFee"));
        assert!(validator.content.contains("reserve_a + min_received"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("PoolDatum"));
        assert!(types.content.contains("PoolRedeemer"));
    }

    #[test]
    fn test_render_lending_pool() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::lending_pool("myorg", "my-lending");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Supply"));
        assert!(validator.content.contains("Withdraw"));
        assert!(validator.content.contains("Borrow"));
        assert!(validator.content.contains("Repay"));
        assert!(validator.content.contains("Liquidate"));
        assert!(validator.content.contains("min_collateral_ratio_bps"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("LendingDatum"));
        assert!(types.content.contains("LendingRedeemer"));
    }

    #[test]
    fn test_render_dao_governance() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::dao_governance("myorg", "my-dao");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Deposit"));
        assert!(validator.content.contains("ExecuteProposal"));
        assert!(validator.content.contains("UpdateAdmin"));
        assert!(validator.content.contains("proposal_count"));
        assert!(validator.content.contains("2_000_000"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("GovernanceDatum"));
        assert!(types.content.contains("GovernanceRedeemer"));
    }

    #[test]
    fn test_render_streaming_payments() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::streaming_payments("myorg", "my-stream");
        let result = gen.render(&opts).unwrap();

        let validator = result.files.iter().find(|f| f.path.starts_with("validators/")).unwrap();
        assert!(validator.content.contains("Claim"));
        assert!(validator.content.contains("Cancel"));
        assert!(validator.content.contains("TopUp"));
        assert!(validator.content.contains("is_entirely_after"));
        assert!(validator.content.contains("claimed_amount"));

        let types = result.files.iter().find(|f| f.path.contains("types.ak")).unwrap();
        assert!(types.content.contains("StreamDatum"));
        assert!(types.content.contains("StreamRedeemer"));
    }

    #[test]
    fn test_render_custom_lib_structure() {
        use crate::features::types::{DatumField, RedeemerAction};

        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::custom(
            "myorg",
            "my-custom",
            "spend",
            vec![DatumField { name: "admin".to_string(), aiken_type: "ByteArray".to_string() }],
            vec![RedeemerAction { name: "Execute".to_string(), fields: vec![] }],
            vec!["signature-auth".to_string()],
        );
        let result = gen.render(&opts).unwrap();

        // CRITICAL: lib files must use snake_case
        assert!(result.files.iter().any(|f| f.path == "lib/myorg/my_custom/types.ak"));
    }

    #[test]
    fn test_render_sdk_supported_template() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::simple_mint("myorg", "my-token", "MyToken", "MY_TOKEN", false);
        let result = gen.render_sdk(&opts).unwrap();

        assert!(result.files.iter().any(|f| f.path == "sdk/package.json"));
        assert!(result.files.iter().any(|f| f.path == "sdk/tsconfig.json"));
        assert!(result.files.iter().any(|f| f.path == "sdk/src/types.ts"));
        assert!(result.files.iter().any(|f| f.path == "sdk/src/client.ts"));
    }

    #[test]
    fn test_render_sdk_unsupported_template_returns_clear_error() {
        let gen = ProjectGenerator::new().unwrap();
        let opts = GenerateOptions::dex_pool("myorg", "my-dex");
        let err = gen.render_sdk(&opts).expect_err("dex sdk should be unsupported");

        match err {
            KaidoError::InvalidOption(msg) => {
                assert!(msg.contains("TypeScript SDK is not available"));
                assert!(msg.contains("dex_pool"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}

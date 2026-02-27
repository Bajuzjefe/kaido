use std::collections::HashSet;

use super::types::{DatumField, RedeemerAction};
use super::{feature_spec, Feature};
use crate::error::{KaidoError, Result};

/// Result of composing multiple features into a single validator
#[derive(Debug, Clone)]
pub struct ComposedValidator {
    /// Deduplicated, sorted import lines
    pub imports: Vec<String>,
    /// Validator parameters (name, type)
    pub validator_params: Vec<(String, String)>,
    /// Code before the redeemer match
    pub preamble: String,
    /// Code inside each redeemer branch
    pub action_checks: Vec<String>,
    /// Test helper constants and functions
    pub test_helpers: String,
    /// Test cases
    pub test_cases: Vec<String>,
}

/// Resolve feature dependencies and check for conflicts.
/// Returns features in composition order (sorted by Feature::order()).
pub fn resolve_features(selected: &[Feature], purpose: &str) -> Result<Vec<Feature>> {
    let mut resolved: Vec<Feature> = selected.to_vec();

    // Add dependencies
    let mut changed = true;
    while changed {
        changed = false;
        let current = resolved.clone();
        for f in &current {
            let spec = feature_spec(*f);
            for dep in &spec.depends_on {
                if !resolved.contains(dep) {
                    resolved.push(*dep);
                    changed = true;
                }
            }
        }
    }

    // Check purpose compatibility
    for f in &resolved {
        let spec = feature_spec(*f);
        if let Some(required_purpose) = spec.purpose {
            if required_purpose != purpose {
                return Err(KaidoError::InvalidOption(format!(
                    "Feature '{}' requires purpose '{}', but got '{}'",
                    f.name(),
                    required_purpose,
                    purpose
                )));
            }
        }
    }

    // Check conflicts
    for f in &resolved {
        let spec = feature_spec(*f);
        for conflict in &spec.conflicts_with {
            if resolved.contains(conflict) {
                return Err(KaidoError::InvalidOption(format!(
                    "Feature '{}' conflicts with '{}'",
                    f.name(),
                    conflict.name()
                )));
            }
        }
    }

    // Sort by composition order
    resolved.sort_by_key(|f| f.order());
    resolved.dedup();

    Ok(resolved)
}

/// Compose resolved features into a single validator specification.
pub fn compose(
    features: &[Feature],
    purpose: &str,
    datum_fields: &[DatumField],
    redeemer_actions: &[RedeemerAction],
    validator_name: &str,
) -> Result<ComposedValidator> {
    let mut import_set: HashSet<String> = HashSet::new();
    let mut params: Vec<(String, String)> = Vec::new();
    let mut preamble_parts: Vec<String> = Vec::new();
    let mut action_parts: Vec<String> = Vec::new();
    let mut param_names: HashSet<String> = HashSet::new();

    // Base imports depending on purpose
    // Need both `use cardano/transaction` (for transaction.placeholder in tests)
    // and specific type imports
    import_set.insert("use cardano/transaction".to_string());
    if purpose == "spend" {
        import_set.insert("use cardano/transaction.{OutputReference, Transaction}".to_string());
    } else {
        // Mint needs bare `cardano/assets` for assets.tokens() / assets.from_asset()
        import_set.insert("use cardano/assets".to_string());
        import_set.insert("use cardano/assets.{PolicyId}".to_string());
        import_set.insert("use cardano/transaction.{Transaction}".to_string());
    }

    // Collect from each feature
    for f in features {
        let spec = feature_spec(*f);

        for imp in &spec.imports {
            import_set.insert(imp.to_string());
        }

        for (name, ty) in &spec.validator_params {
            if param_names.insert(name.to_string()) {
                params.push((name.to_string(), ty.to_string()));
            }
        }

        if !spec.preamble_code.is_empty() {
            let code = spec
                .preamble_code
                .replace("CustomDatum", &datum_type_name(datum_fields));
            preamble_parts.push(code);
        }

        if !spec.per_action_code.is_empty() {
            // For TimeLock, replace datum.deadline with the actual field name
            let mut code = spec.per_action_code.to_string();
            if *f == Feature::TimeLock {
                if let Some(deadline_field) = find_deadline_field(datum_fields) {
                    code = code.replace("datum.deadline", &format!("datum.{}", deadline_field));
                }
            }
            action_parts.push(code);
        }
    }

    // Merge imports from the same module path
    // e.g., "use cardano/transaction.{A, B}" + "use cardano/transaction.{C, D}"
    //     => "use cardano/transaction.{A, B, C, D}"
    let imports = merge_imports(import_set);

    // Build preamble
    let preamble = preamble_parts.join("\n\n");

    // Build test helpers and test cases
    let (test_helpers, test_cases) = build_tests(
        features,
        purpose,
        datum_fields,
        redeemer_actions,
        &params,
        validator_name,
    );

    Ok(ComposedValidator {
        imports,
        validator_params: params,
        preamble,
        action_checks: action_parts,
        test_helpers,
        test_cases,
    })
}

/// Merge imports from the same module into a single line.
/// e.g., "use cardano/transaction.{A}" + "use cardano/transaction.{B, C}"
///     => "use cardano/transaction.{A, B, C}"
fn merge_imports(import_set: HashSet<String>) -> Vec<String> {
    use std::collections::BTreeMap;
    let mut module_types: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut plain_imports: Vec<String> = Vec::new();

    for imp in &import_set {
        if let Some(dot_pos) = imp.find(".{") {
            let module = imp[..dot_pos].to_string();
            let types_str = &imp[dot_pos + 2..imp.len() - 1]; // strip ".{" and "}"
            let types: Vec<String> = types_str.split(',').map(|s| s.trim().to_string()).collect();
            module_types.entry(module).or_default().extend(types);
        } else {
            plain_imports.push(imp.clone());
        }
    }

    let mut result = Vec::new();

    // Add plain imports first
    plain_imports.sort();
    result.extend(plain_imports);

    // Add merged module imports
    for (module, mut types) in module_types {
        types.sort();
        types.dedup();
        result.push(format!("{}.{{{}}}", module, types.join(", ")));
    }

    result.sort();
    result
}

fn datum_type_name(_fields: &[DatumField]) -> String {
    "CustomDatum".to_string()
}

/// Find the first Int field that looks like a deadline
fn find_deadline_field(fields: &[DatumField]) -> Option<String> {
    // Look for fields named deadline, lock_until, expiry, etc.
    let deadline_names = [
        "deadline",
        "lock_until",
        "expiry",
        "expires_at",
        "lock_time",
    ];
    for name in &deadline_names {
        if fields.iter().any(|f| f.name == *name) {
            return Some(name.to_string());
        }
    }
    // Fall back to first Int field
    fields
        .iter()
        .find(|f| f.aiken_type == "Int")
        .map(|f| f.name.clone())
}

fn build_tests(
    features: &[Feature],
    purpose: &str,
    datum_fields: &[DatumField],
    redeemer_actions: &[RedeemerAction],
    params: &[(String, String)],
    validator_name: &str,
) -> (String, Vec<String>) {
    let mut helpers = String::new();
    let mut cases = Vec::new();

    let has_sig = features.contains(&Feature::SignatureAuth);
    let has_timelock = features.contains(&Feature::TimeLock);
    let has_continuity = features.contains(&Feature::DatumContinuity);
    let _has_value = features.contains(&Feature::ValuePreservation);
    let has_ref_safety = features.contains(&Feature::ReferenceSafety);
    let _has_bounded = features.contains(&Feature::BoundedOperations);

    // Test constants
    if has_sig {
        helpers.push_str("const test_admin: ByteArray = #\"aabbccdd\"\n");
    }
    if purpose == "mint" {
        helpers.push_str("const test_policy: ByteArray = #\"eeff0011\"\n");
    }

    // Deadline constant
    let deadline_field = find_deadline_field(datum_fields);
    if has_timelock {
        helpers.push_str("const test_deadline: Int = 1_000_000\n");
    }

    helpers.push('\n');

    // Datum builder
    if purpose == "spend" {
        helpers.push_str("fn test_datum() -> CustomDatum {\n");
        helpers.push_str("  CustomDatum {\n");
        for field in datum_fields {
            let value = test_value_for_type(
                &field.aiken_type,
                &field.name,
                has_timelock,
                &deadline_field,
            );
            helpers.push_str(&format!("    {}: {},\n", field.name, value));
        }
        helpers.push_str("  }\n");
        helpers.push_str("}\n\n");

        // OutputReference helper
        helpers.push_str("fn test_oref() -> OutputReference {\n");
        helpers.push_str("  OutputReference {\n");
        helpers.push_str("    transaction_id: #\"0000000000000000000000000000000000000000000000000000000000000001\",\n");
        helpers.push_str("    output_index: 0,\n");
        helpers.push_str("  }\n");
        helpers.push_str("}\n");

        if has_continuity {
            helpers.push_str("\nfn script_addr() -> Address {\n");
            helpers.push_str(
                "  Address { payment_credential: Script(#\"ee\"), stake_credential: None }\n",
            );
            helpers.push_str("}\n");

            helpers.push_str("\nfn script_input() -> Input {\n");
            helpers.push_str("  Input {\n");
            helpers.push_str("    output_reference: test_oref(),\n");
            helpers.push_str("    output: Output {\n");
            helpers.push_str("      address: script_addr(),\n");
            helpers.push_str("      value: assets.from_lovelace(10_000_000),\n");
            helpers.push_str("      datum: InlineDatum(test_datum()),\n");
            helpers.push_str("      reference_script: None,\n");
            helpers.push_str("    },\n");
            helpers.push_str("  }\n");
            helpers.push_str("}\n");

            helpers.push_str("\nfn cont_output_ok() -> Output {\n");
            helpers.push_str("  Output {\n");
            helpers.push_str("    address: script_addr(),\n");
            helpers.push_str("    value: assets.from_lovelace(10_000_000),\n");
            helpers.push_str("    datum: InlineDatum(test_datum()),\n");
            helpers.push_str("    reference_script: None,\n");
            helpers.push_str("  }\n");
            helpers.push_str("}\n");
        }
    }

    // Build param args string for test calls
    let param_args: Vec<String> = params
        .iter()
        .map(|(name, ty)| match ty.as_str() {
            "ByteArray" => {
                if name == "admin_pkh" {
                    "test_admin".to_string()
                } else {
                    "#\"00\"".to_string()
                }
            }
            "Int" => {
                if name == "min_lovelace" {
                    "2_000_000".to_string()
                } else {
                    "100".to_string()
                }
            }
            _ => "todo".to_string(),
        })
        .collect();
    let params_str = param_args.join(", ");

    // Generate test cases per action
    if purpose == "spend" {
        let first_action = redeemer_actions.first();

        if let Some(action) = first_action {
            let action_expr = redeemer_action_expr(action);

            // Positive test
            let mut tx_fields = Vec::new();
            if has_sig {
                tx_fields.push("      extra_signatories: [test_admin],".to_string());
            }
            if has_timelock {
                tx_fields
                    .push("      validity_range: interval.after(test_deadline + 1),".to_string());
            }
            if has_continuity {
                tx_fields.push("      inputs: [script_input()],".to_string());
                tx_fields.push("      outputs: [cont_output_ok()],".to_string());
            }

            let tx_body = tx_fields.join("\n");
            let call_params = if params_str.is_empty() {
                format!("Some(test_datum()), {}, test_oref(), tx", action_expr)
            } else {
                format!(
                    "{}, Some(test_datum()), {}, test_oref(), tx",
                    params_str, action_expr
                )
            };

            cases.push(format!(
                "test {action_name}_valid() {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.spend({call_params})\n}}",
                action_name = action.name.to_lowercase(),
                vname = validator_name,
                tx_body = tx_body,
                call_params = call_params,
            ));

            // Negative: wrong signer
            if has_sig {
                let mut tx_fields_bad = Vec::new();
                tx_fields_bad.push("      extra_signatories: [#\"deadbeef\"],".to_string());
                if has_timelock {
                    tx_fields_bad.push(
                        "      validity_range: interval.after(test_deadline + 1),".to_string(),
                    );
                }
                if has_continuity {
                    tx_fields_bad.push("      inputs: [script_input()],".to_string());
                    tx_fields_bad.push("      outputs: [cont_output_ok()],".to_string());
                }
                let tx_body_bad = tx_fields_bad.join("\n");

                cases.push(format!(
                    "test {action_name}_wrong_signer_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.spend({call_params})\n}}",
                    action_name = action.name.to_lowercase(),
                    vname = validator_name,
                    tx_body = tx_body_bad,
                    call_params = call_params,
                ));
            }

            // Negative: before deadline
            if has_timelock {
                let mut tx_fields_time = Vec::new();
                if has_sig {
                    tx_fields_time.push("      extra_signatories: [test_admin],".to_string());
                }
                tx_fields_time
                    .push("      validity_range: interval.before(test_deadline - 1),".to_string());
                if has_continuity {
                    tx_fields_time.push("      inputs: [script_input()],".to_string());
                    tx_fields_time.push("      outputs: [cont_output_ok()],".to_string());
                }
                let tx_body_time = tx_fields_time.join("\n");

                cases.push(format!(
                    "test {action_name}_before_deadline_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.spend({call_params})\n}}",
                    action_name = action.name.to_lowercase(),
                    vname = validator_name,
                    tx_body = tx_body_time,
                    call_params = call_params,
                ));
            }

            // Negative: no datum
            let mut tx_fields_nd = Vec::new();
            if has_sig {
                tx_fields_nd.push("      extra_signatories: [test_admin],".to_string());
            }
            if has_timelock {
                tx_fields_nd
                    .push("      validity_range: interval.after(test_deadline + 1),".to_string());
            }
            if has_continuity {
                tx_fields_nd.push("      inputs: [script_input()],".to_string());
                tx_fields_nd.push("      outputs: [cont_output_ok()],".to_string());
            }
            let tx_body_nd = tx_fields_nd.join("\n");
            let call_params_nd = if params_str.is_empty() {
                format!("None, {}, test_oref(), tx", action_expr)
            } else {
                format!("{}, None, {}, test_oref(), tx", params_str, action_expr)
            };

            cases.push(format!(
                "test no_datum_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.spend({call_params})\n}}",
                vname = validator_name,
                tx_body = tx_body_nd,
                call_params = call_params_nd,
            ));

            // Negative: reference script injection
            if has_ref_safety {
                let mut tx_fields_ref = Vec::new();
                if has_sig {
                    tx_fields_ref.push("      extra_signatories: [test_admin],".to_string());
                }
                if has_timelock {
                    tx_fields_ref.push(
                        "      validity_range: interval.after(test_deadline + 1),".to_string(),
                    );
                }
                tx_fields_ref.push("      inputs: [script_input()],".to_string());
                tx_fields_ref.push("      outputs: [\n        Output {\n          address: script_addr(),\n          value: assets.from_lovelace(10_000_000),\n          datum: InlineDatum(test_datum()),\n          reference_script: Some(#\"deadbeef\"),\n        },\n      ],".to_string());
                let tx_body_ref = tx_fields_ref.join("\n");
                let call_params_ref = if params_str.is_empty() {
                    format!("Some(test_datum()), {}, test_oref(), tx", action_expr)
                } else {
                    format!(
                        "{}, Some(test_datum()), {}, test_oref(), tx",
                        params_str, action_expr
                    )
                };

                cases.push(format!(
                    "test reference_script_injection_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.spend({call_params})\n}}",
                    vname = validator_name,
                    tx_body = tx_body_ref,
                    call_params = call_params_ref,
                ));
            }
        }
    } else {
        // Mint purpose tests
        let has_burn = features.contains(&Feature::BurnVerification);

        // Positive: mint
        let mut tx_fields = Vec::new();
        if has_sig {
            tx_fields.push("      extra_signatories: [test_admin],".to_string());
        }
        tx_fields.push("      mint: assets.from_asset(test_policy, \"token\", 1),".to_string());
        let tx_body = tx_fields.join("\n");

        let first_action = redeemer_actions.first();
        if let Some(action) = first_action {
            let action_expr = redeemer_action_expr(action);
            let call_params = if params_str.is_empty() {
                format!("{}, test_policy, tx", action_expr)
            } else {
                format!("{}, {}, test_policy, tx", params_str, action_expr)
            };

            cases.push(format!(
                "test mint_valid() {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.mint({call_params})\n}}",
                vname = validator_name,
                tx_body = tx_body,
                call_params = call_params,
            ));

            // Negative: no sig
            if has_sig {
                let tx_bad = [
                    "      extra_signatories: [],".to_string(),
                    "      mint: assets.from_asset(test_policy, \"token\", 1),".to_string(),
                ];
                let tx_body_bad = tx_bad.join("\n");

                cases.push(format!(
                    "test mint_no_signature_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n{tx_body}\n    }}\n  {vname}.mint({call_params})\n}}",
                    vname = validator_name,
                    tx_body = tx_body_bad,
                    call_params = call_params,
                ));
            }
        }

        // Burn test
        if has_burn {
            let burn_action = redeemer_actions
                .iter()
                .find(|a| a.name.to_lowercase() == "burn");
            if let Some(action) = burn_action {
                let action_expr = redeemer_action_expr(action);
                let call_params = if params_str.is_empty() {
                    format!("{}, test_policy, tx", action_expr)
                } else {
                    format!("{}, {}, test_policy, tx", params_str, action_expr)
                };

                cases.push(format!(
                    "test burn_valid() {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n      mint: assets.from_asset(test_policy, \"token\", -1),\n    }}\n  {vname}.mint({call_params})\n}}",
                    vname = validator_name,
                    call_params = call_params,
                ));

                cases.push(format!(
                    "test burn_positive_fails() fail {{\n  let tx =\n    Transaction {{\n      ..transaction.placeholder,\n      mint: assets.from_asset(test_policy, \"token\", 1),\n    }}\n  {vname}.mint({call_params})\n}}",
                    vname = validator_name,
                    call_params = call_params,
                ));
            }
        }
    }

    (helpers, cases)
}

fn test_value_for_type(
    aiken_type: &str,
    name: &str,
    has_timelock: bool,
    deadline_field: &Option<String>,
) -> String {
    match aiken_type {
        "ByteArray" => {
            if name.contains("admin") || name.contains("owner") || name.contains("signer") {
                "test_admin".to_string()
            } else {
                "#\"aabbccdd\"".to_string()
            }
        }
        "Int" => {
            if has_timelock && deadline_field.as_deref() == Some(name) {
                "test_deadline".to_string()
            } else if name.contains("amount") || name.contains("balance") || name.contains("total")
            {
                "10_000_000".to_string()
            } else {
                "0".to_string()
            }
        }
        "Bool" => "True".to_string(),
        t if t.starts_with("List<") => "[]".to_string(),
        _ => "todo".to_string(),
    }
}

fn redeemer_action_expr(action: &RedeemerAction) -> String {
    if action.fields.is_empty() {
        action.name.clone()
    } else {
        let field_defaults: Vec<String> = action
            .fields
            .iter()
            .map(|(name, ty)| {
                let val = match ty.as_str() {
                    "Int" => "5_000_000".to_string(),
                    "ByteArray" => "#\"aabb\"".to_string(),
                    "Bool" => "True".to_string(),
                    _ => "todo".to_string(),
                };
                format!("{}: {}", name, val)
            })
            .collect();
        format!("{} {{ {} }}", action.name, field_defaults.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_adds_dependencies() {
        let features = vec![Feature::ValuePreservation];
        let resolved = resolve_features(&features, "spend").unwrap();
        assert!(resolved.contains(&Feature::DatumContinuity));
        assert!(resolved.contains(&Feature::ValuePreservation));
    }

    #[test]
    fn test_resolve_sorted_by_order() {
        let features = vec![Feature::ValuePreservation, Feature::SignatureAuth];
        let resolved = resolve_features(&features, "spend").unwrap();
        let sig_pos = resolved
            .iter()
            .position(|f| *f == Feature::SignatureAuth)
            .unwrap();
        let val_pos = resolved
            .iter()
            .position(|f| *f == Feature::ValuePreservation)
            .unwrap();
        assert!(sig_pos < val_pos);
    }

    #[test]
    fn test_resolve_purpose_mismatch() {
        let features = vec![Feature::BurnVerification];
        let result = resolve_features(&features, "spend");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_purpose_compatible() {
        let features = vec![Feature::SignatureAuth];
        let result = resolve_features(&features, "spend");
        assert!(result.is_ok());
        let result = resolve_features(&features, "mint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compose_basic() {
        let features = vec![Feature::SignatureAuth];
        let datum_fields = vec![DatumField {
            name: "admin".to_string(),
            aiken_type: "ByteArray".to_string(),
        }];
        let actions = vec![RedeemerAction {
            name: "Execute".to_string(),
            fields: vec![],
        }];
        let composed = compose(
            &features,
            "spend",
            &datum_fields,
            &actions,
            "test_validator",
        )
        .unwrap();

        assert!(!composed.imports.is_empty());
        assert!(composed
            .validator_params
            .iter()
            .any(|(n, _)| n == "admin_pkh"));
        assert!(!composed.action_checks.is_empty());
    }

    #[test]
    fn test_compose_with_continuity() {
        let features = vec![
            Feature::SignatureAuth,
            Feature::DatumContinuity,
            Feature::ReferenceSafety,
        ];
        let datum_fields = vec![
            DatumField {
                name: "admin".to_string(),
                aiken_type: "ByteArray".to_string(),
            },
            DatumField {
                name: "balance".to_string(),
                aiken_type: "Int".to_string(),
            },
        ];
        let actions = vec![
            RedeemerAction {
                name: "Deposit".to_string(),
                fields: vec![],
            },
            RedeemerAction {
                name: "Withdraw".to_string(),
                fields: vec![],
            },
        ];
        let composed = compose(
            &features,
            "spend",
            &datum_fields,
            &actions,
            "test_validator",
        )
        .unwrap();

        assert!(composed.preamble.contains("own_input"));
        assert!(composed.preamble.contains("cont_output"));
        assert!(composed.preamble.contains("reference_script == None"));
    }

    #[test]
    fn test_compose_mint() {
        let features = vec![Feature::SignatureAuth, Feature::BurnVerification];
        let datum_fields = vec![];
        let actions = vec![
            RedeemerAction {
                name: "Mint".to_string(),
                fields: vec![],
            },
            RedeemerAction {
                name: "Burn".to_string(),
                fields: vec![],
            },
        ];
        let composed = compose(&features, "mint", &datum_fields, &actions, "test_mint").unwrap();

        assert!(composed.imports.iter().any(|i| i.contains("PolicyId")));
        assert!(!composed.test_cases.is_empty());
    }
}

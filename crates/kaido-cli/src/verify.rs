use std::path::Path;
use std::process::Command;

use kaido_core::error::{KaidoError, Result};

/// Raw aikido JSON output
#[derive(Debug, serde::Deserialize)]
struct AikidoOutput {
    #[serde(default)]
    findings: Vec<AikidoFinding>,
    #[serde(default)]
    total: usize,
}

/// Represents a single aikido finding
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
pub struct AikidoFinding {
    pub detector: String,
    pub severity: String,
    #[serde(alias = "description")]
    pub message: String,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Aikido scan result
#[derive(Debug)]
#[allow(dead_code)]
pub struct AikidoResult {
    pub findings: Vec<AikidoFinding>,
    pub high_or_critical: usize,
    pub total: usize,
}

/// Verifies generated Aiken projects compile and pass tests
pub struct AikenVerifier;

impl AikenVerifier {
    /// Run `aiken build` on the generated project
    pub fn build(project_dir: &Path) -> Result<()> {
        let output = Command::new("aiken")
            .arg("build")
            .current_dir(project_dir)
            .output()
            .map_err(|e| KaidoError::AikenBuildFailed(format!("Failed to run aiken: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(KaidoError::AikenBuildFailed(format!(
                "stdout:\n{}\nstderr:\n{}",
                stdout, stderr
            )));
        }

        Ok(())
    }

    /// Run `aiken check` on the generated project (builds + runs tests)
    pub fn check(project_dir: &Path) -> Result<()> {
        let output = Command::new("aiken")
            .arg("check")
            .current_dir(project_dir)
            .output()
            .map_err(|e| KaidoError::AikenCheckFailed(format!("Failed to run aiken: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(KaidoError::AikenCheckFailed(format!(
                "stdout:\n{}\nstderr:\n{}",
                stdout, stderr
            )));
        }

        Ok(())
    }

    /// Check if aiken is available on PATH
    pub fn is_available() -> bool {
        Command::new("aiken")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get aiken version string
    pub fn version() -> Option<String> {
        Command::new("aiken")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }
}

/// Runs aikido static analysis on generated Aiken projects
pub struct AikidoVerifier;

impl AikidoVerifier {
    /// Check if aikido is available on PATH
    pub fn is_available() -> bool {
        Command::new("aikido")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get aikido version string
    pub fn version() -> Option<String> {
        Command::new("aikido")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Run aikido scan on a project, returning findings
    pub fn scan(project_dir: &Path) -> Result<AikidoResult> {
        let output = Command::new("aikido")
            .arg(project_dir)
            .arg("--format")
            .arg("json")
            .arg("--quiet")
            .arg("--fail-on")
            .arg("high")
            .output()
            .map_err(|e| KaidoError::AikidoScanFailed(format!("Failed to run aikido: {}", e)))?;

        parse_scan_output(
            output.status.success(),
            output.status.code(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        )
    }

    /// Run scan and fail if high/critical findings exist
    #[allow(dead_code)]
    pub fn scan_and_fail_on_high(project_dir: &Path) -> Result<AikidoResult> {
        let result = Self::scan(project_dir)?;

        if result.high_or_critical > 0 {
            let summary: Vec<String> = result
                .findings
                .iter()
                .filter(|f| {
                    f.severity.eq_ignore_ascii_case("high")
                        || f.severity.eq_ignore_ascii_case("critical")
                })
                .map(|f| {
                    format!(
                        "[{}] {}: {}",
                        f.severity.to_uppercase(),
                        f.detector,
                        f.message
                    )
                })
                .collect();

            return Err(KaidoError::AikidoScanFailed(summary.join("\n")));
        }

        Ok(result)
    }
}

fn parse_scan_output(
    success: bool,
    code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> Result<AikidoResult> {
    if stdout.trim().is_empty() {
        return Err(KaidoError::AikidoScanFailed(format!(
            "aikido returned empty output (exit {:?}). stderr:\n{}",
            code, stderr
        )));
    }

    let parsed: AikidoOutput = serde_json::from_str(stdout).map_err(|e| {
        KaidoError::AikidoScanFailed(format!(
            "Failed to parse aikido JSON output: {}\nstdout:\n{}\nstderr:\n{}",
            e, stdout, stderr
        ))
    })?;

    if !success && parsed.findings.is_empty() {
        return Err(KaidoError::AikidoScanFailed(format!(
            "aikido exited non-zero ({:?}) without findings.\nstdout:\n{}\nstderr:\n{}",
            code, stdout, stderr
        )));
    }

    let high_or_critical = parsed
        .findings
        .iter()
        .filter(|f| {
            f.severity.eq_ignore_ascii_case("high") || f.severity.eq_ignore_ascii_case("critical")
        })
        .count();

    Ok(AikidoResult {
        findings: parsed.findings,
        high_or_critical,
        total: parsed.total,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_scan_output;

    #[test]
    fn parse_scan_output_rejects_empty_stdout() {
        let err = parse_scan_output(false, Some(1), "", "boom").expect_err("must fail");
        assert!(err.to_string().contains("empty output"));
    }

    #[test]
    fn parse_scan_output_rejects_invalid_json() {
        let err = parse_scan_output(true, Some(0), "not-json", "stderr").expect_err("must fail");
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn parse_scan_output_accepts_non_zero_when_findings_present() {
        let json =
            r#"{"findings":[{"detector":"d","severity":"critical","description":"x"}],"total":1}"#;
        let out = parse_scan_output(false, Some(2), json, "").expect("must parse");
        assert_eq!(out.findings.len(), 1);
        assert_eq!(out.high_or_critical, 1);
    }

    #[test]
    fn parse_scan_output_rejects_non_zero_without_findings() {
        let json = r#"{"findings":[],"total":0}"#;
        let err = parse_scan_output(false, Some(3), json, "stderr").expect_err("must fail");
        assert!(err.to_string().contains("non-zero"));
    }
}

use anyhow::{Context, Result};
use clap::Args;
use iris_agentic_dev_core::iris::connection::CompileResult;
use std::path::PathBuf;

use super::connection_args::ConnectionArgs;

#[derive(Args)]
pub struct CompileCommand {
    /// .cls file(s) to compile directly, bypassing iris-dev.toml.
    /// With no files: reads iris-dev.toml (existing behavior).
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    #[command(flatten)]
    pub conn: ConnectionArgs,

    #[arg(long, default_value = "cuk")]
    pub flags: String,

    #[arg(long)]
    pub force_writable: bool,

    #[arg(long, default_value = "text")]
    pub format: String,
}

impl CompileCommand {
    pub async fn run(self) -> Result<()> {
        let namespace = self.conn.namespace.clone();
        let flags = self.flags.clone();
        let format = self.format.clone();

        let iris = self.conn.resolve().await?;
        let client = iris_agentic_dev_core::iris::connection::IrisConnection::http_client()?;

        if self.files.is_empty() {
            // Legacy toml-based compile (original behavior preserved)
            let target = ".";
            let code = format!(
                "Set sc=$SYSTEM.OBJ.CompileAll(\"{}\") If $System.Status.IsOK(sc) {{Write \"OK\"}} Else {{Write $System.Status.GetErrorText(sc)}}",
                flags
            );
            let out = iris
                .execute_via_generator(&code, &namespace, &client)
                .await
                .context("CompileAll failed")?;
            let out = out.trim();
            if out.ends_with("OK") || out == "OK" {
                let result =
                    serde_json::json!({"success": true, "target": target, "namespace": namespace});
                output_result(&result, &format);
            } else {
                let result = serde_json::json!({"success": false, "error_code": "IRIS_COMPILE_FAILED", "error": out, "target": target});
                output_result(&result, &format);
                std::process::exit(1);
            }
            return Ok(());
        }

        // File-args mode: compile each file directly
        let mut any_error = false;
        for path in &self.files {
            let target = path.to_string_lossy();
            let cls_text =
                std::fs::read_to_string(path).with_context(|| format!("reading {}", target))?;
            let cls_name = cls_text
                .lines()
                .find(|l| l.trim_start().starts_with("Class "))
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    target
                        .trim_end_matches(".cls")
                        .replace(['/', '\\'], ".")
                        .trim_start_matches('.')
                        .to_string()
                });
            let doc_name = format!("{}.cls", cls_name);

            // Upload
            let put_url = iris.versioned_ns_url(
                &namespace,
                &format!("/doc/{}?ignoreConflict=1", urlencoding::encode(&doc_name)),
            );
            let lines: Vec<&str> = cls_text.lines().collect();
            let put_resp = client
                .put(&put_url)
                .basic_auth(&iris.username, Some(&iris.password))
                .json(&serde_json::json!({"enc": false, "content": lines}))
                .send()
                .await
                .context("PUT /doc failed")?;
            if !put_resp.status().is_success() {
                eprintln!(
                    "error: upload failed for {}: HTTP {}",
                    target,
                    put_resp.status()
                );
                any_error = true;
                continue;
            }
            let put_body: serde_json::Value = put_resp.json().await.unwrap_or_default();
            if let Some(errs) = put_body["status"]["errors"].as_array() {
                if !errs.is_empty() {
                    let msg = errs[0]["error"].as_str().unwrap_or("Upload failed");
                    eprintln!("error: {}", msg);
                    any_error = true;
                    continue;
                }
            }

            // Compile
            let compile_result = iris
                .compile_document(&doc_name, &namespace, &flags, &client)
                .await
                .context("compile request failed")?;

            if compile_result.success() {
                if format == "json" {
                    let result = compile_result_to_json(&compile_result, &target, &namespace);
                    println!("{}", result);
                } else {
                    println!("OK: {}", cls_name);
                }
            } else {
                any_error = true;
                if format == "json" {
                    let result = compile_result_to_json(&compile_result, &target, &namespace);
                    println!("{}", result);
                } else {
                    for err in &compile_result.errors {
                        println!("ERROR: {}: {}", cls_name, err);
                    }
                }
            }
        }

        if any_error {
            std::process::exit(1);
        }
        Ok(())
    }
}

fn compile_result_to_json(r: &CompileResult, target: &str, namespace: &str) -> serde_json::Value {
    let errors: Vec<serde_json::Value> = r
        .errors
        .iter()
        .map(|e| serde_json::json!({"severity":"error","text":e}))
        .collect();
    serde_json::json!({
        "success": r.success(),
        "target": target,
        "namespace": namespace,
        "errors": errors,
        "console": r.console,
    })
}

fn output_result(result: &serde_json::Value, format: &str) {
    if format == "json" {
        println!("{}", result);
    } else if result["success"] == true {
        println!("✓ Compiled: {}", result["target"].as_str().unwrap_or(""));
    } else {
        eprintln!(
            "error: [{}]: {}",
            result["error_code"].as_str().unwrap_or(""),
            result["error"].as_str().unwrap_or("")
        );
    }
}

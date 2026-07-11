use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::io::Read;
use std::path::PathBuf;

use super::connection_args::ConnectionArgs;

#[derive(Subcommand)]
pub enum DocAction {
    /// Fetch a class document and print to stdout
    Get {
        /// Class name (e.g. Config.MapMirrors or %Dictionary.ClassDefinition)
        #[arg(value_name = "CLASSNAME")]
        name: String,
    },
    /// Write a class document from file or stdin.
    /// Use `-` as CLASSNAME to read content from stdin.
    Put {
        /// Class name (e.g. MyApp.MyClass), or `-` to read class content from stdin
        #[arg(value_name = "CLASSNAME", allow_hyphen_values = true)]
        name: String,

        /// Read content from file
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<PathBuf>,
    },
}

#[derive(Args)]
pub struct DocCommand {
    #[command(subcommand)]
    pub action: DocAction,

    #[command(flatten)]
    pub conn: ConnectionArgs,
}

impl DocCommand {
    pub async fn run(self) -> Result<()> {
        let namespace = self.conn.namespace.clone();
        let iris = self.conn.resolve().await?;
        let client = iris_agentic_dev_core::iris::connection::IrisConnection::http_client()?;

        match self.action {
            DocAction::Get { name } => {
                let doc_name = ensure_cls_extension(&name);
                let url = iris.versioned_ns_url(
                    &namespace,
                    &format!("/doc/{}", urlencoding::encode(&doc_name)),
                );
                let resp = client
                    .get(&url)
                    .basic_auth(&iris.username, Some(&iris.password))
                    .send()
                    .await
                    .context("GET /doc failed")?;
                let status = resp.status();
                if status.as_u16() == 404 {
                    eprintln!("error: document not found: {}", doc_name);
                    std::process::exit(1);
                }
                if !status.is_success() {
                    eprintln!("error: HTTP {}", status);
                    std::process::exit(1);
                }
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let content = doc_content_to_string(&body);
                // Print raw UDL source — no framing, pipe-safe
                print!("{}", content);
                if !content.ends_with('\n') {
                    println!();
                }
            }
            DocAction::Put { name, file } => {
                if !iris.is_write_allowed() {
                    eprintln!(
                        "error: write operations are suppressed on production IRIS instances.\n\
                         Set IRIS_ALLOW_PROD=1 to override."
                    );
                    std::process::exit(1);
                }

                let doc_name = ensure_cls_extension(&name);
                let content = if name == "-" {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .context("reading doc content from stdin")?;
                    buf
                } else if let Some(path) = file {
                    std::fs::read_to_string(&path)
                        .with_context(|| format!("reading {}", path.display()))?
                } else {
                    eprintln!("error: `doc put` requires --file <path> or `-` to read from stdin");
                    std::process::exit(1);
                };

                let lines: Vec<&str> = content.lines().collect();
                let url = iris.versioned_ns_url(
                    &namespace,
                    &format!("/doc/{}?ignoreConflict=1", urlencoding::encode(&doc_name)),
                );
                let resp = client
                    .put(&url)
                    .basic_auth(&iris.username, Some(&iris.password))
                    .json(&serde_json::json!({"enc": false, "content": lines}))
                    .send()
                    .await
                    .context("PUT /doc failed")?;
                if !resp.status().is_success() {
                    eprintln!("error: HTTP {}", resp.status());
                    std::process::exit(1);
                }
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                if let Some(errs) = body["status"]["errors"].as_array() {
                    if !errs.is_empty() {
                        let msg = errs[0]["error"].as_str().unwrap_or("write failed");
                        eprintln!("error: {}", msg);
                        std::process::exit(1);
                    }
                }
                println!("OK: {}", doc_name);
            }
        }
        Ok(())
    }
}

fn ensure_cls_extension(name: &str) -> String {
    if name.contains('.')
        && (name.ends_with(".cls")
            || name.ends_with(".CLS")
            || name.ends_with(".mac")
            || name.ends_with(".inc"))
    {
        name.to_string()
    } else if !name.contains('.') {
        // No dot at all — treat as-is
        name.to_string()
    } else {
        // Has dots but no known extension — append .cls
        format!("{}.cls", name)
    }
}

fn doc_content_to_string(body: &serde_json::Value) -> String {
    body["result"]["content"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

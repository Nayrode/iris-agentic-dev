use anyhow::{Context, Result};
use clap::Args;
use std::io::Read;
use std::path::PathBuf;

use super::connection_args::ConnectionArgs;

/// How the ObjectScript code is supplied.
pub enum CodeSource {
    Inline(String),
    Stdin,
    File(PathBuf),
}

#[derive(Args)]
pub struct ExecCommand {
    /// ObjectScript code to execute. Use `-` to read from stdin.
    #[arg(value_name = "CODE")]
    pub code: Option<String>,

    /// Read code from a file (mutually exclusive with inline CODE argument)
    #[arg(long, short = 'f', value_name = "FILE", conflicts_with = "code")]
    pub file: Option<PathBuf>,

    #[command(flatten)]
    pub conn: ConnectionArgs,
}

impl ExecCommand {
    /// Resolve which code source applies.
    pub fn source(&self) -> CodeSource {
        if let Some(path) = &self.file {
            return CodeSource::File(path.clone());
        }
        match &self.code {
            Some(s) if s == "-" => CodeSource::Stdin,
            Some(s) => CodeSource::Inline(s.clone()),
            None => CodeSource::Stdin,
        }
    }

    pub async fn run(self) -> Result<()> {
        let namespace = self.conn.namespace.clone();

        let code = match self.source() {
            CodeSource::Inline(s) => s,
            CodeSource::Stdin => {
                let mut buf = String::new();
                std::io::stdin()
                    .read_to_string(&mut buf)
                    .context("reading code from stdin")?;
                buf
            }
            CodeSource::File(path) => std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?,
        };

        let iris = self.conn.resolve().await?;

        if !iris.is_write_allowed() {
            eprintln!(
                "error: write operations are suppressed on production IRIS instances.\n\
                 Set IRIS_ALLOW_PROD=1 to override."
            );
            std::process::exit(1);
        }

        let client = iris_agentic_dev_core::iris::connection::IrisConnection::http_client()?;
        match iris.execute_via_generator(&code, &namespace, &client).await {
            Ok(output) => {
                // Print raw IRIS output — no framing, pipe-safe
                print!("{}", output);
            }
            Err(e) => {
                let msg = e.to_string();
                // IRIS runtime errors are printed to stdout (matching MCP tool behavior)
                println!("{}", msg);
                std::process::exit(1);
            }
        }
        Ok(())
    }
}

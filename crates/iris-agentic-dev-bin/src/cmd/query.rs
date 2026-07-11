use anyhow::Result;
use clap::Args;

use super::connection_args::ConnectionArgs;
use super::tsv::{extract_columns, extract_rows, rows_to_tsv, tsv_header};

#[derive(Args)]
pub struct QueryCommand {
    /// SQL statement to execute
    #[arg(value_name = "SQL")]
    pub sql: String,

    #[command(flatten)]
    pub conn: ConnectionArgs,
}

impl QueryCommand {
    pub async fn run(self) -> Result<()> {
        let namespace = self.conn.namespace.clone();
        let iris = self.conn.resolve().await?;
        let client = iris_agentic_dev_core::iris::connection::IrisConnection::http_client()?;

        let body = iris
            .query(&self.sql, vec![], &namespace, &client)
            .await
            .map_err(|e| {
                eprintln!("error: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        let cols = extract_columns(&body);
        if !cols.is_empty() {
            let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
            println!("{}", tsv_header(&col_refs));
            let rows = extract_rows(&body);
            if !rows.is_empty() {
                print!("{}", rows_to_tsv(&rows));
            }
        }

        Ok(())
    }
}

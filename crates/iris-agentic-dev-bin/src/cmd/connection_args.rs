use clap::Args;
use iris_agentic_dev_core::iris::{
    connection::{DiscoverySource, IrisConnection},
    discovery::{discover_iris, IrisDiscovery},
    workspace_config::apply_workspace_config,
};

/// Shared IRIS connection flags reused by all CLI subcommands.
/// Precedence (highest to lowest):
///   1. Explicit CLI flags (--host, --port, --namespace, ...)
///   2. iris-dev.toml workspace config
///   3. Environment variables (IRIS_HOST, IRIS_WEB_PORT, IRIS_CONTAINER, ...)
///   4. Auto-discovery cascade (localhost scan → Docker scan → VS Code settings)
#[derive(Args, Clone)]
pub struct ConnectionArgs {
    /// IRIS web hostname (overrides discovery)
    #[arg(long, env = "IRIS_HOST")]
    pub host: Option<String>,

    /// IRIS web port
    #[arg(long, env = "IRIS_WEB_PORT", default_value = "52773")]
    pub web_port: u16,

    /// IRIS namespace
    #[arg(long, short = 'n', env = "IRIS_NAMESPACE", default_value = "USER")]
    pub namespace: String,

    /// IRIS username
    #[arg(long, short = 'u', env = "IRIS_USERNAME")]
    pub username: Option<String>,

    /// IRIS password
    #[arg(long, short = 'p', env = "IRIS_PASSWORD")]
    pub password: Option<String>,

    /// Named Docker container for IRIS (overrides auto-discovery)
    #[arg(long, env = "IRIS_CONTAINER")]
    pub container: Option<String>,
}

impl ConnectionArgs {
    /// Resolve this `ConnectionArgs` into a live `IrisConnection`.
    /// Runs the same discovery cascade as the MCP server.
    /// Exits the process (printing to stderr) on connection failure.
    pub async fn resolve(self) -> anyhow::Result<IrisConnection> {
        let explicit = self.host.as_ref().map(|host| {
            let base_url = format!("http://{}:{}", host, self.web_port);
            let username = self.username.as_deref().unwrap_or("_SYSTEM");
            let password = self.password.as_deref().unwrap_or("SYS");
            IrisConnection::new(
                base_url,
                &self.namespace,
                username,
                password,
                DiscoverySource::ExplicitFlag,
            )
        });

        // Apply workspace config — sits between CLI flags and env/auto-discovery
        let ws_path = std::env::var("OBJECTSCRIPT_WORKSPACE").ok();
        let explicit = apply_workspace_config(explicit, ws_path.as_deref(), &self.namespace);

        match discover_iris(explicit).await {
            IrisDiscovery::Found(c) => Ok(c),
            IrisDiscovery::NotFound => {
                anyhow::bail!(
                    "No IRIS connection found — set IRIS_HOST or run `iris-agentic-dev mcp` for auto-discovery"
                );
            }
            IrisDiscovery::Explained => {
                std::process::exit(1);
            }
        }
    }
}

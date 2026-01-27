//! ABCDODAF Executor Daemon Binary
//!
//! Persistent service that manages workflow execution, database persistence,
//! and real-time event streaming to clients.
//!
//! ## Usage
//!
//! ```bash
//! # Start with default configuration
//! abcdodaf-executor
//!
//! # Start with custom config file
//! abcdodaf-executor --config /path/to/config.toml
//!
//! # Override database URL
//! DATABASE_URL=postgresql://localhost/abcdodaf abcdodaf-executor
//! ```

use abcdodaf::executor::{ExecutorConfig, ExecutorDaemon};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "abcdodaf-executor",
    version,
    about = "ABCDODAF Executor Daemon - Workflow execution and management service",
    long_about = "Persistent service that manages BPMN workflow execution, PostgreSQL persistence,\n\
                  DODAF metadata tracking, and real-time event streaming to connected clients."
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Database URL (overrides config file)
    #[arg(long)]
    database_url: Option<String>,

    /// API bind address (overrides config file)
    #[arg(long, default_value = "127.0.0.1:8080")]
    api_bind: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the executor daemon
    Start,

    /// Initialize the database schema
    InitDb,

    /// Check configuration and connections
    Check,

    /// Generate default configuration file
    GenConfig {
        /// Output path for config file
        #[arg(short, long, default_value = "executor-config.toml")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Load configuration
    let mut config = load_config(&cli)?;

    // Apply CLI overrides
    if let Some(db_url) = cli.database_url {
        config.database_url = db_url;
    } else if let Ok(db_url) = std::env::var("DATABASE_URL") {
        config.database_url = db_url;
    }
    if let Some(api_bind) = cli.api_bind {
        config.api_bind = api_bind;
    }

    match cli.command {
        Some(Commands::Start) | None => {
            // Start daemon
            let daemon = ExecutorDaemon::new(config).await?;
            daemon.start().await?;
        },

        Some(Commands::InitDb) => {
            // Initialize database only
            println!("Initializing database schema...");
            let daemon = ExecutorDaemon::new(config).await?;
            println!("✅ Database schema initialized successfully");
            let _ = daemon; // Keep daemon alive until done
        },

        Some(Commands::Check) => {
            // Check configuration and connections
            println!("Checking configuration...");
            println!("  Database URL: {}", config.database_url);
            println!("  API Bind: {}", config.api_bind);
            println!("  Health Check Interval: {}s", config.health_check_interval_secs);
            println!();

            println!("Testing database connection...");
            let daemon = ExecutorDaemon::new(config).await?;
            println!("✅ Database connection successful");

            // Check health
            let health = daemon.health_monitor().overall_health().await;
            println!();
            println!("Connection Health:");
            println!("  Total: {}", health.total_connections);
            println!("  Connected: {}", health.connected);
            println!("  Disconnected: {}", health.disconnected);
            println!("  Error: {}", health.error);
            println!("  Health: {:.1}%", health.health_percentage);
        },

        Some(Commands::GenConfig { output }) => {
            // Generate default config file
            println!("Generating default configuration...");
            let config = ExecutorConfig::default();
            let toml = toml::to_string_pretty(&config)?;
            std::fs::write(&output, toml)?;
            println!("✅ Configuration written to: {}", output.display());
        },
    }

    Ok(())
}

/// Load configuration from file or use defaults
fn load_config(cli: &Cli) -> Result<ExecutorConfig, Box<dyn std::error::Error>> {
    if let Some(config_path) = &cli.config {
        // Load from file
        println!("Loading configuration from: {}", config_path.display());
        let contents = std::fs::read_to_string(config_path)?;
        let config: ExecutorConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        // Use defaults
        Ok(ExecutorConfig::default())
    }
}

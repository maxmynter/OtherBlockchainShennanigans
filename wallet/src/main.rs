mod core;
mod tasks;
mod ui;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use core::Core;
use cursive::views::TextContent;
use std::path::PathBuf;
use std::sync::Arc;
use tasks::{handle_transactions, ui_task, update_balance, update_utxos};
use tracing::{debug, info};
use utils::generate_dummy_config;
use utils::{big_mode_btc, setup_panic_hook, setup_tracing};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, value_name = "FILE", default_value_os_t = PathBuf::from("wallet_config.toml"))]
    config: PathBuf,

    #[arg(short, long, value_name = "ADDRESS")]
    node: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    GenerateConfig {
        #[arg(short, long, value_name = "FILE", default_value_os_t = PathBuf::from("wallet_config.toml"))]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;
    setup_panic_hook();
    info!("Starting Wallet");
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::GenerateConfig { output }) => {
            debug!("Generating dummy config at: {:?}", output);
            return generate_dummy_config(output);
        }
        None => (),
    }
    info!("Loading config from: {:?}", cli.config);
    let mut core = Core::load(cli.config.clone()).await?;
    if let Some(node) = cli.node {
        info!("Overriding default node with: {}", node);
        core.config.default_node = node;
    }
    let (tx_sender, tx_receiver) = kanal::bounded(10);
    core.tx_sender = tx_sender;
    let core = Arc::new(core);
    info!("Starting backgrounf tasks");
    let balance_content = TextContent::new(big_mode_btc(&core));
    tokio::select! {
        _ = ui_task(core.clone(), balance_content.clone()).await => (),
        _ = update_utxos(core.clone()).await => (),
        _ = handle_transactions(tx_receiver. clone_async(), core.clone()).await => (),
        _ = update_balance(core.clone(), balance_content).await => ()
    }
    info!("Application Shutdown!");
    Ok(())
}

mod core;
mod tasks;
mod ui;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use core::Core;
use cursive::views::TextContent;
use kanal;
use std::io::{self, Write};
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

async fn run_cli(core: Arc<Core>) -> Result<()> {
    loop {
        print!(">");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        match parts[0] {
            "balance" => {
                println!("Current balancd: {} satoshis", core.get_balance());
            }
            "send" => {
                if parts.len() != 3 {
                    println!("Usagee: send <recipient> <amount>");
                    continue;
                }
                let recipient = parts[1];
                let amount: u64 = parts[2].parse()?;
                let recipient_key = core
                    .config
                    .contacts
                    .iter()
                    .find(|r| r.name == recipient)
                    .ok_or_else(|| anyhow::anyhow!("Recipient not found"))?
                    .load()?
                    .key;
                if let Err(e) = core.fetch_utxos().await {
                    println!("failed to fetch utxos: {e}");
                };
                let transaction = core.create_transaction(&recipient_key, amount)?;
                core.tx_sender.send(transaction);
                println!("Transaction sent successfully");
                core.fetch_utxos().await?;
            }
            "exit" => break,
            _ => println!("Unkown command"),
        }
    }
    Ok(())
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

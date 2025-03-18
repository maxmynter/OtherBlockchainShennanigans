mod core;
mod utils;

use anyhow::Result;
use btclib::types::Transaction;
use clap::{Parser, Subcommand};
use core::Core;
use kanal;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::*;
use utils::generate_dummy_config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, value_name = "ADDRESS")]
    node: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    GenerateConfig {
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },
}

async fn update_utxos(core: Arc<Core>) {
    let mut interval = time::interval(Duration::from_secs(20));
    loop {
        interval.tick().await;
        if let Err(e) = core.fetch_utxos().await {
            eprint!("Failed to update UTXOs: {}", e);
        }
    }
}

async fn handle_transactions(rx: kanal::AsyncReceiver<Transaction>, core: Arc<Core>) {
    while let Ok(transaction) = rx.recv().await {
        if let Err(e) = core.send_transaction(transaction).await {
            eprintln!("Failed to send transaction: {}", e);
        }
    }
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
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::GenerateConfig { output }) => {
            return generate_dummy_config(output);
        }
        None => {}
    }
    let config_path = cli
        .config
        .unwrap_or_else(|| PathBuf::from("wallet_config.toml"));
    let mut core = Core::load(config_path.clone()).await?;
    if let Some(node) = cli.node {
        core.config.default_node = node;
    }
    let (tx_sender, tx_receiver) = kanal::bounded(10);
    core.tx_sender = tx_sender.clone();
    let core = Arc::new(core);
    tokio::spawn(update_utxos(core.clone()));
    tokio::spawn(handle_transactions(tx_receiver.clone_async(), core.clone()));
    run_cli(core).await?;
    Ok(())
}

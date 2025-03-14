use anyhow::{anyhow, Result};
use btclib::crypto::PublicKey;
use btclib::network::Message;
use btclib::types::Block;
use btclib::util::Saveable;
use clap::Parser;
use std::process::exit;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::{env, thread};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Parser)]
#[command(author, version, about, long_about = "None")]
struct Cli {
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    public_key_file: String,
}

struct Miner;
impl Miner {
    async fn new(address: String, public_key: PublicKey) -> Result<Self> {}
    async fn run(&self) -> Result<()> {}
    fn spawn_mining_thread(&self) -> thread::JoinHandle<()> {}
    async fn fetch_template(&self) -> Result<()> {}
    async fn validate_template(&self) -> Result<()> {}
    async fn submit_block(&self, block: Block) -> Result<()> {}
}

fn usage() -> ! {
    eprintln!(
        "Usage: {} <address> <public_key_file>",
        env::args().next().unwrap()
    );
    exit(1);
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let public_key = PublicKey::load_from_file(&cli.public_key_file)
        .map_err(|e| anyhow!("Error loading public key: {}", e))?;
    let miner = Miner::new(cli.address, public_key).await?;
    miner.run().await
}

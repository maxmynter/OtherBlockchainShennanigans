use btclib::crypto::PublicKey;
use btclib::network::Message;
use btclib::types::Block;
use btclib::util::Saveable;
use std::env;
use std::process::exit;
use tokio::net::TcpStream;

fn usage() -> ! {
    eprintln!(
        "Usage: {} <address> <public_key_file>",
        env::args().next().unwrap()
    );
    exit(1);
}

#[tokio::main]
async fn main() {
    let address = match env::args().nth(1) {
        Some(address) => address,
        None => usage(),
    };
    let public_key_file = match env::args().nth(2) {
        Some(public_key_file) => public_key_file,
        None => usage(),
    };
    let Ok(public_key) = PublicKey::load_from_file(&public_key_file) else {
        eprintln!("Error reading public key file {}", public_key_file);
        exit(1);
    };
    println!("Connecting to {address} to mine with {public_key:?}")
}

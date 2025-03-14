use btclib::types::Block;
use btclib::util::Saveable;
use std::env;
use std::process::exit;

fn main() {
    let (path, steps) = if let (Some(arg1), Some(arg2)) = (env::args().nth(1), env::args().nth(2)) {
        (arg1, arg2)
    } else {
        eprintln!("Usage: miner <block_file> <steps>");
        exit(1);
    };

    let steps: usize = if let Ok(s @ 1..=usize::MAX) = steps.parse() {
        s
    } else {
        eprintln!("<steps> should be a positive integer");
        exit(1);
    };
    let og_block = Block::load_from_file(path).expect("Failed to load block");
    println!("original: {:#?}", og_block);
    println!("hash: {}", og_block.header.hash());

    let mut block = og_block.clone();
    while !block.header.mine(steps) {
        println!("Mining...")
    }
    println!("final: {:#?}", block);
    println!("hash: {}", block.header.hash());
}

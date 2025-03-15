use anyhow::{Context, Result};
use btclib::network::Message;
use btclib::types::Blockchain;
use btclib::util::Saveable;
use tokio::net::TcpStream;
use tokio::time;

pub async fn load_blockchain(blockchain_file: &str) -> Result<()> {
    println!("Blockchain file exists, loading...");
    let new_blockchain = Blockchain::load_from_file(blockchain_file)?;
    println!("blockchain loaded");

    let mut blockchain = crate::BLOCKCHAIN.write().await;
    *blockchain = new_blockchain;
    println!("rebuilding utxos...");
    blockchain.rebuild_utxos();
    println!("utxos rebuilt");
    println!("checking if target needs to be adjusted");
    println!("Current target {}", blockchain.target());
    blockchain.try_adjust_target();
    println!("new target: {}", blockchain.target());
    println!("Initialization complete");
    Ok(())
}

pub async fn find_longest_chain_node() -> Result<()> {
    println!("finding nodes with the highest blockchain length");
    let mut longest_name = String::new();
    let mut longest_count = 0;
    let all_nodes = crate::NODES
        .iter()
        .map(|x| x.key().clone())
        .collect::<Vec<_>>();
    for node in all_nodes {
        println!("asking {} for blockchain length", node);
        let mut stream = crate::NODES.get_mut(&node).context("no node")?;
        let message = Message::AskDifference(0);
        message.send_async(&mut *stream).await.unwrap();
        println!("sent AskDifference to {}", node);
        let message = Message::receive_async(&mut *stream).await?;
        match message {
            Message::Difference(count) => {
                println!("received Difference from {}", node);
                if count > longest_count {
                    println!(
                        "new longest blockchain: \
                        {} from {node}",
                        count
                    );
                    longest_count = count;
                    longest_name = node;
                }
            }
            e => {
                println!("unexpected message from {}: {:?}", node, e);
            }
        }
    }
    Ok(())
}

pub async fn download_blockchain(node: &str, count: u32) -> Result<()> {
    let mut stream = crate::NODES.get_mut(node).unwrap();
    for i in 0..count as usize {
        let message = Message::FetchBlock(i);
        message.send_async(&mut *stream).await?;
        let message = Message::receive_async(&mut *stream).await?;
        match message {
            Message::NewBlock(block) => {
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                blockchain.add_block(block)?;
            }
            _ => {
                println!("unexpected message form {}", node);
            }
        }
    }
    Ok(())
}

pub async fn populate_connections(nodes: &[String]) -> Result<()> {
    println!("trying to connect to other nodes...");
    for node in nodes {
        println!("connecting to {}", node);
        let mut stream = TcpStream::connect(&node).await?;
        let message = Message::DiscoverNodes;
        message.send_async(&mut stream).await?;
        println!("sent DiscoverNodes to {}", node);
        let message = Message::receive_async(&mut stream).await?;
        match message {
            Message::NodeList(child_nodes) => {
                println!("receive NodeList from {}", node);
                for child_node in child_nodes {
                    println!("adding node {}", child_node);
                    let new_stream = TcpStream::connect(&child_node).await?;
                    crate::NODES.insert(child_node, new_stream);
                }
            }
            _ => println!("unexpected message from {}", node),
        }
        crate::NODES.insert(node.clone(), stream);
    }
    Ok(())
}

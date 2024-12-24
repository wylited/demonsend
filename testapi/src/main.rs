use std::{collections::HashMap, path::{Path, PathBuf}, time::Duration};

use localsend::{models::file::FileMetadata, Client};

#[tokio::main]
async fn main() -> localsend::error::Result<()> {
    // Initialize the client
    let client = Client::default().await?;

    // Start background tasks
    let (server_handle, udp_handle, announcement_handle) = client.start().await?;

    let file_path = PathBuf::from("/home/wyli/Repos/demonsend/example/IMG-20240924-WA0002.jpg");

    // wait until a peer is found
    while client.peers.lock().await.len() < 1 {
        println!("waiting for peers... currently {:?}", client.peers.lock().await.len());
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("received a peer!");

    let first_key = client.peers.lock().await.keys().next().unwrap().clone();
    println!("{:?}", client.send_file(first_key.clone(), file_path).await);
    server_handle.await?;
    udp_handle.await?;
    announcement_handle.await?;

    Ok(())
}

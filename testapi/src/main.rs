use std::{collections::HashMap, path::Path, time::Duration};

use localsend::{models::file::FileMetadata, Client};

#[tokio::main]
async fn main() -> localsend::error::Result<()> {
    // Initialize the client
    let client = Client::default().await?;

    // Start background tasks
    let (server_handle, udp_handle, announcement_handle) = client.start().await?;

    let file_path = Path::new("/home/wyli/Repos/demonsend/example/snowcat.png");
    let metadata = FileMetadata::from_path(&file_path)?;
    let files = HashMap::from([(metadata.id.clone(), metadata)]);

    // wait until a peer is found
    while client.peers.lock().await.len() < 1 {
        println!("waiting for peers... currently {:?}", client.peers.lock().await.len());
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("received a peer!");

    let first_key = client.peers.lock().await.keys().next().unwrap().clone();
    println!("response: {:?}", client.prepare_upload(first_key.clone(), files).await);

    server_handle.await?;
    udp_handle.await?;
    announcement_handle.await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let client = localsend::Client::default().unwrap();
    println!("{:?}", client.announce().await);
    println!("{:?}", client.listen().await);
}

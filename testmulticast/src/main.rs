use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("0.0.0.0:53317").await.unwrap();
    socket
        .join_multicast_v4("224.0.0.167".parse().unwrap(), "0.0.0.0".parse().unwrap())
        .unwrap();
    let mut buf = [0; 1024];
    loop {
        if let Ok((size, _)) = socket.recv_from(&mut buf).await {
            println!("Received {} bytes", size);
            // output the buffer as a string
            println!("{:?}", &buf[..size]);
        }
    }
    println!("Hello, world!");
}

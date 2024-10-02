use std::env;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn sender() -> io::Result<()> {
    // hard-coded for now
    let recvs = vec!["127.0.0.1:8080", "127.0.0.1:8081"];

    for recv in recvs {
        let mut stream = TcpStream::connect(recv).await?;
        for i in 1..=10 {
            let s = format!("hello from sender with i = {}", i);
            println!("sending i = {} to recv {}", i, recv);
            stream.write_all(s.as_bytes()).await?;
            stream.flush().await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    Ok(())
}

#[tokio::main]
async fn receiver(port: &str) -> io::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (mut socket, _) = listener.accept().await?;

        let mut buffer = [0; 1024];

        loop {
            match socket.read(&mut buffer).await {
                Ok(n) if n == 0 => break,
                Ok(n) => {
                    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
                }
                Err(e) => {
                    eprintln!("Failed to read from socket; err = {:?}", e);
                }
            };
        }
    }
}

// usage:
// cargo run -- send
// cargo run -- recv <port>
// (receiver port should match the sender, so <port> = 8080 | 8081)
pub fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = match args[1].as_str() {
        "send" => sender(),
        "recv" => receiver(args[2].as_str()),
        _ => panic!("not expected"),
    };
}

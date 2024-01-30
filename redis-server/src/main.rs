use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use redis_server::protocol::handle_connection;

// use tokio_http_project::http::{self as http_lib, status_codes::HTTP_NOT_FOUND, HTTPRequest, HTTPResponse, HttpSendable};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            println!("Handling connection!");
            let result = handle_connection(&mut stream).await;
            match result {
                Err(x) => {
                    println!("Failed to handle connection: {x}");
                }
                _ => {
                    println!("Closing connection!");
                }
            }
            stream.shutdown().await.unwrap_or(());
        });
    }
}


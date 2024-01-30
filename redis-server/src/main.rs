use std::error::Error;
use std::sync::Arc;
use redis_server::server::Server;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use redis_server::protocol::stream_parser::handle_connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = Arc::from(Server::new());
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut stream, _) = listener.accept().await?;

        let server_clone = server.clone();
        tokio::spawn(async move {
            println!("Handling connection!");
            let result = handle_connection(server_clone, &mut stream).await;
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

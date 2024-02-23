use std::error::Error;
use std::net::TcpListener;
use redis_server::single_server::Server;
use redis_server::protocol::stream_parser_std::handle_connection;

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new();
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    loop {
        let (mut stream, _) = listener.accept()?;

        let result = handle_connection(&mut server, &mut stream);

        match result {
            Err(x) => {
                println!("Failed to handle connection: {x}");
            }
            _ => {
                println!("Closing connection!");
            }
        }
    }
}

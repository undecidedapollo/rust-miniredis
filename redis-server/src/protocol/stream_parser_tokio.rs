use std::sync::Arc;
use std::time::Duration;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;
use crate::datatypes::DataType;
use crate::protocol::string_parser::{ParseResult, Parser};
use crate::multi_server::Server;

pub async fn handle_connection(server: Arc<Server>, stream: &mut TcpStream) -> Result<(), String> {
    let (read_stream, mut write_stream) = stream.split();
    let reader = BufReader::new(read_stream);
    let mut lines: io::Lines<BufReader<tokio::net::tcp::ReadHalf<'_>>> = reader.lines();

    loop {
        let mut parser = Parser::new();
        let mut parsed_lines = false;
        while let Some(line) = lines.next_line().await.map_err(|err| err.to_string())? {
            if line.is_empty() {
                break;
            }
            parsed_lines = true;
    
            // println!("Parsing Line: {line}");
            let parse_res = parser.next(&line).map_err(|err| err.to_string())?;
            if let ParseResult::Complete = parse_res {
                break;
            }
        }

        if !parsed_lines {
            println!("No parse!");
            sleep(Duration::from_millis(10)).await;
            return Ok(());
        }
    
        let res = parser.to_datatype()?;
        // println!("Parser output: {:?}", res);
        // println!("Parser wire representation: {:?}", res.to_wire_protocol().escape_debug());
    
        let command = res.to_command();
    
        let response = match command {
            Ok(command) => {
                let result = server.process_command(command).await?;
                result
            },
            Err(err) => {
                DataType::Error(err)
            }
        };
    
        write_stream
            .writable()
            .await
            .map_err(|err| err.to_string())?;
        let resp = format!("{}",response.to_wire_protocol());
        // println!("Writing response: {}", resp.escape_debug());
        write_stream
            .write_all(resp.as_bytes())
            .await
            .map_err(|err| err.to_string())?;
        // write_stream.flush().await.map_err(|err| err.to_string())?;
        // println!("Done");
    }
    // Ok(())
}

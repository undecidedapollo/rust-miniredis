mod parser;
mod serializer;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use crate::datatypes::DataType;
use crate::protocol::parser::{ParseResult, Parser};

pub async fn handle_connection(stream: &mut TcpStream) -> Result<(), String> {
    let (read_stream, mut write_stream) = stream.split();
    let reader = BufReader::new(read_stream);
    let mut lines: io::Lines<BufReader<tokio::net::tcp::ReadHalf<'_>>> = reader.lines();

    let mut parser = Parser::new();
    while let Some(line) = lines.next_line().await.map_err(|err| err.to_string())? {
        if line.is_empty() {
            break;
        }

        println!("Parsing Line: {line}");
        let parse_res = parser.next(&line).map_err(|err| err.to_string())?;
        if let ParseResult::Complete = parse_res {
            break;
        }
    }

    let res = parser.to_datatype()?;
    let wire_output = res.to_wire_protocol();

    println!("Parser output: {:?}\n{}", res, wire_output.escape_debug());

    write_stream
        .writable()
        .await
        .map_err(|err| err.to_string())?;
    let resp = DataType::BulkString("1".to_string()).to_wire_protocol();
    write_stream
        .write_all(resp.as_bytes())
        .await
        .map_err(|err| err.to_string())?;
    println!("Done");
    Ok(())
}

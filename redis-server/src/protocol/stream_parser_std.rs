use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use crate::datatypes::DataType;
use crate::protocol::string_parser::{ParseResult, Parser};
use crate::single_server::Server;

pub fn handle_connection(server: &mut Server, stream: &mut TcpStream) -> Result<(), String> {
    // let (read_stream, mut write_stream) = stream.;
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    // let mut lines: io::Lines<BufReader<tokio::net::tcp::ReadHalf<'_>>> = reader.lines();

    loop {
        let mut parser = Parser::new();
        let mut parsed_lines = false;
        loop {
            let mut str = String::new();
            let res = reader.read_line(&mut str).map_err(|err| err.to_string())?;
            if res == 0 || str.is_empty() {
                break;
            }
            let str = str.trim();
            parsed_lines = true;
    
            // println!("Parsing Line: {str}");
            let parse_res = parser.next(&str).map_err(|err| err.to_string())?;
            if let ParseResult::Complete = parse_res {
                break;
            }
        }

        if !parsed_lines {
            println!("No parse!");
            return Ok(());
        }
    
        let res = parser.to_datatype()?;
        // println!("Parser output: {:?}", res);
        // println!("Parser wire representation: {:?}", res.to_wire_protocol().escape_debug());
    
        let command = res.to_command();
    
        let response = match command {
            Ok(command) => {
                let result = server.process_command(command)?;
                result
            },
            Err(err) => {
                DataType::Error(err)
            }
        };

        let resp = format!("{}",response.to_wire_protocol());
        stream.write_all(resp.as_bytes()).map_err(|err| err.to_string())?;
    }
    // Ok(())
}

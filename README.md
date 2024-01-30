# rust-miniredis

This is a simple Redis server written in Rust. My motivation was I wanted to go farther than the Tokio tutorial of using the built in parsers and try to parse / serialize the Redis serialization protocol (RESP v2) myself.

## Usage

```bash
cargo run -p redis-server
```

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

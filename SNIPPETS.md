Snippet for reading a raw string from a TCP socket. Switched to buf reader instead.

```rs
async fn handle_connection(stream: &mut TcpStream) -> Result<(), String> {


    loop {
        // Wait till ready
        let ready = stream
            .ready(Interest::READABLE)
            .await
            .map_err(|err| err.to_string())?;
        if ready.is_readable() {
            let mut data = vec![0; 1024];
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(&mut data) {
                Ok(n) => {
                    println!("read {} bytes", n);
                    if n == 0 {
                        break;
                    }

                    let mut buf_str = String::with_capacity(n);
                    let mut x = &data[0..n];
                    let res = x.read_to_string(&mut buf_str).await.map_err(|err| err.to_string())?;
                    if res == 0 {
                        println!("Something happened? read_to_string was size 0");
                    }

                    println!("Read string: {buf_str}");
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    println!("Would block");
                    continue;
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            }
        }

        // let n = stream.read(&mut buffer[..]).await.map_err(|err| err.to_string())?;
        // println!("Read {n} bytes: {:#?}", buffer);
        // if n == 0 {break};
    }

    Ok(())
}
```
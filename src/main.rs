mod http;

use tokio::net::TcpListener;

use crate::http::HttpRequest;

const _MAIN_SERVER: &str = "127.0.0.1:4001";
const _SHADOW_SERVER: &str = "127.0.0.1:4002";
const PROXY: &str = "127.0.0.1:1234";

fn main() -> Result<(), std::io::Error> {
    // TODO: configure the amount of main and comparison threads with external
    // configuration (JSON/cli/...)
    let main_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10) /* use 10 threads for handling connections */
        .enable_io()
        .build()?;

    main_rt.block_on(async {
        let listener = TcpListener::bind(PROXY);
        let listener = listener.await.expect("proxy is not available");

        loop {
            let (tcpstream, addr) = listener
                .accept()
                .await
                .expect("could not accept incoming tcp stream");

            main_rt.spawn(handle_connection(tcpstream));

            eprintln!("{}", addr);
        }
    });

    Ok(())
}

async fn handle_connection(client_stream: tokio::net::TcpStream) {
    const BUFSIZE: usize = 1500;
    let mut localbuf = [0u8; BUFSIZE];
    let mut request = HttpRequest::default();

    // FIX: I need to check when the request is ending, there is no nice way
    // to do this now.

    loop {
        client_stream
            .readable()
            .await
            .expect("stream should be readable");

        match client_stream.try_read(&mut localbuf) {
            Ok(0) => {
                eprintln!("read 0 bytes, stop reading");
                break;
            }
            Ok(n) => {
                eprintln!("read {n} bytes from the client");
                match request.add_bytes(&localbuf[0..n]) {
                    Ok(true) => break,
                    Ok(false) => (),
                    Err(e) => {
                        // TODO: problem parsing means invalid request, send
                        // back 400 bad request and log the issue
                        eprintln!("problem adding bytes");
                        eprintln!("{e}");
                    }
                };

                if n < BUFSIZE {
                    // FIX: use this for testing now. later, let the http
                    // request parser deal with deciding if finished
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                eprintln!("error reading tcp stream: {}", e);
                break;
            }
        }
    }

    loop {
        match client_stream.try_write(b"HTTP/1.1 200 OK\n") {
            Ok(n) => {
                eprintln!("written {n} bytes to client");
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        }
    }

    eprintln!("handled client request");
}

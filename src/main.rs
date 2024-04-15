mod http;

use std::io::BufRead;

use tokio::net::TcpListener;

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
    let mut localbuf = [0u8; 1500];
    let mut incoming: Vec<u8> = Vec::new();

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
                incoming.extend_from_slice(&localbuf[0..n]);

                let c = incoming.clone();
                c.lines()
                    .map(|x| x.unwrap())
                    .for_each(|val| {
                        eprintln!("val:|{}|", val);
                    });

                eprintln!("current content:\n{}", String::from_utf8(incoming.clone()).unwrap());

                // TODO: parse here. should parse untill http request is
                // completely read, use some sort of type state pattern here
                // maybe? it all depends on the content AS it is parsed...
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

    eprintln!("handled client request");
}

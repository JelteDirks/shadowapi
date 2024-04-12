mod http;
use crate::http::*;

use tokio::net::TcpListener;

const MAIN_SERVER: &str = "127.0.0.1:4001";
const SHADOW_SERVER: &str = "127.0.0.1:4002";
const PROXY: &str = "127.0.0.1:1234";

fn main() -> Result<(), std::io::Error> {
    // TODO: configure the amount of main and comparison threads with external
    // configuration (JSON/cli/...)
    let main_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(10) /* use 10 threads for handling connections */
        .enable_io()
        .build()?;
    let compare_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2) /* use 2 threads for comparing */
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

async fn handle_connection(tcpstream: tokio::net::TcpStream) {
    let mut localbuf = [0u8; 1024];
    let mut request_builder: HttpRequestBuilder = HttpRequestBuilder::new();

    loop {
        tcpstream
            .readable()
            .await
            .expect("stream should be readable");

        match tcpstream.try_read(&mut localbuf) {
            Ok(0) => {
                eprintln!("finished reading or zero len buffer");
                break;
            }
            Ok(n) => {
                request_builder.insert_bytes(&localbuf[0..n]);
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

    let request = request_builder.build();
}

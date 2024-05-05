mod http;
mod util;

// NOTE: Maybe in the future, replace 'home made' logging with a crate that has
// better configurable logging. Might be nice.

use chrono::Utc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{http::request::RawHttpRequest, util::log};

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

            log::timed_msg(format!("client connected: {}", addr), Utc::now());
        }
    });

    Ok(())
}

async fn handle_connection(mut client_stream: tokio::net::TcpStream) {
    const BUFSIZE: usize = 1500;
    let mut localbuf = [0u8; BUFSIZE];
    let mut request = RawHttpRequest::default();

    // FIX: I need to check when the request is ending, there is no nice way
    // to do this now.

    loop {
        client_stream
            .readable()
            .await
            .expect("stream should be readable"); // readable

        match client_stream.try_read(&mut localbuf) {
            Ok(0) => {
                log::timed_msg(format!("read 0 bytes, stop reading"), Utc::now());
                break;
            }
            Ok(n) => {
                log::timed_msg(format!("read {n} bytes from the client"), Utc::now());
                request.add_bytes(&localbuf[0..n], n);
                if n < BUFSIZE {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                log::timed_msg(format!("error reading tcp stream: {}", e), Utc::now());
                break;
            }
        }
    }

    let main_server = TcpStream::connect(_MAIN_SERVER).await;

    if let Err(e) = main_server {
        log::timed_msg(
            format!("could not connect to main server, aborting: {e}"),
            Utc::now(),
        );
        let res = client_stream
            .write_all(b"HTTP/1.1 503 Service Unavailable\n")
            .await;
        if let Err(e) = res {
            log::timed_msg(format!("failed to write error to client: {e}"), Utc::now());
            log::timed_msg(
                format!("the client might not know what happend now"),
                Utc::now(),
            );
        }
        log::timed_msg(format!("trying to shut down the connection"), Utc::now());
        let res = client_stream.shutdown().await;
        if let Err(e) = res {
            log::timed_msg(
                format!("could not shut down gracefully, dropping connection: {e}"),
                Utc::now(),
            );
        } else {
            log::timed_msg(format!("connection is shut down"), Utc::now());
        }
        return;
    }

    let mut main_server = main_server.unwrap();

    let res = main_server.write_all(request.bytes.as_slice()).await;

    if let Err(e) = res {
        log::timed_msg(
            format!("problem forwarding request to main server: {e}"),
            Utc::now(),
        );

        log::timed_msg(format!("trying to shut down gracefully"), Utc::now());
        let res = client_stream.shutdown().await;
        if let Err(e) = res {
            log::timed_msg(
                format!("could not shut down gracefully, dropping connection: {e}"),
                Utc::now(),
            );
        } else {
            log::timed_msg(format!("connection is shut down"), Utc::now());
        }
        return;
    }

    let mut response: Vec<_> = Vec::new();

    let res = main_server.read_buf(&mut response).await;
    if let Err(e) = res {
        log::timed_msg(
            format!("problem reading response from main server: {e}"),
            Utc::now(),
        );
        let res = client_stream.write_all(b"HTTP/1.1 500 \n").await;
        if let Err(e) = res {
            log::timed_msg(format!("failed to write error to client: {e}"), Utc::now());
            log::timed_msg(
                format!("the client might not know what happend now"),
                Utc::now(),
            );
        }

        log::timed_msg(format!("trying to shut down gracefully"), Utc::now());
        let res = client_stream.shutdown().await;
        if let Err(e) = res {
            log::timed_msg(
                format!("could not shut down gracefully, dropping connection: {e}"),
                Utc::now(),
            );
        } else {
            log::timed_msg(format!("connection is shut down"), Utc::now());
        }
        return;
    }

    let res = client_stream.write_all(&response).await;
    if let Err(e) = res {
        log::timed_msg(format!("problem responding to client: {e}"), Utc::now());
        log::timed_msg(
            format!("aborting now, client is not informed..."),
            Utc::now(),
        );
        let res = client_stream.shutdown().await;
        if let Err(e) = res {
            log::timed_msg(
                format!("could not shut down gracefully, dropping connection: {e}"),
                Utc::now(),
            );
        } else {
            log::timed_msg(format!("connection is shut down"), Utc::now());
        }
        return;
    }

    let res = client_stream.shutdown().await;
    if let Err(e) = res {
        log::timed_msg(
            format!("could not shut down gracefully, dropping connection: {e}"),
            Utc::now(),
        );
    } else {
        log::timed_msg(format!("connection is shut down"), Utc::now());
    }

    log::timed_msg(format!("handled client request"), Utc::now());
}

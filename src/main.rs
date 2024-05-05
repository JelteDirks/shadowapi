mod http;
mod util;

// NOTE: Maybe in the future, replace 'home made' logging with a crate that has
// better configurable logging. Might be nice.

use std::error::Error;

use chrono::Utc;

use http::{error::ServerError, response::RawHttpResponse};
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

    let main_response = request_server(_MAIN_SERVER, &request).await;

    if let Err(e) = main_response {
        log::timed_msg(format!("error with main: {e}"), Utc::now());
        let response = match e {
            ServerError::Unresponsive(_, _) => "HTTP/1.1 503 Service Unavailable",
            ServerError::ServerWriteError(_, _) => "HTTP/1.1 500 Internal Server Error",
            ServerError::ServerReadError(_, _) => "HTTP/1.1 500 Internal Server Error",
        };
        client_stream
            .write_all(response.as_bytes())
            .await
            .expect("expect client to be okay for now");
    } else {
        client_stream
            .write_all(&main_response.unwrap().bytes)
            .await
            .expect("expect client to be okay for now");
    }

    let _ = client_stream.shutdown();

    log::timed_msg(format!("handled client request"), Utc::now());
}

async fn request_server<T>(
    target: T,
    request: &RawHttpRequest,
) -> Result<RawHttpResponse, ServerError>
where
    T: Into<String>,
{
    let target = String::from(target.into());
    let main_server = TcpStream::connect(target.clone()).await;

    if let Err(e) = main_server {
        return Err(ServerError::Unresponsive(target, Box::new(e)));
    }

    let mut main_server = main_server.unwrap();

    let res = main_server.write_all(request.bytes.as_slice()).await;

    if let Err(e) = res {
        return Err(ServerError::ServerWriteError(target, Box::new(e)));
    }

    let mut response: Vec<_> = Vec::new();

    let res = main_server.read_buf(&mut response).await;

    if let Err(e) = res {
        return Err(ServerError::ServerReadError(target, Box::new(e)));
    }

    return Ok(RawHttpResponse::from(response));
}

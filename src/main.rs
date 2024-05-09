mod http;
mod util;

// NOTE: Maybe in the future, replace 'home made' logging with a crate that has
// better configurable logging. Might be nice.

use std::time::Duration;

use chrono::Utc;

use http::{error::ServerError, response::RawHttpResponse};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Runtime,
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
        .enable_time()
        .build()?;

    let parsing_rt: Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_io()
        .enable_time()
        .build()?;

    let (mut tx, mut rx) = tokio::sync::mpsc::channel::<RawHttpRequest>(10_000);

    parsing_rt.spawn(async move {
        loop {
            let v = rx.recv().await;

            if let Some(request) = v {
                tokio::time::sleep(Duration::from_secs(3)).await;
                dbg!("got request");
            }
        }
    });

    main_rt.block_on(async {
        let listener = TcpListener::bind(PROXY);
        let listener = listener.await.expect("proxy is not available");

        loop {
            let (tcpstream, addr) = listener
                .accept()
                .await
                .expect("could not accept incoming tcp stream");

            let ltx = tx.clone();

            main_rt.spawn(async move {
                let result = handle_connection(tcpstream).await;

                if let Err(e) = result {
                    log::timed_msg(format!("issue in main server: {}", e), Utc::now());
                    return;
                }

                let (req, res) = result.unwrap();

                let result = request_server(_SHADOW_SERVER, &req).await;

                if let Err(e) = result {
                    log::timed_msg(format!("issue in shadow server: {}", e), Utc::now());
                    return;
                }

                // TODO: maybe investigate if this makes sure that there is
                // a possibility to reduce the amount of bytes that might have
                // be moved. I don't know if moving an entire struct incurs
                // more overhead than just moving a box pointing to the struct
                let _ = ltx.send(req).await;

                dbg!("debugging call");
            });

            log::timed_msg(format!("client connected: {}", addr), Utc::now());
        }
    });

    Ok(())
}

async fn handle_connection(
    mut client_stream: tokio::net::TcpStream,
) -> Result<(RawHttpRequest, RawHttpResponse), ServerError> {
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
            ServerError::ServerWriteError(_, _) | ServerError::ServerReadError(_, _) => {
                "HTTP/1.1 500 Internal Server Error"
            }
        };
        client_stream
            .write_all(response.as_bytes())
            .await
            .expect("expect client to be okay for now");

        return Err(e);
    } else {
        client_stream
            .write_all(&main_response.as_ref().unwrap().bytes)
            .await
            .expect("expect client to be okay for now");
    }

    let _ = client_stream.shutdown();
    log::timed_msg(format!("handled client request"), Utc::now());

    // NOTE: the fun part starts here

    // TODO: before the shadow server is called and handeled, maybe already
    // save some information about the request and response from main, maybe
    // to a database or somewhere that can generate statistics and comparison
    // results later.

    return Ok((request, main_response.unwrap()));
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

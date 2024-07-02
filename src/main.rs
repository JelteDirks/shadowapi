#![allow(dead_code)]

mod http;
mod util;

// NOTE: Maybe in the future, replace 'home made' logging with a crate that has
// better configurable logging. Might be nice. Nice crate called 'tracing'.
//
// NOTE: it might be nice to configure the BUFSIZE when the user of the system
// knows that incoming requests are always short/long and need less/more memory
// to copy at once. Just for optimization.

// https://httpwg.org/specs/rfc9112.html#message.format
// https://datatracker.ietf.org/doc/html/rfc9110

use chrono::Utc;

use http::{error::ServerError, response::RawHttpResponse};
use tokio::{
    io::AsyncWriteExt,
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
        .build()?;

    let parsing_rt: Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_io()
        .build()?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(RawHttpRequest, RawHttpResponse)>(1_000);

    parsing_rt.spawn(async move {
        loop {
            let v = rx.recv().await;

            if let None = v {
                log::timed_msg(format!("received None from channel"), Utc::now());
                continue;
            }

            tokio::spawn(async move {
                let (raw_request, main_response) = v.unwrap();
                let shadow_response = request_server(_SHADOW_SERVER, &raw_request).await;

                if let Err(e) = shadow_response {
                    log::timed_msg(format!("error parsing shadow response: {}", e), Utc::now());
                    return;
                }

                let shadow_response = shadow_response.unwrap();

                let parsed_request = raw_request.decode();
                if let Err(e) = parsed_request {
                    log::timed_msg(format!("error parsing request: {}", e), Utc::now());
                } else {
                    // TODO: write the result somewhere?
                }

                let main_parsed = main_response.decode();
                if let Err(e) = main_parsed {
                    log::timed_msg(format!("error parsing main response: {}", e), Utc::now());
                } else {
                    // TODO: write the result somewhere?
                }

                let shadow_parsed = shadow_response.decode();
                if let Err(e) = main_parsed {
                    log::timed_msg(format!("error parsing shadow response: {}", e), Utc::now());
                } else {
                    // TODO: write the result somewhere?
                }

                if main_parsed.is_ok() && shadow_parsed.is_ok() {
                    // TODO: compare the results and store
                }
            });
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

                let sent = ltx.send(result.unwrap()).await;

                if let Err(_) = sent {
                    log::timed_msg(format!("receiver dropped"), Utc::now());
                    // NOTE: the send method can return SendError which holds
                    // the T that was sent but failed. Send blocks if there
                    // is no capacity, so the receiver has probably been
                    // dropped. I don't think the I can restart the receiver in
                    // an ergonomic way here.
                }
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

    loop {
        let readable = client_stream.readable().await;

        if let Err(e) = readable {
            return Err(ServerError::ServerReadError(
                String::from("client"),
                Box::new(e),
            ));
        }

        match client_stream.try_read(&mut localbuf) {
            Ok(0) => break,
            Ok(n) => {
                request.add_bytes(&localbuf[0..n], n);
                if n < BUFSIZE {
                    // HACK: assume that last chunk of message was received
                    // here. Not verified.
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(ServerError::ServerReadError(
                    String::from("client"),
                    Box::new(e),
                ));
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

    let _ = client_stream.shutdown().await;
    log::timed_msg(format!("handled client request"), Utc::now());

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
    let target: String = target.into();
    let server = TcpStream::connect(target.clone()).await;

    if let Err(e) = server {
        return Err(ServerError::Unresponsive(target, Box::new(e)));
    }

    let mut server = server.unwrap();

    let res = server.write_all(request.bytes.as_slice()).await;

    if let Err(e) = res {
        return Err(ServerError::ServerWriteError(target, Box::new(e)));
    }

    const BUFSIZE: usize = 1500;
    let mut localbuf = [0u8; BUFSIZE];
    let mut response: Vec<_> = Vec::with_capacity(BUFSIZE);
    loop {
        let readable = server.readable().await;

        if let Err(e) = readable {
            return Err(ServerError::ServerReadError(target, Box::new(e)));
        }

        match server.try_read(&mut localbuf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&localbuf[0..n]);
                if n < BUFSIZE {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(ServerError::ServerReadError(target, Box::new(e)));
            }
        }
    }

    if let Err(e) = res {
        return Err(ServerError::ServerReadError(target, Box::new(e)));
    }

    return Ok(RawHttpResponse::from(response));
}

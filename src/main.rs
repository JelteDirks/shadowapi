use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

mod http;

use crate::http::*;

const MAIN_SERVER: &str = "127.0.0.1:4001";
const SHADOW_SERVER: &str = "127.0.0.1:4002";
const PROXY: &str = "127.0.0.1:1234";

fn main() {
    let listener = TcpListener::bind(PROXY).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match handle_connection(&mut stream) {
            Ok(rl) => {
                dbg!(&rl);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<RequestWrapper, ConnectionError> {
    let mut bufreader = BufReader::new(stream);
    let mut content = String::new();
    let mut request_line = None;

    match bufreader.read_line(&mut content) {
        Ok(_) => {
            if let Ok(rl) = RequestLine::from_string(&content) {
                request_line = Some(rl);
            } else {
                return Err(ConnectionError::BadRequestLine);
            }
        }
        Err(e) => {
            dbg!(e);
        }
    }

    return Ok(RequestWrapper { request_line });
}

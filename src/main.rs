use std::fs;
use std::io;
use std::net::SocketAddr;
use std::sync::mpsc;

use futures::prelude::*;
use futures::try_ready;

use tokio::codec::{BytesCodec, Decoder, Encoder, Framed};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use failure::{Error, Fail, Fallible};

use serde_derive::*;
use serde_json;

use bytes::{ BufMut, BytesMut };
use core::borrow::BorrowMut;

mod server_node;
mod control_node;
mod utils;


fn main() {
//    let addr = "127.0.0.1:3030".parse::<SocketAddr>().unwrap();
//    let listener = TcpListener::bind(&addr).unwrap();
//
//    println!("I'm Listening {}", addr.port());
//
//    let get = b"GET / HTTP/1.1\r\n";
//
//    let server = listener
//        .incoming()
//        .for_each(move |sock| {
//            find_node();
//            connect_node(sock);
//            Ok(())
//        })
//        .map_err(|e| eprintln!("error in incoming: {}", e));
//
//    tokio::run(server);
//
//    eprintln!("server shutdown");
}

fn find_node() {

}

fn connect_node(sock: TcpStream) {
//    let mut buf: Vec<u8> = vec![];
//    let rw_async = ReadWrite::new(sock);
//
//    let conn = rw_async.into_future()
//        .map_err(|(e, _)| e)
//        .and_then(|(req, mut stream)| {
//
//            let get = req.clone().expect("No request found");
//
//            println!("Request: {}", String::from_utf8_lossy(req.unwrap().as_ref()));
//
//            if get.starts_with(b"GET / HTTP/1.1") {
//
//                stream.buffer_u8(resp.unwrap().as_bytes());
//                stream.poll_flush()?;
//            }
//
//            Ok(())
//        })
//        .map_err(|e| eprintln!("{:#?}", e));
//
//    tokio::spawn(conn);
}

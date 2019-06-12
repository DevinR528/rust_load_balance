use std::fs;
use std::io;
use std::net::SocketAddr;
//use std::sync::mpsc;

use futures::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;

use tokio::codec::{BytesCodec, Decoder, Encoder, Framed};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

use failure::{Error, Fail, Fallible};

use serde_derive::*;
use serde_json;

use bytes::{ BufMut, BytesMut };
use core::borrow::BorrowMut;
use tokio::net::tcp::ConnectFuture;

use super::control_node::{ ControlNode };
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeMsg {
    id: SocketAddr,
    count: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeState {
    id: SocketAddr,
    count: u8,
}

pub type NodeSender = Arc<Mutex<mpsc::Sender<NodeMsg>>>;

#[derive(Debug)]
pub struct ServerNode {
    addr: SocketAddr,
    // this may change
    sock: Option<TcpStream>,
    recv: mpsc::Receiver<NodeMsg>,
    tx: NodeSender,
}

impl ServerNode {
    pub fn new(
        addr: &SocketAddr,
        recv: mpsc::Receiver<NodeMsg>,
        tx: mpsc::Sender<NodeMsg>,
    ) -> ServerNode {

        ServerNode {
            addr: addr.to_owned(),
            // TODO
            sock: None,
            recv,
            tx: Arc::new(Mutex::new(tx)),
        }
    }
}

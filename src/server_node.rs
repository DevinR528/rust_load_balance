use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;

use serde_derive::*;

use tokio::net::{ TcpStream };
use tokio::prelude::*;

use bytes::{ BytesMut };

use super::err;


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

#[derive(Debug, Clone)]
struct TcpBytes {
    in_out_vec: BytesMut,
    first: bool,
}

impl TcpBytes {
    fn default() -> Self {
        TcpBytes {
            in_out_vec: BytesMut::default(),
            first: true
        }
    }

    pub fn first_connected(&self) -> bool {
        self.in_out_vec.is_empty() && self.first
    }
}

#[derive(Debug)]
pub struct StreamHold(Option<TcpStream>);

impl StreamHold {
    fn default() -> Self {
        StreamHold(None)
    }
}

pub type NodeSender = Arc<Mutex<mpsc::UnboundedSender<NodeMsg>>>;

#[derive(Debug)]
pub struct ServerNode {
    /// address and sort of id for node
    pub addr: SocketAddr,
    /// Sender for node to controller
    tx: NodeSender,
    /// state and storage of tcp stream
    stream_bytes: TcpBytes,
    tcp_stream: Option<TcpStream>,
}

impl ServerNode {
    pub fn new(
        addr: &SocketAddr,
        tx: mpsc::UnboundedSender<NodeMsg>,
    ) -> Self {

        ServerNode {
            addr: addr.to_owned(),
            // TODO
            tx: Arc::new(Mutex::new(tx)),

            /// holds bytes from tcp stream also checks first bytes to
            /// send back connected status to controller
            stream_bytes: TcpBytes::default(),
            /// tcp stream from controller
            tcp_stream: None,
        }
    }

    pub fn process_conn(&mut self, sock: TcpStream)
        -> impl Future<Item=(), Error=()> + 'static
    {
        ServerNode {
            addr: self.addr.clone(),
            tcp_stream: Some(sock),
            tx: self.tx.clone(),
            stream_bytes: self.stream_bytes.clone(),
        }
    }

    fn poll_read_tcp(&mut self) -> Poll<Option<()>, err::LBError> {
        loop {
            let did_conn = self.stream_bytes.first_connected();
            // tcp stream has to be here by now or error
            match self.tcp_stream.as_mut().unwrap().poll_read(self.stream_bytes.in_out_vec.as_mut())? {
                Async::Ready(_n) => {
                    println!("[Poll] {:#?}", self.stream_bytes.in_out_vec);

                    if did_conn {
                        self.tx.lock().unwrap().unbounded_send(
                            NodeMsg {id: self.addr, count: 1}
                        ).expect("Send failed work on this");

                        self.stream_bytes.first = false;
                    }

                    return Ok(Async::Ready(Some(())));
                },
                Async::NotReady => {
                    println!("[Poll not ready] {:#?}", self.stream_bytes.in_out_vec);
                    return Ok(Async::NotReady);
                },
            }
        }
    }

    fn poll_write_tcp(&mut self) -> Poll<(), err::LBError> {
        while self.stream_bytes.in_out_vec.len() != 0 {
            // tcp stream needs to be there.  TODO: handle cases sep no tryready
            let num = try_ready!(self.tcp_stream.as_mut().unwrap()
                .poll_write(&self.stream_bytes.in_out_vec));
            self.stream_bytes.in_out_vec.split_to(num);
        }
        self.tcp_stream.as_mut().unwrap().poll_flush().map_err(|e| e.into())
//        self.tcp_stream.unwrap().poll_flush().map_err(|e| e.into())?;
//        Ok(Async::Ready(()))
    }

}

impl Future for ServerNode {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match try_ready!(self.poll_read_tcp()) {
            Some(_m) => {
                self.poll_write_tcp().map_err(|e| ())
            },
            None => return Err(()),
        }
    }
}

impl Drop for ServerNode {
    fn drop(&mut self) {
        println!("%% ServerNode DROP %%");
    }
}

struct FutureNode {

}

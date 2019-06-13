use std::io;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::{ BufMut, BytesMut };

use failure::{ err_msg, Error, Fail, Fallible};

use futures::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;

use serde_derive::*;
use serde_json;

use tokio::codec::{BytesCodec, Decoder, Encoder, Framed};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{ AsyncWrite, AsyncRead };
use tokio::prelude::*;

use super::server_node::{ NodeMsg, ServerNode, NodeState, NodeSender };
use super::utils::*;

use tokio::reactor::Handle;
use tokio::io::ErrorKind;
use std::sync::mpsc::TrySendError::{ Disconnected, Full };


struct CurrentNodeTx(NodeSender);

pub struct ControlNode {
    /// Reciving end of msg from node
    recv: mpsc::Receiver<NodeMsg>,
    /// hashmap of soket addr and each nodes trans channel
    nodes: HashMap<SocketAddr, NodeSender>,
    /// hashmap of sacket addr and node state (how many conns)
    node_states: HashMap<SocketAddr, NodeState>,
    /// sorted haha que of nodes by socket addr
    load_order: VecDeque<SocketAddr>,
    // TOD0
    /// stored msg from node probably dont need
    msg_from_node: Option<NodeMsg>,
    /// holds temp node tx for ease of use
    curr_node_tx: Option<CurrentNodeTx>,
}

impl ControlNode {
    pub fn new(recv: mpsc::Receiver<NodeMsg>,) -> Self {
        ControlNode {
            recv,
            nodes: HashMap::new(),
            node_states: HashMap::new(),
            load_order: VecDeque::new(),
            // used by stream and sink
            msg_from_node: None,
            curr_node_tx: None,
        }
    }

    pub fn add_node(
        &mut self,
        addr: &SocketAddr,
        send: Arc<Mutex<mpsc::Sender<NodeMsg>>>,
    ) {
        self.nodes.insert(addr.clone(), send);
    }

    pub fn update(&self)  {
        println!("Called update do something")

    }

    pub fn set_curr(&mut self, n: NodeSender) {
        self.curr_node_tx = Some(CurrentNodeTx(n))
    }

    pub fn send_one(&mut self, s_addr: &SocketAddr, msg: NodeMsg)
        -> Result<(), mpsc::TrySendError<NodeMsg>>
    {
        let tx: &mut Arc<Mutex<mpsc::Sender<NodeMsg>>> =
            self.nodes.get_mut(s_addr).expect("Socket Address not found");
        tx.lock().unwrap().try_send(msg)
    }

    pub fn send_all(&self, msg: NodeMsg) -> Result<(), mpsc::TrySendError<NodeMsg>>{

        let n: Vec<Result<(), mpsc::TrySendError<NodeMsg>>> =
            self.nodes.iter()
                .map(|(addr, s)| s.lock().unwrap().try_send(msg.to_owned()))
                .filter(|res| res.is_err())
                .collect();

        if n.is_empty() {
            Ok(())
        } else {
            eprintln!("[send_all] {:#?}", n);
            n[0].clone()
        }
    }

    fn sort_load_order(&self) {

    }

    pub fn find_node(&mut self)
        -> Result<NodeSender, impl std::error::Error>
    {
        let err = std::io::Error::new(ErrorKind::AddrNotAvailable, "Not found");
        let addr = match self.load_order.front() {
            Some(a) => {
                //TODO
                self.sort_load_order();
                a
            },
            // TODO
            None => return Err(err),
        };
        match self.nodes.get_mut(&addr) {
            Some(tx) => Ok(tx.clone()),
            None => Err(err)
        }
    }

//    pub fn send_to_node(&mut self, s: TcpStream)
//        -> Result<(), impl std::error::Error>
//    {
//        let err = std::io::Error::new(ErrorKind::BrokenPipe, "Full or Disconected");
//        let res = self.find_node()
//            .map_err(|e| err)
//            .map(|tx| {
//                let addr = "127.0.0.0.1:4567".parse::<SocketAddr>().expect("not good socket addr");
//                tx.lock().unwrap().try_send(NodeMsg { id: addr, count: 13})
//                    .map_err(|e| err)
//                    .map(|res| res)
//            });
//        Ok(())
//    }
}

impl Stream for ControlNode {

    type Item = NodeMsg;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.recv.poll() {
            Ok(Async::Ready(Some(n))) => {
                println!("[Poll] {:#?}", n);

                Ok(Async::Ready(Some(n)))
            },
            Ok(Async::Ready(None)) => {
                eprintln!("[poll Disconn] disconnected in poll");
                Ok(Async::Ready(None))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(From::from(e)),
        }

    }
}

impl Sink for ControlNode {
    type SinkItem = NodeMsg;
    type SinkError = ();

    fn start_send(&mut self, item: Self::SinkItem)
                  -> StartSend<Self::SinkItem, Self::SinkError>
    {
        if self.load_order.is_empty() {
            println!("Load order empty");
            return Ok(AsyncSink::NotReady(item));
        }

        // get best node
        match self.find_node() {
            Ok(tx) => {
                self.set_curr(tx);
                // TODO
                // try to send item to node eventually this will be conn?
                match tx.clone().lock().unwrap().try_send(item.clone()) {
                    Ok(()) => Ok(AsyncSink::Ready),
                    Err(err) => {
                        if err.is_disconnected() {
                            eprintln!("[Error in try send] Disconnected")
                        } else {
                            eprintln!("[Error in try send] Full")
                        }
                        Err(())
                    },
                }
            },
            Err(e) => {
                eprintln!("[find node] {}", e);
                Err(())
            },
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self.curr_node_tx.unwrap().0.lock().unwrap()
            .poll_complete().unwrap() {
                Async::Ready(_) => Ok(Async::Ready(())),
                Async::NotReady => Ok(Async::NotReady),
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

impl Drop for ControlNode {
    fn drop(&mut self) {
        println!("%% ControlNode DROP %%");
    }
}


#[cfg(test)]
mod tests {
    use bytes::buf::IntoBuf;
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_load() {
        let addr = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let addrs = [
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap(),
            "127.0.0.1:8081".parse::<SocketAddr>().unwrap(),
        ];

        let mut nodes: Vec<ServerNode> = vec![];

        // send to control from node, updates control with new node state
        let (node_send, node_recv) = mpsc::channel::<NodeMsg>(100);
        // how to handle each tcp stream
        let mut control = Arc::new(Mutex::new(ControlNode::new(node_recv)));

        for a in addrs.iter() {
            // send to node from control, sends jobs to node threads
            let (control_send, control_recv) = mpsc::channel::<NodeMsg>(100);

            let mut n = ServerNode::new(a, control_recv, node_send);
            nodes.push(n);
            control.add_node(a, control_send);

        }

        let mut count = 0;
        let serv = listener.incoming()
            .for_each(|sock| {
                let addr = "127.0.0.0.1:4567".parse::<SocketAddr>()
                    .expect("not good socket addr");

                control.frame(sock).split();
                control.lock().unwrap()
                    .start_send(NodeMsg {
                        id: addr,
                        count,
                    });
            });

        tokio::run(serv);
    }

}


use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures::prelude::*;
use futures::sync::mpsc;
use futures::try_ready;

use super::err;
use super::server_node::{ NodeMsg, ServerNode, NodeState };


type ControlSender = Arc<Mutex<mpsc::UnboundedSender<NodeMsg>>>;

#[derive(Debug)]
pub struct ControlNode {
    /// Receiving end of msg from node
    control_recv: mpsc::UnboundedReceiver<NodeMsg>,
    /// Vector of all server nodes
    nodes_map: HashMap<SocketAddr, ServerNode>,
    /// hashmap of socket addr and node state (how many conns)
    node_states: HashMap<SocketAddr, NodeState>,
    /// sorted 'haha' que of nodes by socket addr
    load_order: VecDeque<SocketAddr>,
    // TOD0
    /// stored msg from node
    msg_from_node: Option<NodeMsg>,

}

impl ControlNode {
    pub fn new(r: mpsc::UnboundedReceiver<NodeMsg>,) -> ControlNode {
        ControlNode {
            control_recv: r,
            nodes_map: HashMap::new(),
            node_states: HashMap::new(),
            load_order: VecDeque::new(),
            // used by stream and sink
            msg_from_node: None,
        }
    }
    /// Adds node to hashmap of server nodes.
    pub fn add_node(
        &mut self,
        node: ServerNode,
    ) {
        self.load_order.push_front(node.addr.clone());
        self.nodes_map.insert(node.addr.clone(), node);

    }

    /// Updates the states of all nodes and calls sort for load order.
    pub fn update(&self, msg: NodeMsg)  {
        println!("Called update do something {:#?}", msg)

    }

    /// Sorts load order que.
    fn sort_load_order(&self) {

    }

    /// Returns nodes and resorts load order.
    pub fn find_node(&mut self) -> Result<Box<&mut ServerNode>, err::LBError> {
        let err = err::LBError::InternalThrow("No node found *fatal*.".into());
        let addr = match self.load_order.front() {
            Some(a) => {
                //TODO
                self.sort_load_order();
                a
            },
            // TODO
            None => return Err(err),
        };
        match self.nodes_map.get_mut(&addr) {
            Some(n) => Ok(Box::new(n)),
            None => Err(err)
        }
    }

    fn poll_read_recv(&mut self) -> Poll<Option<()>, err::LBError> {
        match self.control_recv.poll() {
            Ok(Async::Ready(Some(n))) => {
                println!("[Poll] {:#?}", n);

                Ok(Async::Ready(Some(())))
            },
            Ok(Async::Ready(None)) => {
                eprintln!("[poll Disconnected] disconnected in poll");
                Ok(Async::Ready(None))
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(err::LBError::InternalThrow("Error in poll *Control fatal*.".into())),
        }
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
    use tokio::net::TcpListener;
    use futures::future::lazy;
    use std::io::Write;

    #[test]
    fn test_load() {
        let addr = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        let addrs = [
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap(),
            "127.0.0.1:8081".parse::<SocketAddr>().unwrap(),
        ];

        // send to control from node, updates control with new node state
        let (node_send, node_recv) = mpsc::unbounded();
        // how to handle each tcp stream
        let mut control = ControlNode::new(node_recv);

        for a in addrs.iter() {

            let mut n = ServerNode::new(a, node_send.clone());
            control.add_node(n);

        }

        let mut tcp = std::net::TcpStream::connect(&addr).unwrap();
        tcp.write_all("hello world".as_bytes());

        let serv = listener.incoming()
            .map_err(|e| eprintln!("{:#?}", e))
            .for_each(move |sock| {
                println!("connected");
                let mut p = control.find_node().unwrap().process_conn(sock);

                tokio::spawn(p);

                Ok(())
            }).and_then( |r| Ok(()) );



        tokio::run(serv);
    }

}


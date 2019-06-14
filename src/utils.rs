//
//impl Stream for ControlNode {
//
//    type Item = NodeMsg;
//    type Error = err::LBError;
//
//    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
//        match self.control_recv.poll() {
//            Ok(Async::Ready(Some(n))) => {
//                println!("[Poll] {:#?}", n);
//
//                Ok(Async::Ready(Some(n)))
//            },
//            Ok(Async::Ready(None)) => {
//                eprintln!("[poll Disconnected] disconnected in poll");
//                Ok(Async::Ready(None))
//            },
//            Ok(Async::NotReady) => Ok(Async::NotReady),
//            Err(_) => Err(err::LBError::InternalThrow("Error in poll *Control fatal*.".into())),
//        }
//
//    }
//}
//
//impl Sink for ControlNode {
//    type SinkItem = NodeMsg;
//    type SinkError = err::LBError;
//
//    fn start_send(&mut self, item: Self::SinkItem)
//                  -> StartSend<Self::SinkItem, Self::SinkError>
//    {
//        if self.load_order.is_empty() {
//            println!("Load order empty");
//            return Ok(AsyncSink::NotReady(item));
//        }
//
//        // get best node
//        match self.find_node_tx() {
//            Ok(tx) => {
//                // TODO
//                // try to send item to node eventually this will be conn?
//                match tx.lock().unwrap().try_send(item.clone()) {
//                    Ok(()) => Ok(AsyncSink::Ready),
//                    Err(err) => Err(err.into()),
//                }
//            },
//            Err(e) => Err(e),
//        }
//    }
//
//    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
//
//        match self.find_node_tx().expect("We need a node cant really error")
//            .lock().unwrap().poll_complete()? {
//            Async::Ready(_) => Ok(Async::Ready(())),
//            Async::NotReady => Ok(Async::NotReady),
//        }
//    }
//
//    fn close(&mut self) -> Poll<(), Self::SinkError> {
//        self.poll_complete()
//    }
//}
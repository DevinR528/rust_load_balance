use std::io;
use std::fs;
use std::sync::mpsc;

use serde_json;
use serde_derive::*;

use futures::prelude::*;

use tokio::codec::{ Decoder, Encoder };

use failure::{Error, Fail, Fallible};

use bytes::BytesMut;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonMsg {
    item: u32,
    name: String,
    hello: String,
}

#[derive(Debug, Clone)]
enum RequestState {
    Header,
    Body,
}
#[derive(Debug, Clone)]
pub struct JsonCodec {
    state: RequestState,
    cont_len: usize,
}

impl JsonCodec {
    pub fn new() -> JsonCodec {
        JsonCodec {
            state: RequestState::Header,
            cont_len: 0,
        }
    }
}

impl Decoder for JsonCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let raw_str = String::from_utf8_lossy(src);
        println!("{}", raw_str);

        let res = serde_json::from_slice::<JsonMsg>(src)?;
        Ok(Some(src.to_owned()))
    }
}

// writes out, this is sink
impl Encoder for JsonCodec {
    type Item = JsonMsg;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut json_buf = serde_json::to_vec(&item)?;

        println!("{:#?}", json_buf);

        dst.extend_from_slice(&json_buf);
        Ok(())
    }
}

pub struct PassThrough {
    incoming: ReadHalf<TcpStream>,
    outgoing: WriteHalf<TcpStream>,
}

impl Stream for PassThrough {
    type Item = JsonMsg;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

    }
}

impl Sink for PassThrough {
    type SinkItem = JsonMsg;
    type SinkError = ();

    fn start_send(&mut self, item: Self::SinkItem)
        -> StartSend<Self::SinkItem, Self::SinkError>
    {

    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {

    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.poll_complete()
    }
}

#[cfg(test)]
mod tests {
    use bytes::buf::IntoBuf;
    use super::*;

    #[test]
    fn test_sink_stream() {
        assert_eq!(true, true)
    }

}
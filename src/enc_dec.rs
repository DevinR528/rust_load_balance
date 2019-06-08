use std::io;
use std::fs;
use std::sync::mpsc;

use serde_json;
use serde_derive::*;

use futures::prelude::*;

use tokio::codec::{ Decoder, Encoder };

use failure::{Error, Fail, Fallible};

use bytes::BytesMut;


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
    type Item = JsonMsg;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut)
        -> Result<Option<Self::Item>, Self::Error>
    {
        let j_str = String::from_utf8_lossy(src);
        println!("{}", j_str);
        let res = serde_json::from_slice::<JsonMsg>(src)?;
        Ok(Some(res))
    }
}

impl Encoder for JsonCodec {
    type Item = JsonMsg;
    type Error = Error;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut BytesMut
    ) -> Result<(), Self::Error> {
        let j_str = serde_json::to_string(&item)?;

        dst.extend_from_slice(j_str.as_bytes());
        Ok(())
    }
}

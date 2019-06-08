use std::fs;
use std::io;
use std::net::SocketAddr;
use std::sync::mpsc;

use log::{debug, error, warn};

use futures::prelude::*;

use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::codec::{BytesCodec, Decoder, Encoder, Framed};

use failure::{Error, Fail, Fallible};

use serde_derive::*;
use serde_json;

use bytes::BytesMut;
use tokio::io::ErrorKind;
use std::borrow::Cow;
use futures::future::result;

fn main() {
    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    debug!(target: "hmm", "I'm Listening {}", addr.port());

//    let server = listener.incoming()
//        .map_err(|e| {
//           eprintln!("Failure: {:#?}", e);
//            panic!()
//        });

    let server = listener.incoming()
        .for_each(|sock| {
            debug!(target: "[in thread]", "about to open file");
            let async_file = tokio::fs::File::open("j.json")
                .map_err(|e| println!("{}", e))
                .then(|f| f.unwrap());

            let f = BytesCodec::new().framed(file);

            let amt = tokio::io::copy(read, write);

            let req = amt.then(|result| {
                match result {
                    Ok((amt, s_read, s_write)) => println!("wrote {} bytes", amt),
                    Err(e) => eprintln!("whoopsies {}", e),
                }
                Ok(())
            });
            tokio::spawn(req);
            Ok(())
        })
        .map_err(|e| {
            eprintln!("error in incoming: {}", e)
        });

    tokio::run(server);

    eprintln!("server shutdown");
}
//
//fn process<'e>(
//    sock: TcpStream,
//) -> impl Future<Item=(), Error=()> {
//    //let (sink, stream) = JsonCodec::new().framed(sock).split();
//    tokio::fs::File::open("j.json")
//        .map_err(|err| {
//            eprintln!("{}", err);
//            ()
//        })
//        .then(move |async_file| {
//            BytesCodec::new()
//                .framed(async_file.unwrap())
//                .map(Into::into)
//                .forward(BytesCodec::new().framed(sock))
//                .map(|_res| {
//                    println!("after msg sent");
//                    ()
//                })
//        })
//}
//
//fn form_resp(
//    async_file: Result<tokio::fs::File, io::Error>,
//    soc: impl AsyncRead + AsyncWrite + Send + 'static,
//) -> impl Future<Item=(), Error=io::Error> {
//
//    let f = async_file.unwrap();
//
//    let stream = BytesCodec::new()
//        .framed(f)
//        .map(Into::into)
//        .forward(BytesCodec::new().framed(soc))
//        .then(|_res| {
//            println!("after msg sent");
//            Ok(())
//        })
//        .map_err(|e: io::Error| io::Error::new(e.kind(), "uhoh"));
//    stream
//}

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
    type Item =  BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//        let raw_str = String::from_utf8_lossy(src);
//        println!("{}", raw_str);
        //let res = serde_json::from_slice::<JsonMsg>(src)?;
        Ok(Some(src.to_owned()))
    }
}

// writes out, this is sink
impl Encoder for JsonCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let raw_str = String::from_utf8_lossy(&item);
        let mut json_msg = serde_json::from_str::<JsonMsg>(&raw_str)?;

        json_msg.item += 1;

        println!("{:#?}", json_msg);
        let j_str = serde_json::to_string(&json_msg)?;
        dst.extend_from_slice(j_str.as_bytes());
        Ok(())
    }
}

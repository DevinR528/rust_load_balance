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

mod enc_dec;
use crate::enc_dec::JsonCodec;
use crate::enc_dec::JsonMsg;

fn main() {
    let addr = "127.0.0.1:3030".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    println!("I'm Listening {}", addr.port());

    let get = b"GET / HTTP/1.1\r\n";

    let server = listener
        .incoming()
        .for_each(move |sock| {
            process(sock);
            Ok(())
        })
        .map_err(|e| eprintln!("error in incoming: {}", e));

    tokio::run(server);

    eprintln!("server shutdown");
}

fn process(sock: TcpStream) {
    let mut buf: Vec<u8> = vec![];
    let rw_async = ReadWrite::new(sock);


    // return val from and_then x2
    let async_resp = tokio::fs::File::open("j.json")
        .and_then(move |mut file| {
            file.read_buf(&mut buf)
                .and_then(|_f| {

                    let header = format!(
                        "HTTP/1.1 200 OK\r\n{}{}",
                        format!("Content-Length: {}\r\n", buf.len()),
                        "Content-Type: application/json\r\n\r\n",
                    );
                    let content = String::from_utf8_lossy(&buf);

                    let resp = format!("{}{}", header, content);
                    // returns future of string response
                    Ok(resp)
                })
        })
        .map_err(|e| eprintln!("File Open Error: {}", e));

    let conn = rw_async.into_future()
        .map_err(|(e, _)| e)
        .and_then(|(req, mut stream)| {

            let get = req.clone().expect("No request found");

            println!("Request: {}", String::from_utf8_lossy(req.unwrap().as_ref()));

            if get.starts_with(b"GET / HTTP/1.1") {
                let resp = async_resp.wait();
                stream.buffer_u8(resp.unwrap().as_bytes());
                stream.poll_flush()?;
            }

            Ok(())
        })
        .map_err(|e| eprintln!("{:#?}", e));

    tokio::spawn(conn);
}

#[derive(Debug)]
struct ReadWrite {
    sock: TcpStream,
    rd: BytesMut,
    wr: BytesMut,
}

impl ReadWrite {
    fn new(s: TcpStream) -> Self {
        ReadWrite {
            sock: s,
            rd: BytesMut::new(),
            wr: BytesMut::new(),
        }
    }

    /// Buffer u8 adds to write half of ReadWrite
    ///
    /// # Examples
    /// ```
    /// use bytes::BufMut;
    ///
    /// let rw = ReadWrite::new(TcpStream);
    /// rw.buffer_u8(b"123".as_ref());
    ///
    /// let b_m = BytesMut::new();
    /// b_m.reserve(3);
    /// b_m.put(b"123".as_ref());
    ///
    /// assert_eq!(b_m, rw.rd);
    /// ```
    fn buffer_u8(&mut self, line: &[u8]) {
        self.wr.reserve(line.len());

        self.wr.put(line);
    }

    /// Writes to the TcpStream and
    /// removes all bytes successfully writen from buffer
    ///
    /// # Examples
    /// ```
    /// use bytes::BufMut;
    ///
    /// let rw = ReadWrite::new(TcpStream);
    ///
    /// rw.buffer(b"123".as_ref());
    /// rw.poll_flush();
    ///
    /// assert_eq!(BytesMut::new(), rw.wr);
    /// ```
    fn poll_flush(&mut self) -> Poll<(), tokio::io::Error> {
        while !self.wr.is_empty() {
            let n = try_ready!(self.sock.poll_write(&self.wr));
            assert!(n > 0);
            let _throw_away = self.wr.split_to(n);
        }

        Ok(Async::Ready(()))
    }

    /// Reads from the TcpStream,
    /// loops until reading from socket returns
    /// zero bytes read.
    ///
    /// # Examples
    /// ```                         calls fill_read_buf
    /// let f = ReadWrite::new(sock).into_future()
    ///     .map_err(|(e, _)| e)
    ///     .and_then(|(first, mut rw)|{
    ///         assert_eq!(b"123", first.unwrap().as_ref());
    ///         assert_eq!(b"123".as_ref(), rw.rd.as_ref());
    ///         Ok(())
    ///     }).map_err(|e| eprintln!("{}", e));
    ///     tokio::spawn(f);
    ///
    /// ```
    fn fill_read_buf(&mut self) -> Poll<(), tokio::io::Error> {
        loop {
            self.rd.reserve(512);
            let n = try_ready!(self.sock.read_buf(&mut self.rd));
            if n == 0 {
                return Ok(Async::Ready(()))
            }
        }
    }
}

impl Stream for ReadWrite {
    type Item = BytesMut;
    type Error = tokio::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let soc_closed = self.fill_read_buf()?.is_ready();

        let pos = self.rd.windows(2)
            .enumerate()
            .find(|&(_, bytes)| bytes == b"\r\n")
            .map(|(i, _)| i);

        if let Some(pos) = pos {
            let mut line = self.rd.split_to(pos + 2);
            // drop last new line
            line.split_off(pos);
            return Ok(Async::Ready(Some(line)));
        }

        if soc_closed {
            Ok(Async::Ready(None))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Sink for ReadWrite {
    type SinkItem = JsonMsg;
    type SinkError = ();

    fn start_send(&mut self, item: Self::SinkItem)
                  -> StartSend<Self::SinkItem, Self::SinkError>
    {
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self.poll_flush().unwrap() {
            Async::Ready(_) => Ok(Async::Ready(())),
            Async::NotReady => Ok(Async::NotReady),
        }
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
    fn test_write_buffer() {

        let addr = "127.0.0.1:3001".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        match std::net::TcpStream::connect("localhost:3001") {
            Ok(_s) => println!("connected"),
            Err(e) => eprintln!("{:#?}", e),
        }

        let server = listener.incoming().take(1)
            .for_each(move |sock| {
                let mut rw = ReadWrite::new(sock);
                rw.buffer_u8(b"123".as_ref());

                let mut b_m = BytesMut::new();
                b_m.reserve(3);
                b_m.put(b"123".as_ref());
                assert_eq!(b_m, rw.wr);
                Ok(())
            })
            .map_err(|e| {
                eprintln!("error in incoming: {}", e)
            });



        tokio::run(server);
    }

    #[test]
    fn test_read_buffer() {

        let addr = "127.0.0.1:3002".parse::<SocketAddr>().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        println!("listening");

        let mut conn = std::net::TcpStream::connect("localhost:3002").unwrap();
        conn.write_all("123\r\n123".as_bytes());

        let server = listener.incoming().take(1)
            .for_each(move |sock| {
                //                         this calls fill_read_buffer
                let f = ReadWrite::new(sock).into_future()
                    .map_err(|(e, _)| e)
                    .and_then(|(first, mut rw)|{
                        assert_eq!(b"123", first.unwrap().as_ref());

                        assert_eq!(b"123".as_ref(), rw.rd.as_ref());
                        Ok(())
                    }).map_err(|e| eprintln!("{}", e));
                tokio::spawn(f);
                Ok(())
            })
            .map_err(|e| {
                eprintln!("error in incoming: {}", e)
            });

        tokio::run(server);

    }

}



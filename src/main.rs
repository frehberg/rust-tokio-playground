//! Franks server handson
//!
//! A simple server that accepts connections, writes "hello world\n", and closes
//! the connection.
//!
//! Start this application and in another terminal run:
//!
//!     telnet localhost 6142
//!

#![allow(warnings)]
#![allow(unused_variables)]

extern crate tokio;
extern crate futures;

use tokio::io;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::prelude::*;
//use bytes::Buf;
use futures::sync::mpsc;
use futures::sync::mpsc::Sender;
use futures::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct RawMesg {
    portname: String,
    timestamp: Instant,
    origin: std::net::SocketAddr,
    mesg: Vec<u8>,
}

#[derive(Debug)]
enum PortEvent {
    RawMesg(RawMesg),
}

///
struct PortStream {
    portname: String,
    socket: TcpStream,
    tx : Sender<PortEvent>,
}

///
impl io::Write for PortStream {
    ///
    fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }

    ///
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.write(buf)
    }
}

///
impl io::AsyncWrite for PortStream {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        <&TcpStream>::shutdown(&mut  &(self.socket))
    }
}


pub fn main() {
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let addr = "127.0.0.1:6142".parse().unwrap();

    let (tx, rx) = mpsc::channel(1);

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).unwrap();

    // The server task asynchronously iterates over and processes each
    // incoming connection.
    let server =
        listener.incoming().for_each(move |socket| {
            let peer =  socket.peer_addr().unwrap();

            println!("accepted socket; addr={:?}", peer);

            let tx = tx.clone();

           //tx.send(PortEvent {});
            let portname = "Server1".to_string();

            let port = PortStream{portname, socket, tx,};

            let connection =
                io::write_all(port, "hello world\n")
                    .then( move | res | {
                        let (port, buf) = res.ok().unwrap();
                        port.tx.send( PortEvent::RawMesg(
                            RawMesg{portname: port.portname,
                                timestamp: Instant::now(),
                                origin: peer,
                                mesg: vec!{0,1,2} } ) ) } )
                    .then( |res| {
                        println!("wrote message; success={:?}",res.is_ok());

                        Ok(())
                    });

            // Spawn a new task that processes the socket:
            tokio::spawn(connection);

            Ok(())
        })
            .map_err(|err| {
                // All tasks must have an `Error` type of `()`. This forces error
                // handling and helps avoid silencing failures.
                //
                // In our example, we are only going to log the error to STDOUT.
                println!("accept error = {:?}", err);
            });


    let f2 = rx.for_each(|event| {
        println!("Message = {:?}", event);

        // The stream will stop on `Err`, so we need to return `Ok`.
        Ok(())
    });

    println!("server running on localhost:6142");
    runtime.spawn(server);
    runtime.spawn(f2);

    // Start the Tokio runtime
    runtime.shutdown_on_idle().wait().unwrap();
}
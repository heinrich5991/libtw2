#![cfg(not(test))]

#[macro_use]
extern crate log;
extern crate serverbrowse;
extern crate tools;

use serverbrowse::protocol as browse_protocol;
use serverbrowse::protocol::CountResponse;
use serverbrowse::protocol::Response;

use std::net::SocketAddr;
use std::net::UdpSocket;

const BUFSIZE: usize = 2048;

fn do_(socket: UdpSocket, addr: SocketAddr) {
    let mut buf = [0; BUFSIZE];

    socket.send_to(&browse_protocol::request_count(), addr).unwrap();

    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!("received response from non-peer, wanted={} got={}", addr, from);
            continue;
        }

        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Count(CountResponse(x))) => {
                println!("{}", x);
                break;
            },
            _ => {
                error!("received non-info response from peer");
            },
        }
    }
}

fn main() {
    tools::client::client(do_);
}

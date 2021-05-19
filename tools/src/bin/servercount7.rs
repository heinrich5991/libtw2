#![cfg(not(test))]

#[macro_use]
extern crate log;
extern crate serverbrowse;
extern crate tools;

use serverbrowse::protocol as browse_protocol;
use serverbrowse::protocol::Count7Response;
use serverbrowse::protocol::Response;
use serverbrowse::protocol::Token7Response;

use std::net::SocketAddr;
use std::net::UdpSocket;

const BUFSIZE: usize = 2048;

fn do_(socket: UdpSocket, addr: SocketAddr) {
    let mut buf = [0; BUFSIZE];

    socket.send_to(&browse_protocol::request_token_7(0), addr).unwrap();

    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!("received response from non-peer, wanted={} got={}", addr, from);
            continue;
        }
        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Token7(Token7Response(0, their_token))) => {
                info!("token={:08x}", their_token);
                socket.send_to(&browse_protocol::request_count_7(0, their_token), addr).unwrap();
                break;
            },
            _ => {
                error!("received non-token response from peer");
            },
        }
    }
    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!("received response from non-peer, wanted={} got={}", addr, from);
            continue;
        }
        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Count7(Count7Response(_, _, x))) => {
                println!("{}", x);
                break;
            },
            _ => {
                error!("received non-count response from peer");
            },
        }
    }
}

fn main() {
    tools::client::client(do_);
}

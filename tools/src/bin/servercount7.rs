#![cfg(not(test))]

#[macro_use]
extern crate log;

use libtw2_serverbrowse::protocol as browse_protocol;
use libtw2_serverbrowse::protocol::Count7Response;
use libtw2_serverbrowse::protocol::Response;
use libtw2_serverbrowse::protocol::Token7;
use libtw2_serverbrowse::protocol::Token7Response;
use std::net::SocketAddr;
use std::net::UdpSocket;

const BUFSIZE: usize = 2048;

fn do_(socket: UdpSocket, addr: SocketAddr) {
    let mut buf = [0; BUFSIZE];

    socket
        .send_to(&browse_protocol::request_token_7(Token7([0; 4])), addr)
        .unwrap();

    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!(
                "received response from non-peer, wanted={} got={}",
                addr, from
            );
            continue;
        }
        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Token7(Token7Response(Token7([0, 0, 0, 0]), their_token))) => {
                info!("token={}", their_token);
                socket
                    .send_to(
                        &browse_protocol::request_count_7(Token7([0; 4]), their_token),
                        addr,
                    )
                    .unwrap();
                break;
            }
            _ => {
                error!("received non-token response from peer");
            }
        }
    }
    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!(
                "received response from non-peer, wanted={} got={}",
                addr, from
            );
            continue;
        }
        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Count7(Count7Response(_, _, x))) => {
                println!("{}", x);
                break;
            }
            _ => {
                error!("received non-count response from peer");
            }
        }
    }
}

fn main() {
    libtw2_tools::client::client(do_);
}

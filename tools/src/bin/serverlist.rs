#![cfg(not(test))]

#[macro_use]
extern crate log;

use libtw2_serverbrowse::protocol as browse_protocol;
use libtw2_serverbrowse::protocol::List6Response;
use libtw2_serverbrowse::protocol::Response;
use std::net::SocketAddr;
use std::net::UdpSocket;

const BUFSIZE: usize = 2048;

fn do_(socket: UdpSocket, addr: SocketAddr) {
    let mut buf = [0; BUFSIZE];

    socket
        .send_to(&browse_protocol::request_list_6(), addr)
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
            Some(Response::List6(List6Response(list))) => {
                for &s in list.iter() {
                    println!("{}", s.unpack());
                }
            }
            _ => {
                error!("received non-list response from peer");
            }
        }
    }
}

fn main() {
    libtw2_tools::client::client(do_);
}

#![cfg(not(test))]

#![feature(io)]

#[macro_use]
extern crate log;

extern crate serverbrowse;

use serverbrowse::protocol as browse_protocol;
use serverbrowse::protocol::Response;

use std::old_io::net::ip::Ipv4Addr;
use std::old_io::net::ip::SocketAddr;
use std::old_io::net::udp::UdpSocket;

const BUFSIZE: usize = 2048;

fn main() {
    let bindaddr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 0 };
    //let addr = SocketAddr { ip: Ipv4Addr(198, 251, 81, 153), port: 8300 };
    let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 8303 };
    let mut socket = UdpSocket::bind(bindaddr).unwrap();

    let mut buf = [0; BUFSIZE];

    //browse_protocol::request_list_6(|x| socket.send_to(x, addr).unwrap());
    browse_protocol::request_info_6(|x| socket.send_to(x, addr).unwrap());

    socket.set_timeout(Some(1000));

    loop {
        let (len, from) = socket.recv_from(&mut buf).unwrap();
        if from != addr {
            error!("received response from non-peer, wanted={} got={}", addr, from);
            continue;
        }

        match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Info6(x)) => {
	    	println!("{:?}", x.parse().unwrap());
                break;
            },
            _ => {
                error!("received non-info response from peer");
            },
        }

        //let browse_protocol::ListResponse(server_addrs)
        //    = browse_protocol::parse_response(&buf[..len])
        //      .map(|x| x.to_list()).unwrap_or(None).unwrap();
        //let server_addrs: &[browse_protocol::AddrPacked] = server_addrs;

        //for &s in server_addrs.iter() {
        //    println!("{}", s.unpack());
        //}
    }
}

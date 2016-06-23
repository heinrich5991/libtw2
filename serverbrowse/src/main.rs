#![cfg(not(test))]

#[macro_use]
extern crate log;
extern crate logger;

extern crate serverbrowse;

use serverbrowse::protocol as browse_protocol;
use serverbrowse::protocol::Response;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;

const BUFSIZE: usize = 2048;

fn to_socket_addr_or_panic(addr: &str) -> SocketAddr {
    addr.to_socket_addrs().unwrap().next().unwrap()
    //   |                 |        |      |
    //   io::Result        Iterator Option SocketAddr
}

fn main() {
    logger::init();

    let bindaddr = "0.0.0.0:0";
    //let addr = "198.251.81.153:8300";
    let addr = "127.0.0.1:8303";
    let addr = to_socket_addr_or_panic(addr);
    let socket = UdpSocket::bind(bindaddr).unwrap();

    let mut buf = [0; BUFSIZE];

    //browse_protocol::request_list_6(|x| socket.send_to(x, addr).unwrap());
    browse_protocol::request_info_6(|x| socket.send_to(x, &addr).unwrap());

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

use std::env;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;

fn to_socket_addr_or_panic(addr: &str) -> SocketAddr {
    addr.to_socket_addrs().unwrap().next().unwrap()
    //   |                 |        |      |
    //   io::Result        Iterator Option SocketAddr
}

pub fn client<D: FnOnce(UdpSocket, SocketAddr)>(do_: D) {
    logger::init();

    let mut args = env::args();
    let program_name = args.next().unwrap();
    let param = args.next();
    if let (Some(param), None) = (param, args.next()) {
        let bindaddr = "0.0.0.0:0";
        let addr = to_socket_addr_or_panic(&param);
        let socket = UdpSocket::bind(bindaddr).unwrap();
        do_(socket, addr);
    } else {
        println!("USAGE: {} <MAP>...", program_name);
    }
}

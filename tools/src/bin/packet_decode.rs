extern crate arrayvec;
extern crate buffer;
extern crate gamenet_teeworlds_0_6 as gamenet;
extern crate hexdump;
extern crate net;
extern crate packer;
extern crate tools;
extern crate warn;

use arrayvec::ArrayVec;
use buffer::ReadBuffer;
use gamenet::msg;
use gamenet::msg::Connless;
use hexdump::hexdump;
use net::protocol::ChunksIter;
use net::protocol::ConnectedPacketType;
use net::protocol::Packet;
use packer::Unpacker;
use std::io;
use tools::unhexdump::Unhexdump;
use tools::warn_stdout::Stdout;

fn main() {
    let mut un = Unhexdump::new();
    let mut buf: ArrayVec<[u8; 4096]> = ArrayVec::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    while {
        buf.clear();
        stdin.read_buffer(&mut buf).unwrap().len() != 0
    } {
        un.feed(&buf).unwrap();
    }

    let bytes = un.into_inner().unwrap();

    println!("packet");
    hexdump(&bytes);
    let p = match Packet::read(&mut Stdout, &bytes, None, &mut buf) {
        Err(e) => {
            println!("ERROR: {:?}", e);
            return;
        }
        Ok(p) => p,
    };

    let cp = match p {
        Packet::Connless(data) => {
            println!("connless");
            let msg = match Connless::decode(&mut Stdout, &mut Unpacker::new(data)) {
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    return;
                }
                Ok(m) => m,
            };
            println!("{:?}", msg);
            return;
        }
        Packet::Connected(cp) => cp,
    };

    if let Some(token) = cp.token {
        println!("token={}", token);
    }

    let (request_resend, num_chunks, payload) = match cp.type_ {
        ConnectedPacketType::Control(control) => {
            println!("control ack={}", cp.ack);
            println!("{:?}", control);
            return;
        }
        ConnectedPacketType::Chunks(r, n, p) => (r, n, p),
    };
    println!(
        "chunks ack={} request_resend={} num_chunks={}",
        cp.ack, request_resend, num_chunks
    );
    hexdump(payload);
    let mut i = 0;
    let mut chunks_iter = ChunksIter::new(payload, num_chunks);
    loop {
        if chunks_iter.clone().next().is_some() {
            println!("chunk {}", i);
        }
        let chunk = if let Some(chunk) = chunks_iter.next_warn(&mut Stdout) {
            i += 1;
            chunk
        } else {
            break;
        };

        match chunk.vital {
            Some((sequence, resend)) => {
                println!("vital=true sequence={} resend={}", sequence, resend)
            }
            None => println!("vital=false"),
        }
        hexdump(chunk.data);

        let msg = match msg::decode(&mut Stdout, &mut Unpacker::new(chunk.data)) {
            Err(e) => {
                println!("ERROR: {:?}", e);
                continue;
            }
            Ok(m) => m,
        };

        println!("{:?}", msg);
    }
}

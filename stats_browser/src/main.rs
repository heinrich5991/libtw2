#![cfg(not(test))]

#![allow(unstable)]
#![feature(int_uint)]

#[macro_use] extern crate log;
extern crate "time" as rust_time;
extern crate "rustc-serialize" as rustc_serialize;

extern crate serverbrowse;

use serverbrowse::protocol::CountResponse;
use serverbrowse::protocol::Info5Response;
use serverbrowse::protocol::Info6Response;
use serverbrowse::protocol::List5Response;
use serverbrowse::protocol::List6Response;
use serverbrowse::protocol::MASTERSERVER_PORT;
use serverbrowse::protocol::NzU8SliceExt;
use serverbrowse::protocol::PlayerInfo;
use serverbrowse::protocol::Response;
use serverbrowse::protocol::ServerInfo;
use serverbrowse::protocol;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecMap;
use std::collections::hash_map;
use std::default::Default;
use std::io::net::addrinfo;
use std::io::timer;
use std::mem;
use std::num::SignedInt;

use addr::Addr;
use addr::ProtocolVersion;
use addr::ServerAddr;
use base64::b64;
use entry::MasterServerEntry;
use entry::ServerEntry;
use entry::ServerResponse;
use socket::NonBlockExt;
use socket::UdpSocket;
use socket::WouldBlock;
use time::Limit;
use work_queue::TimedWorkQueue;

pub mod addr;
pub mod base64;
pub mod config;
pub mod entry;
pub mod socket;
pub mod time;
pub mod work_queue;

trait HashMapEntryToInner<'a> {
    type Key;
    type Value;
    fn into_occupied(self) -> Option<hash_map::OccupiedEntry<'a,<Self as HashMapEntryToInner<'a>>::Key,<Self as HashMapEntryToInner<'a>>::Value>>;
    fn into_vacant(self) -> Option<hash_map::VacantEntry<'a,<Self as HashMapEntryToInner<'a>>::Key,<Self as HashMapEntryToInner<'a>>::Value>>;
}

impl<'a,K,V> HashMapEntryToInner<'a> for hash_map::Entry<'a,K,V> {
    type Key = K;
    type Value = V;
    fn into_occupied(self) -> Option<hash_map::OccupiedEntry<'a,K,V>> {
        match self {
            hash_map::Entry::Occupied(o) => Some(o),
            hash_map::Entry::Vacant(_) => None,
        }
    }
    fn into_vacant(self) -> Option<hash_map::VacantEntry<'a,K,V>> {
        match self {
            hash_map::Entry::Occupied(_) => None,
            hash_map::Entry::Vacant(v) => Some(v),
        }
    }
}

trait StatsBrowserCb {
    fn on_server_new(&mut self, addr: ServerAddr, info: &ServerInfo);
    fn on_server_change(&mut self, addr: ServerAddr, old: &ServerInfo, new: &ServerInfo);
    fn on_server_remove(&mut self, addr: ServerAddr, last: &ServerInfo);
}

#[derive(Copy, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, RustcEncodable, Show)]
struct MasterId(uint);

impl MasterId {
    fn get_and_inc(&mut self) -> MasterId {
        let MasterId(value) = *self;
        *self = MasterId(value + 1);
        MasterId(value)
    }
}

enum Work {
    Resolve(MasterId),
    RequestList(MasterId),
    ExpectList(MasterId),
    RequestInfo(ServerAddr),
    ExpectInfo(ServerAddr),
}

struct StatsBrowser<'a> {
    master_servers: VecMap<MasterServerEntry>,
    servers: HashMap<ServerAddr,ServerEntry>,

    next_master_id: MasterId,

    list_limit: Limit,
    info_limit: Limit,

    work_queue: TimedWorkQueue<Work>,

    socket: UdpSocket,

    cb: &'a mut (StatsBrowserCb+'a),
}

impl<'a> StatsBrowser<'a> {
    fn new(cb: &mut StatsBrowserCb) -> Option<StatsBrowser> {
        StatsBrowser::new_without_masters(cb).map(|mut browser| {
            for i in range(0u32, 4).map(|x| x + 1) {
                browser.add_master(format!("master{}.teeworlds.com", i));
            }
            browser
        })
    }
    fn new_without_masters(cb: &mut StatsBrowserCb) -> Option<StatsBrowser> {
        let socket = match UdpSocket::open() {
            Ok(s) => s,
            Err(e) => {
                error!("Couldn't open socket, {:?}", e);
                return None;
            }
        };
        let mut work_queue = TimedWorkQueue::new();
        work_queue.add_duration(config::RESOLVE_REPEAT_MS.to_duration());
        work_queue.add_duration(config::LIST_REPEAT_MS.to_duration());
        work_queue.add_duration(config::LIST_EXPECT_MS.to_duration());
        work_queue.add_duration(config::INFO_REPEAT_MS.to_duration());
        work_queue.add_duration(config::INFO_EXPECT_MS.to_duration());
        Some(StatsBrowser {
            master_servers: Default::default(),
            servers: Default::default(),

            next_master_id: Default::default(),

            list_limit: Limit::new(config::MAX_LISTS, config::MAX_LISTS_MS.to_duration()),
            info_limit: Limit::new(config::MAX_INFOS, config::MAX_INFOS_MS.to_duration()),

            work_queue: work_queue,

            socket: socket,

            cb: cb,
        })
    }
    fn add_master(&mut self, domain: String) {
        let MasterId(id) = self.next_master_id.get_and_inc();
        assert!(self.master_servers.insert(id, MasterServerEntry::new(domain)).is_none());
        self.work_queue.push_now(Work::Resolve(MasterId(id)));
    }
    fn do_resolve(&mut self, master_id: MasterId) -> Result<(),()> {
        let MasterId(idx) = master_id;
        let master = self.master_servers.get_mut(&idx).unwrap();
        match addrinfo::get_host_addresses(master.domain.as_slice()).map(|x| x.get(0).cloned()) {
            Ok(Some(ip_address)) => {
                let addr = Addr::new(ip_address, MASTERSERVER_PORT);
                info!("Resolved {} to {}", master.domain, addr);
                match mem::replace(&mut master.addr, Some(addr)) {
                    Some(_) => {},
                    None => { self.work_queue.push_now(Work::RequestList(master_id)); },
                }
            },
            Ok(None) => { info!("Resolved {}, no address found", master.domain); },
            Err(x) => { warn!("Error while resolving {}, {}", master.domain, x); },
        }
        self.work_queue.push(config::RESOLVE_REPEAT_MS.to_duration(), Work::Resolve(master_id));
        Ok(())
    }
    fn do_expect_list(&mut self, master_id: MasterId) -> Result<(),()> {
        if self.check_complete_list(master_id).is_ok() {
            self.work_queue.push(config::LIST_REPEAT_MS.to_duration(), Work::RequestList(master_id));
        } else {
            let MasterId(idx) = master_id;
            let master = self.master_servers.get_mut(&idx).unwrap();
            info!("Re-requesting list for {}", master.domain);
            self.work_queue.push_now(Work::RequestList(master_id));
        }
        Ok(())
    }
    fn do_request_list(&mut self, master_id: MasterId) -> Result<(),()> {
        let MasterId(idx) = master_id;
        let master = self.master_servers.get_mut(&idx).unwrap();

        if !self.list_limit.acquire().is_ok() {
            return Err(());
        }

        let socket = &mut self.socket;
        let mut send = |&mut: data: &[u8]| socket.send_to(data, master.addr.unwrap()).unwrap();

        debug!("Requesting count and list from {}", master.domain);
        if protocol::request_count(|y| send(y)).would_block()
            || protocol::request_list_5(|y| send(y)).would_block()
            || protocol::request_list_6(|y| send(y)).would_block()
        {
            debug!("Failed to send count or list request, would block");
            return Err(());
        }

        self.work_queue.push(config::LIST_EXPECT_MS.to_duration(), Work::ExpectList(master_id));
        Ok(())
    }
    fn do_expect_info(&mut self, server_addr: ServerAddr) -> Result<(),()> {
        let server = self.servers.entry(server_addr).into_occupied().unwrap();

        if server.get().num_missing_resp == 0 {
            self.work_queue.push(config::INFO_REPEAT_MS.to_duration(), Work::RequestInfo(server_addr));
        } else {
            if server.get().num_missing_resp >= 10 {
                // Throw the server out after ten missing replies.
                match server.remove().resp {
                    Some(ref y) => self.cb.on_server_remove(server_addr, &y.info),
                    None => {},
                }
            } else {
                info!("Re-requesting info from {}", server_addr);
                self.work_queue.push_now(Work::RequestInfo(server_addr));
            }
        }
        Ok(())
    }
    fn do_request_info(&mut self, server_addr: ServerAddr) -> Result<(),()> {
        let server = self.servers.get_mut(&server_addr).unwrap();

        if !self.info_limit.acquire().is_ok() {
            return Err(());
        }

        debug!("Requesting info from {}", server_addr);
        let socket = &mut self.socket;

        let mut send = |&mut: data: &[u8]| socket.send_to(data, server_addr.addr).unwrap();

        let would_block = match server_addr.version {
            ProtocolVersion::V5 => protocol::request_info_5(|x| send(x)).would_block(),
            ProtocolVersion::V6 => protocol::request_info_6(|x| send(x)).would_block(),
        };

        if would_block {
            debug!("Failed to send info request, would block");
            return Err(());
        }

        server.num_missing_resp += 1;

        self.work_queue.push(config::INFO_EXPECT_MS.to_duration(), Work::ExpectInfo(server_addr));
        Ok(())
    }
    fn get_master_id(&self, addr: Addr) -> Option<MasterId> {
        for (i, master) in self.master_servers.iter() {
            if master.addr == Some(addr) {
                return Some(MasterId(i));
            }
        }
        None
    }
    fn check_complete_list(&mut self, id: MasterId) -> Result<(),()> {
        let MasterId(idx) = id;
        let master = self.master_servers.get_mut(&idx).unwrap();

        let updated_count = master.updated_count.take();
        let updated_list = mem::replace(&mut master.updated_list, HashSet::new());

        if let Some(updated_count) = updated_count {
            if (updated_count as int - updated_list.len() as int).abs() <= 5 {
                let _old_list = mem::replace(&mut master.list, updated_list);
                // TODO: diff
                return Ok(());
            }
        }
        Err(())
    }
    fn process_count(&mut self, from: MasterId, count: u16) {
        let MasterId(idx) = from;
        let master = self.master_servers.get_mut(&idx).unwrap();

        debug!("Received count from {}, {}", master.domain, count);

        match mem::replace(&mut master.updated_count, Some(count)) {
            Some(x) => {
                warn!("Received double count message, old={}", x);
            },
            None => {},
        }
    }
    fn process_list<I>(&mut self, from: MasterId, mut servers_iter: I)
        where I: Iterator<Item=ServerAddr>+ExactSizeIterator,
              
    {
        let MasterId(idx) = from;
        let master = self.master_servers.get_mut(&idx).unwrap();

        debug!("Received list from {}, length {}", master.domain, servers_iter.len());

        for s in servers_iter {
            if !master.updated_list.insert(s) {
                warn!("Double-received {}", s);
            }
            if let Some(v) = self.servers.entry(s).into_vacant() {
                v.insert(ServerEntry::new());
                self.work_queue.push_now(Work::RequestInfo(s));
            }
        }
    }
    fn process_info(&mut self, from: ServerAddr, info: Option<ServerInfo>, raw: &[u8]) {
        let server = match self.servers.get_mut(&from) {
            Some(x) => x,
            None => {
                warn!("Received info from unknown server {}, {:?}", from, raw);
                return;
            }
        };
        match info {
            None => {
                if server.num_malformed_resp < config::MAX_MALFORMED_RESP {
                    warn!("Received unparsable info from {}, {:?}", from, raw);
                }
                server.num_malformed_resp += 1;
            },
            Some(x) => {
                if server.num_missing_resp == 0 {
                    if server.num_extra_resp < config::MAX_EXTRA_RESP {
                        warn!("Received info while not expecting it, from {}, {:?}", from, x);
                    }
                    server.num_extra_resp += 1;
                    return;
                }
                server.num_missing_resp = 0;
                debug!("Received server info from {}, {:?}", from, x);
                match server.resp {
                    Some(ref y) => self.cb.on_server_change(from, &y.info, &x),
                    None => self.cb.on_server_new(from, &x)
                }
                server.resp = Some(ServerResponse::new(x));
            },
        }
    }
    fn process_packet(&mut self, from: Addr, data: &[u8]) {
        match protocol::parse_response(data) {
            Some(Response::Count(CountResponse(count))) => {
                match self.get_master_id(from) {
                    Some(id) => {
                        self.process_count(id, count);
                    },
                    None => {
                        warn!("Received count message from non-master {}, count={}", from, count);
                    },
                }
            },
            Some(Response::List5(List5Response(servers))) => {
                match self.get_master_id(from) {
                    Some(id) => {
                        self.process_list(id, servers.iter().map(|x|
                            ServerAddr::new(ProtocolVersion::V5, Addr::from_srvbrowse_addr(x.unpack()))
                        ));
                    },
                    None => {
                        let servers: Vec<_> = servers.iter().map(|x| x.unpack()).collect();
                        warn!("Received list message from non-master {}, servers={:?}", from, servers);
                    },
                }
            },
            Some(Response::List6(List6Response(servers))) => {
                match self.get_master_id(from) {
                    Some(id) => {
                        self.process_list(id, servers.iter().map(|x|
                            ServerAddr::new(ProtocolVersion::V6, Addr::from_srvbrowse_addr(x.unpack()))
                        ));
                    },
                    None => {
                        let servers: Vec<_> = servers.iter().map(|x| x.unpack()).collect();
                        warn!("Received list message from non-master {}, servers={:?}", from, servers);
                    },
                }
            },
            Some(Response::Info5(info)) => {
                let Info5Response(raw_data) = info;
                self.process_info(ServerAddr::new(ProtocolVersion::V5, from), info.parse(), raw_data);
            },
            Some(Response::Info6(info)) => {
                let Info6Response(raw_data) = info;
                self.process_info(ServerAddr::new(ProtocolVersion::V6, from), info.parse(), raw_data);
            },
            _ => {
                warn!("Received unknown message from {}, {:?}", from, data);
            },
        }
    }
    fn pump_network(&mut self) {
        let mut buffer: [u8; 2048] = unsafe { mem::uninitialized() };

        loop {
            match self.socket.recv_from(&mut buffer) {
                Err(x) => { panic!("socket error, {:?}", x); },
                Ok(Err(WouldBlock)) => return,
                Ok(Ok((read_len, from))) => {
                    self.process_packet(from, buffer.as_slice().slice_to(read_len));
                },
            }
        }
    }
    fn run(&mut self) {
        loop {
            self.pump_network();
            while let Some(work) = self.work_queue.pop() {
                let result = match work {
                    Work::Resolve(id)       => self.do_resolve(id),
                    Work::RequestList(id)   => self.do_request_list(id),
                    Work::ExpectList(id)    => self.do_expect_list(id),
                    Work::RequestInfo(addr) => self.do_request_info(addr),
                    Work::ExpectInfo(addr)  => self.do_expect_info(addr),
                };
                if !result.is_ok() {
                    self.work_queue.push_now_front(work);
                    break;
                }
            }
            timer::sleep(config::SLEEP_MS.to_duration());
        }
    }
}

struct Tracker {
    player_count: u32,
}

fn print_start() {
    println!("START\t1.0\tlibtw2\t0.1");
}

fn print_player_new(addr: ServerAddr, info: &PlayerInfo) {
    println!("PLADD\t{}\t{}\t{}\t{}\t{}", addr.addr, b64(&info.name), b64(&info.clan), info.is_player, info.country);
}

fn print_player_remove(addr: ServerAddr, info: &PlayerInfo) {
    if info.name.as_slice().as_bytes() == "(connecting)".as_bytes() { return; }
    println!("PLDEL\t{}\t{}", addr.addr, b64(&info.name));
}

fn print_player_change(addr: ServerAddr, old: &PlayerInfo, new: &PlayerInfo) {
    print_player_remove(addr, new);
    print_player_new(addr, old);
}

fn print_server_remove(addr: ServerAddr, info: &ServerInfo) {
    let _ = info;
    println!("SVDEL\t{}", addr.addr);
}

fn print_server_change_impl(addr: ServerAddr, new: bool, info: &ServerInfo) {
    println!("{}\t{}\t{}\t{}\t{}\t{}\t{}",
        if new { "SVADD" } else { "SVCHG" },
        addr.addr,
        info.flags,
        b64(&info.version),
        b64(&info.game_type),
        b64(&info.map),
        b64(&info.name),
    );
}

fn print_server_new(addr: ServerAddr, info: &ServerInfo) {
    print_server_change_impl(addr, true, info);
}

fn print_server_change(addr: ServerAddr, old: &ServerInfo, new: &ServerInfo) {
    let _ = old;
    print_server_change_impl(addr, false, new);
}

fn player_ignore(addr: ServerAddr, info: &PlayerInfo) -> bool {
    let _ = addr;
    info.name.as_slice().as_bytes() == "(connecting)".as_bytes()
}

impl Tracker {
    fn new() -> Tracker {
        Tracker {
            player_count: 0,
        }
    }
    fn start(&mut self) {
        print_start();
    }
    fn server_ignore(addr: ServerAddr) -> bool {
        addr.version != ProtocolVersion::V6
    }
    fn on_player_new(&mut self, addr: ServerAddr, info: &PlayerInfo) {
        if player_ignore(addr, info) { return; }
        print_player_new(addr, info);
        self.player_count += 1;
    }

    fn on_player_change(&mut self, addr: ServerAddr, old: &PlayerInfo, new: &PlayerInfo) {
        if player_ignore(addr, old) || player_ignore(addr, new) { return; }
        if old.clan != new.clan
            || old.is_player != new.is_player
            || old.country != new.country
        {
            print_player_change(addr, old, new);
        }
    }

    fn on_player_remove(&mut self, addr: ServerAddr, last: &PlayerInfo) {
        if player_ignore(addr, last) { return; }
        print_player_remove(addr, last);
        self.player_count -= 1;
    }

    fn diff_players(&mut self, addr: ServerAddr, slice_old: &[PlayerInfo], slice_new: &[PlayerInfo]) {
        let mut iter_old = slice_old.iter();
        let mut iter_new = slice_new.iter();
        let mut maybe_old: Option<&PlayerInfo> = iter_old.next();
        let mut maybe_new: Option<&PlayerInfo> = iter_new.next();
        loop {
            match (maybe_old, maybe_new) {
                (None, None) => break,
                (None, Some(new)) => {
                    self.on_player_new(addr, new);
                    maybe_new = iter_new.next();
                }
                (Some(old), None) => {
                    self.on_player_remove(addr, old);
                    maybe_old = iter_old.next();
                }
                (Some(old), Some(new)) => {
                    match Ord::cmp(&*old.name, &*new.name) {
                        Ordering::Less => {
                            self.on_player_remove(addr, old);
                            maybe_old = iter_old.next();
                        }
                        Ordering::Equal => {
                            self.on_player_change(addr, old, new);
                            maybe_old = iter_old.next();
                            maybe_new = iter_new.next();
                        }
                        Ordering::Greater => {
                            self.on_player_new(addr, new);
                            maybe_new = iter_new.next();
                        }
                    }
                }
            }
        }
    }
}

impl StatsBrowserCb for Tracker {
    fn on_server_new(&mut self, addr: ServerAddr, info: &ServerInfo) {
        if Tracker::server_ignore(addr) { return; }
        print_server_new(addr, info);
        self.diff_players(addr, &[], info.clients());
    }

    fn on_server_change(&mut self, addr: ServerAddr, old: &ServerInfo, new: &ServerInfo) {
        if Tracker::server_ignore(addr) { return; }
        if old.flags != new.flags
            || old.version != new.version
            || old.game_type != new.game_type
            || old.map != new.map
            || old.name != new.name
        {
            print_server_change(addr, old, new);
        }
        self.diff_players(addr, old.clients(), new.clients());
    }

    fn on_server_remove(&mut self, addr: ServerAddr, last: &ServerInfo) {
        if Tracker::server_ignore(addr) { return; }
        self.diff_players(addr, last.clients(), &[]);
        print_server_remove(addr, last);
    }
}

fn main() {
    let mut tracker = Tracker::new();
    tracker.start();
    let mut browser = match StatsBrowser::new(&mut tracker) {
        Some(b) => b,
        None => {
            panic!("Failed to bind socket.");
        },
    };
    browser.run();
}

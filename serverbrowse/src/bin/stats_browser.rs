#![cfg(not(test))]

#![feature(phase)]

#[phase(plugin, link)]
extern crate log;
extern crate time;
extern crate "rustc-serialize" as serialize;

extern crate mio;

extern crate serverbrowse;

use serverbrowse::protocol::Addr;
use serverbrowse::protocol::CountResponse;
use serverbrowse::protocol::Info5Response;
use serverbrowse::protocol::Info6Response;
use serverbrowse::protocol::List5Response;
use serverbrowse::protocol::List6Response;
use serverbrowse::protocol::NzU8Slice;
use serverbrowse::protocol::PString64;
use serverbrowse::protocol::PlayerInfo;
use serverbrowse::protocol::Response;
use serverbrowse::protocol::ServerInfo;
use serverbrowse::protocol;

use mio::NonBlock;
use mio::buf::Buf;
use mio::buf::MutSliceBuf;
use mio::buf::SliceBuf;
use mio::net::SockAddr;
use mio::net::UnconnectedSocket;
use mio::net::udp::UdpSocket;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::RingBuf;
use std::collections::VecMap;
use std::collections::hash_map::Entry;
use std::default::Default;
use std::fmt;
use std::io::net::addrinfo;
use std::io::timer;
use std::mem;
use std::num::SignedInt;
use std::time::duration::Duration;

use serialize::base64;
use serialize::base64::ToBase64;

#[deriving(Clone)]
pub struct TimedWorkQueue<T> {
    now_queue: RingBuf<T>,
    other_queues: HashMap<u64,RingBuf<Timed<T>>>,
}

impl<T> Default for TimedWorkQueue<T> {
    fn default() -> TimedWorkQueue<T> {
        TimedWorkQueue::new()
    }
}

impl<T> TimedWorkQueue<T> {
    pub fn new() -> TimedWorkQueue<T> {
        TimedWorkQueue {
            now_queue: RingBuf::new(),
            other_queues: HashMap::new(),
        }
    }
    pub fn add_duration(&mut self, dur: Duration) {
        let dur_k = TimedWorkQueue::<T>::duration_to_key(dur);
        if let Entry::Vacant(v) = self.other_queues.entry(dur_k) {
            v.set(RingBuf::new());
        }
    }
    fn duration_to_key(dur: Duration) -> u64 {
        dur.num_milliseconds().to_u64().expect("Expected positive duration")
    }
    pub fn push(&mut self, dur: Duration, data: T) {
        let dur_k = TimedWorkQueue::<T>::duration_to_key(dur);
        let queue = self.other_queues.get_mut(&dur_k);
        let queue = queue.expect("Need to `add_duration` before pushing with it.");
        queue.push_back(Timed::new(data, Time::now() + dur));
    }
    pub fn push_now(&mut self, data: T) {
        self.now_queue.push_back(data);
    }
    pub fn push_now_front(&mut self, data: T) {
        self.now_queue.push_front(data);
    }
    pub fn pop(&mut self) -> Option<T> {
        if let Some(data) = self.now_queue.pop_front() {
            return Some(data);
        }
        let now = Time::now();
        for (_, q) in self.other_queues.iter_mut() {
            // Only pop the first element if there actually is an element in
            // the front and it's time to process it.
            if q.front().map(|timed| timed.time <= now).unwrap_or(false) {
                return Some(q.pop_front().unwrap().data);
            }
        }
        None
    }
}

// TODO: What happens on time overflow?
// TODO: What happens on time backward jump?

// Config
const MAX_LISTS:          u32 =  1;
const MAX_INFOS:          u32 = 10;
const MAX_MALFORMED_RESP: u32 = 10;
const MAX_EXTRA_RESP:     u32 = 10;
const MAX_LISTS_MS:      Ms = Ms(  1_000);
const MAX_INFOS_MS:      Ms = Ms(     25);
const INFO_EXPECT_MS:    Ms = Ms(  1_000);
const INFO_REPEAT_MS:    Ms = Ms(  5_000);
const LIST_EXPECT_MS:    Ms = Ms(  5_000);
const LIST_REPEAT_MS:    Ms = Ms( 30_000);
const RESOLVE_REPEAT_MS: Ms = Ms(120_000);
const SLEEP_MS:          Ms = Ms(      5);

struct Ms(u32);

impl Ms {
    fn to_duration(self) -> Duration {
        let Ms(ms) = self;
        Duration::milliseconds(ms.to_i64().unwrap())
    }
}

const MASTERSRV_PORT: u16 = 8300;

#[deriving(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Show)]
pub struct Time(u64); // In milliseconds.

impl Add<Duration,Time> for Time {
    fn add(self, rhs: Duration) -> Time {
        let Time(ms) = self;
        Time(ms + rhs.num_milliseconds() as u64)
    }
}

impl Time {
    fn now() -> Time {
        Time(time::precise_time_ns() / 1_000_000)
    }
}

fn addr_to_sockaddr(addr: Addr) -> SockAddr {
    SockAddr::InetAddr(addr.ip_address, addr.port)
}

fn sockaddr_to_addr(addr: SockAddr) -> Addr {
    match addr {
        SockAddr::InetAddr(ip_address, port) => Addr { ip_address: ip_address, port: port },
        x => { panic!("Invalid sockaddr: {}", x); }
    }
}

#[deriving(Copy, Clone)]
struct B64<'a>(&'a [u8]);

fn b64(string: &PString64) -> B64 {
    B64(string.as_slice().as_bytes())
}

impl<'a> fmt::Show for B64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let B64(bytes) = *self;
        const CONFIG: base64::Config = base64::Config {
            char_set: base64::CharacterSet::Standard,
            newline: base64::Newline::LF,
            pad: true,
            line_length: None,
        };
        //write!(f, "{}", String::from_utf8_lossy(bytes))
        write!(f, "{}", bytes.to_base64(CONFIG))
    }
}

#[deriving(Clone)]
struct MasterServerEntry {
    domain: String,
    addr: Option<Addr>,

    count: u16,
    list: HashSet<ServerAddr>,
    updated_count: Option<u16>,
    updated_list: HashSet<ServerAddr>,
    completely_updated: bool,
}

impl MasterServerEntry {
    fn new(domain: String) -> MasterServerEntry {
        MasterServerEntry {
            domain: domain,
            addr: None,

            count: 0,
            list: HashSet::new(),
            updated_count: None,
            updated_list: HashSet::new(),
            completely_updated: false,
        }
    }
}

#[deriving(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, Show)]
enum ProtocolVersion {
    V5,
    V6,
}

#[deriving(Clone, Copy, Eq, Hash, PartialEq)]
struct ServerAddr {
    version: ProtocolVersion,
    addr: Addr,
}

impl fmt::Show for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.version, self.addr)
    }
}

impl ServerAddr {
    fn new(version: ProtocolVersion, addr: Addr) -> ServerAddr {
        ServerAddr {
            version: version,
            addr: addr,
        }
    }
}

#[deriving(Copy, Clone)]
struct ServerEntry {
    num_missing_resp: u32,
    num_malformed_resp: u32,
    num_extra_resp: u32,
    resp: Option<ServerResponse>,
}

impl ServerEntry {
    fn new() -> ServerEntry {
        ServerEntry {
            num_missing_resp: 0,
            num_malformed_resp: 0,
            num_extra_resp: 0,
            resp: None,
        }
    }
}

#[deriving(Copy, Clone)]
struct ServerResponse {
    info: ServerInfo,
}

impl ServerResponse {
    fn new(info: ServerInfo) -> ServerResponse {
        ServerResponse {
            info: info,
        }
    }
}

trait StatsBrowserCb {
    fn on_server_new(&mut self, addr: ServerAddr, info: &ServerInfo);
    fn on_server_change(&mut self, addr: ServerAddr, old: &ServerInfo, new: &ServerInfo);
    fn on_server_remove(&mut self, addr: ServerAddr, last: &ServerInfo);
}

#[deriving(Copy, Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Show)]
struct MasterId(uint);

impl MasterId {
    fn get_and_inc(&mut self) -> MasterId {
        let MasterId(value) = *self;
        *self = MasterId(value + 1);
        MasterId(value)
    }
}

#[deriving(Copy, Clone, Eq, Hash, PartialEq, Show)]
struct Timed<T> {
    data: T,
    time: Time,
}

#[deriving(Copy, Clone)]
struct Limit {
    remaining: u32,
    reset: Time,
    max: u32,
    duration: Duration,
}

impl Limit {
    fn new(max: u32, duration: Duration) -> Limit {
        Limit {
            remaining: max,
            reset: Time::now(),
            max: max,
            duration: duration,
        }
    }
    fn acquire_at(&mut self, time: Time) -> Result<(),()> {
        if time >= self.reset {
            self.remaining = self.max;
            self.reset = time + self.duration;
        }
        if self.remaining != 0 {
            self.remaining -= 1;
            Ok(())
        } else {
            Err(())
        }
    }
    fn acquire(&mut self) -> Result<(),()> {
        self.acquire_at(Time::now())
    }
}

impl<T> Timed<T> {
    fn new(data: T, time: Time) -> Timed<T> {
        Timed { data: data, time: time }
    }
    fn new_now(data: T) -> Timed<T> {
        Timed::new(data, Time::now())
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
        let socket = match UdpSocket::v4() {
            Ok(s) => s,
            Err(e) => {
                error!("Couldn't open socket, {}", e);
                return None;
            }
        };
        let mut work_queue = TimedWorkQueue::new();
        work_queue.add_duration(RESOLVE_REPEAT_MS.to_duration());
        work_queue.add_duration(LIST_REPEAT_MS.to_duration());
        work_queue.add_duration(LIST_EXPECT_MS.to_duration());
        work_queue.add_duration(INFO_REPEAT_MS.to_duration());
        work_queue.add_duration(INFO_EXPECT_MS.to_duration());
        Some(StatsBrowser {
            master_servers: Default::default(),
            servers: Default::default(),

            next_master_id: Default::default(),

            list_limit: Limit::new(MAX_LISTS, MAX_LISTS_MS.to_duration()),
            info_limit: Limit::new(MAX_INFOS, MAX_INFOS_MS.to_duration()),

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
            Ok(Some(x)) => {
                let addr = Addr { ip_address: x, port: MASTERSRV_PORT };
                info!("Resolved {} to {}", master.domain, addr);
                match mem::replace(&mut master.addr, Some(addr)) {
                    Some(_) => {},
                    None => { self.work_queue.push_now(Work::RequestList(master_id)); },
                }
            },
            Ok(None) => { info!("Resolved {}, no address found", master.domain); },
            Err(x) => { warn!("Error while resolving {}, {}", master.domain, x); },
        }
        self.work_queue.push(RESOLVE_REPEAT_MS.to_duration(), Work::Resolve(master_id));
	Ok(())
    }
    fn do_expect_list(&mut self, master_id: MasterId) -> Result<(),()> {
        if self.check_complete_list(master_id).is_ok() {
            self.work_queue.push(LIST_REPEAT_MS.to_duration(), Work::RequestList(master_id));
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

        let socket = &mut self.socket;
        let mut send = |&mut: y: &[u8]| socket.send_to(
            &mut SliceBuf::wrap(y),
            &addr_to_sockaddr(master.addr.unwrap()),
        ).unwrap();

        debug!("Requesting count and list from {}", master.domain);
        if protocol::request_count(|y| send(y)).would_block()
            || protocol::request_list_5(|y| send(y)).would_block()
            || protocol::request_list_6(|y| send(y)).would_block()
        {
            debug!("Failed to send count or list request, would block");
            self.work_queue.push_now_front(Work::RequestList(master_id));
            return Err(());
        }

        self.work_queue.push(LIST_EXPECT_MS.to_duration(), Work::ExpectList(master_id));
        Ok(())
    }
    fn do_expect_info(&mut self, server_addr: ServerAddr) -> Result<(),()> {
        let server = *self.servers.get_mut(&server_addr).unwrap();

        let now = Time::now();
        if server.num_missing_resp == 0 {
            self.work_queue.push(INFO_REPEAT_MS.to_duration(), Work::RequestInfo(server_addr));
        } else {
            if server.num_missing_resp >= 10 {
                // Throw the server out after ten missing replies.
                match self.servers.remove(&server_addr).unwrap().resp {
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
        server.num_missing_resp += 1;

        debug!("Requesting info from {}", server_addr);
        let socket = &mut self.socket;

        let mut send = |&mut: data: &[u8]| socket.send_to(
            &mut SliceBuf::wrap(data),
            &addr_to_sockaddr(server_addr.addr),
        ).unwrap();

        let would_block = match server_addr.version {
            ProtocolVersion::V5 => protocol::request_info_5(|x| send(x)).would_block(),
            ProtocolVersion::V6 => protocol::request_info_6(|x| send(x)).would_block(),
        };

        if would_block {
            debug!("Failed to send info request, would block");
            self.work_queue.push_now_front(Work::RequestInfo(server_addr));
            return Err(());
        }

        self.work_queue.push(INFO_EXPECT_MS.to_duration(), Work::ExpectInfo(server_addr));
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
                let _old_count = mem::replace(&mut master.count, updated_count);
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
        where I: Iterator<ServerAddr>+ExactSizeIterator<ServerAddr>,
    {
        let MasterId(idx) = from;
        let master = self.master_servers.get_mut(&idx).unwrap();

        debug!("Received list from {}, length {}", master.domain, servers_iter.len());

        let now = Time::now();
        for s in servers_iter {
            if !master.updated_list.insert(s) {
                warn!("Double-received {}", s);
            }
            if let Entry::Vacant(v) = self.servers.entry(s) {
                v.set(ServerEntry::new());
                self.work_queue.push_now(Work::RequestInfo(s));
            }
        }
    }
    fn process_info(&mut self, from: ServerAddr, info: Option<ServerInfo>, raw: &[u8]) {
        let server = match self.servers.get_mut(&from) {
            Some(x) => x,
            None => {
                warn!("Received info from unknown server {}, {}", from, raw);
                return;
            }
        };
        match info {
            None => {
                if server.num_malformed_resp < MAX_MALFORMED_RESP {
                    warn!("Received unparsable info from {}, {}", from, raw);
                }
                server.num_malformed_resp += 1;
            },
            Some(x) => {
                if server.num_missing_resp == 0 {
                    if server.num_extra_resp < MAX_EXTRA_RESP {
                        warn!("Received info while not expecting it, from {}, {}", from, x);
                    }
                    server.num_extra_resp += 1;
                    return;
                }
                server.num_missing_resp = 0;
                debug!("Received server info from {}, {}", from, x);
                match server.resp {
                    Some(y) => self.cb.on_server_change(from, &y.info, &x),
                    None => self.cb.on_server_new(from, &x)
                }
                server.resp = Some(ServerResponse::new(x));
            },
        }
    }
    fn process_packet(&mut self, from: Addr, data: &[u8]) {
        match protocol::parse_response(data) {
            None => {
                warn!("Received unknown message from {}, {}", from, data);
            },
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
                        self.process_list(id, servers.iter().map(|x| ServerAddr::new(ProtocolVersion::V5, x.unpack())));
                    },
                    None => {
                        let servers: Vec<_> = servers.iter().map(|x| x.unpack()).collect();
                        warn!("Received list message from non-master {}, servers={}", from, servers);
                    },
                }
            },
            Some(Response::List6(List6Response(servers))) => {
                match self.get_master_id(from) {
                    Some(id) => {
                        self.process_list(id, servers.iter().map(|x| ServerAddr::new(ProtocolVersion::V6, x.unpack())));
                    },
                    None => {
                        let servers: Vec<_> = servers.iter().map(|x| x.unpack()).collect();
                        warn!("Received list message from non-master {}, servers={}", from, servers);
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
        }
    }
    fn pump_network(&mut self) {
        let mut storage: [u8, ..2048] = unsafe { mem::uninitialized() };

        loop {
            let from;
            let remaining;
            {
                let mut buffer = MutSliceBuf::wrap(storage.as_mut_slice());
                match self.socket.recv_from(&mut buffer) {
                    Err(x) => { panic!("socket error, {}", x); },
                    Ok(NonBlock::WouldBlock) => return,
                    Ok(NonBlock::Ready(from_sockaddr)) => {
                        from = sockaddr_to_addr(from_sockaddr);
                        remaining = buffer.remaining();
                    },
                }
            }
            let read_len = storage.len() - remaining;
            self.process_packet(from, storage.as_slice().slice_to(read_len));
        }
    }
    fn run(&mut self) {
        loop {
            self.pump_network();
            while let Some(work) = self.work_queue.pop() {
                match work {
                    Work::Resolve(id)       => { if !self.do_resolve(id).is_ok()        { break; } },
                    Work::RequestList(id)   => { if !self.do_request_list(id).is_ok()   { break; } },
                    Work::ExpectList(id)    => { if !self.do_expect_list(id).is_ok()    { break; } },
                    Work::RequestInfo(addr) => { if !self.do_request_info(addr).is_ok() { break; } },
                    Work::ExpectInfo(addr)  => { if !self.do_expect_info(addr).is_ok()  { break; } },
                }
            }
            timer::sleep(SLEEP_MS.to_duration());
        }
    }
}

struct Tracker {
    player_count: u32,
}

fn print_player_new(addr: ServerAddr, info: &PlayerInfo) {
    println!("PLADD\t{}\t{}\t{}\t{}\t{}", addr, b64(&info.name), b64(&info.clan), info.is_player, info.country);
}

fn print_player_remove(addr: ServerAddr, info: &PlayerInfo) {
    if info.name.as_slice().as_bytes() == "(connecting)".as_bytes() { return; }
    println!("PLDEL\t{}\t{}", addr, b64(&info.name));
}

fn print_player_change(addr: ServerAddr, old: &PlayerInfo, new: &PlayerInfo) {
    print_player_remove(addr, new);
    print_player_new(addr, old);
}

fn print_server_remove(addr: ServerAddr, info: &ServerInfo) {
    let _ = info;
    println!("SVDEL\t{}", addr);
}

fn print_server_change_impl(addr: ServerAddr, new: bool, info: &ServerInfo) {
    println!("{}\t{}\t{}\t{}\t{}\t{}\t{}",
        if new { "SVADD" } else { "SVCHG" },
        addr,
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
        print_server_new(addr, info);
        self.diff_players(addr, &[], info.clients());
    }

    fn on_server_change(&mut self, addr: ServerAddr, old: &ServerInfo, new: &ServerInfo) {
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
        print_server_remove(addr, last);
        self.diff_players(addr, last.clients(), &[]);
    }
}

fn main() {
    let mut tracker = Tracker::new();
    let mut browser = match StatsBrowser::new(&mut tracker) {
        Some(b) => b,
        None => {
            panic!("Failed to bind socket.");
        },
    };
    browser.run();
}

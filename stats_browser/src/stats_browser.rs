use common::num::Cast;
use common::pretty::Bytes;
use serverbrowse::protocol::CountResponse;
use serverbrowse::protocol::Info5Response;
use serverbrowse::protocol::Info6ExMoreResponse;
use serverbrowse::protocol::Info6ExResponse;
use serverbrowse::protocol::Info6Response;
use serverbrowse::protocol::List5Response;
use serverbrowse::protocol::List6Response;
use serverbrowse::protocol::MASTERSERVER_PORT;
use serverbrowse::protocol::PartialServerInfo;
use serverbrowse::protocol::Response;
use serverbrowse::protocol::ServerInfo;
use serverbrowse::protocol;

use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::default::Default;
use std::mem;
use std::thread;

use addr::Addr;
use addr::ProtocolVersion;
use addr::ServerAddr;
use config;
use entry::MasterServerEntry;
use entry::ServerEntry;
use entry::ServerResponse;
use entry::Token;
use hashmap_ext::HashMapEntryIntoInner;
use lookup::lookup_host;
use socket::NonBlockExt;
use socket::UdpSocket;
use socket::WouldBlock;
use time::Limit;
use vec_map::VecMap;
use vec_map;
use work_queue::TimedWorkQueue;

pub trait StatsBrowserCb {
    fn on_server_new(&mut self, addr: ServerAddr, info: &ServerInfo);
    fn on_server_change(&mut self, addr: ServerAddr, old: &ServerInfo, new: &ServerInfo);
    fn on_server_remove(&mut self, addr: ServerAddr, last: &ServerInfo);
}

#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, RustcEncodable)]
struct MasterId(usize);

impl vec_map::Index for MasterId {
    fn to_usize(self) -> usize { let MasterId(val) = self; val }
    fn from_usize(val: usize) -> MasterId { MasterId(val) }
}

enum Work {
    Resolve(MasterId),
    RequestList(MasterId),
    ExpectList(MasterId),
    RequestInfo(ServerAddr),
    ExpectInfo(ServerAddr),
}

pub struct StatsBrowser<'a> {
    master_servers: VecMap<MasterId, MasterServerEntry>,
    servers: HashMap<ServerAddr,ServerEntry>,

    list_limit: Limit,
    info_limit: Limit,

    work_queue: TimedWorkQueue<Work>,
    socket: UdpSocket,
    rng: StdRng,
    cb: &'a mut (dyn StatsBrowserCb+'a),
}

impl<'a> StatsBrowser<'a> {
    pub fn new(cb: &mut dyn StatsBrowserCb) -> Option<StatsBrowser> {
        const MASTER_MIN: u32 = 1;
        const MASTER_MAX: u32 = 4;
        StatsBrowser::new_without_masters(cb).map(|mut browser| {
            for i in MASTER_MIN..MASTER_MAX+1 {
                browser.add_master(format!("master{}.teeworlds.com", i));
            }
            browser
        })
    }
    pub fn new_without_masters(cb: &mut dyn StatsBrowserCb) -> Option<StatsBrowser> {
        let socket = match UdpSocket::open() {
            Ok(s) => s,
            Err(e) => {
                error!("Couldn't open socket, {:?}", e);
                return None;
            }
        };
        let mut work_queue = TimedWorkQueue::new();
        work_queue.add_duration(config::RESOLVE_REPEAT_MS);
        work_queue.add_duration(config::LIST_REPEAT_MS);
        work_queue.add_duration(config::LIST_EXPECT_MS);
        work_queue.add_duration(config::INFO_REPEAT_MS);
        work_queue.add_duration(config::INFO_EXPECT_MS);
        Some(StatsBrowser {
            master_servers: Default::default(),
            servers: Default::default(),

            list_limit: Limit::new(config::MAX_LISTS, config::MAX_LISTS_MS),
            info_limit: Limit::new(config::MAX_INFOS, config::MAX_INFOS_MS),

            work_queue: work_queue,
            socket: socket,
            rng: StdRng::from_entropy(),
            cb: cb,
        })
    }
    pub fn add_master(&mut self, domain: String) {
        let master_id = self.master_servers.push(MasterServerEntry::new(domain));
        self.work_queue.push_now(Work::Resolve(master_id));
    }
    fn do_resolve(&mut self, master_id: MasterId) -> Result<(),()> {
        let master = &mut self.master_servers[master_id];
        match lookup_host(&master.domain, MASTERSERVER_PORT) {
            Ok(Some(addr)) => {
                info!("Resolved {} to {}", master.domain, addr);
                match mem::replace(&mut master.addr, Some(addr)) {
                    Some(_) => {},
                    None => { self.work_queue.push_now(Work::RequestList(master_id)); },
                }
            },
            Ok(None) => { info!("Resolved {}, no address found", master.domain); },
            Err(x) => { warn!("Error while resolving {}, {}", master.domain, x); },
        }
        self.work_queue.push(config::RESOLVE_REPEAT_MS, Work::Resolve(master_id));
        Ok(())
    }
    fn do_expect_list(&mut self, master_id: MasterId) -> Result<(),()> {
        if self.check_complete_list(master_id).is_ok() {
            self.work_queue.push(config::LIST_REPEAT_MS, Work::RequestList(master_id));
        } else {
            let master = &mut self.master_servers[master_id];
            info!("Re-requesting list for {}", master.domain);
            self.work_queue.push_now(Work::RequestList(master_id));
        }
        Ok(())
    }
    fn do_request_list(&mut self, master_id: MasterId) -> Result<(),()> {
        let master = &mut self.master_servers[master_id];

        if !self.list_limit.acquire().is_ok() {
            return Err(());
        }

        let socket = &mut self.socket;
        let mut send = |data: &[u8]| socket.send_to(data, master.addr.unwrap()).unwrap();

        debug!("Requesting count and list from {}", master.domain);
        if send(&protocol::request_count()).would_block()
            || send(&protocol::request_list_5()).would_block()
            || send(&protocol::request_list_6()).would_block()
        {
            debug!("Failed to send count or list request, would block");
            return Err(());
        }

        self.work_queue.push(config::LIST_EXPECT_MS, Work::ExpectList(master_id));
        Ok(())
    }
    fn do_expect_info(&mut self, server_addr: ServerAddr) -> Result<(),()> {
        let server = self.servers.entry(server_addr).into_occupied().unwrap();

        if server.get().missing_resp.is_empty() {
            self.work_queue.push(config::INFO_REPEAT_MS, Work::RequestInfo(server_addr));
        } else {
            if server.get().missing_resp.len() >= 10 {
                info!("Missing responses from {}, removing", server_addr);
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

        let mut send = |data: &[u8]| socket.send_to(data, server_addr.addr).unwrap();

        let mut token: Token = self.rng.gen();
        while server.missing_resp.iter().any(|&t| t.u8() == token.u8()) {
            token = self.rng.gen();
        }
        let token = token;
        let would_block = match server_addr.version {
            ProtocolVersion::V5 => send(&protocol::request_info_5(token.u8())).would_block(),
            ProtocolVersion::V6 => send(&protocol::request_info_6_ex(token.u24())).would_block(),
        };

        if would_block {
            debug!("Failed to send info request, would block");
            return Err(());
        }

        server.missing_resp.push(token);

        self.work_queue.push(config::INFO_EXPECT_MS, Work::ExpectInfo(server_addr));
        Ok(())
    }
    fn get_master_id(&self, addr: Addr) -> Option<MasterId> {
        for (id, master) in self.master_servers.iter() {
            if master.addr == Some(addr) {
                return Some(id);
            }
        }
        None
    }
    fn check_complete_list(&mut self, master_id: MasterId) -> Result<(),()> {
        let master = &mut self.master_servers[master_id];

        let updated_count = master.updated_count.take();
        let updated_list = mem::replace(&mut master.updated_list, HashSet::new());

        if let Some(updated_count) = updated_count {
            if (updated_count as isize - updated_list.len() as isize).abs() <= 5 {
                let _old_list = mem::replace(&mut master.list, updated_list);
                // TODO: diff
                return Ok(());
            }
        }
        Err(())
    }
    fn process_count(&mut self, from: MasterId, count: u16) {
        let master = &mut self.master_servers[from];

        debug!("Received count from {}, {}", master.domain, count);

        match mem::replace(&mut master.updated_count, Some(count)) {
            Some(x) => {
                warn!("Received double count message, old={}", x);
            },
            None => {},
        }
    }
    fn process_list<I>(&mut self, from: MasterId, servers_iter: I)
        where I: Iterator<Item=ServerAddr>+ExactSizeIterator,
    {
        let master = &mut self.master_servers[from];

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
                    warn!("Received unparsable info from {}, {:?}", from, Bytes::new(raw));
                }
                server.num_malformed_resp += 1;
            },
            Some(x) => {
                if server.missing_resp.is_empty() {
                    if server.num_extra_resp < config::MAX_EXTRA_RESP {
                        warn!("Received info while not expecting it, from {}, {:?}", from, x);
                    }
                    server.num_extra_resp += 1;
                    return;
                }
                if !server.missing_resp.iter().any(|&t| t.u8().i32() == x.token) {
                    if server.num_invalid_resp < config::MAX_INVALID_RESP {
                        warn!("Received info with wrong token from {}, {:?}", from, x);
                    }
                    server.num_invalid_resp += 1;
                    return;
                }
                server.missing_resp.clear();
                server.partial_resp.clear();
                debug!("Received server info from {}, {:?}", from, x);
                match server.resp {
                    Some(ref y) => self.cb.on_server_change(from, &y.info, &x),
                    None => self.cb.on_server_new(from, &x)
                }
                server.resp = Some(ServerResponse::new(x));
            },
        }
    }
    fn process_partial_info(
        &mut self,
        from: ServerAddr,
        info: Option<PartialServerInfo>,
        raw: &[u8],
    ) {
        let server = match self.servers.get_mut(&from) {
            Some(x) => x,
            None => {
                warn!("Received partial info from unknown server {}, {:?}", from, raw);
                return;
            }
        };
        match info {
            None => {
                if server.num_malformed_resp < config::MAX_MALFORMED_RESP {
                    warn!("Received unparsable partial info from {}, {:?}", from, Bytes::new(raw));
                }
                server.num_malformed_resp += 1;
            },
            Some(x) => {
                if server.missing_resp.is_empty() {
                    if server.num_extra_resp < config::MAX_EXTRA_RESP {
                        warn!("Received partial info while not expecting it, from {}, {:?}", from, x);
                    }
                    server.num_extra_resp += 1;
                    return;
                }
                if !server.missing_resp.iter().any(|&t| t.u24().assert_i32() == x.token()) {
                    if server.num_invalid_resp < config::MAX_INVALID_RESP {
                        warn!("Received partial info with wrong token from {}, {:?}", from, x);
                    }
                    server.num_invalid_resp += 1;
                    return;
                }
                debug!("Received partial server info from {}, {:?}", from, x);
                let index;
                if let Some(i) = server.partial_resp.iter().position(|r| r.token() == x.token()) {
                    index = i;
                    if let Err(e) = server.partial_resp[i].merge(x) {
                        warn!("Received partial server info {:?} incompatible with {:?}: {:?}", raw, server.partial_resp[i], e);
                        return;
                    }
                } else {
                    index = server.partial_resp.len();
                    server.partial_resp.push(x);
                }
                let info = match server.partial_resp[index].take_info() {
                    None => return,
                    Some(i) => i,
                };
                server.missing_resp.clear();
                server.partial_resp.clear();
                debug!("Partial server info from {} complete, {:?}", from, info);
                match server.resp {
                    Some(ref y) => self.cb.on_server_change(from, &y.info, &info),
                    None => self.cb.on_server_new(from, &info)
                }
                server.resp = Some(ServerResponse::new(info));
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
            Some(Response::Info6Ex(partial)) => {
                let Info6ExResponse(raw_data) = partial;
                self.process_partial_info(ServerAddr::new(ProtocolVersion::V6, from), partial.parse(), raw_data);
            },
            Some(Response::Info6ExMore(partial)) => {
                let Info6ExMoreResponse(raw_data) = partial;
                self.process_partial_info(ServerAddr::new(ProtocolVersion::V6, from), partial.parse(), raw_data);
            },
            _ => {
                warn!("Received unknown message from {}, {:?}", from, data);
            },
        }
    }
    fn pump_network(&mut self) {
        let mut buffer = [0u8; 2048];

        loop {
            match self.socket.recv_from(&mut buffer) {
                Err(x) => { panic!("socket error, {:?}", x); },
                Ok(Err(WouldBlock)) => return,
                Ok(Ok((read_len, from))) => {
                    self.process_packet(from, &buffer[..read_len]);
                },
            }
        }
    }
    pub fn run(&mut self) {
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
            thread::sleep(config::SLEEP_MS.to_std());
        }
    }
}

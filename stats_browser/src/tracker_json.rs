use StatsBrowserCb;
use arrayvec::ArrayString;
use addr::ProtocolVersion;
use addr::ServerAddr;
use csv;
use ipnet::Ipv4Net;
use serverbrowse::protocol::ClientInfo;
use serverbrowse::protocol::IpAddr;
use serverbrowse::protocol::ServerInfo;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::fs;
use std::io::BufWriter;
use std::io::Write;
use std::mem;
use std::process;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

mod json {
    use addr;
    use arrayvec::ArrayString;
    use serverbrowse::protocol;
    use std::collections::BTreeMap;
    use std::convert::TryFrom;
    use std::convert::TryInto;
    use std::fmt::Write;
    use super::Timestamp;
    use uuid::Uuid;

    #[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
    pub struct Addr(pub addr::ServerAddr);

    #[derive(Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum EntryKind {
        Backcompat,
    }

    #[derive(Serialize)]
    pub struct Dump<'a> {
        pub now: Timestamp,
        // Use `BTreeMap`s so the serialization is stable.
        pub addresses: BTreeMap<Addr, AddrInfo>,
        pub servers: BTreeMap<Uuid, Server<'a>>,
    }
    #[derive(Serialize)]
    pub struct AddrInfo {
        pub kind: EntryKind,
        pub ping_time: Timestamp,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub location: Option<ArrayString<[u8; 15]>>,
        pub secret: Uuid,
    }
    #[derive(Serialize)]
    pub struct Server<'a> {
        pub info_serial: Timestamp,
        pub info: &'a ServerInfo,
    }
    #[derive(Serialize)]
    pub struct ServerInfo {
        pub max_clients: i32,
        pub max_players: i32,
        pub passworded: bool,
        pub game_type: ArrayString<[u8; 32]>,
        pub name: ArrayString<[u8; 64]>,
        pub map: MapInfo,
        pub version: ArrayString<[u8; 32]>,
        pub clients: Vec<ClientInfo>,
    }
    #[derive(Serialize)]
    pub struct MapInfo {
        pub name: ArrayString<[u8; 32]>,
    }
    #[derive(Serialize)]
    pub struct ClientInfo {
        pub name: ArrayString<[u8; 15]>,
        pub clan: ArrayString<[u8; 11]>,
        pub country: i32,
        pub score: i32,
        pub is_player: bool,
    }

    impl serde::Serialize for Addr {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
            S: serde::Serializer,
        {
            let mut result: ArrayString<[u8; 64]> = ArrayString::new();
            result.push_str(match self.0.version {
                addr::ProtocolVersion::V5 => "tw-0.5+udp://",
                addr::ProtocolVersion::V6 => "tw-0.6+udp://",
                addr::ProtocolVersion::V7 => "tw-0.7+udp://",
            });
            write!(result, "{}", self.0.addr).unwrap();
            serializer.serialize_str(&result)
        }
    }

    pub struct Error;

    impl<'a> TryFrom<&'a super::ClientInfo> for ClientInfo {
        type Error = Error;
        fn try_from(i: &'a super::ClientInfo) -> Result<ClientInfo, Error> {
            Ok(ClientInfo {
                name: i.name,
                clan: i.clan,
                country: i.country,
                score: i.score,
                is_player: i.is_player != 0,
            })
        }
    }
    impl<'a> TryFrom<&'a super::ServerInfo> for ServerInfo {
        type Error = Error;
        fn try_from(i: &'a super::ServerInfo) -> Result<ServerInfo, Error> {
            let mut result = ServerInfo {
                max_clients: i.max_clients,
                max_players: i.max_players,
                passworded: i.flags & protocol::SERVERINFO_FLAG_PASSWORDED != 0,
                game_type: i.game_type,
                name: i.name,
                map: MapInfo {
                    name: i.map,
                },
                version: i.version,
                clients: Vec::new(),
            };
            result.clients.reserve_exact(i.clients.len());
            for c in &i.clients {
                result.clients.push(c.try_into()?);
            }
            Ok(result)
        }
    }
}

/// Time in milliseconds since the epoch of the timekeeper.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Timestamp(i64);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy)]
struct Timekeeper {
    instant: Instant,
}

impl Timekeeper {
    fn new() -> Timekeeper {
        Timekeeper {
            instant: Instant::now(),
        }
    }
    fn now(&self) -> Timestamp {
        Timestamp(self.instant.elapsed().as_millis() as i64)
    }
}

#[derive(Deserialize)]
struct LocationRecord {
    network: Ipv4Net,
    location: ArrayString<[u8; 15]>,
}

pub struct ServerEntry {
    location: Option<ArrayString<[u8; 15]>>,
    info: Option<json::ServerInfo>,
    ping_time: Timestamp,
}

pub struct Tracker {
    filename: String,
    locations: Vec<LocationRecord>,
    secret_seed: Uuid,
    servers: Arc<Mutex<HashMap<ServerAddr, ServerEntry>>>,
    timekeeper: Timekeeper,
}

const PROTOCOL_VERSIONS_PRIORITY: &'static [ProtocolVersion] = &[
    ProtocolVersion::V5,
    ProtocolVersion::V7,
    ProtocolVersion::V6,
];

impl Tracker {
    pub fn new(filename: String, locations_filename: Option<String>, secret_seed: Option<Uuid>)
        -> Tracker
    {
        let locations: Result<Vec<_>, _>;
        if let Some(l) = locations_filename {
            let mut locations_reader = csv::Reader::from_path(l).unwrap();
            locations = locations_reader.deserialize().collect();
        } else {
            locations = Ok(Vec::new());
        }
        Tracker {
            filename,
            locations: locations.unwrap(),
            secret_seed: secret_seed.unwrap_or_else(Uuid::new_v4),
            servers: Default::default(),
            timekeeper: Timekeeper::new(),
        }
    }
    pub fn start(&mut self) {
        let mut tracker_thread = Tracker {
            filename: mem::replace(&mut self.filename, String::new()),
            locations: Vec::new(),
            secret_seed: self.secret_seed,
            servers: self.servers.clone(),
            timekeeper: self.timekeeper,
        };
        thread::spawn(move || tracker_thread.handle_writeout());
    }
    fn lookup_location(&self, addr: ServerAddr) -> Option<ArrayString<[u8; 15]>> {
        let ip_addr = match addr.addr.to_srvbrowse_addr().ip_address {
            IpAddr::V4(a) => a,
            IpAddr::V6(_) => return None, // sad smiley
        };
        for LocationRecord { network, location } in &self.locations {
            if network.contains(&ip_addr) {
                return Some(*location);
            }
        }
        None
    }
    fn handle_writeout(&mut self) {
        let temp_filename = format!("{}.tmp.{}", self.filename, process::id());

        thread::sleep(Duration::from_secs(15));

        let start = Instant::now();
        let mut iteration = 0;
        loop {
            {
                let servers = self.servers.lock().unwrap();
                let mut addresses: Vec<_> = servers.keys()
                    .map(|a| a.addr).collect();
                addresses.sort_unstable();
                addresses.dedup();

                let mut dump = json::Dump {
                    now: self.timekeeper.now(),
                    addresses: BTreeMap::new(),
                    servers: BTreeMap::new(),
                };
                for &addr in &addresses {
                    let secret = Uuid::new_v5(&self.secret_seed, addr.to_string().as_bytes());
                    let mut entry = None;
                    for &version in PROTOCOL_VERSIONS_PRIORITY {
                        let server_addr = ServerAddr::new(version, addr);
                        if let Some(e) = servers.get(&server_addr) {
                            assert!(dump.addresses.insert(json::Addr(server_addr), json::AddrInfo {
                                kind: json::EntryKind::Backcompat,
                                ping_time: e.ping_time,
                                location: e.location,
                                secret,
                            }).is_none());
                            entry = Some(e);
                        }
                    }
                    let entry = entry.unwrap();
                    if let Some(i) = &entry.info {
                        dump.servers.insert(secret, json::Server {
                            info_serial: entry.ping_time,
                            info: i,
                        });
                    }
                }

                {
                    let temp_file = File::create(&temp_filename).unwrap();
                    let mut temp_file = BufWriter::new(temp_file);
                    serde_json::to_writer(&mut temp_file, &dump).unwrap();
                    temp_file.flush().unwrap();
                    // Drop the temporary file.
                }

                fs::rename(&temp_filename, &self.filename).unwrap();
                // Drop the mutex.
            }
            let elapsed = start.elapsed();
            if elapsed.as_secs() <= iteration {
                let remaining_ns = 1_000_000_000 - elapsed.subsec_nanos();
                thread::sleep(Duration::new(0, remaining_ns));
                iteration += 1;
            } else {
                iteration = elapsed.as_secs();
            }
        }
    }
}

impl StatsBrowserCb for Tracker {
    fn on_server_new(&mut self, addr: ServerAddr, info: &ServerInfo) {
        let mut servers = self.servers.lock().unwrap();
        let info = json::ServerInfo::try_from(info).ok();
        assert!(servers.insert(addr, ServerEntry {
            location: self.lookup_location(addr),
            info,
            ping_time: self.timekeeper.now(),
        }).is_none());
    }
    fn on_server_change(
        &mut self,
        addr: ServerAddr,
        _old: &ServerInfo,
        new: &ServerInfo,
    ) {
        let mut servers = self.servers.lock().unwrap();
        let server = servers.get_mut(&addr).unwrap();
        server.info = json::ServerInfo::try_from(new).ok();
        server.ping_time = self.timekeeper.now();
    }
    fn on_server_remove(&mut self, addr: ServerAddr, _last: &ServerInfo) {
        let mut servers = self.servers.lock().unwrap();
        assert!(servers.remove(&addr).is_some());
    }
}

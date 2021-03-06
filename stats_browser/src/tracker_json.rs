use StatsBrowserCb;
use addr::ALL_PROTOCOL_VERSIONS;
use addr::ServerAddr;
use serverbrowse::protocol::ClientInfo;
use serverbrowse::protocol::ServerInfo;
use std::collections::HashMap;
use std::convert::TryFrom;
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

mod json {
    use addr;
    use arrayvec::Array;
    use arrayvec::ArrayString;
    use serverbrowse::protocol;
    use std::convert::TryFrom;
    use std::convert::TryInto;
    use std::fmt::Write;
    use std::str;

    pub struct Addr(pub addr::ServerAddr);

    #[derive(Serialize)]
    pub struct MasterInfo<'a> {
        pub servers: &'a [Server<'a>],
    }
    #[derive(Serialize)]
    pub struct Server<'a> {
        pub addresses: Vec<Addr>,
        pub info: &'a ServerInfo,
    }
    #[derive(Serialize)]
    pub struct ServerInfo {
        pub max_clients: i32,
        pub max_players: i32,
        pub passworded: bool,
        pub game_type: ArrayString<[u8; 16]>,
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
        pub name: ArrayString<[u8; 16]>,
        pub clan: ArrayString<[u8; 16]>,
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
            });
            write!(result, "{}", self.0.addr).unwrap();
            serializer.serialize_str(&result)
        }
    }

    pub struct Error;

    fn s<A: Array<Item=u8> + Copy>(bytes: &[u8]) -> Result<ArrayString<A>, Error> {
        let string = str::from_utf8(bytes).map_err(|_| Error)?;
        let mut result = ArrayString::new();
        result.try_push_str(string).map_err(|_| Error)?;
        Ok(result)
    }
    impl<'a> TryFrom<&'a super::ClientInfo> for ClientInfo {
        type Error = Error;
        fn try_from(i: &'a super::ClientInfo) -> Result<ClientInfo, Error> {
            Ok(ClientInfo {
                name: s(&i.name)?,
                clan: s(&i.clan)?,
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
                game_type: s(&i.game_type)?,
                name: s(&i.name)?,
                map: MapInfo {
                    name: s(&i.map)?,
                },
                version: s(&i.version)?,
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

pub struct Tracker {
    filename: String,
    servers: Arc<Mutex<HashMap<ServerAddr, Option<json::ServerInfo>>>>,
}

impl Tracker {
    pub fn new(filename: String) -> Tracker {
        Tracker {
            filename,
            servers: Default::default(),
        }
    }
    pub fn start(&mut self) {
        let mut tracker_thread = Tracker {
            filename: mem::replace(&mut self.filename, String::new()),
            servers: self.servers.clone(),
        };
        thread::spawn(move || tracker_thread.handle_writeout());
    }
    fn handle_writeout(&mut self) {
        let temp_filename = format!("{}.tmp.{}", self.filename, process::id());

        let start = Instant::now();
        let mut iteration = 0;
        loop {
            {
                let servers = self.servers.lock().unwrap();
                let mut addresses: Vec<_> = servers.keys()
                    .map(|a| a.addr).collect();
                addresses.sort_unstable();
                addresses.dedup();

                let mut result = Vec::new();
                for &addr in &addresses {
                    let mut info = None;
                    let mut addresses = Vec::new();
                    for &version in ALL_PROTOCOL_VERSIONS {
                        let server_addr = ServerAddr::new(version, addr);
                        if let Some(i) = servers.get(&server_addr) {
                            addresses.push(json::Addr(server_addr));
                            info = Some(i);
                        }
                    }
                    if let Some(i) = info.unwrap() {
                        result.push(json::Server {
                            addresses,
                            info: i,
                        });
                    }
                }

                let master = json::MasterInfo {
                    servers: &result,
                };

                {
                    let temp_file = File::create(&temp_filename).unwrap();
                    let mut temp_file = BufWriter::new(temp_file);
                    serde_json::to_writer(&mut temp_file, &master).unwrap();
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
        assert!(servers.insert(addr, json::ServerInfo::try_from(info).ok()).is_none());
    }
    fn on_server_change(
        &mut self,
        addr: ServerAddr,
        _old: &ServerInfo,
        new: &ServerInfo,
    ) {
        let mut servers = self.servers.lock().unwrap();
        *servers.get_mut(&addr).unwrap() = json::ServerInfo::try_from(new).ok();
    }
    fn on_server_remove(&mut self, addr: ServerAddr, _last: &ServerInfo) {
        let mut servers = self.servers.lock().unwrap();
        assert!(servers.remove(&addr).is_some());
    }
}

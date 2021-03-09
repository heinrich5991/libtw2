use serverbrowse::protocol::ServerInfo;

use std::collections::HashSet;

use arrayvec::ArrayVec;
use addr::Addr;
use addr::ServerAddr;

/// Describes a master server.
#[derive(Clone)]
pub struct MasterServerEntry {
    /// Domain of the master server.
    pub domain: String,
    /// Address of the master server if resolved, `None` otherwise.
    pub addr: Option<Addr>,

    /// Servers that the master server lists.
    pub list: HashSet<ServerAddr>,

    /// Field that is used when requesting the number of servers from the
    /// master server.
    pub updated_count: Option<u16>,
    /// Field that is used when requesting the list of servers from the master
    /// server.
    pub updated_list: HashSet<ServerAddr>,
}

impl MasterServerEntry {
    /// Creates a new master server entry with empty responses from a domain.
    pub fn new(domain: String) -> MasterServerEntry {
        MasterServerEntry {
            domain: domain,
            addr: None,

            list: HashSet::new(),
            updated_count: None,
            updated_list: HashSet::new(),
        }
    }
}

/// Describes a server.
#[derive(Clone)]
pub struct ServerEntry {
    /// Tokens with missing responses since the last successful info request.
    pub missing_resp: ArrayVec<[u8; 16]>,
    /// Total number of malformed responses from this server.
    pub num_malformed_resp: u32,
    /// Total number of responses with invalid token from this server.
    pub num_invalid_resp: u32,
    /// Total number of excess responses from this server.
    pub num_extra_resp: u32,
    /// The last response from a server if received, `None` otherwise.
    pub resp: Option<ServerResponse>,
    /// Whether the server supports the 0.6_64 protocol, only interesting if
    /// the server is from a 0.6 master server.
    pub server_664_support: Option<bool>,
}

impl ServerEntry {
    /// Creates a new server entry with empty responses.
    pub fn new() -> ServerEntry {
        ServerEntry {
            missing_resp: ArrayVec::new(),
            num_malformed_resp: 0,
            num_invalid_resp: 0,
            num_extra_resp: 0,
            resp: None,
            server_664_support: None,
        }
    }
}

/// Describes a server info response.
#[derive(Clone)]
pub struct ServerResponse {
    /// The server info received from the info request.
    pub info: ServerInfo,
}

impl ServerResponse {
    /// Creates a new server response from the received server info.
    pub fn new(info: ServerInfo) -> ServerResponse {
        ServerResponse {
            info: info,
        }
    }
}

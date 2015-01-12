use serverbrowse::protocol::ServerInfo;

use std::collections::HashSet;

use addr::Addr;
use addr::ServerAddr;

/// Describes a master server.
#[derive(Clone)]
pub struct MasterServerEntry {
    /// Domain of the master server.
    pub domain: String,
    /// Address of the master server if resolved, `None` otherwise.
    pub addr: Option<Addr>,

    /// Number of servers the master server advertised to have.
    pub count: u16,
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

            count: 0,
            list: HashSet::new(),
            updated_count: None,
            updated_list: HashSet::new(),
        }
    }
}

/// Describes a server.
#[derive(Copy, Clone)]
pub struct ServerEntry {
    /// Number of missing responses since the last successful info request.
    pub num_missing_resp: u32,
    /// Total number of malformed responses from this server.
    pub num_malformed_resp: u32,
    /// Total number of excess responses from this server.
    pub num_extra_resp: u32,
    /// The last response from a server if received, `None` otherwise.
    pub resp: Option<ServerResponse>,
}

impl ServerEntry {
    /// Creates a new server entry with empty responses.
    pub fn new() -> ServerEntry {
        ServerEntry {
            num_missing_resp: 0,
            num_malformed_resp: 0,
            num_extra_resp: 0,
            resp: None,
        }
    }
}

/// Describes a server info response.
#[derive(Copy, Clone)]
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

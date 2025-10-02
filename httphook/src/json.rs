use libtw2_serverbrowse::protocol as browse_protocol;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Server {
    pub max_clients: i32,
    pub max_players: i32,
    pub passworded: bool,
    pub game_type: String,
    pub name: String,
    pub map: Map,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_login: Option<bool>,
    pub clients: Vec<Client>,
}

#[derive(Serialize)]
pub struct Map {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tw_crc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

#[derive(Serialize)]
pub struct Client {
    pub name: String,
    pub clan: String,
    pub country: i32,
    pub score: i32,
    pub is_player: bool,
}

impl From<&browse_protocol::ServerInfo> for Server {
    fn from(info: &browse_protocol::ServerInfo) -> Server {
        Server {
            max_clients: info.max_clients,
            max_players: info.max_players,
            passworded: info.flags & browse_protocol::SERVERINFO_FLAG_PASSWORDED != 0,
            game_type: (&*info.game_type).into(),
            name: (&*info.name).into(),
            map: Map {
                name: (&*info.map).into(),
                tw_crc: info.map_crc.map(|crc| format!("{:08x}", crc)),
                size: info.map_size,
            },
            version: (&*info.version).into(),
            requires_login: None,
            clients: info.clients.iter().map(Client::from).collect(),
        }
    }
}

impl From<&browse_protocol::ClientInfo> for Client {
    fn from(info: &browse_protocol::ClientInfo) -> Client {
        Client {
            name: (&*info.name).into(),
            clan: (&*info.clan).into(),
            country: info.country,
            score: info.score,
            is_player: info.flags & browse_protocol::CLIENTINFO_FLAG_SPECTATOR == 0,
        }
    }
}

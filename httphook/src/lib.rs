#![cfg(not(test))]

#[macro_use]
extern crate log;

use libtw2_polyfill_1_63::OnceLock;
use libtw2_register::Register;
use libtw2_serverbrowse::protocol as browse_protocol;
use libtw2_serverbrowse::protocol::Response;
use serde_derive::Deserialize;
use std::collections::btree_map;
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Once;
use std::sync::RwLock;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

use self::runtime::spawn;

mod json;
mod runtime;

// TODO: maybe support teeworlds 0.7 servers?

#[derive(Deserialize)]
struct Config {
    community_token: Option<Box<str>>,
    log: Option<Box<str>>,
    override_requires_login: Option<bool>,
    register_url: Option<Box<str>>,
    protocols: Option<libtw2_register::Protocols>,
}

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| envy::prefixed("LIBTW2_HTTPHOOK_").from_env().unwrap())
}

// TODO (MSRV 1.63): Remove the `Option`.
static REGISTERS: RwLock<Option<BTreeMap<u16, Arc<OnceLock<Register>>>>> = RwLock::new(None);

pub fn on_packet(data: &[u8]) {
    if data.starts_with(browse_protocol::CHALLENGE_6) {
        if let Some(registers) = &*REGISTERS.read().unwrap() {
            for register in registers.values() {
                if let Some(register) = register.get() {
                    register.on_udp_packet(data);
                }
            }
        }
    }
}

pub fn register_server_6(port: u16) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut logger = env_logger::LogBuilder::new();
        logger.filter(None, log::LogLevelFilter::Info);
        if let Some(filters) = &config().log {
            logger.parse(&filters);
        }
        logger.format(|record| {
            use log::LogLevel::*;
            let level = match record.level() {
                Error => 'E',
                Warn => 'W',
                Info => 'I',
                Debug => 'D',
                Trace => 'T',
            };
            format!(
                "                    {} {}: {}",
                level,
                record.metadata().target(),
                record.args(),
            )
        });
        let _ = logger.init();
    });

    if port == 0 {
        error!("can't register server on port 0");
        return;
    }
    let register = match REGISTERS
        .write()
        .unwrap()
        .get_or_insert_with(BTreeMap::new)
        .entry(port)
    {
        btree_map::Entry::Occupied(_) => return, // already started
        btree_map::Entry::Vacant(v) => v.insert(Arc::new(OnceLock::new())).clone(),
    };
    spawn(register_server_6_impl(port, register));
}

async fn request_server_info_6(
    socket: &UdpSocket,
    addr: SocketAddr,
) -> browse_protocol::ServerInfo {
    let mut buf = [0; 2048];
    socket
        .send_to(&browse_protocol::request_info_6_ex(0), addr)
        .await
        .unwrap();

    let mut partial: Option<browse_protocol::PartialServerInfo> = None;
    loop {
        let (len, from) = socket.recv_from(&mut buf).await.unwrap();
        if from != addr {
            error!(
                "received response from non-peer, wanted={} got={}",
                addr, from,
            );
            continue;
        }
        let new_partial = match browse_protocol::parse_response(&buf[..len]) {
            Some(Response::Info6(info)) => {
                if let Some(info) = info.parse() {
                    return info;
                } else {
                    error!("received bad info6 response from peer");
                }
                continue;
            }
            Some(Response::Info6Ex(new_partial)) => {
                if let Some(new_partial) = new_partial.parse() {
                    new_partial
                } else {
                    error!("received bad info6_ex response from peer");
                    continue;
                }
            }
            Some(Response::Info6ExMore(new_partial)) => {
                if let Some(new_partial) = new_partial.parse() {
                    new_partial
                } else {
                    error!("received bad info6_ex_more response from peer");
                    continue;
                }
            }
            _ => {
                error!("received non-info response from peer");
                continue;
            }
        };
        if let Some(partial) = &mut partial {
            if let Err(err) = partial.merge(new_partial) {
                error!("error merging infos: {err:?}");
                continue;
            }
        } else {
            partial = Some(new_partial);
        }
        if let Some(partial) = &mut partial {
            if let Some(info) = partial.take_info() {
                return info;
            }
        }
    }
}

fn build_register(port: u16, info: Arc<str>) -> Register {
    let mut builder = Register::builder()
        .require_external_heartbeats()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")).into());
    let config = config();
    if let Some(community_token) = &config.community_token {
        builder = builder.community_token((&**community_token).into());
    }
    if let Some(register_url) = &config.register_url {
        builder = builder.register_url((&**register_url).into());
    }
    if let Some(protocols) = config.protocols {
        builder = builder.protocols(protocols);
    }
    builder.build(port, info.into())
}

fn apply_overrides(mut info: json::Server) -> json::Server {
    let config = &config();
    info.requires_login = config.override_requires_login.or(info.requires_login);
    info
}

async fn register_server_6_impl(port: u16, register: Arc<OnceLock<Register>>) {
    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    const LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let addr = SocketAddr::new(LOCALHOST, port);

    let mut interval = time::interval(Duration::from_millis(1_000));
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    loop {
        interval.tick().await;

        let info = match time::timeout(
            Duration::from_millis(800),
            request_server_info_6(&socket, addr),
        )
        .await
        {
            Ok(info) => info,
            Err(_) => continue,
        };
        let info = serde_json::to_string(&apply_overrides(json::Server::from(&info))).unwrap();

        if let Some(register) = register.get() {
            register.on_new_info(info.into());
            register.on_heartbeat();
        } else {
            register
                .set(build_register(port, info.into()))
                .ok()
                .expect("register cannot be set concurrently");
        }
    }
}

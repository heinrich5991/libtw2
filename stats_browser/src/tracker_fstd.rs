use serverbrowse::protocol::NzU8SliceExt;
use serverbrowse::protocol::PString64;
use serverbrowse::protocol::PlayerInfo;
use serverbrowse::protocol::ServerInfo;
use serverbrowse::protocol::ServerInfoVersion;

use std::cmp::Ordering;
use std::fmt;

use rust_time;

use addr::ProtocolVersion;
use addr::ServerAddr;
use base64::B64;

use StatsBrowserCb;

#[allow(missing_copy_implementations)]
pub struct Tracker {
    player_count: u32,
}

impl Tracker {
    pub fn new() -> Tracker {
        Tracker {
            player_count: 0,
        }
    }
    pub fn start(&mut self) {
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

/// Returns a `B64` for a `PString64`.
fn b64(string: &PString64) -> B64 {
    B64(string.as_slice().as_bytes())
}

#[derive(Clone, Copy)]
struct LogVersion(ServerInfoVersion);

impl fmt::String for LogVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let LogVersion(inner) = *self;
        let output = match inner {
            ServerInfoVersion::V5 => "5",
            ServerInfoVersion::V6 => "6",
            ServerInfoVersion::V664 => "6_64",
            ServerInfoVersion::V7 => "7",
        };
        fmt::String::fmt(&output, f)
    }
}

fn print(command: &str, args: &[&fmt::Display]) {
    print!("{}\t{}", rust_time::get_time().sec, command);
    for a in args.iter() {
        print!("\t{}", a);
    }
    println!("");
}

fn print_start() {
    print("START", &[&"1.0", &"libtw2_statsbrowser", &"0.1"]);
}

fn print_player_new(ver: LogVersion, addr: ServerAddr, info: &PlayerInfo) {
    print("PLADD", &[
        &ver,
        &addr.addr,
        &b64(&info.name),
        &b64(&info.clan),
        &info.is_player,
        &info.country,
    ]);
}

fn print_player_remove(ver: LogVersion, addr: ServerAddr, info: &PlayerInfo) {
    print("PLDEL", &[
        &ver,
        &addr.addr,
        &b64(&info.name),
    ]);
}

fn print_player_change(ver: LogVersion, addr: ServerAddr, old: &PlayerInfo, new: &PlayerInfo) {
    print_player_remove(ver, addr, new);
    print_player_new(ver, addr, old);
}

fn print_server_remove(ver: LogVersion, addr: ServerAddr, info: &ServerInfo) {
    let _ = info;
    print("SVDEL", &[
        &addr.addr,
        &ver,
    ]);
}

fn print_server_change_impl(ver: LogVersion, addr: ServerAddr, new: bool, info: &ServerInfo) {
    print(if new { "SVADD" } else { "SVCHG" }, &[
        &addr.addr,
        &LogVersion(info.info_version),
        &info.flags,
        &b64(&info.version),
        &b64(&info.game_type),
        &b64(&info.map),
        &b64(&info.name),
    ]);
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

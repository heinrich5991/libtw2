#[cfg(feature = "buffer")]
pub use libtw2_buffer as buffer;
#[cfg(feature = "common")]
pub use libtw2_common as common;
#[cfg(feature = "datafile")]
pub use libtw2_datafile as datafile;
#[cfg(feature = "demo")]
pub use libtw2_demo as demo;
#[cfg(feature = "event_loop")]
pub use libtw2_event_loop as event_loop;
#[cfg(feature = "httphook")]
pub use libtw2_httphook as httphook;
#[cfg(feature = "logger")]
pub use libtw2_logger as logger;
#[cfg(feature = "map")]
pub use libtw2_map as map;
#[cfg(feature = "net")]
pub use libtw2_net as net;
#[cfg(feature = "packer")]
pub use libtw2_packer as packer;
#[cfg(feature = "polyfill_1_63")]
pub use libtw2_polyfill_1_63 as polyfill_1_63;
#[cfg(feature = "register")]
pub use libtw2_register as register;
#[cfg(feature = "serverbrowse")]
pub use libtw2_serverbrowse as serverbrowse;
#[cfg(feature = "socket")]
pub use libtw2_socket as socket;
#[cfg(feature = "stats_browser")]
pub use libtw2_stats_browser as stats_browser;
#[cfg(feature = "teehistorian")]
pub use libtw2_teehistorian as teehistorian;
#[cfg(feature = "tools")]
pub use libtw2_tools as tools;
#[cfg(feature = "warn")]
pub use libtw2_warn as warn;
#[cfg(feature = "world")]
pub use libtw2_world as world;
#[cfg(feature = "zlib_minimal")]
pub use libtw2_zlib_minimal as zlib_minimal;

pub mod gamenet {
    #[cfg(feature = "gamenet_common")]
    pub use libtw2_gamenet_common as common;
    #[cfg(feature = "gamenet_ddnet")]
    pub use libtw2_gamenet_ddnet as ddnet;
    #[cfg(feature = "gamenet_snap")]
    pub use libtw2_gamenet_snap as snap;
    #[cfg(feature = "gamenet_spec")]
    pub use libtw2_gamenet_spec as spec;
    #[cfg(feature = "gamenet_teeworlds_0_5")]
    pub use libtw2_gamenet_teeworlds_0_5 as teeworlds_0_5;
    #[cfg(feature = "gamenet_teeworlds_0_6")]
    pub use libtw2_gamenet_teeworlds_0_6 as teeworlds_0_6;
    #[cfg(feature = "gamenet_teeworlds_0_7")]
    pub use libtw2_gamenet_teeworlds_0_7 as teeworlds_0_7;
}

pub mod huffman {
    #[cfg(feature = "huffman")]
    pub use libtw2_huffman as huffman;
    #[cfg(feature = "huffman_reference")]
    pub use libtw2_huffman_reference as reference;
    #[cfg(all(feature = "huffman_reference_sys", feature = "c_toolchain"))]
    pub use libtw2_huffman_reference_sys as reference_sys;
}

pub mod snapshot {
    #[cfg(feature = "snapshot")]
    pub use libtw2_snapshot as snapshot;
    #[cfg(feature = "snapshot_reference")]
    pub use libtw2_snapshot_reference as reference;
    #[cfg(all(feature = "snapshot_reference_sys", feature = "c_toolchain"))]
    pub use libtw2_snapshot_reference_sys as reference_sys;
}

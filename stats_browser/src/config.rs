use time::Duration;

/// Maximum number of malformed responses to report per server.
pub const MAX_MALFORMED_RESP: u32 = 10;
/// Maximum number of excess responses to report per server.
pub const MAX_EXTRA_RESP:     u32 = 10;
/// Maximum number of responses with invalid token to report per server.
pub const MAX_INVALID_RESP:   u32 = 10;
/// Maximum number of excess token responses to report per server.
pub const MAX_EXTRA_TOKEN:    u32 = 10;
/// Maximum number of token responses with invalid token to report per server.
pub const MAX_INVALID_TOKEN:  u32 = 10;
/// Maximum number of list requests per time span.
pub const MAX_LISTS:          u32 =  1;
/// Maximum number of info requests per time span.
pub const MAX_INFOS:          u32 = 10;
/// Time span for `MAX_LISTS`.
pub const MAX_LISTS_MS:      Duration = Duration(  1_000);
/// Time span for `MAX_INFOS`.
pub const MAX_INFOS_MS:      Duration = Duration(     25);
/// Time span in which info responses are expected.
pub const INFO_EXPECT_MS:    Duration = Duration(  1_000);
/// Time span after which a successful info request is repeated.
pub const INFO_REPEAT_MS:    Duration = Duration(  5_000);
/// Time span in which list responses are expected.
pub const LIST_EXPECT_MS:    Duration = Duration(  5_000);
/// Time span after which a successful list request is repeated.
pub const LIST_REPEAT_MS:    Duration = Duration( 30_000);
/// Time span for re-resolving master servers.
pub const RESOLVE_REPEAT_MS: Duration = Duration(120_000);
/// Sleep time in the main loop.
pub const SLEEP_MS:          Duration = Duration(      5);

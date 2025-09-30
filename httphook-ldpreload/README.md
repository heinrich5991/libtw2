httphook-ldpreload
==================

Use this to register a Teeworlds 0.6/DDNet game server with the HTTPS
mastersrv. This method only works on Linux.

Building
--------

You need at least Rust 1.63 installed. This should be available via your
package manager or https://rustup.rs/.

```sh
git clone https://github.com/heinrich5991/libtw2
cd libtw2/httphook-ldpreload
cargo build --release
ls ../target/release/liblibtw2_httphook_ldpreload.so
```

You'll find the built library in `../target/release/liblibtw2_httphook_ldpreload.so`.

Usage
-----

Use the `LD_PRELOAD` variable to instruct the dynamic linker to load this
library into the Teeworlds 0.6/DDNet game server process:

```sh
LD_PRELOAD=/path/to/liblibtw2_httphook_ldpreload.so ./teeworlds_srv "sv_register 0"
LD_PRELOAD=/path/to/liblibtw2_httphook_ldpreload.so ./DDNet-Server "sv_register 0"
```

You should disable its internal mastersrv registration using `"sv_register 0"`
as it would conflict with the library's registration process.

Configuration
-------------

The library accepts a couple of optional environment variables to configure the
registration process:

- `LIBTW2_HTTPHOOK_COMMUNITY_TOKEN` (unset by default): Use this token to
  register with a community (server group) on the mastersrv. Example:
  `ddtc_6DnZq5Ix0J2kvDHbkPNtb6bsZxOVQg4ly2jw`.
- `LIBTW2_HTTPHOOK_LOG` (default: `info`): Specify log level of the library.
  Examples: `debug`, `error`. [Documentation of the
  syntax](https://docs.rs/env_logger/0.3.5/env_logger/#enabling-logging).
- `LIBTW2_HTTPHOOK_REGISTER_URL` (default:
  `https://master1.ddnet.org/ddnet/15/register`): Contact the mastersrv given
  by this URL. Example: `http://localhost:8080/ddnet/15/register` for local testing.

Example:
```
LIBTW2_HTTPHOOK_COMMUNITY_TOKEN=ddtc_6DnZq5Ix0J2kvDHbkPNtb6bsZxOVQg4ly2jw LD_PRELOAD=/path/to/liblibtw2_httphook_ldpreload.so ./teeworlds_srv "sv_register 0"
```

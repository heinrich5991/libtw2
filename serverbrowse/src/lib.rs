#![feature(core)]
#![feature(hash)]
#![feature(io)]

#[macro_use] extern crate common;
#[macro_use] extern crate log;
extern crate "rustc-serialize" as rustc_serialize;

pub mod protocol;
